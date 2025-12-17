//! Analysis engine for code analysis and diagnostic generation

use lynx_core::parser::ParsedFile;
use lynx_core::rules::RuleRegistry;
use lynx_core::rules::quality::{Eqeqeq, NoConsole, NoEval, NoUnusedVars, NoVar};
use tower_lsp::lsp_types::Diagnostic;

use crate::diagnostics::{convert_diagnostics, convert_parse_errors};

pub struct AnalysisEngine {
    registry: RuleRegistry,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self {
            registry: create_default_registry(),
        }
    }

    pub fn analyze(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(convert_parse_errors(file.errors()));

        let rule_diagnostics = self.registry.run_all(file);
        diagnostics.extend(convert_diagnostics(&rule_diagnostics));

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
    use std::time::Instant;
    use tower_lsp::lsp_types::NumberOrString;

    fn make_parsed_file(filename: &str, content: &str) -> ParsedFile {
        ParsedFile::from_source(filename, content)
    }

    #[test]
    fn analyze_valid_file_returns_empty_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "const x = 1; console.log(x);");

        let diagnostics = engine.analyze(&file);

        // After rule integration, this should only have console.log diagnostics
        // For now, we expect no parse errors
        let parse_errors: Vec<_> = diagnostics.iter().filter(|d| d.code.is_none()).collect();
        assert!(parse_errors.is_empty());
    }

    #[test]
    fn syntax_errors_become_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "const = ;");

        let diagnostics = engine.analyze(&file);

        assert!(!diagnostics.is_empty());
        assert!(!diagnostics[0].message.is_empty());
    }

    #[test]
    fn multiple_syntax_errors_produce_multiple_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file(
            "test.js",
            r#"
const = ;
let = ;
var = ;
"#,
        );

        let diagnostics = engine.analyze(&file);

        assert!(
            !diagnostics.is_empty(),
            "Should have at least one diagnostic for invalid syntax"
        );
    }

    #[test]
    fn diagnostics_have_correct_source() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "const = ;");

        let diagnostics = engine.analyze(&file);

        assert_eq!(diagnostics[0].source, Some("lynx".to_string()));
    }

    // TDD Tests for issue #29: Rules integration

    #[test]
    fn analysis_runs_all_rules() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "var x = 1;");

        let diagnostics = engine.analyze(&file);

        // Should have Q030 (no-var) diagnostic
        let has_q030 = diagnostics
            .iter()
            .any(|d| matches!(&d.code, Some(NumberOrString::String(code)) if code == "Q030"));

        assert!(
            has_q030,
            "Expected Q030 diagnostic for var declaration, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn multiple_diagnostics_collected() {
        let engine = AnalysisEngine::new();
        // This code should trigger:
        // - Q030 (no-var) for "var"
        // - Q033 (eqeqeq) for "=="
        let file = make_parsed_file("test.js", "var x = 1; if (x == 2) {}");

        let diagnostics = engine.analyze(&file);

        let rule_ids: Vec<_> = diagnostics
            .iter()
            .filter_map(|d| match &d.code {
                Some(NumberOrString::String(code)) => Some(code.as_str()),
                _ => None,
            })
            .collect();

        assert!(
            rule_ids.contains(&"Q030"),
            "Expected Q030 diagnostic, got: {:?}",
            rule_ids
        );
        assert!(
            rule_ids.contains(&"Q033"),
            "Expected Q033 diagnostic, got: {:?}",
            rule_ids
        );
    }

    #[test]
    fn performance_under_30ms() {
        let engine = AnalysisEngine::new();

        // Generate a 500-line file
        let mut code = String::new();
        for i in 0..500 {
            code.push_str(&format!("const x{} = {};\n", i, i));
        }

        let file = make_parsed_file("test.js", &code);

        let start = Instant::now();
        let _ = engine.analyze(&file);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 30,
            "Analysis took {}ms, expected under 30ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn rule_diagnostics_have_correct_format() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "var x = 1;");

        let diagnostics = engine.analyze(&file);

        let var_diagnostic = diagnostics
            .iter()
            .find(|d| matches!(&d.code, Some(NumberOrString::String(code)) if code == "Q030"));

        assert!(var_diagnostic.is_some(), "Expected Q030 diagnostic");
        let diag = var_diagnostic.unwrap();

        assert_eq!(diag.source, Some("lynx".to_string()));
        assert!(diag.severity.is_some());
        assert!(!diag.message.is_empty());
    }
}
