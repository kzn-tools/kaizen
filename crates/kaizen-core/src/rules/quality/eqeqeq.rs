//! eqeqeq rule (Q033): Require === and !== instead of == and !=
//!
//! Exception: `== null` and `!= null` are allowed as they check for both null and undefined.

use std::ops::ControlFlow;

use swc_common::Spanned;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Lit};

use crate::declare_rule;
use crate::diagnostic::{Diagnostic, Fix};
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
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct EqeqeqVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl EqeqeqVisitor<'_> {
    /// Check if this is a null comparison (x == null or x != null).
    /// These are intentional patterns to check for both null and undefined.
    fn is_null_comparison(&self, node: &BinExpr) -> bool {
        Self::is_null_literal(&node.left) || Self::is_null_literal(&node.right)
    }

    fn is_null_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::Lit(Lit::Null(_)))
    }

    fn find_operator_position(&self, node: &BinExpr, op_str: &str) -> Option<(usize, usize)> {
        let source = self.ctx.file().source();
        let left_end = node.left.span().hi.0 as usize;
        let right_start = node.right.span().lo.0 as usize;

        if left_end >= source.len() || right_start > source.len() {
            return None;
        }

        let between = &source[left_end..right_start];
        if let Some(offset) = between.find(op_str) {
            let op_byte_pos = left_end + offset;
            let prefix = &source[..op_byte_pos];
            let line = prefix.matches('\n').count() + 1;
            let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let column = op_byte_pos - last_newline + 1;
            Some((line, column))
        } else {
            None
        }
    }
}

impl AstVisitor for EqeqeqVisitor<'_> {
    fn visit_bin_expr(&mut self, node: &BinExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        // Allow == null and != null as they check for both null and undefined
        if self.is_null_comparison(node) {
            return ControlFlow::Continue(());
        }

        match node.op {
            BinaryOp::EqEq => {
                let (line, column, _, _) = self.ctx.span_to_range(node.span);

                let (op_line, op_column) = self
                    .find_operator_position(node, "==")
                    .unwrap_or((line, column));

                let fix = Fix::replace(
                    "Replace '==' with '==='",
                    "===",
                    op_line,
                    op_column,
                    op_line,
                    op_column + 1,
                );

                let diagnostic = Diagnostic::new(
                    "Q033",
                    Severity::Warning,
                    "Expected '===' but found '=='",
                    &self.file_path,
                    op_line,
                    op_column,
                )
                .with_end(op_line, op_column + 2)
                .with_suggestion("Replace '==' with '===' for strict equality")
                .with_fix(fix);

                self.diagnostics.push(diagnostic);
            }
            BinaryOp::NotEq => {
                let (line, column, _, _) = self.ctx.span_to_range(node.span);

                let (op_line, op_column) = self
                    .find_operator_position(node, "!=")
                    .unwrap_or((line, column));

                let fix = Fix::replace(
                    "Replace '!=' with '!=='",
                    "!==",
                    op_line,
                    op_column,
                    op_line,
                    op_column + 1,
                );

                let diagnostic = Diagnostic::new(
                    "Q033",
                    Severity::Warning,
                    "Expected '!==' but found '!='",
                    &self.file_path,
                    op_line,
                    op_column,
                )
                .with_end(op_line, op_column + 2)
                .with_suggestion("Replace '!=' with '!==' for strict inequality")
                .with_fix(fix);

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

    #[test]
    fn fix_for_double_equals() {
        let diagnostics = run_eqeqeq("x == y");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].fixes.len(), 1);

        let fix = &diagnostics[0].fixes[0];
        assert_eq!(fix.title, "Replace '==' with '==='");
        assert!(matches!(
            &fix.kind,
            crate::diagnostic::FixKind::ReplaceWith { new_text } if new_text == "==="
        ));
    }

    #[test]
    fn fix_for_not_equals() {
        let diagnostics = run_eqeqeq("x != y");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].fixes.len(), 1);

        let fix = &diagnostics[0].fixes[0];
        assert_eq!(fix.title, "Replace '!=' with '!=='");
        assert!(matches!(
            &fix.kind,
            crate::diagnostic::FixKind::ReplaceWith { new_text } if new_text == "!=="
        ));
    }

    // === Null comparison exception tests ===

    #[test]
    fn allows_equals_null() {
        let diagnostics = run_eqeqeq("if (x == null) {}");

        assert!(
            diagnostics.is_empty(),
            "x == null should be allowed (checks for both null and undefined)"
        );
    }

    #[test]
    fn allows_not_equals_null() {
        let diagnostics = run_eqeqeq("if (x != null) {}");

        assert!(
            diagnostics.is_empty(),
            "x != null should be allowed (checks for both null and undefined)"
        );
    }

    #[test]
    fn allows_null_equals_variable() {
        let diagnostics = run_eqeqeq("if (null == x) {}");

        assert!(
            diagnostics.is_empty(),
            "null == x should be allowed (checks for both null and undefined)"
        );
    }

    #[test]
    fn allows_null_not_equals_variable() {
        let diagnostics = run_eqeqeq("if (null != x) {}");

        assert!(
            diagnostics.is_empty(),
            "null != x should be allowed (checks for both null and undefined)"
        );
    }

    #[test]
    fn allows_null_check_in_ternary() {
        let diagnostics = run_eqeqeq("const result = value != null ? value : defaultValue;");

        assert!(
            diagnostics.is_empty(),
            "null check in ternary should be allowed"
        );
    }

    #[test]
    fn allows_null_check_in_logical_expression() {
        let diagnostics = run_eqeqeq("if (a != null && a.length > 0) {}");

        assert!(
            diagnostics.is_empty(),
            "null check in logical expression should be allowed"
        );
    }

    #[test]
    fn still_detects_undefined_comparison() {
        // undefined comparisons are NOT the same as null comparisons
        // x == undefined only checks for undefined, not null
        let diagnostics = run_eqeqeq("if (x == undefined) {}");

        assert_eq!(
            diagnostics.len(),
            1,
            "x == undefined should still be flagged"
        );
    }
}
