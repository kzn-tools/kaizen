use lynx_core::parser::ParseError;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};

pub fn convert_parse_error(error: &ParseError) -> Diagnostic {
    let start = Position {
        line: error.line.saturating_sub(1) as u32,
        character: error.column as u32,
    };

    let end = Position {
        line: error.line.saturating_sub(1) as u32,
        character: (error.column + 1) as u32,
    };

    Diagnostic {
        range: Range { start, end },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("lynx".to_string()),
        message: error.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

pub fn convert_parse_errors(errors: &[ParseError]) -> Vec<Diagnostic> {
    errors.iter().map(convert_parse_error).collect()
}

#[allow(dead_code)]
pub fn create_diagnostic(
    line: u32,
    start_col: u32,
    end_col: u32,
    message: String,
    severity: DiagnosticSeverity,
    tags: Option<Vec<DiagnosticTag>>,
) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line,
                character: start_col,
            },
            end: Position {
                line,
                character: end_col,
            },
        },
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("lynx".to_string()),
        message,
        related_information: None,
        tags,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parse_error(line: usize, column: usize, message: &str) -> ParseError {
        ParseError {
            line,
            column,
            span_lo: 0,
            span_hi: 1,
            message: message.to_string(),
        }
    }

    #[test]
    fn convert_single_diagnostic() {
        let error = make_parse_error(1, 6, "Expected identifier");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.message, "Expected identifier");
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.source, Some("lynx".to_string()));
    }

    #[test]
    fn diagnostic_has_correct_range() {
        let error = make_parse_error(5, 10, "Unexpected token");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.range.start.line, 4);
        assert_eq!(diagnostic.range.start.character, 10);
        assert_eq!(diagnostic.range.end.line, 4);
    }

    #[test]
    fn convert_multiple_errors() {
        let errors = vec![
            make_parse_error(1, 0, "Error 1"),
            make_parse_error(2, 5, "Error 2"),
            make_parse_error(3, 10, "Error 3"),
        ];

        let diagnostics = convert_parse_errors(&errors);

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(diagnostics[0].message, "Error 1");
        assert_eq!(diagnostics[1].message, "Error 2");
        assert_eq!(diagnostics[2].message, "Error 3");
    }

    #[test]
    fn empty_errors_returns_empty_diagnostics() {
        let errors: Vec<ParseError> = vec![];

        let diagnostics = convert_parse_errors(&errors);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn line_number_converts_from_one_based_to_zero_based() {
        let error = make_parse_error(1, 0, "Line 1 error");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.range.start.line, 0);
    }

    #[test]
    fn column_preserves_zero_based_index() {
        let error = make_parse_error(1, 0, "Column 0 error");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.range.start.character, 0);
    }

    #[test]
    fn diagnostic_severity_is_error_for_parse_errors() {
        let error = make_parse_error(1, 0, "Any error");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn diagnostic_source_is_lynx() {
        let error = make_parse_error(1, 0, "Any error");

        let diagnostic = convert_parse_error(&error);

        assert_eq!(diagnostic.source, Some("lynx".to_string()));
    }

    #[test]
    fn create_diagnostic_with_unnecessary_tag() {
        let diagnostic = create_diagnostic(
            0,
            0,
            5,
            "Unused variable 'x'".to_string(),
            DiagnosticSeverity::WARNING,
            Some(vec![DiagnosticTag::UNNECESSARY]),
        );

        assert_eq!(diagnostic.tags, Some(vec![DiagnosticTag::UNNECESSARY]));
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn create_diagnostic_with_deprecated_tag() {
        let diagnostic = create_diagnostic(
            5,
            10,
            20,
            "Deprecated API usage".to_string(),
            DiagnosticSeverity::HINT,
            Some(vec![DiagnosticTag::DEPRECATED]),
        );

        assert_eq!(diagnostic.tags, Some(vec![DiagnosticTag::DEPRECATED]));
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::HINT));
    }

    #[test]
    fn create_diagnostic_without_tags() {
        let diagnostic = create_diagnostic(
            0,
            0,
            10,
            "Error message".to_string(),
            DiagnosticSeverity::ERROR,
            None,
        );

        assert_eq!(diagnostic.tags, None);
    }

    #[test]
    fn create_diagnostic_has_correct_range() {
        let diagnostic = create_diagnostic(
            10,
            5,
            15,
            "Test".to_string(),
            DiagnosticSeverity::ERROR,
            None,
        );

        assert_eq!(diagnostic.range.start.line, 10);
        assert_eq!(diagnostic.range.start.character, 5);
        assert_eq!(diagnostic.range.end.line, 10);
        assert_eq!(diagnostic.range.end.character, 15);
    }
}
