//! eqeqeq rule (Q033): Require === and !== instead of == and !=

use std::ops::ControlFlow;

use swc_ecma_ast::{BinExpr, BinaryOp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    Eqeqeq,
    id = "Q033",
    name = "eqeqeq",
    description = "Require === and !== instead of == and !=",
    category = Quality,
    severity = Warning,
    examples =
        "// Bad\nif (x == y) { }\nif (x != y) { }\n\n// Good\nif (x === y) { }\nif (x !== y) { }"
);

impl Rule for Eqeqeq {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = EqeqeqVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct EqeqeqVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
}

impl AstVisitor for EqeqeqVisitor {
    fn visit_bin_expr(&mut self, node: &BinExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        match node.op {
            BinaryOp::EqEq => {
                let (line, column) = ctx.span_to_location(node.span);
                let diagnostic = Diagnostic::new(
                    "Q033",
                    Severity::Warning,
                    "Expected '===' but found '=='",
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion("Replace '==' with '===' for strict equality");

                self.diagnostics.push(diagnostic);
            }
            BinaryOp::NotEq => {
                let (line, column) = ctx.span_to_location(node.span);
                let diagnostic = Diagnostic::new(
                    "Q033",
                    Severity::Warning,
                    "Expected '!==' but found '!='",
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion("Replace '!=' with '!==' for strict inequality");

                self.diagnostics.push(diagnostic);
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_eqeqeq(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = Eqeqeq::new();
        rule.check(&file)
    }

    #[test]
    fn detects_double_equals() {
        let diagnostics = run_eqeqeq("if (x == y) {}");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q033");
        assert_eq!(diagnostics[0].message, "Expected '===' but found '=='");
        assert!(diagnostics[0].suggestion.is_some());
    }

    #[test]
    fn detects_not_equals() {
        let diagnostics = run_eqeqeq("if (x != y) {}");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q033");
        assert_eq!(diagnostics[0].message, "Expected '!==' but found '!='");
    }

    #[test]
    fn ignores_triple_equals() {
        let diagnostics = run_eqeqeq("if (x === y) {}");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_strict_not_equals() {
        let diagnostics = run_eqeqeq("if (x !== y) {}");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_loose_comparisons() {
        let code = r#"
if (a == b) {}
if (c != d) {}
if (e == f) {}
"#;
        let diagnostics = run_eqeqeq(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn ignores_other_binary_operators() {
        let code = r#"
const sum = a + b;
const diff = a - b;
const less = a < b;
const greater = a > b;
"#;
        let diagnostics = run_eqeqeq(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_in_nested_expression() {
        let diagnostics = run_eqeqeq("const result = (a == b) && (c === d);");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Expected '===' but found '=='");
    }

    #[test]
    fn detects_in_ternary() {
        let diagnostics = run_eqeqeq("const x = a == b ? 1 : 2;");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_function() {
        let code = r#"
function compare(a, b) {
    return a == b;
}
"#;
        let diagnostics = run_eqeqeq(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = Eqeqeq::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q033");
        assert_eq!(metadata.name, "eqeqeq");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn suggestion_for_double_equals() {
        let diagnostics = run_eqeqeq("x == y");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].suggestion,
            Some("Replace '==' with '===' for strict equality".to_string())
        );
    }

    #[test]
    fn suggestion_for_not_equals() {
        let diagnostics = run_eqeqeq("x != y");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].suggestion,
            Some("Replace '!=' with '!==' for strict inequality".to_string())
        );
    }
}
