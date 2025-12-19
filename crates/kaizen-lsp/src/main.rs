mod analysis;
mod capabilities;
mod cli;
mod code_actions;
mod debouncer;
mod diagnostics;
mod document;
mod handlers;
mod logging;
mod server;

use clap::Parser;
use server::KaizenLanguageServer;
use tower_lsp::{LspService, Server};

use crate::cli::Cli;
use crate::logging::init_logging;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let _guard = init_logging(&cli);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(KaizenLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
