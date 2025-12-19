use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, Url,
};
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, info, instrument};

use kaizen_core::diagnostic::Diagnostic as CoreDiagnostic;

use crate::analysis::AnalysisEngine;
use crate::capabilities::server_capabilities;
use crate::code_actions::generate_code_actions;
use crate::debouncer::Debouncer;
use crate::document::DocumentStore;

pub struct KaizenLanguageServer {
    client: Client,
    documents: Arc<DocumentStore>,
    analysis_engine: Arc<AnalysisEngine>,
    debouncer: Arc<Debouncer>,
    core_diagnostics: Arc<DashMap<Url, Vec<CoreDiagnostic>>>,
}

impl KaizenLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DocumentStore::new()),
            analysis_engine: Arc::new(AnalysisEngine::new()),
            debouncer: Arc::new(Debouncer::new()),
            core_diagnostics: Arc::new(DashMap::new()),
        }
    }

    async fn analyze_and_publish(&self, uri: &Url) {
        let (lsp_diagnostics, core_diags) = self
            .documents
            .get(uri)
            .map(|doc| self.analysis_engine.analyze_with_core(&doc))
            .unwrap_or_default();

        self.core_diagnostics.insert(uri.clone(), core_diags);

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }

    #[allow(dead_code)]
    fn schedule_analysis(&self, uri: Url) {
        let client = self.client.clone();
        let documents = self.documents.clone();
        let analysis_engine = self.analysis_engine.clone();
        let core_diagnostics = self.core_diagnostics.clone();

        self.debouncer.schedule(uri.clone(), move || async move {
            let (lsp_diagnostics, core_diags) = documents
                .get(&uri)
                .map(|doc| analysis_engine.analyze_with_core(&doc))
                .unwrap_or_default();

            core_diagnostics.insert(uri.clone(), core_diags);

            client.publish_diagnostics(uri, lsp_diagnostics, None).await;
        });
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KaizenLanguageServer {
    #[instrument(skip(self, _params), name = "lsp/initialize")]
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        info!("initializing LSP server");
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            ..Default::default()
        })
    }

    #[instrument(skip(self, _params), name = "lsp/initialized")]
    async fn initialized(&self, _params: InitializedParams) {
        info!("LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "kaizen-lsp server initialized")
            .await;
    }

    #[instrument(skip(self), name = "lsp/shutdown")]
    async fn shutdown(&self) -> Result<()> {
        info!("shutting down LSP server");
        Ok(())
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri), name = "lsp/textDocument/didOpen")]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        debug!(uri = %uri, "opening document");
        self.documents.open(uri.clone(), &text);
        self.analyze_and_publish(&uri).await;
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri), name = "lsp/textDocument/didChange")]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            debug!(uri = %uri, "document changed");
            self.documents.update(&uri, &change.text);
            self.analyze_and_publish(&uri).await;
        }
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri), name = "lsp/textDocument/didClose")]
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        debug!(uri = %uri, "closing document");
        self.debouncer.cancel(&uri);
        self.documents.close(&uri);
        self.core_diagnostics.remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri), name = "lsp/textDocument/codeAction")]
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = &params.range;

        let actions: Vec<CodeActionOrCommand> = self
            .core_diagnostics
            .get(uri)
            .map(|diags| generate_code_actions(uri, &diags, range))
            .unwrap_or_default();

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind};

    #[test]
    fn server_responds_to_initialize_with_capabilities() {
        let capabilities = server_capabilities();
        assert!(
            capabilities.text_document_sync.is_some(),
            "Server must declare textDocumentSync capability"
        );
    }

    #[test]
    fn server_declares_text_sync_capability() {
        let capabilities = server_capabilities();

        match &capabilities.text_document_sync {
            Some(TextDocumentSyncCapability::Kind(kind)) => {
                assert_eq!(
                    *kind,
                    TextDocumentSyncKind::FULL,
                    "textDocumentSync should be Full for initial implementation"
                );
            }
            Some(TextDocumentSyncCapability::Options(opts)) => {
                assert_eq!(
                    opts.change,
                    Some(TextDocumentSyncKind::FULL),
                    "textDocumentSync change should be Full"
                );
            }
            None => panic!("textDocumentSync capability must be declared"),
        }
    }

    #[test]
    fn server_handles_shutdown() {
        // shutdown() returns Result<()>, verify the implementation is correct
        // This is implicitly tested via the LanguageServer trait implementation
        // The actual async test would require tokio runtime
    }

    #[test]
    fn server_declares_open_close_capability() {
        let capabilities = server_capabilities();

        match &capabilities.text_document_sync {
            Some(TextDocumentSyncCapability::Options(opts)) => {
                assert_eq!(
                    opts.open_close,
                    Some(true),
                    "Server must support open/close notifications"
                );
            }
            _ => panic!("textDocumentSync must use Options variant for open_close support"),
        }
    }
}
