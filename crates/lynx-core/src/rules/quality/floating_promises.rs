//! no-floating-promises rule (Q021): Detect Promises that are not awaited, caught, or returned
//!
//! This rule detects Promise-returning function calls that are not properly handled,
//! which can lead to unhandled rejections and silent failures.

use swc_ecma_ast::{
    CallExpr, Callee, Decl, Expr, ExprStmt, MemberExpr, MemberProp, ModuleItem, Stmt, UnaryOp,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::VisitorContext;

declare_rule!(
    FloatingPromises,
    id = "Q021",
    name = "no-floating-promises",
    description = "Require Promises to be awaited, caught, or explicitly voided",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nfetchData();\npromise.then(handler);\n\n// Good\nawait fetchData();\nfetchData().catch(handleError);\nvoid fetchData();"
);

impl Rule for FloatingPromises {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);

        let mut visitor = FloatingPromisesVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        for item in &module.body {
            visitor.check_module_item(item);
        }

        visitor.diagnostics
    }
}

struct FloatingPromisesVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl<'a> FloatingPromisesVisitor<'a> {
    fn check_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::Stmt(stmt) => self.check_stmt(stmt),
            ModuleItem::ModuleDecl(_) => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.check_expr_stmt(expr_stmt);
            }
            Stmt::Block(block) => {
                for s in &block.stmts {
                    self.check_stmt(s);
                }
            }
            Stmt::If(if_stmt) => {
                self.check_stmt(&if_stmt.cons);
                if let Some(alt) = &if_stmt.alt {
                    self.check_stmt(alt);
                }
            }
            Stmt::For(for_stmt) => {
                self.check_stmt(&for_stmt.body);
            }
            Stmt::ForIn(for_in) => {
                self.check_stmt(&for_in.body);
            }
            Stmt::ForOf(for_of) => {
                self.check_stmt(&for_of.body);
            }
            Stmt::While(while_stmt) => {
                self.check_stmt(&while_stmt.body);
            }
            Stmt::DoWhile(do_while) => {
                self.check_stmt(&do_while.body);
            }
            Stmt::Try(try_stmt) => {
                for s in &try_stmt.block.stmts {
                    self.check_stmt(s);
                }
                if let Some(handler) = &try_stmt.handler {
                    for s in &handler.body.stmts {
                        self.check_stmt(s);
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for s in &finalizer.stmts {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::Switch(switch_stmt) => {
                for case in &switch_stmt.cases {
                    for s in &case.cons {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::With(with_stmt) => {
                self.check_stmt(&with_stmt.body);
            }
            Stmt::Labeled(labeled) => {
                self.check_stmt(&labeled.body);
            }
            Stmt::Decl(Decl::Fn(fn_decl)) => {
                if let Some(body) = &fn_decl.function.body {
                    for s in &body.stmts {
                        self.check_stmt(s);
                    }
                }
            }
            _ => {}
        }
    }

    fn check_expr_stmt(&mut self, expr_stmt: &ExprStmt) {
        let expr = &*expr_stmt.expr;

        // Check if this is a void expression (intentionally ignored)
        if let Expr::Unary(unary) = expr {
            if unary.op == UnaryOp::Void {
                return;
            }
        }

        // Check if this is an await expression (properly handled)
        if matches!(expr, Expr::Await(_)) {
            return;
        }

        // Check if expression is a potentially floating promise
        if let Some(call_info) = self.extract_floating_promise_info(expr) {
            let (line, column) = self.ctx.span_to_location(expr_stmt.span);

            let message = format!(
                "Floating Promise: '{}' returns a Promise that is not awaited or caught",
                call_info.name
            );

            let diagnostic = Diagnostic::new(
                "Q021",
                Severity::Warning,
                message,
                &self.file_path,
                line,
                column,
            )
            .with_suggestion(
                "Add 'await' before the call, handle with '.catch()', or use 'void' if intentional",
            );

            self.diagnostics.push(diagnostic);
        }
    }

    fn extract_floating_promise_info(&self, expr: &Expr) -> Option<CallInfo> {
        match expr {
            Expr::Call(call) => self.analyze_call_expr(call),
            _ => None,
        }
    }

    fn analyze_call_expr(&self, call: &CallExpr) -> Option<CallInfo> {
        // Check if this is a .catch() or .finally() call (properly handled)
        if self.is_promise_error_handler(call) {
            return None;
        }

        // Extract the function name being called
        let call_name = self.extract_call_name(&call.callee)?;

        // Check if this looks like an async function call
        if self.looks_like_async_call(&call_name, call) {
            return Some(CallInfo { name: call_name });
        }

        None
    }

    fn is_promise_error_handler(&self, call: &CallExpr) -> bool {
        if let Callee::Expr(callee_expr) = &call.callee {
            if let Expr::Member(member) = callee_expr.as_ref() {
                if let MemberProp::Ident(ident) = &member.prop {
                    let method_name = ident.sym.as_ref();
                    // .catch() and .finally() are valid error handlers
                    if method_name == "catch" || method_name == "finally" {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn extract_call_name(&self, callee: &Callee) -> Option<String> {
        match callee {
            Callee::Expr(expr) => self.extract_name_from_expr(expr),
            _ => None,
        }
    }

    fn extract_name_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => Some(ident.sym.to_string()),
            Expr::Member(member) => self.extract_member_chain(member),
            _ => None,
        }
    }

    fn extract_member_chain(&self, member: &MemberExpr) -> Option<String> {
        let prop_name = match &member.prop {
            MemberProp::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let obj_name = self.extract_name_from_expr(&member.obj)?;
        Some(format!("{}.{}", obj_name, prop_name))
    }

    fn looks_like_async_call(&self, name: &str, _call: &CallExpr) -> bool {
        let lower_name = name.to_lowercase();

        // Check common async function prefixes
        let async_prefixes = [
            "fetch",
            "get",
            "post",
            "put",
            "delete",
            "patch",
            "request",
            "load",
            "save",
            "send",
            "create",
            "update",
            "remove",
            "read",
            "write",
            "connect",
            "disconnect",
            "subscribe",
            "publish",
            "upload",
            "download",
        ];

        for prefix in async_prefixes {
            if lower_name.starts_with(prefix) {
                return true;
            }
        }

        // Check async suffix
        if lower_name.ends_with("async") {
            return true;
        }

        // Check if it's a .then() call (promise chaining without catch)
        if name.ends_with(".then") {
            return true;
        }

        // Check specific async API patterns
        let async_patterns = [
            "promise.all",
            "promise.race",
            "promise.any",
            "promise.allsettled",
            "promise.resolve",
            "promise.reject",
        ];

        for pattern in async_patterns {
            if lower_name == pattern {
                return true;
            }
        }

        false
    }
}

struct CallInfo {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_floating_promises(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = FloatingPromises::new();
        rule.check(&file)
    }

    #[test]
    fn detects_floating_fetch() {
        let diagnostics = run_floating_promises("fetchData();");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q021");
        assert!(diagnostics[0].message.contains("fetchData"));
    }

    #[test]
    fn ignores_awaited_promise() {
        let diagnostics = run_floating_promises("await fetchData();");

        assert!(
            diagnostics.is_empty(),
            "Awaited promises should not be flagged"
        );
    }

    #[test]
    fn ignores_caught_promise() {
        let diagnostics = run_floating_promises("fetchData().catch(handleError);");

        assert!(
            diagnostics.is_empty(),
            "Promises with .catch() should not be flagged"
        );
    }

    #[test]
    fn ignores_void_promise() {
        let diagnostics = run_floating_promises("void fetchData();");

        assert!(
            diagnostics.is_empty(),
            "void expressions should not be flagged"
        );
    }

    #[test]
    fn detects_various_async_prefixes() {
        let test_cases = [
            ("getData();", "getData"),
            ("postUser();", "postUser"),
            ("putItem();", "putItem"),
            ("deleteRecord();", "deleteRecord"),
            ("loadConfig();", "loadConfig"),
            ("saveFile();", "saveFile"),
            ("sendMessage();", "sendMessage"),
            ("createUser();", "createUser"),
            ("updateProfile();", "updateProfile"),
        ];

        for (code, expected_name) in test_cases {
            let diagnostics = run_floating_promises(code);
            assert_eq!(
                diagnostics.len(),
                1,
                "Should detect floating promise in: {}",
                code
            );
            assert!(
                diagnostics[0].message.contains(expected_name),
                "Should mention function name {} in: {}",
                expected_name,
                code
            );
        }
    }

    #[test]
    fn detects_async_suffix() {
        let diagnostics = run_floating_promises("doSomethingAsync();");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("doSomethingAsync"));
    }

    #[test]
    fn detects_promise_then_without_catch() {
        let diagnostics = run_floating_promises("promise.then(handler);");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("promise.then"));
    }

    #[test]
    fn ignores_finally_handler() {
        let diagnostics = run_floating_promises("promise.finally(cleanup);");

        assert!(
            diagnostics.is_empty(),
            ".finally() should not be flagged as floating"
        );
    }

    #[test]
    fn detects_in_nested_block() {
        let code = r#"
if (condition) {
    fetchData();
}
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_try_block() {
        let code = r#"
try {
    fetchData();
} catch (e) {}
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_multiple_violations() {
        let code = r#"
fetchData();
loadConfig();
sendMessage();
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn ignores_non_async_functions() {
        let code = r#"
console.log("test");
Array.from(items);
Object.keys(obj);
"#;
        let diagnostics = run_floating_promises(code);

        assert!(
            diagnostics.is_empty(),
            "Non-async functions should not be flagged"
        );
    }

    #[test]
    fn detects_promise_all() {
        let diagnostics = run_floating_promises("Promise.all([p1, p2]);");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Promise.all"));
    }

    #[test]
    fn detects_promise_race() {
        let diagnostics = run_floating_promises("Promise.race([p1, p2]);");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn suggestion_is_provided() {
        let diagnostics = run_floating_promises("fetchData();");

        assert!(diagnostics[0].suggestion.is_some());
        let suggestion = diagnostics[0].suggestion.as_ref().unwrap();
        assert!(suggestion.contains("await"));
        assert!(suggestion.contains("catch"));
        assert!(suggestion.contains("void"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = FloatingPromises::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q021");
        assert_eq!(metadata.name, "no-floating-promises");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn detects_in_function_body() {
        let code = r#"
function test() {
    fetchData();
}
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_while_loop() {
        let code = r#"
while (true) {
    sendHeartbeat();
}
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_for_loop() {
        let code = r#"
for (let i = 0; i < 10; i++) {
    sendRequest();
}
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_connect_prefix() {
        let diagnostics = run_floating_promises("connectToDatabase();");

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_read_write_prefix() {
        let code = r#"
readFile();
writeFile();
"#;
        let diagnostics = run_floating_promises(code);

        assert_eq!(diagnostics.len(), 2);
    }
}
