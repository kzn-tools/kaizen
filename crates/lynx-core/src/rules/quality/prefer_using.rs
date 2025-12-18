//! prefer-using rule (Q020): Detect disposable resources not declared with using/await using
//!
//! This rule detects variables initialized with functions that return disposable resources
//! (like FileHandle from fs/promises.open) that should be declared with `using` or `await using`
//! for proper automatic resource cleanup.

use swc_ecma_ast::{
    AwaitExpr, CallExpr, Callee, Decl, Expr, MemberExpr, MemberProp, ModuleItem, Pat, ReturnStmt,
    Stmt, VarDecl, VarDeclKind,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::semantic::types::DisposableTypesRegistry;
use crate::visitor::VisitorContext;

declare_rule!(
    PreferUsing,
    id = "Q020",
    name = "prefer-using",
    description = "Require using/await using for disposable resources",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nconst file = await open('./data.txt');\n\n// Good\nawait using file = await open('./data.txt');"
);

impl Rule for PreferUsing {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let registry = DisposableTypesRegistry::with_defaults();

        let returned_vars = collect_returned_vars(&module.body);

        let mut visitor = PreferUsingVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            registry,
            returned_vars,
            ctx: &ctx,
        };

        for item in &module.body {
            visitor.check_module_item(item);
        }

        visitor.diagnostics
    }
}

fn collect_returned_vars(items: &[ModuleItem]) -> Vec<String> {
    let mut vars = Vec::new();
    for item in items {
        if let ModuleItem::Stmt(stmt) = item {
            collect_returned_vars_from_stmt(stmt, &mut vars);
        }
    }
    vars
}

fn collect_returned_vars_from_stmt(stmt: &Stmt, vars: &mut Vec<String>) {
    match stmt {
        Stmt::Return(ReturnStmt { arg: Some(arg), .. }) => {
            if let Expr::Ident(ident) = arg.as_ref() {
                vars.push(ident.sym.to_string());
            }
        }
        Stmt::Block(block) => {
            for s in &block.stmts {
                collect_returned_vars_from_stmt(s, vars);
            }
        }
        Stmt::If(if_stmt) => {
            collect_returned_vars_from_stmt(&if_stmt.cons, vars);
            if let Some(alt) = &if_stmt.alt {
                collect_returned_vars_from_stmt(alt, vars);
            }
        }
        Stmt::Decl(Decl::Fn(fn_decl)) => {
            if let Some(body) = &fn_decl.function.body {
                for s in &body.stmts {
                    collect_returned_vars_from_stmt(s, vars);
                }
            }
        }
        Stmt::Try(try_stmt) => {
            for s in &try_stmt.block.stmts {
                collect_returned_vars_from_stmt(s, vars);
            }
            if let Some(handler) = &try_stmt.handler {
                for s in &handler.body.stmts {
                    collect_returned_vars_from_stmt(s, vars);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for s in &finalizer.stmts {
                    collect_returned_vars_from_stmt(s, vars);
                }
            }
        }
        _ => {}
    }
}

struct PreferUsingVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    registry: DisposableTypesRegistry,
    returned_vars: Vec<String>,
    ctx: &'a VisitorContext<'a>,
}

impl<'a> PreferUsingVisitor<'a> {
    fn check_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::Stmt(stmt) => self.check_stmt(stmt),
            ModuleItem::ModuleDecl(_) => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(Decl::Var(var_decl)) => {
                self.check_var_decl(var_decl);
            }
            Stmt::Decl(Decl::Using(_)) => {}
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

    fn check_var_decl(&mut self, var_decl: &VarDecl) {
        if var_decl.kind == VarDeclKind::Var {
            return;
        }

        for declarator in &var_decl.decls {
            let var_name = match &declarator.name {
                Pat::Ident(ident) => ident.sym.to_string(),
                _ => continue,
            };

            if self.returned_vars.contains(&var_name) {
                continue;
            }

            let Some(init) = &declarator.init else {
                continue;
            };

            let (call_expr, is_awaited) = match init.as_ref() {
                Expr::Await(AwaitExpr { arg, .. }) => {
                    if let Expr::Call(call) = arg.as_ref() {
                        (call, true)
                    } else {
                        continue;
                    }
                }
                Expr::Call(call) => (call, false),
                _ => continue,
            };

            if let Some((confidence, type_name, is_async)) =
                self.analyze_call_for_disposable(call_expr)
            {
                let severity = match confidence {
                    Confidence::High => Severity::Warning,
                    Confidence::Medium => Severity::Info,
                };

                let using_keyword = if is_async || is_awaited {
                    "await using"
                } else {
                    "using"
                };

                let message = format!(
                    "Variable '{}' holds a disposable resource ({}) and should be declared with '{}'",
                    var_name, type_name, using_keyword
                );

                let (line, column) = self.ctx.span_to_location(declarator.span);

                let diagnostic =
                    Diagnostic::new("Q020", severity, message, &self.file_path, line, column)
                        .with_suggestion(format!(
                            "Replace 'const {}' or 'let {}' with '{} {}'",
                            var_name, var_name, using_keyword, var_name
                        ));

                self.diagnostics.push(diagnostic);
            }
        }
    }

    fn analyze_call_for_disposable(&self, call: &CallExpr) -> Option<(Confidence, String, bool)> {
        let callee_info = self.extract_callee_info(&call.callee)?;

        if let Some(return_type) = self.registry.get_return_type(&callee_info.full_path) {
            let is_async = self.registry.is_async_disposable(return_type);
            return Some((Confidence::High, return_type.to_string(), is_async));
        }

        if self.registry.matches_heuristic_pattern(&callee_info.name) {
            return Some((Confidence::Medium, "possible disposable".to_string(), true));
        }

        None
    }

    fn extract_callee_info(&self, callee: &Callee) -> Option<CalleeInfo> {
        match callee {
            Callee::Expr(expr) => self.extract_callee_from_expr(expr),
            _ => None,
        }
    }

    fn extract_callee_from_expr(&self, expr: &Expr) -> Option<CalleeInfo> {
        match expr {
            Expr::Ident(ident) => {
                let name = ident.sym.to_string();
                Some(CalleeInfo {
                    name: name.clone(),
                    full_path: name,
                })
            }
            Expr::Member(member) => self.extract_member_path(member),
            _ => None,
        }
    }

    fn extract_member_path(&self, member: &MemberExpr) -> Option<CalleeInfo> {
        let prop_name = match &member.prop {
            MemberProp::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let obj_path = self.extract_object_path(&member.obj)?;

        let full_path = format!("{}.{}", obj_path, prop_name);

        Some(CalleeInfo {
            name: prop_name,
            full_path,
        })
    }

    fn extract_object_path(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => Some(ident.sym.to_string()),
            Expr::Member(member) => {
                let obj_path = self.extract_object_path(&member.obj)?;
                let prop_name = match &member.prop {
                    MemberProp::Ident(ident) => ident.sym.to_string(),
                    _ => return None,
                };
                Some(format!("{}.{}", obj_path, prop_name))
            }
            _ => None,
        }
    }
}

enum Confidence {
    High,
    Medium,
}

struct CalleeInfo {
    name: String,
    full_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_prefer_using(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = PreferUsing::new();
        rule.check(&file)
    }

    #[test]
    fn detects_open_without_using() {
        let code = "const file = await open('./data.txt');";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q020");
        assert!(diagnostics[0].message.contains("file"));
        assert!(diagnostics[0].message.contains("await using"));
    }

    #[test]
    fn detects_fs_promises_open() {
        let code = "const handle = await fsPromises.open('./file');";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q020");
        assert!(diagnostics[0].message.contains("FileHandle"));
    }

    #[test]
    fn ignores_using_declaration() {
        let code = "await using file = await open('./data.txt');";
        let diagnostics = run_prefer_using(code);

        assert!(
            diagnostics.is_empty(),
            "using declaration should not be flagged"
        );
    }

    #[test]
    fn ignores_sync_using_declaration() {
        let code = "using resource = getResource();";
        let diagnostics = run_prefer_using(code);

        assert!(
            diagnostics.is_empty(),
            "sync using declaration should not be flagged"
        );
    }

    #[test]
    fn detects_heuristic_pattern_acquire() {
        let code = "const lock = await acquireLock();";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn detects_heuristic_pattern_connect() {
        let code = "const conn = await connectToDatabase();";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("conn"));
    }

    #[test]
    fn detects_heuristic_pattern_open_file() {
        let code = "const handle = openFileHandle('./path');";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_returned_resource() {
        let code = r#"
async function getFile() {
    const file = await open('./data.txt');
    return file;
}
"#;
        let diagnostics = run_prefer_using(code);

        assert!(
            diagnostics.is_empty(),
            "Returned resources should not be flagged"
        );
    }

    #[test]
    fn ignores_non_disposable_calls() {
        let code = r#"
const data = await fetch('https://api.example.com');
const result = await getData();
const value = computeValue();
"#;
        let diagnostics = run_prefer_using(code);

        assert!(
            diagnostics.is_empty(),
            "Non-disposable calls should not be flagged"
        );
    }

    #[test]
    fn ignores_var_declarations() {
        let code = "var file = await open('./data.txt');";
        let diagnostics = run_prefer_using(code);

        assert!(
            diagnostics.is_empty(),
            "var declarations should not be flagged (legacy code)"
        );
    }

    #[test]
    fn detects_in_nested_block() {
        let code = r#"
if (condition) {
    const file = await open('./data.txt');
}
"#;
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_in_try_block() {
        let code = r#"
try {
    const file = await open('./data.txt');
} catch (e) {}
"#;
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn high_confidence_for_known_types() {
        let code = "const file = await open('./data.txt');";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn medium_confidence_for_heuristic() {
        let code = "const conn = await createPoolConnection();";
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn suggestion_includes_using_keyword() {
        let code = "const file = await open('./data.txt');";
        let diagnostics = run_prefer_using(code);

        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("await using")
        );
    }

    #[test]
    fn detects_multiple_violations() {
        let code = r#"
const file1 = await open('./a.txt');
const file2 = await open('./b.txt');
"#;
        let diagnostics = run_prefer_using(code);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = PreferUsing::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q020");
        assert_eq!(metadata.name, "prefer-using");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }
}
