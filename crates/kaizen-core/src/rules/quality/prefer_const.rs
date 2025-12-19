//! prefer-const rule (Q031): Require const declarations for variables never reassigned
//!
//! This rule detects `let` declarations where the variable is never reassigned
//! and suggests using `const` instead.

use std::collections::HashSet;

use swc_common::Span;
use swc_ecma_ast::{
    AssignExpr, AssignTarget, Decl, Expr, ForHead, Module, ModuleItem, Pat, SimpleAssignTarget,
    Stmt, UpdateExpr, VarDecl, VarDeclKind,
};

use crate::declare_rule;
use crate::diagnostic::{Diagnostic, Fix};
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::VisitorContext;

declare_rule!(
    PreferConst,
    id = "Q031",
    name = "prefer-const",
    description = "Require const declarations for variables that are never reassigned",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nlet x = 1;\nconsole.log(x);\n\n// Good\nconst x = 1;\nconsole.log(x);"
);

impl Rule for PreferConst {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut analyzer = PreferConstAnalyzer::new(&ctx, file.metadata().filename.clone());
        analyzer.analyze(module);
        analyzer.diagnostics
    }
}

struct LetDeclaration {
    span: Span,
    decl_span: Span,
    scope_depth: usize,
}

struct PreferConstAnalyzer<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
    let_declarations: Vec<(String, LetDeclaration)>,
    reassigned: HashSet<(String, usize)>,
    current_scope_depth: usize,
}

impl<'a> PreferConstAnalyzer<'a> {
    fn new(ctx: &'a VisitorContext<'a>, file_path: String) -> Self {
        Self {
            diagnostics: Vec::new(),
            file_path,
            ctx,
            let_declarations: Vec::new(),
            reassigned: HashSet::new(),
            current_scope_depth: 0,
        }
    }

    fn analyze(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }

        self.finalize_scope(0);
    }

    fn finalize_scope(&mut self, scope_depth: usize) {
        let to_check: Vec<_> = self
            .let_declarations
            .iter()
            .filter(|(_, decl)| decl.scope_depth == scope_depth)
            .map(|(name, decl)| (name.clone(), decl.span, decl.decl_span))
            .collect();

        for (name, span, decl_span) in to_check {
            if !self.reassigned.contains(&(name.clone(), scope_depth)) {
                let (line, column) = self.ctx.span_to_location(span);
                let (decl_line, decl_column) = self.ctx.span_to_location(decl_span);

                let fix = Fix::replace(
                    "Replace 'let' with 'const'",
                    "const",
                    decl_line,
                    decl_column,
                    decl_line,
                    decl_column + 2,
                );

                let diagnostic = Diagnostic::new(
                    "Q031",
                    Severity::Warning,
                    format!("'{}' is never reassigned. Use 'const' instead", name),
                    &self.file_path,
                    line,
                    column,
                )
                .with_end(line, column + name.len())
                .with_suggestion(format!("Replace 'let {}' with 'const {}'", name, name))
                .with_fix(fix);

                self.diagnostics.push(diagnostic);
            }
        }

        self.let_declarations
            .retain(|(_, decl)| decl.scope_depth != scope_depth);
    }

    fn enter_scope(&mut self) {
        self.current_scope_depth += 1;
    }

    fn exit_scope(&mut self) {
        self.finalize_scope(self.current_scope_depth);
        self.current_scope_depth = self.current_scope_depth.saturating_sub(1);
    }

    fn find_declaration_scope(&self, name: &str) -> Option<usize> {
        for (decl_name, decl) in self.let_declarations.iter().rev() {
            if decl_name == name && decl.scope_depth <= self.current_scope_depth {
                return Some(decl.scope_depth);
            }
        }
        None
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
            ModuleItem::ModuleDecl(_) => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => {
                self.enter_scope();
                for s in &block.stmts {
                    self.visit_stmt(s);
                }
                self.exit_scope();
            }
            Stmt::Decl(Decl::Var(var_decl)) => {
                self.visit_var_decl(var_decl);
            }
            Stmt::Decl(Decl::Fn(fn_decl)) => {
                if let Some(body) = &fn_decl.function.body {
                    self.enter_scope();
                    for param in &fn_decl.function.params {
                        self.mark_pattern_as_reassignable(&param.pat);
                    }
                    for s in &body.stmts {
                        self.visit_stmt(s);
                    }
                    self.exit_scope();
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.visit_expr(&expr_stmt.expr);
            }
            Stmt::If(if_stmt) => {
                self.visit_expr(&if_stmt.test);
                self.visit_stmt(&if_stmt.cons);
                if let Some(alt) = &if_stmt.alt {
                    self.visit_stmt(alt);
                }
            }
            Stmt::For(for_stmt) => {
                self.enter_scope();
                if let Some(init) = &for_stmt.init {
                    match init {
                        swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
                            self.visit_var_decl(var_decl);
                        }
                        swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
                            self.visit_expr(expr);
                        }
                    }
                }
                if let Some(test) = &for_stmt.test {
                    self.visit_expr(test);
                }
                if let Some(update) = &for_stmt.update {
                    self.visit_expr(update);
                }
                self.visit_stmt(&for_stmt.body);
                self.exit_scope();
            }
            Stmt::ForIn(for_in) => {
                self.enter_scope();
                self.mark_for_head_reassignable(&for_in.left);
                self.visit_expr(&for_in.right);
                self.visit_stmt(&for_in.body);
                self.exit_scope();
            }
            Stmt::ForOf(for_of) => {
                self.enter_scope();
                self.mark_for_head_reassignable(&for_of.left);
                self.visit_expr(&for_of.right);
                self.visit_stmt(&for_of.body);
                self.exit_scope();
            }
            Stmt::While(while_stmt) => {
                self.visit_expr(&while_stmt.test);
                self.visit_stmt(&while_stmt.body);
            }
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body);
                self.visit_expr(&do_while.test);
            }
            Stmt::Switch(switch_stmt) => {
                self.visit_expr(&switch_stmt.discriminant);
                for case in &switch_stmt.cases {
                    if let Some(test) = &case.test {
                        self.visit_expr(test);
                    }
                    for s in &case.cons {
                        self.visit_stmt(s);
                    }
                }
            }
            Stmt::Try(try_stmt) => {
                self.enter_scope();
                for s in &try_stmt.block.stmts {
                    self.visit_stmt(s);
                }
                self.exit_scope();

                if let Some(handler) = &try_stmt.handler {
                    self.enter_scope();
                    if let Some(param) = &handler.param {
                        self.mark_pattern_as_reassignable(param);
                    }
                    for s in &handler.body.stmts {
                        self.visit_stmt(s);
                    }
                    self.exit_scope();
                }

                if let Some(finalizer) = &try_stmt.finalizer {
                    self.enter_scope();
                    for s in &finalizer.stmts {
                        self.visit_stmt(s);
                    }
                    self.exit_scope();
                }
            }
            Stmt::Return(ret) => {
                if let Some(arg) = &ret.arg {
                    self.visit_expr(arg);
                }
            }
            Stmt::Throw(throw) => {
                self.visit_expr(&throw.arg);
            }
            Stmt::With(with_stmt) => {
                self.visit_expr(&with_stmt.obj);
                self.visit_stmt(&with_stmt.body);
            }
            Stmt::Labeled(labeled) => {
                self.visit_stmt(&labeled.body);
            }
            _ => {}
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        if var_decl.kind != VarDeclKind::Let {
            return;
        }

        for declarator in &var_decl.decls {
            if let Pat::Ident(ident) = &declarator.name {
                if declarator.init.is_some() {
                    let name = ident.sym.to_string();
                    self.let_declarations.push((
                        name,
                        LetDeclaration {
                            span: ident.span,
                            decl_span: var_decl.span,
                            scope_depth: self.current_scope_depth,
                        },
                    ));
                }
            }

            if let Some(init) = &declarator.init {
                self.visit_expr(init);
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign(assign) => {
                self.handle_assignment(assign);
            }
            Expr::Update(update) => {
                self.handle_update(update);
            }
            Expr::Call(call) => {
                if let swc_ecma_ast::Callee::Expr(callee) = &call.callee {
                    self.visit_expr(callee);
                }
                for arg in &call.args {
                    self.visit_expr(&arg.expr);
                }
            }
            Expr::New(new_expr) => {
                self.visit_expr(&new_expr.callee);
                if let Some(args) = &new_expr.args {
                    for arg in args {
                        self.visit_expr(&arg.expr);
                    }
                }
            }
            Expr::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    self.visit_expr(&elem.expr);
                }
            }
            Expr::Object(obj) => {
                for prop in &obj.props {
                    match prop {
                        swc_ecma_ast::PropOrSpread::Spread(spread) => {
                            self.visit_expr(&spread.expr);
                        }
                        swc_ecma_ast::PropOrSpread::Prop(prop) => match prop.as_ref() {
                            swc_ecma_ast::Prop::KeyValue(kv) => {
                                self.visit_expr(&kv.value);
                            }
                            swc_ecma_ast::Prop::Shorthand(ident) => {
                                let _ = ident;
                            }
                            _ => {}
                        },
                    }
                }
            }
            Expr::Member(member) => {
                self.visit_expr(&member.obj);
                if let swc_ecma_ast::MemberProp::Computed(computed) = &member.prop {
                    self.visit_expr(&computed.expr);
                }
            }
            Expr::Cond(cond) => {
                self.visit_expr(&cond.test);
                self.visit_expr(&cond.cons);
                self.visit_expr(&cond.alt);
            }
            Expr::Bin(bin) => {
                self.visit_expr(&bin.left);
                self.visit_expr(&bin.right);
            }
            Expr::Unary(unary) => {
                self.visit_expr(&unary.arg);
            }
            Expr::Seq(seq) => {
                for e in &seq.exprs {
                    self.visit_expr(e);
                }
            }
            Expr::Paren(paren) => {
                self.visit_expr(&paren.expr);
            }
            Expr::Arrow(arrow) => {
                self.enter_scope();
                for param in &arrow.params {
                    self.mark_pattern_as_reassignable(param);
                }
                match &*arrow.body {
                    swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
                        for s in &block.stmts {
                            self.visit_stmt(s);
                        }
                    }
                    swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
                        self.visit_expr(expr);
                    }
                }
                self.exit_scope();
            }
            Expr::Fn(fn_expr) => {
                if let Some(body) = &fn_expr.function.body {
                    self.enter_scope();
                    for param in &fn_expr.function.params {
                        self.mark_pattern_as_reassignable(&param.pat);
                    }
                    for s in &body.stmts {
                        self.visit_stmt(s);
                    }
                    self.exit_scope();
                }
            }
            Expr::Tpl(tpl) => {
                for e in &tpl.exprs {
                    self.visit_expr(e);
                }
            }
            Expr::TaggedTpl(tagged) => {
                self.visit_expr(&tagged.tag);
                for e in &tagged.tpl.exprs {
                    self.visit_expr(e);
                }
            }
            Expr::Await(await_expr) => {
                self.visit_expr(&await_expr.arg);
            }
            Expr::Yield(yield_expr) => {
                if let Some(arg) = &yield_expr.arg {
                    self.visit_expr(arg);
                }
            }
            _ => {}
        }
    }

    fn handle_assignment(&mut self, assign: &AssignExpr) {
        if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = &assign.left {
            let name = ident.sym.to_string();
            if let Some(scope_depth) = self.find_declaration_scope(&name) {
                self.reassigned.insert((name, scope_depth));
            }
        }
        self.visit_expr(&assign.right);
    }

    fn handle_update(&mut self, update: &UpdateExpr) {
        if let Expr::Ident(ident) = &*update.arg {
            let name = ident.sym.to_string();
            if let Some(scope_depth) = self.find_declaration_scope(&name) {
                self.reassigned.insert((name, scope_depth));
            }
        }
    }

    fn mark_for_head_reassignable(&mut self, head: &ForHead) {
        match head {
            ForHead::VarDecl(var_decl) => {
                for declarator in &var_decl.decls {
                    self.mark_pattern_as_reassignable(&declarator.name);
                }
            }
            ForHead::Pat(pat) => {
                self.mark_pattern_as_reassignable(pat);
            }
            ForHead::UsingDecl(using) => {
                for declarator in &using.decls {
                    self.mark_pattern_as_reassignable(&declarator.name);
                }
            }
        }
    }

    fn mark_pattern_as_reassignable(&mut self, pat: &Pat) {
        match pat {
            Pat::Ident(ident) => {
                let name = ident.sym.to_string();
                if let Some(scope_depth) = self.find_declaration_scope(&name) {
                    self.reassigned.insert((name, scope_depth));
                }
            }
            Pat::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    self.mark_pattern_as_reassignable(elem);
                }
            }
            Pat::Object(obj) => {
                for prop in &obj.props {
                    match prop {
                        swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                            self.mark_pattern_as_reassignable(&kv.value);
                        }
                        swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                            let name = assign.key.sym.to_string();
                            if let Some(scope_depth) = self.find_declaration_scope(&name) {
                                self.reassigned.insert((name, scope_depth));
                            }
                        }
                        swc_ecma_ast::ObjectPatProp::Rest(rest) => {
                            self.mark_pattern_as_reassignable(&rest.arg);
                        }
                    }
                }
            }
            Pat::Rest(rest) => {
                self.mark_pattern_as_reassignable(&rest.arg);
            }
            Pat::Assign(assign) => {
                self.mark_pattern_as_reassignable(&assign.left);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_prefer_const(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = PreferConst::new();
        rule.check(&file)
    }

    #[test]
    fn detects_never_reassigned_let() {
        let diagnostics = run_prefer_const("let x = 1; console.log(x);");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q031");
        assert!(diagnostics[0].message.contains("x"));
        assert!(diagnostics[0].message.contains("never reassigned"));
    }

    #[test]
    fn ignores_reassigned_let() {
        let code = r#"
let x = 1;
x = 2;
console.log(x);
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_const_declaration() {
        let diagnostics = run_prefer_const("const x = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_var_declaration() {
        let diagnostics = run_prefer_const("var x = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_in_nested_function() {
        let code = r#"
function foo() {
    let x = 1;
    return x;
}
"#;
        let diagnostics = run_prefer_const(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("x"));
    }

    #[test]
    fn ignores_reassigned_with_increment() {
        let code = r#"
let x = 1;
x++;
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_reassigned_with_compound_assignment() {
        let code = r#"
let x = 1;
x += 2;
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_never_reassigned() {
        let code = r#"
let a = 1;
let b = 2;
let c = 3;
c = 4;
console.log(a, b, c);
"#;
        let diagnostics = run_prefer_const(code);

        assert_eq!(diagnostics.len(), 2);
        let names: Vec<_> = diagnostics.iter().map(|d| &d.message).collect();
        assert!(names.iter().any(|m| m.contains("'a'")));
        assert!(names.iter().any(|m| m.contains("'b'")));
    }

    #[test]
    fn ignores_for_loop_variable() {
        let code = r#"
for (let i = 0; i < 10; i++) {
    console.log(i);
}
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_for_in_loop_variable() {
        let code = r#"
const obj = { a: 1, b: 2 };
for (let key in obj) {
    console.log(key);
}
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_for_of_loop_variable() {
        let code = r#"
const arr = [1, 2, 3];
for (let item of arr) {
    console.log(item);
}
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_let_without_initializer() {
        let code = r#"
let x;
x = 1;
console.log(x);
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn metadata_is_correct() {
        let rule = PreferConst::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q031");
        assert_eq!(metadata.name, "prefer-const");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn fix_provided() {
        let diagnostics = run_prefer_const("let x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].fixes.len(), 1);

        let fix = &diagnostics[0].fixes[0];
        assert_eq!(fix.title, "Replace 'let' with 'const'");
        assert!(matches!(
            &fix.kind,
            crate::diagnostic::FixKind::ReplaceWith { new_text } if new_text == "const"
        ));
    }

    #[test]
    fn ignores_arrow_function_params() {
        let code = r#"
const fn = (x) => {
    x = 2;
    return x;
};
"#;
        let diagnostics = run_prefer_const(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn scopes_are_handled_correctly() {
        let code = r#"
let x = 1;
function foo() {
    let x = 2;
    x = 3;
}
console.log(x);
"#;
        let diagnostics = run_prefer_const(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("'x'"));
    }
}
