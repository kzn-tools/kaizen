//! prefer-nullish-coalescing rule (Q023): Suggest ?? instead of || when appropriate

use std::ops::ControlFlow;

use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Lit};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    PreferNullishCoalescing,
    id = "Q023",
    name = "prefer-nullish-coalescing",
    description = "Suggest nullish coalescing (??) instead of || for default values",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nvalue || 'default'\nconfig.timeout || 5000\n\n// Good\nvalue ?? 'default'\nconfig.timeout ?? 5000"
);

impl Rule for PreferNullishCoalescing {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = PreferNullishCoalescingVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct PreferNullishCoalescingVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
}

impl PreferNullishCoalescingVisitor {
    fn is_likely_default_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Lit(lit) => match lit {
                Lit::Str(_) | Lit::Num(_) | Lit::Bool(_) => true,
                Lit::Null(_) => false,
                _ => false,
            },
            Expr::Array(_) => true,
            Expr::Object(_) => true,
            _ => false,
        }
    }

    fn is_likely_boolean_context(&self, left: &Expr, right: &Expr) -> bool {
        if let (Expr::Ident(left_id), Expr::Ident(right_id)) = (left, right) {
            let left_name = left_id.sym.to_string().to_lowercase();
            let right_name = right_id.sym.to_string().to_lowercase();

            let boolean_prefixes = ["is", "has", "can", "should", "will", "did", "was"];
            let is_left_boolean = boolean_prefixes.iter().any(|p| left_name.starts_with(p));
            let is_right_boolean = boolean_prefixes.iter().any(|p| right_name.starts_with(p));

            if is_left_boolean && is_right_boolean {
                return true;
            }
        }

        if let Expr::Lit(Lit::Bool(_)) = left {
            return true;
        }

        if let Expr::Lit(Lit::Bool(_)) = right {
            return false;
        }

        false
    }

    fn get_expr_summary(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(ident) => ident.sym.to_string(),
            Expr::Member(member) => {
                let obj = self.get_expr_summary(&member.obj);
                match &member.prop {
                    swc_ecma_ast::MemberProp::Ident(ident) => {
                        format!("{}.{}", obj, ident.sym)
                    }
                    _ => format!("{}[...]", obj),
                }
            }
            Expr::Lit(lit) => match lit {
                Lit::Str(s) => format!("'{}'", s.value),
                Lit::Num(n) => n.value.to_string(),
                Lit::Bool(b) => b.value.to_string(),
                _ => "...".to_string(),
            },
            Expr::Array(_) => "[...]".to_string(),
            Expr::Object(_) => "{...}".to_string(),
            _ => "...".to_string(),
        }
    }
}

impl AstVisitor for PreferNullishCoalescingVisitor {
    fn visit_bin_expr(&mut self, node: &BinExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        if node.op != BinaryOp::LogicalOr {
            return ControlFlow::Continue(());
        }

        let left = node.left.as_ref();
        let right = node.right.as_ref();

        if self.is_likely_boolean_context(left, right) {
            return ControlFlow::Continue(());
        }

        if !self.is_likely_default_value(right) {
            return ControlFlow::Continue(());
        }

        let (line, column) = ctx.span_to_location(node.span);
        let left_str = self.get_expr_summary(left);
        let right_str = self.get_expr_summary(right);

        let diagnostic = Diagnostic::new(
            "Q023",
            Severity::Warning,
            format!(
                "Prefer nullish coalescing (??) over || for default values: '{} || {}'",
                left_str, right_str
            ),
            &self.file_path,
            line,
            column,
        )
        .with_suggestion(format!(
            "Replace with '{} ?? {}' to only fall through on null/undefined",
            left_str, right_str
        ));

        self.diagnostics.push(diagnostic);

        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = PreferNullishCoalescing::new();
        rule.check(&file)
    }

    #[test]
    fn detects_string_literal_default() {
        let diagnostics = run_rule("const x = value || 'default';");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q023");
        assert!(diagnostics[0].message.contains("??"));
    }

    #[test]
    fn detects_number_literal_default() {
        let diagnostics = run_rule("const x = value || 0;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q023");
    }

    #[test]
    fn detects_array_literal_default() {
        let diagnostics = run_rule("const x = items || [];");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q023");
    }

    #[test]
    fn detects_object_literal_default() {
        let diagnostics = run_rule("const x = config || {};");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q023");
    }

    #[test]
    fn detects_member_expression_with_default() {
        let diagnostics = run_rule("const timeout = config.timeout || 5000;");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("config.timeout"));
    }

    #[test]
    fn suggests_nullish_coalescing() {
        let diagnostics = run_rule("const x = value || 'default';");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].suggestion.as_ref().unwrap().contains("??"));
    }

    #[test]
    fn ignores_boolean_identifiers() {
        let diagnostics = run_rule("const x = isEnabled || hasPermission;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_nullish_coalescing() {
        let diagnostics = run_rule("const x = value ?? 'default';");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_identifier_on_right() {
        let diagnostics = run_rule("const x = value || other;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_function_call_on_right() {
        let diagnostics = run_rule("const x = value || getValue();");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_in_function_body() {
        let code = r#"
function test(value) {
    return value || 'default';
}
"#;
        let diagnostics = run_rule(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_arrow_function() {
        let diagnostics = run_rule("const fn = (x) => x || 0;");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_ternary() {
        let diagnostics = run_rule("const x = cond ? (value || 'a') : 'b';");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn handles_multiple_patterns() {
        let code = r#"
const a = x || 'default';
const b = y || 0;
const c = z || [];
"#;
        let diagnostics = run_rule(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = PreferNullishCoalescing::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q023");
        assert_eq!(metadata.name, "prefer-nullish-coalescing");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn ignores_boolean_literal_on_left() {
        let diagnostics = run_rule("const x = false || 'fallback';");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_boolean_literal_default_on_right() {
        let diagnostics = run_rule("const x = value || false;");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_logical_and() {
        let diagnostics = run_rule("const x = value && 'result';");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_nested_member_with_default() {
        let diagnostics = run_rule("const x = obj.a.b.c || 'default';");

        assert_eq!(diagnostics.len(), 1);
    }
}
