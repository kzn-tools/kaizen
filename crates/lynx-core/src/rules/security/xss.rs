//! no-xss rule (S002): Detects XSS vulnerabilities via taint analysis

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::taint::{TaintAnalyzer, TaintSinkCategory};
use crate::visitor::VisitorContext;

declare_rule!(
    Xss,
    id = "S002",
    name = "no-xss",
    description = "Disallow untrusted HTML from being inserted into the DOM",
    category = Security,
    severity = Error,
    examples = "// Bad\nconst html = req.query.html;\nelement.innerHTML = html;\n\n// Good\nconst safe = DOMPurify.sanitize(html);\nelement.innerHTML = safe;"
);

impl Rule for Xss {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|finding| finding.sink_category == TaintSinkCategory::XssSink)
            .map(|finding| {
                let (sink_line, sink_column) = ctx.span_to_location(finding.sink_span);
                let (source_line, _) = ctx.span_to_location(finding.source_span);

                let message = format!(
                    "Potential XSS: untrusted data from line {} flows to {}",
                    source_line, finding.sink_description
                );

                Diagnostic::new(
                    "S002",
                    Severity::Error,
                    message,
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
                .with_suggestion(
                    "Use DOMPurify.sanitize() or escapeHtml() to sanitize HTML content",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_xss(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = Xss::new();
        rule.check(&file)
    }

    #[test]
    fn detects_inner_html_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const html = req.body.html;
                element.innerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(!diagnostics.is_empty(), "should detect XSS via innerHTML");
        assert_eq!(diagnostics[0].rule_id, "S002");
        assert!(diagnostics[0].message.contains("XSS"));
    }

    #[test]
    fn detects_document_write_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const content = req.query.content;
                document.write(content);
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via document.write"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_document_writeln_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const content = req.body.content;
                document.writeln(content);
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via document.writeln"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_outer_html_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const html = req.query.html;
                element.outerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(!diagnostics.is_empty(), "should detect XSS via outerHTML");
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_insert_adjacent_html_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const html = req.body.content;
                element.insertAdjacentHTML("beforeend", html);
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via insertAdjacentHTML"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_jquery_html_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const content = req.query.content;
                $(selector).html(content);
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via jQuery .html()"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_inner_html_with_template_literal() {
        let code = r#"
            function handler(req, res) {
                const name = req.query.name;
                element.innerHTML = `<h1>Hello ${name}</h1>`;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via template literal in innerHTML"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_inner_html_with_string_concatenation() {
        let code = r#"
            function handler(req, res) {
                const name = req.body.name;
                element.innerHTML = "<h1>Hello " + name + "</h1>";
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect XSS via string concatenation in innerHTML"
        );
        assert_eq!(diagnostics[0].rule_id, "S002");
    }

    #[test]
    fn detects_indirect_taint_flow() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const processed = input;
                const html = "<div>" + processed + "</div>";
                element.innerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect taint flow through assignments"
        );
    }

    #[test]
    fn no_false_positive_for_static_html() {
        let code = r#"
            function render() {
                element.innerHTML = "<div>Static content</div>";
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(diagnostics.is_empty(), "should not flag static HTML");
    }

    #[test]
    fn no_false_positive_for_literal_only() {
        let code = r#"
            function render() {
                const html = "<p>Hello World</p>";
                element.innerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag HTML from literals only"
        );
    }

    #[test]
    fn no_false_positive_for_dompurify_sanitized() {
        let code = r#"
            function handler(req, res) {
                const html = req.body.html;
                const safe = DOMPurify.sanitize(html);
                element.innerHTML = safe;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag DOMPurify sanitized input"
        );
    }

    #[test]
    fn no_false_positive_for_escape_html_sanitized() {
        let code = r#"
            function handler(req, res) {
                const content = req.body.content;
                const safe = escapeHtml(content);
                element.innerHTML = safe;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag escapeHtml sanitized input"
        );
    }

    #[test]
    fn no_false_positive_for_sanitize_html_sanitized() {
        let code = r#"
            function handler(req, res) {
                const html = req.query.html;
                const safe = sanitizeHtml(html);
                element.innerHTML = safe;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag sanitizeHtml sanitized input"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"
            function handler(req, res) {
                const html = req.body.html;
                element.innerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("DOMPurify")
        );
    }

    #[test]
    fn diagnostic_shows_source_line() {
        let code = r#"
            function handler(req, res) {
                const html = req.body.html;
                element.innerHTML = html;
            }
        "#;

        let diagnostics = run_xss(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("line"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = Xss::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S002");
        assert_eq!(metadata.name, "no-xss");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
    }
}
