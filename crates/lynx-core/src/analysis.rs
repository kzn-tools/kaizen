//! Analysis engine for code analysis and diagnostic generation
//!
//! Provides the core analysis functionality for CLI and other consumers.

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::RuleRegistry;
use crate::rules::quality::{
    Eqeqeq, FloatingPromises, MaxComplexity, MaxDepth, NoConsole, NoEval, NoUnusedVars, NoVar,
    PreferNullishCoalescing, PreferOptionalChaining, PreferUsing,
};

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
        let disable_directives = file.disable_directives();

        for error in file.errors() {
            let diagnostic = Diagnostic::new(
                "PARSE",
                crate::rules::Severity::Error,
                &error.message,
                &file.metadata().filename,
                error.line,
                error.column,
            );
            if !disable_directives.is_disabled(diagnostic.line, &diagnostic.rule_id) {
                diagnostics.push(diagnostic);
            }
        }

        let rule_diagnostics = self.registry.run_all(file);
        for diagnostic in rule_diagnostics {
            if !disable_directives.is_disabled(diagnostic.line, &diagnostic.rule_id) {
                diagnostics.push(diagnostic);
            }
        }

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

    registry.register(Box::new(MaxComplexity::new()));
    registry.register(Box::new(MaxDepth::new()));
    registry.register(Box::new(NoVar::new()));
    registry.register(Box::new(Eqeqeq::new()));
    registry.register(Box::new(NoConsole::new()));
    registry.register(Box::new(NoEval::new()));
    registry.register(Box::new(NoUnusedVars::new()));
    registry.register(Box::new(PreferUsing::new()));
    registry.register(Box::new(FloatingPromises::new()));
    registry.register(Box::new(PreferOptionalChaining::new()));
    registry.register(Box::new(PreferNullishCoalescing::new()));

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

    #[test]
    fn disable_next_line_suppresses_diagnostic() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"// lynx-disable-next-line Q030
var x = 1;"#,
        );

        let diagnostics = engine.analyze(&file);

        assert!(
            !diagnostics.iter().any(|d| d.rule_id == "Q030"),
            "Q030 should be suppressed by disable comment"
        );
    }

    #[test]
    fn disable_line_suppresses_diagnostic() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "var x = 1; // lynx-disable-line Q030");

        let diagnostics = engine.analyze(&file);

        assert!(
            !diagnostics.iter().any(|d| d.rule_id == "Q030"),
            "Q030 should be suppressed by disable comment"
        );
    }

    #[test]
    fn disable_next_line_all_rules() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"// lynx-disable-next-line
var x = 1; if (x == 2) {}"#,
        );

        let diagnostics = engine.analyze(&file);

        assert!(
            diagnostics.is_empty(),
            "All diagnostics should be suppressed"
        );
    }

    #[test]
    fn disable_specific_rule_does_not_affect_others() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"// lynx-disable-next-line Q030
var x = 1; if (x == 2) {}"#,
        );

        let diagnostics = engine.analyze(&file);

        assert!(
            !diagnostics.iter().any(|d| d.rule_id == "Q030"),
            "Q030 should be suppressed"
        );
        assert!(
            diagnostics.iter().any(|d| d.rule_id == "Q033"),
            "Q033 should NOT be suppressed"
        );
    }

    #[test]
    fn disable_multiple_rules() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"// lynx-disable-next-line Q030, Q033
var x = 1; if (x == 2) {}"#,
        );

        let diagnostics = engine.analyze(&file);

        assert!(
            !diagnostics.iter().any(|d| d.rule_id == "Q030"),
            "Q030 should be suppressed"
        );
        assert!(
            !diagnostics.iter().any(|d| d.rule_id == "Q033"),
            "Q033 should be suppressed"
        );
    }

    #[test]
    fn disable_does_not_affect_other_lines() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"// lynx-disable-next-line Q030
var x = 1;
var y = 2;"#,
        );

        let diagnostics = engine.analyze(&file);

        let line_2_q030 = diagnostics
            .iter()
            .any(|d| d.rule_id == "Q030" && d.line == 2);
        let line_3_q030 = diagnostics
            .iter()
            .any(|d| d.rule_id == "Q030" && d.line == 3);

        assert!(!line_2_q030, "Q030 on line 2 should be suppressed");
        assert!(line_3_q030, "Q030 on line 3 should NOT be suppressed");
    }
}
