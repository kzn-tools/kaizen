//! Pretty formatter for human-readable terminal output
//!
//! Displays diagnostics with colors, source code context, and summary.

use colored::{ColoredString, Colorize};
use lynx_core::diagnostic::Diagnostic;
use lynx_core::rules::Severity;
use std::collections::HashMap;
use std::fs;

pub struct PrettyFormatter {
    sources: HashMap<String, String>,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn with_sources(sources: HashMap<String, String>) -> Self {
        Self { sources }
    }

    pub fn format(&self, diagnostics: &[Diagnostic]) -> String {
        let mut output = String::new();

        for diag in diagnostics {
            output.push_str(&self.format_diagnostic(diag));
            output.push('\n');
        }

        if !diagnostics.is_empty() {
            output.push_str(&self.format_summary(diagnostics));
        }

        output
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        let mut lines = Vec::new();

        let severity_str = self.colorize_severity(&diag.severity);
        let header = format!(
            "{}[{}]: {}",
            severity_str,
            diag.rule_id.dimmed(),
            diag.message
        );
        lines.push(header);

        let location = format!(
            "  {} {}:{}:{}",
            "-->".blue(),
            diag.file,
            diag.line,
            diag.column
        );
        lines.push(location);

        if let Some(source_line) = self.get_source_line(&diag.file, diag.line) {
            let line_num_width = diag.line.to_string().len();
            let padding = " ".repeat(line_num_width);

            lines.push(format!("{} {}", padding, "|".blue()));

            let line_display = format!(
                "{} {} {}",
                diag.line.to_string().blue(),
                "|".blue(),
                source_line
            );
            lines.push(line_display);

            let caret_col = diag.column.saturating_sub(2);
            let caret_padding = " ".repeat(caret_col);
            let caret_len = if diag.end_column > diag.column && diag.end_line == diag.line {
                diag.end_column - diag.column
            } else {
                3
            };
            let carets = "^".repeat(caret_len.max(1));
            let caret_line = format!(
                "{} {} {}{}",
                padding,
                "|".blue(),
                caret_padding,
                carets.red()
            );
            lines.push(caret_line);

            lines.push(format!("{} {}", padding, "|".blue()));
        }

        if let Some(suggestion) = &diag.suggestion {
            let line_num_width = diag.line.to_string().len();
            let padding = " ".repeat(line_num_width);
            lines.push(format!(
                "{} {} {} {}",
                padding,
                "=".blue(),
                "suggestion:".green(),
                suggestion
            ));
        }

        lines.join("\n")
    }

    fn colorize_severity(&self, severity: &Severity) -> ColoredString {
        match severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue().bold(),
            Severity::Hint => "hint".cyan().bold(),
        }
    }

    fn get_source_line(&self, file: &str, line: usize) -> Option<String> {
        if let Some(source) = self.sources.get(file) {
            return source.lines().nth(line - 1).map(|s| s.to_string());
        }

        if let Ok(content) = fs::read_to_string(file) {
            return content.lines().nth(line - 1).map(|s| s.to_string());
        }

        None
    }

    fn format_summary(&self, diagnostics: &[Diagnostic]) -> String {
        let error_count = diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();

        let total = diagnostics.len();

        let errors_str = if error_count == 1 {
            format!("{} error", error_count)
        } else {
            format!("{} errors", error_count)
        };

        let warnings_str = if warning_count == 1 {
            format!("{} warning", warning_count)
        } else {
            format!("{} warnings", warning_count)
        };

        let problems_str = if total == 1 { "problem" } else { "problems" };

        format!(
            "\nFound {} {} ({}, {})\n",
            total.to_string().bold(),
            problems_str,
            errors_str.red(),
            warnings_str.yellow()
        )
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_diagnostic(severity: Severity, line: usize, column: usize) -> Diagnostic {
        Diagnostic::new(
            "Q002",
            severity,
            "Avoid using 'var', prefer 'let' or 'const'",
            "test.js",
            line,
            column,
        )
        .with_end(line, column + 4)
    }

    #[test]
    fn pretty_format_single_diagnostic() {
        let diag = create_test_diagnostic(Severity::Error, 3, 2);
        let mut sources = HashMap::new();
        sources.insert(
            "test.js".to_string(),
            "// comment\n// comment\nvar x = 1;".to_string(),
        );

        let formatter = PrettyFormatter::with_sources(sources);
        let output = formatter.format(&[diag]);

        assert!(output.contains("error"));
        assert!(output.contains("Q002"));
        assert!(output.contains("Avoid using 'var'"));
        assert!(output.contains("test.js:3:2"));
        assert!(output.contains("var x = 1;"));
    }

    #[test]
    fn colors_match_severity_error() {
        let formatter = PrettyFormatter::new();
        let colored = formatter.colorize_severity(&Severity::Error);
        assert_eq!(colored.to_string(), "error".red().bold().to_string());
    }

    #[test]
    fn colors_match_severity_warning() {
        let formatter = PrettyFormatter::new();
        let colored = formatter.colorize_severity(&Severity::Warning);
        assert_eq!(colored.to_string(), "warning".yellow().bold().to_string());
    }

    #[test]
    fn colors_match_severity_info() {
        let formatter = PrettyFormatter::new();
        let colored = formatter.colorize_severity(&Severity::Info);
        assert_eq!(colored.to_string(), "info".blue().bold().to_string());
    }

    #[test]
    fn colors_match_severity_hint() {
        let formatter = PrettyFormatter::new();
        let colored = formatter.colorize_severity(&Severity::Hint);
        assert_eq!(colored.to_string(), "hint".cyan().bold().to_string());
    }

    #[test]
    fn shows_source_context() {
        let diag = create_test_diagnostic(Severity::Error, 2, 2);
        let mut sources = HashMap::new();
        sources.insert(
            "test.js".to_string(),
            "const a = 1;\nvar x = 1;\nconst b = 2;".to_string(),
        );

        let formatter = PrettyFormatter::with_sources(sources);
        let output = formatter.format(&[diag]);

        assert!(output.contains("var x = 1;"));
        assert!(output.contains("^^^^"));
    }

    #[test]
    fn shows_source_context_with_correct_caret_length() {
        let diag = Diagnostic::new("Q002", Severity::Error, "Test message", "test.js", 1, 2)
            .with_end(1, 12);

        let mut sources = HashMap::new();
        sources.insert("test.js".to_string(), "var longVar = 1;".to_string());

        let formatter = PrettyFormatter::with_sources(sources);
        let output = formatter.format(&[diag]);

        assert!(output.contains("^^^^^^^^^^"));
    }

    #[test]
    fn shows_summary() {
        let diags = vec![
            create_test_diagnostic(Severity::Error, 1, 2),
            create_test_diagnostic(Severity::Error, 2, 2),
            create_test_diagnostic(Severity::Warning, 3, 2),
        ];
        let formatter = PrettyFormatter::new();
        let output = formatter.format(&diags);

        assert!(output.contains("Found"));
        assert!(output.contains("3"));
        assert!(output.contains("problems"));
        assert!(output.contains("2 errors"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn shows_summary_singular() {
        let diags = vec![create_test_diagnostic(Severity::Error, 1, 2)];
        let formatter = PrettyFormatter::new();
        let output = formatter.format(&diags);

        assert!(output.contains("1"));
        assert!(output.contains("problem"));
        assert!(output.contains("1 error"));
    }

    #[test]
    fn shows_suggestion() {
        let diag = create_test_diagnostic(Severity::Error, 1, 2)
            .with_suggestion("Use 'let' or 'const' instead of 'var'");

        let formatter = PrettyFormatter::new();
        let output = formatter.format(&[diag]);

        assert!(output.contains("suggestion:"));
        assert!(output.contains("Use 'let' or 'const' instead of 'var'"));
    }

    #[test]
    fn empty_diagnostics_produces_empty_output() {
        let formatter = PrettyFormatter::new();
        let output = formatter.format(&[]);

        assert!(output.is_empty());
    }

    #[test]
    fn handles_missing_source_file() {
        let diag = create_test_diagnostic(Severity::Error, 1, 2);
        let formatter = PrettyFormatter::new();
        let output = formatter.format(&[diag]);

        assert!(output.contains("error"));
        assert!(output.contains("Q002"));
    }

    #[test]
    fn multiple_diagnostics_same_file() {
        let diags = vec![
            create_test_diagnostic(Severity::Error, 1, 2),
            create_test_diagnostic(Severity::Warning, 3, 2),
        ];
        let mut sources = HashMap::new();
        sources.insert(
            "test.js".to_string(),
            "var a = 1;\nconst b = 2;\nvar c = 3;".to_string(),
        );

        let formatter = PrettyFormatter::with_sources(sources);
        let output = formatter.format(&diags);

        assert!(output.contains("var a = 1;"));
        assert!(output.contains("var c = 3;"));
    }
}
