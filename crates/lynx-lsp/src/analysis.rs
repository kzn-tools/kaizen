//! Analysis engine for code analysis and diagnostic generation

use lynx_core::parser::ParsedFile;
use tower_lsp::lsp_types::Diagnostic;

use crate::diagnostics::convert_parse_errors;

pub struct AnalysisEngine;

impl AnalysisEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(convert_parse_errors(file.errors()));

        diagnostics
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parsed_file(filename: &str, content: &str) -> ParsedFile {
        ParsedFile::from_source(filename, content)
    }

    #[test]
    fn analyze_valid_file_returns_empty_diagnostics() {
        let engine = AnalysisEngine::new();
        let file = make_parsed_file("test.js", "const x = 1;");

        let diagnostics = engine.analyze(&file);

        assert!(diagnostics.is_empty());
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
}
