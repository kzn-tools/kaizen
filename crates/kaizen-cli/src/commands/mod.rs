//! CLI command implementations

pub mod auth;
pub mod check;
pub mod explain;
pub mod init;

pub use auth::AuthArgs;
pub use check::CheckArgs;
pub use explain::ExplainArgs;
pub use init::InitArgs;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Manage authentication")]
    Auth(AuthArgs),

    #[command(about = "Analyze JavaScript/TypeScript files for issues")]
    Check(CheckArgs),

    #[command(about = "Initialize Kaizen configuration in current directory")]
    Init(InitArgs),

    #[command(about = "Show detailed explanation for a specific rule")]
    Explain(ExplainArgs),
}
