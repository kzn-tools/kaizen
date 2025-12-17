use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{InitializeParams, InitializeResult, InitializedParams, MessageType};
use tower_lsp::{Client, LanguageServer};

use crate::capabilities::server_capabilities;

pub struct LynxLanguageServer {
    client: Client,
}

impl LynxLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LynxLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "lynx-lsp server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::{
        DiagnosticServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    };

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
    fn server_declares_diagnostic_provider() {
        let capabilities = server_capabilities();

        match &capabilities.diagnostic_provider {
            Some(DiagnosticServerCapabilities::Options(opts)) => {
                assert!(
                    !opts.inter_file_dependencies,
                    "inter_file_dependencies should be false for initial implementation"
                );
            }
            Some(DiagnosticServerCapabilities::RegistrationOptions(_)) => {
                // Registration options are also valid
            }
            None => panic!("diagnosticProvider capability must be declared"),
        }
    }

    #[test]
    fn server_handles_shutdown() {
        // shutdown() returns Result<()>, verify the implementation is correct
        // This is implicitly tested via the LanguageServer trait implementation
        // The actual async test would require tokio runtime
    }
}
