use zed_extension_api::{self as zed, Result};

struct KaizenExtension;

impl zed::Extension for KaizenExtension {
    fn new() -> Self {
        KaizenExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = worktree.which("kaizen-lsp").ok_or_else(|| {
            "kaizen-lsp not found in PATH. Install with: ./scripts/install-local.sh".to_string()
        })?;

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(KaizenExtension);
