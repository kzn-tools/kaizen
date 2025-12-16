//! Init command - initializes Lynx configuration in a project

use clap::Args;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,
}

impl InitArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("Initializing Lynx configuration...");
        Ok(())
    }
}
