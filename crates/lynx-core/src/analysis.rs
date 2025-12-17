//! Analysis engine for code analysis and diagnostic generation
//!
//! Provides the core analysis functionality for CLI and other consumers.

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::RuleRegistry;
use crate::rules::quality::{Eqeqeq, NoConsole, NoEval, NoUnusedVars, NoVar};

pub struct AnalysisEngine {
    registry: RuleRegistry,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self {
            registry: create_default_registry(),
        }
    }

    pub fn with_config(config: &Config) -> Self {
        let mut registry = create_default_registry();
        registry.configure(&config.rules);
        Self { registry }
    }

    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    pub fn analyze(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for error in file.errors() {
            diagnostics.push(Diagnostic::new(
                "PARSE",
                crate::rules::Severity::Error,
                &error.message,
                &file.metadata().filename,
                error.line,
                error.column,
            ));
        }

        let rule_diagnostics = self.registry.run_all(file);
        diagnostics.extend(rule_diagnostics);

        diagnostics
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn create_default_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::new();

    registry.register(Box::new(NoVar::new()));
    registry.register(Box::new(Eqeqeq::new()));
    registry.register(Box::new(NoConsole::new()));
    registry.register(Box::new(NoEval::new()));
    registry.register(Box::new(NoUnusedVars::new()));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parsed_file(filename: &str, content: &str) -> ParsedFile {
        ParsedFile::from_source(filename, content)
    }

    #[test]
    fn analyze_valid_file_returns_diagnostics_for_issues() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "var x = 1;");

        let diagnostics = engine.analyze(&file);

        assert!(
            diagnostics.iter().any(|d| d.rule_id == "Q030"),
            "Expected Q030 diagnostic for var declaration"
        );
    }

    #[test]
    fn syntax_errors_become_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "const = ;");

        let diagnostics = engine.analyze(&file);

        assert!(
            diagnostics.iter().any(|d| d.rule_id == "PARSE"),
            "Expected PARSE diagnostic for syntax error"
        );
    }

    #[test]
    fn multiple_rules_produce_multiple_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "var x = 1; if (x == 2) {}");

        let diagnostics = engine.analyze(&file);

        let rule_ids: Vec<_> = diagnostics.iter().map(|d| d.rule_id.as_str()).collect();

        assert!(rule_ids.contains(&"Q030"), "Expected Q030 for var");
        assert!(rule_ids.contains(&"Q033"), "Expected Q033 for ==");
    }
}
