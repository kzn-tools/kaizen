//! prefer-optional-chaining rule (Q022): Suggest ?. instead of && for property access

use std::ops::ControlFlow;

use swc_ecma_ast::{BinExpr, BinaryOp, Expr, MemberProp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    PreferOptionalChaining,
    id = "Q022",
    name = "prefer-optional-chaining",
    description = "Suggest optional chaining (?.) instead of && for property access",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nobj && obj.prop\nobj && obj.a && obj.a.b\n\n// Good\nobj?.prop\nobj?.a?.b"
);

impl Rule for PreferOptionalChaining {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = PreferOptionalChainingVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct PreferOptionalChainingVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
}

impl PreferOptionalChainingVisitor {
    fn extract_expr_path(&self, expr: &Expr) -> Option<Vec<String>> {
        match expr {
            Expr::Ident(ident) => Some(vec![ident.sym.to_string()]),
            Expr::Member(member) => {
                let mut path = self.extract_expr_path(&member.obj)?;
                match &member.prop {
                    MemberProp::Ident(ident) => {
                        path.push(ident.sym.to_string());
                    }
                    MemberProp::Computed(_) => {
                        path.push("[computed]".to_string());
                    }
                    MemberProp::PrivateName(name) => {
                        path.push(format!("#{}", name.name));
                    }
                }
                Some(path)
            }
            _ => None,
        }
    }

    fn is_prefix_of(&self, prefix: &[String], full: &[String]) -> bool {
        if prefix.len() >= full.len() {
            return false;
        }
        prefix.iter().zip(full.iter()).all(|(a, b)| a == b)
    }

    fn format_path_with_optional_chaining(&self, path: &[String]) -> String {
        if path.is_empty() {
            return String::new();
        }
        let mut result = path[0].clone();
        for part in &path[1..] {
            result.push_str("?.");
            result.push_str(part);
        }
        result
    }

    fn check_and_report(
        &mut self,
        left: &Expr,
        right: &Expr,
        ctx: &VisitorContext,
        span: swc_common::Span,
    ) {
        let left_path = match self.extract_expr_path(left) {
            Some(path) => path,
            None => return,
        };

        let right_path = match self.extract_expr_path(right) {
            Some(path) => path,
            None => return,
        };

        if self.is_prefix_of(&left_path, &right_path) {
            let (line, column) = ctx.span_to_location(span);
            let suggested = self.format_path_with_optional_chaining(&right_path);
            let left_str = left_path.join(".");
            let right_str = right_path.join(".");

            let diagnostic = Diagnostic::new(
                "Q022",
                Severity::Warning,
                format!(
                    "Prefer optional chaining over && for property access: '{} && {}'",
                    left_str, right_str
                ),
                &self.file_path,
                line,
                column,
            )
            .with_suggestion(format!("Replace with '{}'", suggested));

            self.diagnostics.push(diagnostic);
        }
    }
}

impl AstVisitor for PreferOptionalChainingVisitor {
    fn visit_bin_expr(&mut self, node: &BinExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        if node.op != BinaryOp::LogicalAnd {
            return ControlFlow::Continue(());
        }

        match node.left.as_ref() {
            Expr::Bin(left_bin) if left_bin.op == BinaryOp::LogicalAnd => {
                // For chained && expressions, we only check the rightmost pair
                // The visitor will also visit the left BinExpr separately
                if let Some(innermost_right) = self.get_rightmost_and_operand(left_bin) {
                    self.check_and_report(innermost_right, &node.right, ctx, node.span);
                }
            }
            left_expr => {
                self.check_and_report(left_expr, &node.right, ctx, node.span);
            }
        }

        ControlFlow::Continue(())
    }
}

impl PreferOptionalChainingVisitor {
    fn get_rightmost_and_operand<'a>(&self, expr: &'a BinExpr) -> Option<&'a Expr> {
        if expr.op != BinaryOp::LogicalAnd {
            return None;
        }
        Some(expr.right.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = PreferOptionalChaining::new();
        rule.check(&file)
    }

    #[test]
    fn detects_simple_optional_chaining_candidate() {
        let diagnostics = run_rule("obj && obj.prop");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q022");
        assert!(diagnostics[0].message.contains("obj && obj.prop"));
    }

    #[test]
    fn suggests_optional_chaining() {
        let diagnostics = run_rule("obj && obj.prop");

        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("obj?.prop")
        );
    }

    #[test]
    fn detects_nested_property_access() {
        let diagnostics = run_rule("obj && obj.a.b");

        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("obj?.a?.b")
        );
    }

    #[test]
    fn detects_chained_and_expressions() {
        let diagnostics = run_rule("obj && obj.a && obj.a.b");

        // Should detect both: obj && obj.a, and obj.a && obj.a.b
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_unrelated_and_conditions() {
        let diagnostics = run_rule("obj && other.prop");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_non_member_right_side() {
        let diagnostics = run_rule("obj && x > 0");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_different_base_objects() {
        let diagnostics = run_rule("foo && bar.prop");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_with_member_expression_on_left() {
        let diagnostics = run_rule("obj.a && obj.a.b");

        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("obj?.a?.b")
        );
    }

    #[test]
    fn handles_deeply_nested_chains() {
        let diagnostics = run_rule("obj.a.b && obj.a.b.c");

        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("obj?.a?.b?.c")
        );
    }

    #[test]
    fn ignores_same_expression_both_sides() {
        // obj && obj is not a property access pattern
        let diagnostics = run_rule("obj && obj");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn metadata_is_correct() {
        let rule = PreferOptionalChaining::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q022");
        assert_eq!(metadata.name, "prefer-optional-chaining");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn detects_in_if_condition() {
        let diagnostics = run_rule("if (obj && obj.prop) {}");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_ternary() {
        let diagnostics = run_rule("const x = obj && obj.prop ? 1 : 2;");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_function_body() {
        let code = r#"
function test() {
    return obj && obj.prop;
}
"#;
        let diagnostics = run_rule(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn handles_multiple_independent_patterns() {
        let code = r#"
const a = obj && obj.prop;
const b = foo && foo.bar;
"#;
        let diagnostics = run_rule(code);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_logical_or() {
        let diagnostics = run_rule("obj || obj.prop");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_already_optional_chaining() {
        let diagnostics = run_rule("obj?.prop");

        assert!(diagnostics.is_empty());
    }
}
