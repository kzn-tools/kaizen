//! Explain command - provides detailed explanation of a rule

use clap::Args;

#[derive(Args, Debug)]
pub struct ExplainArgs {
    /// Rule ID to explain (e.g., "no-console", "sql-injection")
    #[arg(value_name = "RULE_ID")]
    pub rule_id: String,
}

impl ExplainArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("Explaining rule: {}", self.rule_id);
        Ok(())
    }
}
