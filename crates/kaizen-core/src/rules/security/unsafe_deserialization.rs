//! no-unsafe-deserialization rule (S022): Detects dangerous deserialization patterns

use std::ops::ControlFlow;

use swc_ecma_ast::{
    ArrowExpr, BlockStmtOrExpr, CallExpr, Callee, Expr, ExprOrSpread, Ident, MemberProp, NewExpr,
    Stmt,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    UnsafeDeserialization,
    id = "S022",
    name = "no-unsafe-deserialization",
    description = "Disallow dangerous deserialization patterns that may lead to code execution",
    category = Security,
    severity = Error,
    min_tier = Pro,
    examples = "// Bad\nJSON.parse(data, (k, v) => eval(v));\neval(JSON.parse(input));\n\n// Good\nJSON.parse(data);\nJSON.parse(data, (k, v) => sanitize(v));"
);

impl Rule for UnsafeDeserialization {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = UnsafeDeserializationVisitor {
            diagnostics: Vec::new(),
            file_path: &file.metadata().filename,
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct UnsafeDeserializationVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: &'a str,
    ctx: &'a VisitorContext<'a>,
}

impl UnsafeDeserializationVisitor<'_> {
    fn is_json_parse_call(call: &CallExpr) -> bool {
        let Callee::Expr(callee_expr) = &call.callee else {
            return false;
        };

        let Expr::Member(member) = callee_expr.as_ref() else {
            return false;
        };

        let Expr::Ident(obj) = member.obj.as_ref() else {
            return false;
        };

        if obj.sym.as_ref() != "JSON" {
            return false;
        }

        let MemberProp::Ident(prop) = &member.prop else {
            return false;
        };

        prop.sym.as_ref() == "parse"
    }

    fn get_dangerous_call_name(ident: &Ident) -> Option<&'static str> {
        match ident.sym.as_ref() {
            "eval" => Some("eval"),
            "Function" => Some("Function"),
            "setTimeout" => Some("setTimeout"),
            "setInterval" => Some("setInterval"),
            _ => None,
        }
    }

    fn is_dangerous_call(ident: &Ident) -> bool {
        Self::get_dangerous_call_name(ident).is_some()
    }

    fn is_dangerous_new_expr(new_expr: &NewExpr) -> bool {
        if let Expr::Ident(ident) = new_expr.callee.as_ref() {
            return ident.sym.as_ref() == "Function";
        }
        false
    }

    fn check_expr_for_dangerous_patterns(&self, expr: &Expr) -> Option<&'static str> {
        match expr {
            Expr::Call(call) => {
                if let Callee::Expr(callee_expr) = &call.callee {
                    match callee_expr.as_ref() {
                        Expr::Ident(ident) => {
                            return Self::get_dangerous_call_name(ident);
                        }
                        Expr::New(new_expr) => {
                            if Self::is_dangerous_new_expr(new_expr) {
                                return Some("new Function");
                            }
                        }
                        Expr::Call(inner_call) => {
                            if let Callee::Expr(inner_callee) = &inner_call.callee {
                                if let Expr::Ident(ident) = inner_callee.as_ref() {
                                    if ident.sym.as_ref() == "Function" {
                                        return Some("Function");
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                None
            }
            Expr::New(new_expr) => {
                if Self::is_dangerous_new_expr(new_expr) {
                    Some("new Function")
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_stmt_for_dangerous_patterns(&self, stmt: &Stmt) -> Option<&'static str> {
        match stmt {
            Stmt::Expr(expr_stmt) => self.check_expr_for_dangerous_patterns(&expr_stmt.expr),
            Stmt::Return(ret_stmt) => {
                if let Some(arg) = &ret_stmt.arg {
                    self.check_expr_for_dangerous_patterns(arg)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_reviver_body_block(&self, stmts: &[Stmt]) -> Option<&'static str> {
        for stmt in stmts {
            if let Some(pattern) = self.check_stmt_for_dangerous_patterns(stmt) {
                return Some(pattern);
            }
        }
        None
    }

    fn check_reviver_function(&self, reviver: &Expr) -> Option<&'static str> {
        match reviver {
            Expr::Arrow(arrow) => self.check_arrow_reviver(arrow),
            Expr::Fn(fn_expr) => {
                if let Some(body) = &fn_expr.function.body {
                    self.check_reviver_body_block(&body.stmts)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_arrow_reviver(&self, arrow: &ArrowExpr) -> Option<&'static str> {
        match &*arrow.body {
            BlockStmtOrExpr::Expr(expr) => self.check_expr_for_dangerous_patterns(expr),
            BlockStmtOrExpr::BlockStmt(block) => self.check_reviver_body_block(&block.stmts),
        }
    }

    fn check_json_parse_with_reviver(&mut self, call: &CallExpr) {
        if call.args.len() < 2 {
            return;
        }

        let reviver_arg = &call.args[1];
        if let Some(dangerous_pattern) = self.check_reviver_function(&reviver_arg.expr) {
            let (line, column) = self.ctx.span_to_location(call.span);
            let diagnostic = Diagnostic::new(
                "S022",
                Severity::Error,
                format!(
                    "Unsafe deserialization: JSON.parse reviver function contains dangerous '{}' call",
                    dangerous_pattern
                ),
                self.file_path,
                line,
                column,
            )
            .with_suggestion(
                "Remove code execution from reviver function. Use safe data transformation instead.",
            );
            self.diagnostics.push(diagnostic);
        }
    }

    fn check_json_parse_in_dangerous_context(&mut self, call: &CallExpr, context: &str) {
        for arg in &call.args {
            if let Expr::Call(inner_call) = arg.expr.as_ref() {
                if Self::is_json_parse_call(inner_call) {
                    let (line, column) = self.ctx.span_to_location(call.span);
                    let diagnostic = Diagnostic::new(
                        "S022",
                        Severity::Error,
                        format!(
                            "Unsafe deserialization: JSON.parse result passed directly to {}",
                            context
                        ),
                        self.file_path,
                        line,
                        column,
                    )
                    .with_suggestion(
                        "Do not pass deserialized data directly to code execution functions.",
                    );
                    self.diagnostics.push(diagnostic);
                    return;
                }
            }
        }
    }

    fn get_first_arg_as_inner_call(args: &[ExprOrSpread]) -> Option<&CallExpr> {
        args.first().and_then(|arg| {
            if let Expr::Call(inner) = arg.expr.as_ref() {
                Some(inner)
            } else {
                None
            }
        })
    }
}

impl AstVisitor for UnsafeDeserializationVisitor<'_> {
    fn visit_call_expr(&mut self, node: &CallExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        if Self::is_json_parse_call(node) {
            self.check_json_parse_with_reviver(node);
        }

        if let Callee::Expr(callee_expr) = &node.callee {
            if let Expr::Ident(ident) = callee_expr.as_ref() {
                if Self::is_dangerous_call(ident) {
                    self.check_json_parse_in_dangerous_context(node, ident.sym.as_ref());
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_new_expr(&mut self, node: &NewExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        if Self::is_dangerous_new_expr(node) {
            if let Some(args) = &node.args {
                if let Some(inner_call) = Self::get_first_arg_as_inner_call(args) {
                    if Self::is_json_parse_call(inner_call) {
                        let (line, column) = self.ctx.span_to_location(node.span);
                        let diagnostic = Diagnostic::new(
                            "S022",
                            Severity::Error,
                            "Unsafe deserialization: JSON.parse result passed directly to new Function".to_string(),
                            self.file_path,
                            line,
                            column,
                        )
                        .with_suggestion(
                            "Do not pass deserialized data directly to code execution functions.",
                        );
                        self.diagnostics.push(diagnostic);
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_unsafe_deserialization(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = UnsafeDeserialization::new();
        rule.check(&file)
    }

    #[test]
    fn detects_eval_in_reviver() {
        let code = r#"JSON.parse(data, (key, value) => eval(value));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(!diagnostics.is_empty(), "should detect eval in reviver");
        assert_eq!(diagnostics[0].rule_id, "S022");
        assert!(diagnostics[0].message.contains("eval"));
    }

    #[test]
    fn detects_function_constructor_in_reviver() {
        let code = r#"JSON.parse(data, (key, value) => new Function(value)());"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect Function constructor in reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
        assert!(diagnostics[0].message.contains("new Function"));
    }

    #[test]
    fn detects_function_call_in_reviver() {
        let code = r#"JSON.parse(data, (key, value) => Function(value)());"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect Function call in reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn detects_settimeout_in_reviver() {
        let code = r#"JSON.parse(data, (key, value) => setTimeout(value, 0));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect setTimeout in reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
        assert!(diagnostics[0].message.contains("setTimeout"));
    }

    #[test]
    fn detects_setinterval_in_reviver() {
        let code = r#"JSON.parse(data, (key, value) => setInterval(value, 1000));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect setInterval in reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn detects_eval_in_block_reviver() {
        let code = r#"
            JSON.parse(data, (key, value) => {
                return eval(value);
            });
        "#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect eval in block reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn detects_eval_in_function_reviver() {
        let code = r#"
            JSON.parse(data, function(key, value) {
                return eval(value);
            });
        "#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect eval in function reviver"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn detects_json_parse_in_eval() {
        let code = r#"eval(JSON.parse(userInput));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(!diagnostics.is_empty(), "should detect JSON.parse in eval");
        assert_eq!(diagnostics[0].rule_id, "S022");
        assert!(diagnostics[0].message.contains("eval"));
    }

    #[test]
    fn detects_json_parse_in_function_constructor() {
        let code = r#"new Function(JSON.parse(userInput))();"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect JSON.parse in Function constructor"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn detects_json_parse_in_settimeout() {
        let code = r#"setTimeout(JSON.parse(input), 100);"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect JSON.parse in setTimeout"
        );
        assert_eq!(diagnostics[0].rule_id, "S022");
    }

    #[test]
    fn no_false_positive_for_safe_json_parse() {
        let code = r#"const data = JSON.parse(input);"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(diagnostics.is_empty(), "should not flag safe JSON.parse");
    }

    #[test]
    fn no_false_positive_for_safe_reviver() {
        let code = r#"
            JSON.parse(data, (key, value) => {
                return value.trim();
            });
        "#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(diagnostics.is_empty(), "should not flag safe reviver");
    }

    #[test]
    fn no_false_positive_for_type_reviver() {
        let code = r#"
            JSON.parse(data, (key, value) => {
                if (typeof value === 'string' && value.match(/^\d{4}-\d{2}-\d{2}$/)) {
                    return new Date(value);
                }
                return value;
            });
        "#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(diagnostics.is_empty(), "should not flag date reviver");
    }

    #[test]
    fn no_false_positive_for_console_log() {
        let code = r#"console.log(JSON.parse(input));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag console.log with JSON.parse"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"JSON.parse(data, (key, value) => eval(value));"#;
        let diagnostics = run_unsafe_deserialization(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
    }

    #[test]
    fn metadata_is_correct() {
        let rule = UnsafeDeserialization::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S022");
        assert_eq!(metadata.name, "no-unsafe-deserialization");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
        assert_eq!(metadata.min_tier, crate::licensing::PremiumTier::Pro);
    }
}
