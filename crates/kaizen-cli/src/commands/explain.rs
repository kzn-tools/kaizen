//! Explain command - provides detailed explanation of a rule

use clap::Args;
use colored::Colorize;
use kaizen_core::analysis::AnalysisEngine;
use kaizen_core::config::load_config_or_default_with_warnings;
use kaizen_core::rules::{RuleCategory, Severity};
use rust_i18n::t;
use std::env;

#[derive(Args, Debug)]
pub struct ExplainArgs {
    #[arg(
        value_name = "RULE_ID",
        help = "Rule ID to explain (e.g., \"Q032\", \"no-console\")"
    )]
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
                println!("{}", t!("explain.rule_title", id = metadata.id).bold());
                println!();
                println!("  {}: {}", t!("explain.name").cyan(), metadata.name);
                println!(
                    "  {}: {}",
                    t!("explain.description").cyan(),
                    metadata.description
                );
                println!(
                    "  {}: {}",
                    t!("explain.category").cyan(),
                    format_category(&metadata.category)
                );
                println!(
                    "  {}: {}",
                    t!("explain.severity_label").cyan(),
                    format_severity(&metadata.severity)
                );

                if let Some(url) = metadata.docs_url {
                    println!("  {}: {}", t!("explain.documentation").cyan(), url);
                }

                if let Some(examples) = metadata.examples {
                    println!();
                    println!("  {}:", t!("explain.examples").cyan());
                    for line in examples.lines() {
                        println!("    {}", line);
                    }
                }

                println!();
                if is_enabled {
                    println!(
                        "  {}: {}",
                        t!("explain.status").cyan(),
                        t!("explain.enabled").green()
                    );
                } else {
                    println!(
                        "  {}: {}",
                        t!("explain.status").cyan(),
                        t!("explain.disabled").red()
                    );
                }
                println!();

                Ok(())
            }
            None => {
                eprintln!(
                    "{} {}",
                    "error:".red().bold(),
                    t!("explain.not_found", id = self.rule_id)
                );
                eprintln!();
                eprintln!("{}", t!("explain.available_rules"));

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
        RuleCategory::Quality => t!("category.quality").to_string(),
        RuleCategory::Security => t!("category.security").to_string(),
    }
}

fn format_severity(severity: &Severity) -> String {
    match severity {
        Severity::Error => t!("output.error").red().to_string(),
        Severity::Warning => t!("output.warning").yellow().to_string(),
        Severity::Info => t!("output.info").blue().to_string(),
        Severity::Hint => t!("output.hint").cyan().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use kaizen_core::analysis::AnalysisEngine;
    use kaizen_core::config::Config;

    #[test]
    fn explain_known_rule_returns_metadata() {
        let config = Config::default();
        let engine = AnalysisEngine::with_config(&config);
        let registry = engine.registry();

        let rule = registry.get_rule("Q030");
        assert!(rule.is_some(), "Q030 rule should exist");

        let metadata = rule.unwrap().metadata();
        assert_eq!(metadata.id, "Q030");
        assert_eq!(metadata.name, "no-var");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn explain_unknown_rule_returns_none() {
        let config = Config::default();
        let engine = AnalysisEngine::with_config(&config);
        let registry = engine.registry();

        let rule = registry.get_rule("Q999");
        assert!(rule.is_none(), "Q999 rule should not exist");
    }

    #[test]
    fn explain_rule_by_name() {
        let config = Config::default();
        let engine = AnalysisEngine::with_config(&config);
        let registry = engine.registry();

        let rule = registry.get_rule_by_name("no-var");
        assert!(rule.is_some(), "no-var rule should exist");
        assert_eq!(rule.unwrap().metadata().id, "Q030");
    }

    #[test]
    fn rule_has_examples() {
        let config = Config::default();
        let engine = AnalysisEngine::with_config(&config);
        let registry = engine.registry();

        let rule = registry.get_rule("Q030").expect("Q030 should exist");
        let metadata = rule.metadata();

        assert!(
            metadata.examples.is_some(),
            "Q030 should have examples defined"
        );
        let examples = metadata.examples.unwrap();
        assert!(examples.contains("var"), "Examples should show var usage");
    }
}
