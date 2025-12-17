//! Explain command - provides detailed explanation of a rule

use clap::Args;
use colored::Colorize;
use lynx_core::analysis::AnalysisEngine;
use lynx_core::config::load_config_or_default_with_warnings;
use lynx_core::rules::{RuleCategory, Severity};
use std::env;

#[derive(Args, Debug)]
pub struct ExplainArgs {
    /// Rule ID to explain (e.g., "Q032", "no-console")
    #[arg(value_name = "RULE_ID")]
    pub rule_id: String,
}

impl ExplainArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let cwd = env::current_dir()?;
        let config_result = load_config_or_default_with_warnings(&cwd);
        let config = config_result.config;
        let engine = AnalysisEngine::with_config(&config);
        let registry = engine.registry();

        let rule = registry
            .get_rule(&self.rule_id)
            .or_else(|| registry.get_rule_by_name(&self.rule_id));

        match rule {
            Some(rule) => {
                let metadata = rule.metadata();
                let is_enabled = registry.is_rule_enabled(&self.rule_id);

                println!();
                println!("{}", format!("Rule: {}", metadata.id).bold());
                println!();
                println!("  {}: {}", "Name".cyan(), metadata.name);
                println!("  {}: {}", "Description".cyan(), metadata.description);
                println!(
                    "  {}: {}",
                    "Category".cyan(),
                    format_category(&metadata.category)
                );
                println!(
                    "  {}: {}",
                    "Severity".cyan(),
                    format_severity(&metadata.severity)
                );

                if let Some(url) = metadata.docs_url {
                    println!("  {}: {}", "Documentation".cyan(), url);
                }

                println!();
                if is_enabled {
                    println!("  {}: {}", "Status".cyan(), "enabled".green());
                } else {
                    println!("  {}: {}", "Status".cyan(), "disabled".red());
                }
                println!();

                Ok(())
            }
            None => {
                eprintln!(
                    "{} Rule '{}' not found",
                    "error:".red().bold(),
                    self.rule_id
                );
                eprintln!();
                eprintln!("Available rules:");

                for rule in registry.rules() {
                    let meta = rule.metadata();
                    eprintln!("  {} ({})", meta.id, meta.name);
                }

                std::process::exit(1);
            }
        }
    }
}

fn format_category(category: &RuleCategory) -> String {
    match category {
        RuleCategory::Quality => "Quality".to_string(),
        RuleCategory::Security => "Security".to_string(),
    }
}

fn format_severity(severity: &Severity) -> String {
    match severity {
        Severity::Error => "Error".red().to_string(),
        Severity::Warning => "Warning".yellow().to_string(),
        Severity::Info => "Info".blue().to_string(),
        Severity::Hint => "Hint".cyan().to_string(),
    }
}
