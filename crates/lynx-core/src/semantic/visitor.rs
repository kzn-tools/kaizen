//! Scope visitor for building ScopeTree and SymbolTable from AST
//!
//! This module provides a visitor that traverses the AST and builds
//! the scope tree and symbol table with proper scoping semantics.

use swc_common::{Span, Spanned};
use swc_ecma_ast::{
    ArrowExpr, BlockStmt, CatchClause, ClassDecl, Decl, FnDecl, ForInStmt, ForOfStmt, ForStmt,
    Module, ModuleDecl, ModuleItem, ObjectPatProp, Pat, Stmt, SwitchStmt, TryStmt, VarDeclKind,
    WhileStmt,
};

use super::scope::{ScopeId, ScopeKind, ScopeTree};
use super::symbols::{DeclarationKind, SymbolKind, SymbolTable};

pub struct ScopeBuilder {
    pub scope_tree: ScopeTree,
    pub symbol_table: SymbolTable,
    current_scope: Option<ScopeId>,
}

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeBuilder {
    pub fn new() -> Self {
        Self {
            scope_tree: ScopeTree::new(),
            symbol_table: SymbolTable::new(),
            current_scope: None,
        }
    }

    pub fn build(module: &Module) -> (ScopeTree, SymbolTable) {
        let mut builder = Self::new();
        builder.visit_module(module);
        (builder.scope_tree, builder.symbol_table)
    }

    fn visit_module(&mut self, module: &Module) {
        let global_scope = self
            .scope_tree
            .create_scope(ScopeKind::Global, None, module.span);
        self.current_scope = Some(global_scope);

        for item in &module.body {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(decl) => self.visit_module_decl(decl),
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }

    fn visit_module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::ExportDecl(export_decl) => {
                self.visit_decl(&export_decl.decl, true);
            }
            ModuleDecl::ExportDefaultDecl(export_default) => {
                if let Some(fn_expr) = &export_default.decl.as_fn_expr() {
                    if let Some(ident) = &fn_expr.ident {
                        self.declare_symbol(
                            &ident.sym,
                            SymbolKind::Function,
                            DeclarationKind::Function,
                            ident.span,
                            true,
                        );
                    }
                    self.visit_function(&fn_expr.function, None);
                } else if let Some(class_expr) = &export_default.decl.as_class() {
                    if let Some(ident) = &class_expr.ident {
                        self.declare_symbol(
                            &ident.sym,
                            SymbolKind::Class,
                            DeclarationKind::Class,
                            ident.span,
                            true,
                        );
                    }
                    self.visit_class(&class_expr.class);
                }
            }
            ModuleDecl::Import(import) => {
                for specifier in &import.specifiers {
                    match specifier {
                        swc_ecma_ast::ImportSpecifier::Named(named) => {
                            self.declare_symbol(
                                &named.local.sym,
                                SymbolKind::Import,
                                DeclarationKind::Import,
                                named.local.span,
                                false,
                            );
                        }
                        swc_ecma_ast::ImportSpecifier::Default(default) => {
                            self.declare_symbol(
                                &default.local.sym,
                                SymbolKind::Import,
                                DeclarationKind::Import,
                                default.local.span,
                                false,
                            );
                        }
                        swc_ecma_ast::ImportSpecifier::Namespace(namespace) => {
                            self.declare_symbol(
                                &namespace.local.sym,
                                SymbolKind::Import,
                                DeclarationKind::Import,
                                namespace.local.span,
                                false,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(decl) => self.visit_decl(decl, false),
            Stmt::Block(block) => self.visit_block_stmt(block),
            Stmt::If(if_stmt) => {
                self.visit_stmt(&if_stmt.cons);
                if let Some(alt) = &if_stmt.alt {
                    self.visit_stmt(alt);
                }
            }
            Stmt::For(for_stmt) => self.visit_for_stmt(for_stmt),
            Stmt::ForIn(for_in) => self.visit_for_in_stmt(for_in),
            Stmt::ForOf(for_of) => self.visit_for_of_stmt(for_of),
            Stmt::While(while_stmt) => self.visit_while_stmt(while_stmt),
            Stmt::Switch(switch_stmt) => self.visit_switch_stmt(switch_stmt),
            Stmt::Try(try_stmt) => self.visit_try_stmt(try_stmt),
            Stmt::With(with_stmt) => {
                self.visit_stmt(&with_stmt.body);
            }
            Stmt::Labeled(labeled) => {
                self.visit_stmt(&labeled.body);
            }
            Stmt::Return(ret) => {
                if let Some(arg) = &ret.arg {
                    self.visit_expr(arg);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.visit_expr(&expr_stmt.expr);
            }
            _ => {}
        }
    }

    fn visit_decl(&mut self, decl: &Decl, is_exported: bool) {
        match decl {
            Decl::Var(var_decl) => {
                let (symbol_kind, decl_kind) = match var_decl.kind {
                    VarDeclKind::Var => (SymbolKind::Variable, DeclarationKind::Var),
                    VarDeclKind::Let => (SymbolKind::Variable, DeclarationKind::Let),
                    VarDeclKind::Const => (SymbolKind::Constant, DeclarationKind::Const),
                };

                for declarator in &var_decl.decls {
                    self.declare_pat(&declarator.name, symbol_kind, decl_kind, is_exported);

                    if let Some(init) = &declarator.init {
                        self.visit_expr(init);
                    }
                }
            }
            Decl::Fn(fn_decl) => self.visit_fn_decl(fn_decl, is_exported),
            Decl::Class(class_decl) => self.visit_class_decl(class_decl, is_exported),
            _ => {}
        }
    }

    fn visit_fn_decl(&mut self, fn_decl: &FnDecl, is_exported: bool) {
        self.declare_symbol(
            &fn_decl.ident.sym,
            SymbolKind::Function,
            DeclarationKind::Function,
            fn_decl.ident.span,
            is_exported,
        );

        self.visit_function(&fn_decl.function, Some(fn_decl.ident.span));
    }

    fn visit_function(&mut self, func: &swc_ecma_ast::Function, name_span: Option<Span>) {
        let span = func
            .body
            .as_ref()
            .map(|b| b.span)
            .unwrap_or_else(|| name_span.unwrap_or(func.span));

        let parent_scope = self.current_scope;
        let func_scope = self
            .scope_tree
            .create_scope(ScopeKind::Function, parent_scope, span);
        self.current_scope = Some(func_scope);

        for param in &func.params {
            self.declare_pat(
                &param.pat,
                SymbolKind::Parameter,
                DeclarationKind::Parameter,
                false,
            );
        }

        if let Some(body) = &func.body {
            for stmt in &body.stmts {
                self.visit_stmt(stmt);
            }
        }

        self.current_scope = parent_scope;
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl, is_exported: bool) {
        self.declare_symbol(
            &class_decl.ident.sym,
            SymbolKind::Class,
            DeclarationKind::Class,
            class_decl.ident.span,
            is_exported,
        );

        self.visit_class(&class_decl.class);
    }

    fn visit_class(&mut self, class: &swc_ecma_ast::Class) {
        let parent_scope = self.current_scope;
        let class_scope = self
            .scope_tree
            .create_scope(ScopeKind::Class, parent_scope, class.span);
        self.current_scope = Some(class_scope);

        for member in &class.body {
            match member {
                swc_ecma_ast::ClassMember::Method(method) => {
                    self.visit_function(&method.function, None);
                }
                swc_ecma_ast::ClassMember::Constructor(ctor) => {
                    let ctor_scope = self.scope_tree.create_scope(
                        ScopeKind::Function,
                        Some(class_scope),
                        ctor.span,
                    );
                    self.current_scope = Some(ctor_scope);

                    for param in &ctor.params {
                        match param {
                            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
                                self.declare_pat(
                                    &p.pat,
                                    SymbolKind::Parameter,
                                    DeclarationKind::Parameter,
                                    false,
                                );
                            }
                            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param) => {
                                if let Some(param) = &ts_param.param.as_ident() {
                                    self.declare_symbol(
                                        &param.sym,
                                        SymbolKind::Parameter,
                                        DeclarationKind::Parameter,
                                        param.span,
                                        false,
                                    );
                                }
                            }
                        }
                    }

                    if let Some(body) = &ctor.body {
                        for stmt in &body.stmts {
                            self.visit_stmt(stmt);
                        }
                    }

                    self.current_scope = Some(class_scope);
                }
                _ => {}
            }
        }

        self.current_scope = parent_scope;
    }

    fn visit_block_stmt(&mut self, block: &BlockStmt) {
        let parent_scope = self.current_scope;
        let block_scope = self
            .scope_tree
            .create_scope(ScopeKind::Block, parent_scope, block.span);
        self.current_scope = Some(block_scope);

        for stmt in &block.stmts {
            self.visit_stmt(stmt);
        }

        self.current_scope = parent_scope;
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) {
        let parent_scope = self.current_scope;
        let for_scope = self
            .scope_tree
            .create_scope(ScopeKind::For, parent_scope, for_stmt.span);
        self.current_scope = Some(for_scope);

        if let Some(init) = &for_stmt.init {
            match init {
                swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
                    let (symbol_kind, decl_kind) = match var_decl.kind {
                        VarDeclKind::Var => (SymbolKind::Variable, DeclarationKind::Var),
                        VarDeclKind::Let => (SymbolKind::Variable, DeclarationKind::Let),
                        VarDeclKind::Const => (SymbolKind::Constant, DeclarationKind::Const),
                    };
                    for declarator in &var_decl.decls {
                        self.declare_pat(&declarator.name, symbol_kind, decl_kind, false);
                    }
                }
                swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
                    self.visit_expr(expr);
                }
            }
        }

        self.visit_stmt(&for_stmt.body);
        self.current_scope = parent_scope;
    }

    fn visit_for_in_stmt(&mut self, for_in: &ForInStmt) {
        let parent_scope = self.current_scope;
        let for_scope = self
            .scope_tree
            .create_scope(ScopeKind::For, parent_scope, for_in.span);
        self.current_scope = Some(for_scope);

        if let swc_ecma_ast::ForHead::VarDecl(var_decl) = &for_in.left {
            let (symbol_kind, decl_kind) = match var_decl.kind {
                VarDeclKind::Var => (SymbolKind::Variable, DeclarationKind::Var),
                VarDeclKind::Let => (SymbolKind::Variable, DeclarationKind::Let),
                VarDeclKind::Const => (SymbolKind::Constant, DeclarationKind::Const),
            };
            for declarator in &var_decl.decls {
                self.declare_pat(&declarator.name, symbol_kind, decl_kind, false);
            }
        }

        self.visit_stmt(&for_in.body);
        self.current_scope = parent_scope;
    }

    fn visit_for_of_stmt(&mut self, for_of: &ForOfStmt) {
        let parent_scope = self.current_scope;
        let for_scope = self
            .scope_tree
            .create_scope(ScopeKind::For, parent_scope, for_of.span);
        self.current_scope = Some(for_scope);

        if let swc_ecma_ast::ForHead::VarDecl(var_decl) = &for_of.left {
            let (symbol_kind, decl_kind) = match var_decl.kind {
                VarDeclKind::Var => (SymbolKind::Variable, DeclarationKind::Var),
                VarDeclKind::Let => (SymbolKind::Variable, DeclarationKind::Let),
                VarDeclKind::Const => (SymbolKind::Constant, DeclarationKind::Const),
            };
            for declarator in &var_decl.decls {
                self.declare_pat(&declarator.name, symbol_kind, decl_kind, false);
            }
        }

        self.visit_stmt(&for_of.body);
        self.current_scope = parent_scope;
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        let parent_scope = self.current_scope;
        let while_scope =
            self.scope_tree
                .create_scope(ScopeKind::While, parent_scope, while_stmt.span);
        self.current_scope = Some(while_scope);

        self.visit_stmt(&while_stmt.body);
        self.current_scope = parent_scope;
    }

    fn visit_switch_stmt(&mut self, switch_stmt: &SwitchStmt) {
        let parent_scope = self.current_scope;
        let switch_scope =
            self.scope_tree
                .create_scope(ScopeKind::Switch, parent_scope, switch_stmt.span);
        self.current_scope = Some(switch_scope);

        for case in &switch_stmt.cases {
            for stmt in &case.cons {
                self.visit_stmt(stmt);
            }
        }

        self.current_scope = parent_scope;
    }

    fn visit_try_stmt(&mut self, try_stmt: &TryStmt) {
        let parent_scope = self.current_scope;
        let try_scope =
            self.scope_tree
                .create_scope(ScopeKind::Try, parent_scope, try_stmt.block.span);
        self.current_scope = Some(try_scope);

        for stmt in &try_stmt.block.stmts {
            self.visit_stmt(stmt);
        }

        self.current_scope = parent_scope;

        if let Some(catch) = &try_stmt.handler {
            self.visit_catch_clause(catch);
        }

        if let Some(finalizer) = &try_stmt.finalizer {
            let finally_scope =
                self.scope_tree
                    .create_scope(ScopeKind::Block, parent_scope, finalizer.span);
            self.current_scope = Some(finally_scope);

            for stmt in &finalizer.stmts {
                self.visit_stmt(stmt);
            }

            self.current_scope = parent_scope;
        }
    }

    fn visit_catch_clause(&mut self, catch: &CatchClause) {
        let parent_scope = self.current_scope;
        let catch_scope = self
            .scope_tree
            .create_scope(ScopeKind::Catch, parent_scope, catch.span);
        self.current_scope = Some(catch_scope);

        if let Some(param) = &catch.param {
            self.declare_pat(
                param,
                SymbolKind::Parameter,
                DeclarationKind::Parameter,
                false,
            );
        }

        for stmt in &catch.body.stmts {
            self.visit_stmt(stmt);
        }

        self.current_scope = parent_scope;
    }

    fn visit_expr(&mut self, expr: &swc_ecma_ast::Expr) {
        match expr {
            swc_ecma_ast::Expr::Arrow(arrow) => self.visit_arrow_expr(arrow),
            swc_ecma_ast::Expr::Fn(fn_expr) => {
                if let Some(ident) = &fn_expr.ident {
                    self.declare_symbol(
                        &ident.sym,
                        SymbolKind::Function,
                        DeclarationKind::Function,
                        ident.span,
                        false,
                    );
                }
                self.visit_function(&fn_expr.function, fn_expr.ident.as_ref().map(|i| i.span));
            }
            swc_ecma_ast::Expr::Call(call) => {
                if let Some(callee_expr) = call.callee.as_expr() {
                    self.visit_expr(callee_expr);
                }
                for arg in &call.args {
                    self.visit_expr(&arg.expr);
                }
            }
            swc_ecma_ast::Expr::Array(arr) => {
                for elem in arr.elems.iter().flatten() {
                    self.visit_expr(&elem.expr);
                }
            }
            swc_ecma_ast::Expr::Object(obj) => {
                for prop in &obj.props {
                    if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
                        match prop.as_ref() {
                            swc_ecma_ast::Prop::Method(method) => {
                                self.visit_function(&method.function, None);
                            }
                            swc_ecma_ast::Prop::KeyValue(kv) => {
                                self.visit_expr(&kv.value);
                            }
                            swc_ecma_ast::Prop::Getter(getter) => {
                                if let Some(body) = &getter.body {
                                    let parent = self.current_scope;
                                    let scope = self.scope_tree.create_scope(
                                        ScopeKind::Function,
                                        parent,
                                        body.span,
                                    );
                                    self.current_scope = Some(scope);
                                    for stmt in &body.stmts {
                                        self.visit_stmt(stmt);
                                    }
                                    self.current_scope = parent;
                                }
                            }
                            swc_ecma_ast::Prop::Setter(setter) => {
                                if let Some(body) = &setter.body {
                                    let parent = self.current_scope;
                                    let scope = self.scope_tree.create_scope(
                                        ScopeKind::Function,
                                        parent,
                                        body.span,
                                    );
                                    self.current_scope = Some(scope);
                                    self.declare_pat(
                                        &setter.param,
                                        SymbolKind::Parameter,
                                        DeclarationKind::Parameter,
                                        false,
                                    );
                                    for stmt in &body.stmts {
                                        self.visit_stmt(stmt);
                                    }
                                    self.current_scope = parent;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            swc_ecma_ast::Expr::Class(class_expr) => {
                if let Some(ident) = &class_expr.ident {
                    self.declare_symbol(
                        &ident.sym,
                        SymbolKind::Class,
                        DeclarationKind::Class,
                        ident.span,
                        false,
                    );
                }
                self.visit_class(&class_expr.class);
            }
            _ => {}
        }
    }

    fn visit_arrow_expr(&mut self, arrow: &ArrowExpr) {
        let span = match &*arrow.body {
            swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => block.span,
            swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => expr.span(),
        };

        let parent_scope = self.current_scope;
        let arrow_scope =
            self.scope_tree
                .create_scope(ScopeKind::ArrowFunction, parent_scope, span);
        self.current_scope = Some(arrow_scope);

        for param in &arrow.params {
            self.declare_pat(
                param,
                SymbolKind::Parameter,
                DeclarationKind::Parameter,
                false,
            );
        }

        match &*arrow.body {
            swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
                for stmt in &block.stmts {
                    self.visit_stmt(stmt);
                }
            }
            swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
                self.visit_expr(expr);
            }
        }

        self.current_scope = parent_scope;
    }

    fn declare_pat(
        &mut self,
        pat: &Pat,
        symbol_kind: SymbolKind,
        decl_kind: DeclarationKind,
        is_exported: bool,
    ) {
        match pat {
            Pat::Ident(binding_ident) => {
                self.declare_symbol(
                    &binding_ident.id.sym,
                    symbol_kind,
                    decl_kind,
                    binding_ident.id.span,
                    is_exported,
                );
            }
            Pat::Array(array_pat) => {
                for elem in array_pat.elems.iter().flatten() {
                    self.declare_pat(elem, symbol_kind, decl_kind, is_exported);
                }
            }
            Pat::Object(object_pat) => {
                for prop in &object_pat.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            self.declare_pat(&kv.value, symbol_kind, decl_kind, is_exported);
                        }
                        ObjectPatProp::Assign(assign) => {
                            self.declare_symbol(
                                &assign.key.sym,
                                symbol_kind,
                                decl_kind,
                                assign.key.span,
                                is_exported,
                            );
                        }
                        ObjectPatProp::Rest(rest) => {
                            self.declare_pat(&rest.arg, symbol_kind, decl_kind, is_exported);
                        }
                    }
                }
            }
            Pat::Rest(rest_pat) => {
                self.declare_pat(&rest_pat.arg, symbol_kind, decl_kind, is_exported);
            }
            Pat::Assign(assign_pat) => {
                self.declare_pat(&assign_pat.left, symbol_kind, decl_kind, is_exported);
            }
            Pat::Invalid(_) | Pat::Expr(_) => {}
        }
    }

    fn declare_symbol(
        &mut self,
        name: &str,
        kind: SymbolKind,
        decl_kind: DeclarationKind,
        span: Span,
        is_exported: bool,
    ) {
        let scope = if decl_kind == DeclarationKind::Var {
            self.find_hoisting_scope()
        } else {
            self.current_scope.expect("no current scope")
        };

        self.symbol_table
            .declare(name, kind, decl_kind, scope, span, is_exported);
    }

    fn find_hoisting_scope(&self) -> ScopeId {
        let current = self.current_scope.expect("no current scope");

        for scope in self.scope_tree.ancestors(current) {
            match scope.kind {
                ScopeKind::Global | ScopeKind::Module | ScopeKind::Function => {
                    return scope.id;
                }
                _ => continue,
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedFile;

    fn build_from_source(code: &str) -> (ScopeTree, SymbolTable) {
        let parsed = ParsedFile::from_source("test.js", code);
        let module = parsed.module().expect("parse failed");
        ScopeBuilder::build(module)
    }

    #[test]
    fn creates_global_scope() {
        let (tree, _) = build_from_source("");
        assert!(tree.root().is_some());
        assert_eq!(tree.get(tree.root().unwrap()).kind, ScopeKind::Global);
    }

    #[test]
    fn collects_const_declaration() {
        let (tree, symbols) = build_from_source("const x = 1;");

        let global = tree.root().unwrap();
        let found = symbols.lookup("x", global, &tree);
        assert!(found.is_some());

        let symbol = symbols.get(found.unwrap());
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.kind, SymbolKind::Constant);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Const);
    }

    #[test]
    fn collects_let_declaration() {
        let (tree, symbols) = build_from_source("let y = 2;");

        let global = tree.root().unwrap();
        let found = symbols.lookup("y", global, &tree);
        assert!(found.is_some());

        let symbol = symbols.get(found.unwrap());
        assert_eq!(symbol.name, "y");
        assert_eq!(symbol.kind, SymbolKind::Variable);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Let);
    }

    #[test]
    fn collects_var_declaration() {
        let (tree, symbols) = build_from_source("var z = 3;");

        let global = tree.root().unwrap();
        let found = symbols.lookup("z", global, &tree);
        assert!(found.is_some());

        let symbol = symbols.get(found.unwrap());
        assert_eq!(symbol.name, "z");
        assert_eq!(symbol.kind, SymbolKind::Variable);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Var);
    }

    #[test]
    fn creates_function_scope_with_params() {
        let code = r#"
function add(a, b) {
    return a + b;
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let add_symbol = symbols.lookup("add", global, &tree);
        assert!(add_symbol.is_some());
        assert_eq!(symbols.get(add_symbol.unwrap()).kind, SymbolKind::Function);

        let func_scope = tree.get(global).children[0];
        assert_eq!(tree.get(func_scope).kind, ScopeKind::Function);

        let a_symbol = symbols.lookup("a", func_scope, &tree);
        let b_symbol = symbols.lookup("b", func_scope, &tree);
        assert!(a_symbol.is_some());
        assert!(b_symbol.is_some());
        assert_eq!(symbols.get(a_symbol.unwrap()).kind, SymbolKind::Parameter);
        assert_eq!(symbols.get(b_symbol.unwrap()).kind, SymbolKind::Parameter);
    }

    #[test]
    fn var_hoists_to_function_scope() {
        let code = r#"
function test() {
    if (true) {
        var hoisted = 1;
    }
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let func_scope = tree.get(global).children[0];

        let hoisted = symbols.lookup("hoisted", func_scope, &tree);
        assert!(hoisted.is_some());

        let symbol = symbols.get(hoisted.unwrap());
        assert_eq!(symbol.scope, func_scope);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Var);
    }

    #[test]
    fn let_respects_block_scope() {
        let code = r#"
function test() {
    if (true) {
        let blockScoped = 1;
    }
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let func_scope = tree.get(global).children[0];

        let not_found_in_func = symbols
            .symbols_in_scope(func_scope)
            .find(|s| s.name == "blockScoped");
        assert!(not_found_in_func.is_none());

        let block_scope = tree.get(func_scope).children[0];
        let found = symbols.lookup("blockScoped", block_scope, &tree);
        assert!(found.is_some());
    }

    #[test]
    fn arrow_function_creates_scope() {
        let code = r#"
const fn = (x, y) => {
    const z = x + y;
    return z;
};
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let fn_symbol = symbols.lookup("fn", global, &tree);
        assert!(fn_symbol.is_some());

        let arrow_scope = tree.get(global).children[0];
        assert_eq!(tree.get(arrow_scope).kind, ScopeKind::ArrowFunction);

        let x = symbols.lookup("x", arrow_scope, &tree);
        let y = symbols.lookup("y", arrow_scope, &tree);
        let z = symbols.lookup("z", arrow_scope, &tree);
        assert!(x.is_some());
        assert!(y.is_some());
        assert!(z.is_some());
    }

    #[test]
    fn for_loop_creates_scope() {
        let code = r#"
for (let i = 0; i < 10; i++) {
    console.log(i);
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let for_scope = tree.get(global).children[0];
        assert_eq!(tree.get(for_scope).kind, ScopeKind::For);

        let i = symbols.lookup("i", for_scope, &tree);
        assert!(i.is_some());

        let not_in_global = symbols.symbols_in_scope(global).find(|s| s.name == "i");
        assert!(not_in_global.is_none());
    }

    #[test]
    fn catch_clause_binds_error() {
        let code = r#"
try {
    throw new Error();
} catch (err) {
    console.log(err);
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let try_scope = tree.get(global).children[0];
        assert_eq!(tree.get(try_scope).kind, ScopeKind::Try);

        let catch_scope = tree.get(global).children[1];
        assert_eq!(tree.get(catch_scope).kind, ScopeKind::Catch);

        let err = symbols.lookup("err", catch_scope, &tree);
        assert!(err.is_some());
    }

    #[test]
    fn destructuring_collects_all_names() {
        let code = r#"
const { a, b: renamed, ...rest } = obj;
const [x, y, ...others] = arr;
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        assert!(symbols.lookup("a", global, &tree).is_some());
        assert!(symbols.lookup("renamed", global, &tree).is_some());
        assert!(symbols.lookup("rest", global, &tree).is_some());
        assert!(symbols.lookup("x", global, &tree).is_some());
        assert!(symbols.lookup("y", global, &tree).is_some());
        assert!(symbols.lookup("others", global, &tree).is_some());
    }

    #[test]
    fn nested_functions_create_nested_scopes() {
        let code = r#"
function outer(a) {
    function inner(b) {
        return a + b;
    }
    return inner;
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let outer_scope = tree.get(global).children[0];
        let inner_scope = tree.get(outer_scope).children[0];

        assert_eq!(tree.get(outer_scope).kind, ScopeKind::Function);
        assert_eq!(tree.get(inner_scope).kind, ScopeKind::Function);

        let a_from_inner = symbols.lookup("a", inner_scope, &tree);
        assert!(a_from_inner.is_some());
        assert_eq!(symbols.get(a_from_inner.unwrap()).scope, outer_scope);
    }

    #[test]
    fn class_declaration_creates_scope() {
        let code = r#"
class MyClass {
    constructor(value) {
        this.value = value;
    }

    getValue() {
        return this.value;
    }
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let class_symbol = symbols.lookup("MyClass", global, &tree);
        assert!(class_symbol.is_some());
        assert_eq!(symbols.get(class_symbol.unwrap()).kind, SymbolKind::Class);

        let class_scope = tree.get(global).children[0];
        assert_eq!(tree.get(class_scope).kind, ScopeKind::Class);
    }

    #[test]
    fn exported_symbols_marked_correctly() {
        let code = r#"
export const exported = 1;
const notExported = 2;
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let exported = symbols.lookup("exported", global, &tree).unwrap();
        assert!(symbols.get(exported).is_exported);

        let not_exported = symbols.lookup("notExported", global, &tree).unwrap();
        assert!(!symbols.get(not_exported).is_exported);
    }

    #[test]
    fn import_declarations_registered() {
        let code = r#"
import defaultExport from 'module';
import { named } from 'module';
import * as namespace from 'module';
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        let default_import = symbols.lookup("defaultExport", global, &tree);
        assert!(default_import.is_some());
        assert_eq!(
            symbols.get(default_import.unwrap()).kind,
            SymbolKind::Import
        );

        let named_import = symbols.lookup("named", global, &tree);
        assert!(named_import.is_some());

        let namespace_import = symbols.lookup("namespace", global, &tree);
        assert!(namespace_import.is_some());
    }

    #[test]
    fn var_in_for_loop_hoists() {
        let code = r#"
function test() {
    for (var i = 0; i < 10; i++) {
        console.log(i);
    }
    console.log(i);
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let func_scope = tree.get(global).children[0];

        let i_symbol = symbols.lookup("i", func_scope, &tree).unwrap();
        assert_eq!(symbols.get(i_symbol).scope, func_scope);
    }

    #[test]
    fn multiple_var_declarations() {
        let code = "var a = 1, b = 2, c = 3;";
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();

        assert!(symbols.lookup("a", global, &tree).is_some());
        assert!(symbols.lookup("b", global, &tree).is_some());
        assert!(symbols.lookup("c", global, &tree).is_some());
    }

    #[test]
    fn switch_statement_creates_scope() {
        let code = r#"
switch (x) {
    case 1:
        let a = 1;
        break;
    case 2:
        let b = 2;
        break;
}
"#;
        let (tree, symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let switch_scope = tree.get(global).children[0];
        assert_eq!(tree.get(switch_scope).kind, ScopeKind::Switch);

        let a = symbols.lookup("a", switch_scope, &tree);
        let b = symbols.lookup("b", switch_scope, &tree);
        assert!(a.is_some());
        assert!(b.is_some());
    }

    #[test]
    fn while_loop_creates_scope() {
        let code = r#"
while (true) {
    let x = 1;
}
"#;
        let (tree, _symbols) = build_from_source(code);

        let global = tree.root().unwrap();
        let while_scope = tree.get(global).children[0];
        assert_eq!(tree.get(while_scope).kind, ScopeKind::While);
    }
}
