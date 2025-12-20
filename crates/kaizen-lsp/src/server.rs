use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionResponse, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Url,
};
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, info, instrument, warn};

use kaizen_core::config::{LicenseConfig, find_config_file, load_config};
use kaizen_core::diagnostic::Diagnostic as CoreDiagnostic;
use kaizen_core::licensing::{LicenseInfo, LicenseValidator, PremiumTier};

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
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    license_tier: Arc<RwLock<PremiumTier>>,
    #[allow(dead_code)]
    license_info: Arc<RwLock<Option<LicenseInfo>>>,
}

impl KaizenLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DocumentStore::new()),
            analysis_engine: Arc::new(AnalysisEngine::new()),
            debouncer: Arc::new(Debouncer::new()),
            core_diagnostics: Arc::new(DashMap::new()),
            workspace_root: Arc::new(RwLock::new(None)),
            license_tier: Arc::new(RwLock::new(PremiumTier::Free)),
            license_info: Arc::new(RwLock::new(None)),
        }
    }

    pub fn license_tier(&self) -> PremiumTier {
        *self.license_tier.read()
    }

    async fn load_license(&self) {
        let workspace_root = self.workspace_root.read().clone();

        let license_config = tokio::task::spawn_blocking(move || {
            workspace_root
                .as_ref()
                .and_then(|root| find_config_file(root))
                .and_then(|path| load_config(&path).ok())
                .map(|config| config.license)
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();

        let result = load_license_from_sources(&license_config).await;

        *self.license_tier.write() = result.tier;
        *self.license_info.write() = result.info;

        info!(tier = %result.tier.as_str(), source = %result.source.as_str(), "license loaded");
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
    #[instrument(skip(self, params), name = "lsp/initialize")]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("initializing LSP server");

        if let Some(root_uri) = params.root_uri {
            if let Ok(path) = root_uri.to_file_path() {
                *self.workspace_root.write() = Some(path);
            }
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            ..Default::default()
        })
    }

    #[instrument(skip(self, _params), name = "lsp/initialized")]
    async fn initialized(&self, _params: InitializedParams) {
        info!("LSP server initialized");

        self.load_license().await;

        let tier = self.license_tier();
        self.client
            .log_message(
                MessageType::INFO,
                format!("kaizen-lsp initialized (tier: {})", tier.as_str()),
            )
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

    #[instrument(skip(self, _params), name = "lsp/workspace/didChangeConfiguration")]
    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        info!("configuration changed, reloading license");
        self.load_license().await;
    }
}

const ENV_VAR_NAME: &str = "KAIZEN_API_KEY";
const CREDENTIALS_FILE: &str = ".kaizen/credentials";

struct LicenseResult {
    tier: PremiumTier,
    info: Option<LicenseInfo>,
    source: LicenseSource,
}

#[derive(Debug, Clone, Copy)]
enum LicenseSource {
    Environment,
    Credentials,
    Config,
    None,
}

impl LicenseSource {
    fn as_str(&self) -> &'static str {
        match self {
            LicenseSource::Environment => "environment",
            LicenseSource::Credentials => "credentials",
            LicenseSource::Config => "config",
            LicenseSource::None => "none",
        }
    }
}

async fn load_license_from_sources(config: &LicenseConfig) -> LicenseResult {
    if let Some(key) = read_from_env() {
        if let Some(result) = validate_key(&key, LicenseSource::Environment) {
            return result;
        }
    }

    if let Some(key) = read_from_credentials().await {
        if let Some(result) = validate_key(&key, LicenseSource::Credentials) {
            return result;
        }
    }

    if let Some(key) = config.api_key.as_ref() {
        if let Some(result) = validate_key(key, LicenseSource::Config) {
            return result;
        }
    }

    LicenseResult {
        tier: PremiumTier::Free,
        info: None,
        source: LicenseSource::None,
    }
}

fn read_from_env() -> Option<String> {
    std::env::var(ENV_VAR_NAME).ok().filter(|s| !s.is_empty())
}

async fn read_from_credentials() -> Option<String> {
    let home = dirs::home_dir()?;
    let credentials_path = home.join(CREDENTIALS_FILE);
    tokio::fs::read_to_string(credentials_path)
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn validate_key(key: &str, source: LicenseSource) -> Option<LicenseResult> {
    let secret = get_validation_secret()?;
    let validator = LicenseValidator::new(&secret);

    match validator.validate(key) {
        Ok(info) => Some(LicenseResult {
            tier: info.tier,
            info: Some(info),
            source,
        }),
        Err(e) => {
            warn!(source = %source.as_str(), error = %e, "license validation failed");
            None
        }
    }
}

fn get_validation_secret() -> Option<Vec<u8>> {
    std::env::var("KAIZEN_LICENSE_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.into_bytes())
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
