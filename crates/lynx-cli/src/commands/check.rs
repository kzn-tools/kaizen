//! Check command - analyzes JavaScript/TypeScript files for issues

use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Path to file or directory to analyze
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// Output format for diagnostics
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Fail on warnings (exit code 1)
    #[arg(long)]
    pub fail_on_warnings: bool,
}

impl CheckArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("Checking: {}", self.path.display());
        Ok(())
    }
}
