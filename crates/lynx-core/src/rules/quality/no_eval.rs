//! no-eval rule (Q034): Detects usage of eval() and similar dangerous patterns

use std::ops::ControlFlow;

use swc_ecma_ast::{CallExpr, Callee, Expr, Lit, NewExpr};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoEval,
    id = "Q034",
    name = "no-eval",
    description = "Disallow eval() and similar dangerous patterns",
    category = Quality,
    severity = Warning
);

impl Rule for NoEval {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoEvalVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct NoEvalVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
}

const DANGEROUS_TIMER_FUNCTIONS: [&str; 2] = ["setTimeout", "setInterval"];

impl NoEvalVisitor {
    fn is_string_argument(arg: &Expr) -> bool {
        matches!(arg, Expr::Lit(Lit::Str(_)) | Expr::Tpl(_))
    }

    fn report(
        &mut self,
        message: String,
        suggestion: &str,
        span: swc_common::Span,
        ctx: &VisitorContext,
    ) {
        let (line, column) = ctx.span_to_location(span);
        let diagnostic = Diagnostic::new(
            "Q034",
            Severity::Warning,
            message,
            &self.file_path,
            line,
            column,
        )
        .with_suggestion(suggestion);

        self.diagnostics.push(diagnostic);
    }
}

impl AstVisitor for NoEvalVisitor {
    fn visit_call_expr(&mut self, node: &CallExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        if let Callee::Expr(callee_expr) = &node.callee {
            if let Expr::Ident(ident) = callee_expr.as_ref() {
                let name = ident.sym.as_ref();

                if name == "eval" {
                    self.report(
                        "eval() is a security risk".to_string(),
                        "Avoid using eval(). Consider JSON.parse() for JSON data or other safer alternatives",
                        node.span,
                        ctx,
                    );
                } else if DANGEROUS_TIMER_FUNCTIONS.contains(&name) {
                    if let Some(first_arg) = node.args.first() {
                        if Self::is_string_argument(&first_arg.expr) {
                            self.report(
                                format!("{}() with string argument is a security risk", name),
                                &format!("Pass a function to {}() instead of a string", name),
                                node.span,
                                ctx,
                            );
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_new_expr(&mut self, node: &NewExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        if let Expr::Ident(ident) = node.callee.as_ref() {
            if ident.sym.as_ref() == "Function" {
                self.report(
                    "new Function() is a security risk".to_string(),
                    "Avoid using new Function(). Consider using regular function declarations",
                    node.span,
                    ctx,
                );
            }
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_no_eval(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoEval::new();
        rule.check(&file)
    }

    #[test]
    fn detects_eval() {
        let diagnostics = run_no_eval("eval(\"console.log('test')\");");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
        assert_eq!(diagnostics[0].message, "eval() is a security risk");
        assert_eq!(diagnostics[0].line, 1);
    }

    #[test]
    fn detects_new_function() {
        let diagnostics = run_no_eval("new Function(\"return 1\");");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
        assert_eq!(diagnostics[0].message, "new Function() is a security risk");
    }

    #[test]
    fn detects_settimeout_string() {
        let diagnostics = run_no_eval("setTimeout(\"alert(1)\", 100);");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
        assert_eq!(
            diagnostics[0].message,
            "setTimeout() with string argument is a security risk"
        );
    }

    #[test]
    fn detects_setinterval_string() {
        let diagnostics = run_no_eval("setInterval(\"alert(1)\", 100);");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
        assert_eq!(
            diagnostics[0].message,
            "setInterval() with string argument is a security risk"
        );
    }

    #[test]
    fn ignores_settimeout_function() {
        let code = r#"
setTimeout(function() { console.log("test"); }, 100);
setTimeout(() => console.log("test"), 100);
"#;
        let diagnostics = run_no_eval(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_setinterval_function() {
        let code = r#"
setInterval(function() { console.log("test"); }, 100);
setInterval(() => console.log("test"), 100);
"#;
        let diagnostics = run_no_eval(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_eval_in_nested_scope() {
        let code = r#"
function test() {
    eval("code");
}
"#;
        let diagnostics = run_no_eval(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
    }

    #[test]
    fn detects_multiple_violations() {
        let code = r#"
eval("code1");
eval("code2");
new Function("return 1");
setTimeout("alert(1)", 100);
"#;
        let diagnostics = run_no_eval(code);

        assert_eq!(diagnostics.len(), 4);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoEval::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q034");
        assert_eq!(metadata.name, "no-eval");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn suggestion_provided() {
        let diagnostics = run_no_eval("eval(\"test\");");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("Avoid using eval()")
        );
    }

    #[test]
    fn detects_settimeout_template_literal() {
        let diagnostics = run_no_eval("setTimeout(`alert(1)`, 100);");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q034");
    }
}
