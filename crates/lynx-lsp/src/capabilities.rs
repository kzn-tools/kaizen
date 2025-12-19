use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
};

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                ..Default::default()
            },
        )),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
            work_done_progress_options: Default::default(),
            resolve_provider: None,
        })),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_declares_code_action_capability() {
        let capabilities = server_capabilities();
        assert!(
            capabilities.code_action_provider.is_some(),
            "Server must declare codeActionProvider capability"
        );
    }

    #[test]
    fn code_action_provider_advertises_quickfix_kind() {
        let capabilities = server_capabilities();

        match &capabilities.code_action_provider {
            Some(CodeActionProviderCapability::Options(opts)) => {
                let kinds = opts.code_action_kinds.as_ref().unwrap();
                assert!(
                    kinds.contains(&CodeActionKind::QUICKFIX),
                    "Server must advertise quickfix code action kind"
                );
            }
            _ => panic!("codeActionProvider must use Options variant with codeActionKinds"),
        }
    }
}
