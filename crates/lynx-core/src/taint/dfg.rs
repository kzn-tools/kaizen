//! Data Flow Graph for tracking value propagation
//!
//! This module provides a DFG data structure for representing how values
//! flow through JavaScript/TypeScript code. It is the foundation for
//! taint analysis and security vulnerability detection.

use id_arena::{Arena, Id};
use std::collections::{HashMap, HashSet};
use swc_common::{Span, Spanned};
use swc_ecma_ast::{
    ArrowExpr, AssignExpr, BinExpr, BlockStmt, CallExpr, Callee, CondExpr, Decl, Expr, FnDecl,
    FnExpr, ForInStmt, ForOfStmt, ForStmt, Ident, MemberExpr, MemberProp, Module, ModuleItem,
    NewExpr, ObjectLit, OptChainExpr, Pat, Prop, PropOrSpread, Stmt, VarDecl, VarDeclarator,
};

use crate::semantic::{ScopeId, SemanticModel};

pub type DfgNodeId = Id<DfgNode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DfgNodeKind {
    Variable {
        name: String,
        scope_id: Option<ScopeId>,
    },
    Literal,
    Call {
        callee_name: String,
    },
    PropertyAccess {
        object: DfgNodeId,
        property: String,
    },
    BinaryOp {
        left: DfgNodeId,
        right: DfgNodeId,
    },
    Parameter {
        name: String,
        index: usize,
    },
    Unknown,
}

#[derive(Debug)]
pub struct DfgNode {
    pub id: DfgNodeId,
    pub kind: DfgNodeKind,
    pub span: Span,
    pub flows_to: Vec<DfgNodeId>,
    pub flows_from: Vec<DfgNodeId>,
}

#[derive(Debug)]
pub struct DataFlowGraph {
    arena: Arena<DfgNode>,
    var_to_node: HashMap<(Option<ScopeId>, String), DfgNodeId>,
}

impl Default for DataFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFlowGraph {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            var_to_node: HashMap::new(),
        }
    }

    pub fn build(module: &Module, semantic: &SemanticModel) -> Self {
        let mut builder = DfgBuilder::new(semantic);
        builder.visit_module(module);
        builder.graph
    }

    fn create_node(&mut self, kind: DfgNodeKind, span: Span) -> DfgNodeId {
        self.arena.alloc_with_id(|id| DfgNode {
            id,
            kind,
            span,
            flows_to: Vec::new(),
            flows_from: Vec::new(),
        })
    }

    fn add_edge(&mut self, from: DfgNodeId, to: DfgNodeId) {
        if from == to {
            return;
        }
        if !self.arena[from].flows_to.contains(&to) {
            self.arena[from].flows_to.push(to);
        }
        if !self.arena[to].flows_from.contains(&from) {
            self.arena[to].flows_from.push(from);
        }
    }

    pub fn get(&self, id: DfgNodeId) -> &DfgNode {
        &self.arena[id]
    }

    pub fn nodes(&self) -> impl Iterator<Item = &DfgNode> {
        self.arena.iter().map(|(_, node)| node)
    }

    pub fn node_count(&self) -> usize {
        self.arena.len()
    }

    pub fn get_variable_node(&self, scope_id: Option<ScopeId>, name: &str) -> Option<DfgNodeId> {
        self.var_to_node.get(&(scope_id, name.to_string())).copied()
    }

    pub fn get_sources(&self, node: DfgNodeId) -> Vec<DfgNodeId> {
        let mut sources = Vec::new();
        let mut visited = HashSet::new();
        self.collect_sources(node, &mut sources, &mut visited);
        sources
    }

    fn collect_sources(
        &self,
        node: DfgNodeId,
        sources: &mut Vec<DfgNodeId>,
        visited: &mut HashSet<DfgNodeId>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        let n = &self.arena[node];
        if n.flows_from.is_empty() {
            sources.push(node);
        } else {
            for &from in &n.flows_from {
                self.collect_sources(from, sources, visited);
            }
        }
    }

    pub fn get_dependents(&self, node: DfgNodeId) -> Vec<DfgNodeId> {
        let mut dependents = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependents(node, &mut dependents, &mut visited);
        dependents
    }

    fn collect_dependents(
        &self,
        node: DfgNodeId,
        dependents: &mut Vec<DfgNodeId>,
        visited: &mut HashSet<DfgNodeId>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        let n = &self.arena[node];
        for &to in &n.flows_to {
            dependents.push(to);
            self.collect_dependents(to, dependents, visited);
        }
    }

    pub fn depends_on(&self, node: DfgNodeId, source: DfgNodeId) -> bool {
        let sources = self.get_sources(node);
        sources.contains(&source)
    }
}

#[allow(dead_code)]
struct DfgBuilder<'a> {
    graph: DataFlowGraph,
    semantic: &'a SemanticModel,
    current_scope: Option<ScopeId>,
}

impl<'a> DfgBuilder<'a> {
    fn new(semantic: &'a SemanticModel) -> Self {
        Self {
            graph: DataFlowGraph::new(),
            semantic,
            current_scope: semantic.scope_tree.root(),
        }
    }

    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
            ModuleItem::ModuleDecl(decl) => {
                if let swc_ecma_ast::ModuleDecl::ExportDecl(export) = decl {
                    self.visit_decl(&export.decl);
                }
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(decl) => self.visit_decl(decl),
            Stmt::Expr(expr_stmt) => {
                self.visit_expr(&expr_stmt.expr);
            }
            Stmt::Block(block) => self.visit_block(block),
            Stmt::If(if_stmt) => {
                self.visit_expr(&if_stmt.test);
                self.visit_stmt(&if_stmt.cons);
                if let Some(alt) = &if_stmt.alt {
                    self.visit_stmt(alt);
                }
            }
            Stmt::For(for_stmt) => self.visit_for_stmt(for_stmt),
            Stmt::ForIn(for_in) => self.visit_for_in_stmt(for_in),
            Stmt::ForOf(for_of) => self.visit_for_of_stmt(for_of),
            Stmt::While(while_stmt) => {
                self.visit_expr(&while_stmt.test);
                self.visit_stmt(&while_stmt.body);
            }
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body);
                self.visit_expr(&do_while.test);
            }
            Stmt::Return(ret) => {
                if let Some(arg) = &ret.arg {
                    self.visit_expr(arg);
                }
            }
            Stmt::Switch(switch_stmt) => {
                self.visit_expr(&switch_stmt.discriminant);
                for case in &switch_stmt.cases {
                    if let Some(test) = &case.test {
                        self.visit_expr(test);
                    }
                    for stmt in &case.cons {
                        self.visit_stmt(stmt);
                    }
                }
            }
            Stmt::Try(try_stmt) => {
                self.visit_block(&try_stmt.block);
                if let Some(handler) = &try_stmt.handler {
                    self.visit_block(&handler.body);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    self.visit_block(finalizer);
                }
            }
            Stmt::Throw(throw) => {
                self.visit_expr(&throw.arg);
            }
            _ => {}
        }
    }

    fn visit_block(&mut self, block: &BlockStmt) {
        for stmt in &block.stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Var(var_decl) => self.visit_var_decl(var_decl),
            Decl::Fn(fn_decl) => self.visit_fn_decl(fn_decl),
            _ => {}
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        for declarator in &var_decl.decls {
            self.visit_var_declarator(declarator);
        }
    }

    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        let var_node = self.create_node_for_pattern(&declarator.name);

        if let Some(init) = &declarator.init {
            let init_node = self.visit_expr(init);
            if let (Some(var_id), Some(init_id)) = (var_node, init_node) {
                self.graph.add_edge(init_id, var_id);
            }
        }
    }

    fn create_node_for_pattern(&mut self, pat: &Pat) -> Option<DfgNodeId> {
        match pat {
            Pat::Ident(ident) => {
                let name = ident.sym.to_string();
                let node_id = self.graph.create_node(
                    DfgNodeKind::Variable {
                        name: name.clone(),
                        scope_id: self.current_scope,
                    },
                    ident.span,
                );
                self.graph
                    .var_to_node
                    .insert((self.current_scope, name), node_id);
                Some(node_id)
            }
            Pat::Array(array_pat) => {
                for elem in array_pat.elems.iter().flatten() {
                    self.create_node_for_pattern(elem);
                }
                None
            }
            Pat::Object(object_pat) => {
                for prop in &object_pat.props {
                    match prop {
                        swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                            self.create_node_for_pattern(&kv.value);
                        }
                        swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                            let name = assign.key.sym.to_string();
                            let node_id = self.graph.create_node(
                                DfgNodeKind::Variable {
                                    name: name.clone(),
                                    scope_id: self.current_scope,
                                },
                                assign.key.span,
                            );
                            self.graph
                                .var_to_node
                                .insert((self.current_scope, name), node_id);
                        }
                        swc_ecma_ast::ObjectPatProp::Rest(rest) => {
                            self.create_node_for_pattern(&rest.arg);
                        }
                    }
                }
                None
            }
            Pat::Rest(rest) => self.create_node_for_pattern(&rest.arg),
            Pat::Assign(assign) => self.create_node_for_pattern(&assign.left),
            _ => None,
        }
    }

    fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
        let saved_scope = self.current_scope;

        for (index, param) in fn_decl.function.params.iter().enumerate() {
            self.create_parameter_node(&param.pat, index);
        }

        if let Some(body) = &fn_decl.function.body {
            self.visit_block(body);
        }

        self.current_scope = saved_scope;
    }

    fn create_parameter_node(&mut self, pat: &Pat, index: usize) -> Option<DfgNodeId> {
        match pat {
            Pat::Ident(ident) => {
                let name = ident.sym.to_string();
                let node_id = self.graph.create_node(
                    DfgNodeKind::Parameter {
                        name: name.clone(),
                        index,
                    },
                    ident.span,
                );
                self.graph
                    .var_to_node
                    .insert((self.current_scope, name), node_id);
                Some(node_id)
            }
            _ => self.create_node_for_pattern(pat),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Option<DfgNodeId> {
        match expr {
            Expr::Ident(ident) => self.visit_ident(ident),
            Expr::Lit(_) => Some(self.graph.create_node(DfgNodeKind::Literal, expr.span())),
            Expr::Call(call) => self.visit_call_expr(call),
            Expr::Member(member) => self.visit_member_expr(member),
            Expr::Bin(bin) => self.visit_bin_expr(bin),
            Expr::Assign(assign) => self.visit_assign_expr(assign),
            Expr::Arrow(arrow) => self.visit_arrow_expr(arrow),
            Expr::Fn(fn_expr) => self.visit_fn_expr(fn_expr),
            Expr::New(new_expr) => self.visit_new_expr(new_expr),
            Expr::Cond(cond) => self.visit_cond_expr(cond),
            Expr::Paren(paren) => self.visit_expr(&paren.expr),
            Expr::Seq(seq) => {
                let mut last = None;
                for expr in &seq.exprs {
                    last = self.visit_expr(expr);
                }
                last
            }
            Expr::Unary(unary) => self.visit_expr(&unary.arg),
            Expr::Update(update) => self.visit_expr(&update.arg),
            Expr::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    self.visit_expr(&elem.expr);
                }
                Some(self.graph.create_node(DfgNodeKind::Unknown, array.span))
            }
            Expr::Object(obj) => self.visit_object_lit(obj),
            Expr::Tpl(tpl) => self.visit_template_literal(tpl),
            Expr::TaggedTpl(tagged) => {
                self.visit_expr(&tagged.tag);
                for expr in &tagged.tpl.exprs {
                    self.visit_expr(expr);
                }
                Some(self.graph.create_node(DfgNodeKind::Unknown, tagged.span))
            }
            Expr::Await(await_expr) => self.visit_expr(&await_expr.arg),
            Expr::Yield(yield_expr) => {
                if let Some(arg) = &yield_expr.arg {
                    self.visit_expr(arg)
                } else {
                    None
                }
            }
            Expr::OptChain(opt_chain) => self.visit_opt_chain_expr(opt_chain),
            _ => None,
        }
    }

    fn visit_ident(&mut self, ident: &Ident) -> Option<DfgNodeId> {
        let name = ident.sym.to_string();
        self.graph
            .var_to_node
            .get(&(self.current_scope, name.clone()))
            .copied()
            .or_else(|| self.graph.var_to_node.get(&(None, name.clone())).copied())
            .or_else(|| {
                for scope_id in self.graph.var_to_node.keys() {
                    if let Some(node_id) = self.graph.var_to_node.get(&(scope_id.0, name.clone())) {
                        return Some(*node_id);
                    }
                }
                None
            })
    }

    fn visit_call_expr(&mut self, call: &CallExpr) -> Option<DfgNodeId> {
        let callee_name = self.extract_callee_name(&call.callee);

        for arg in &call.args {
            self.visit_expr(&arg.expr);
        }

        let call_node = self.graph.create_node(
            DfgNodeKind::Call {
                callee_name: callee_name.clone(),
            },
            call.span,
        );

        for arg in &call.args {
            if let Some(arg_node) = self.visit_expr(&arg.expr) {
                self.graph.add_edge(arg_node, call_node);
            }
        }

        Some(call_node)
    }

    fn extract_callee_name(&self, callee: &Callee) -> String {
        match callee {
            Callee::Expr(expr) => match expr.as_ref() {
                Expr::Ident(ident) => ident.sym.to_string(),
                Expr::Member(member) => {
                    if let MemberProp::Ident(prop) = &member.prop {
                        prop.sym.to_string()
                    } else {
                        "unknown".to_string()
                    }
                }
                _ => "unknown".to_string(),
            },
            _ => "unknown".to_string(),
        }
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) -> Option<DfgNodeId> {
        let object_node = self.visit_expr(&member.obj)?;

        let property = match &member.prop {
            MemberProp::Ident(ident) => ident.sym.to_string(),
            MemberProp::Computed(computed) => {
                self.visit_expr(&computed.expr);
                "[computed]".to_string()
            }
            MemberProp::PrivateName(private) => format!("#{}", private.name),
        };

        let prop_node = self.graph.create_node(
            DfgNodeKind::PropertyAccess {
                object: object_node,
                property,
            },
            member.span,
        );

        self.graph.add_edge(object_node, prop_node);
        Some(prop_node)
    }

    fn visit_bin_expr(&mut self, bin: &BinExpr) -> Option<DfgNodeId> {
        let left_node = self.visit_expr(&bin.left);
        let right_node = self.visit_expr(&bin.right);

        match (left_node, right_node) {
            (Some(left), Some(right)) => {
                let bin_node = self
                    .graph
                    .create_node(DfgNodeKind::BinaryOp { left, right }, bin.span);
                self.graph.add_edge(left, bin_node);
                self.graph.add_edge(right, bin_node);
                Some(bin_node)
            }
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        }
    }

    fn visit_assign_expr(&mut self, assign: &AssignExpr) -> Option<DfgNodeId> {
        let value_node = self.visit_expr(&assign.right);

        match &assign.left {
            swc_ecma_ast::AssignTarget::Simple(simple) => match simple {
                swc_ecma_ast::SimpleAssignTarget::Ident(ident) => {
                    let name = ident.sym.to_string();
                    let target_node = self
                        .graph
                        .var_to_node
                        .get(&(self.current_scope, name.clone()))
                        .copied()
                        .or_else(|| {
                            let node = self.graph.create_node(
                                DfgNodeKind::Variable {
                                    name: name.clone(),
                                    scope_id: self.current_scope,
                                },
                                ident.span,
                            );
                            self.graph
                                .var_to_node
                                .insert((self.current_scope, name), node);
                            Some(node)
                        });

                    if let (Some(value), Some(target)) = (value_node, target_node) {
                        self.graph.add_edge(value, target);
                    }
                    target_node
                }
                swc_ecma_ast::SimpleAssignTarget::Member(member) => {
                    self.visit_member_expr(member);
                    value_node
                }
                _ => value_node,
            },
            swc_ecma_ast::AssignTarget::Pat(pat) => {
                match pat {
                    swc_ecma_ast::AssignTargetPat::Array(arr) => {
                        for elem in arr.elems.iter().flatten() {
                            self.create_node_for_pattern(elem);
                        }
                    }
                    swc_ecma_ast::AssignTargetPat::Object(obj) => {
                        for prop in &obj.props {
                            match prop {
                                swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                                    self.create_node_for_pattern(&kv.value);
                                }
                                swc_ecma_ast::ObjectPatProp::Assign(assign_prop) => {
                                    let name = assign_prop.key.sym.to_string();
                                    let node_id = self.graph.create_node(
                                        DfgNodeKind::Variable {
                                            name: name.clone(),
                                            scope_id: self.current_scope,
                                        },
                                        assign_prop.key.span,
                                    );
                                    self.graph
                                        .var_to_node
                                        .insert((self.current_scope, name), node_id);
                                }
                                swc_ecma_ast::ObjectPatProp::Rest(rest) => {
                                    self.create_node_for_pattern(&rest.arg);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                value_node
            }
        }
    }

    fn visit_arrow_expr(&mut self, arrow: &ArrowExpr) -> Option<DfgNodeId> {
        let saved_scope = self.current_scope;

        for (index, param) in arrow.params.iter().enumerate() {
            self.create_parameter_node(param, index);
        }

        match arrow.body.as_ref() {
            swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
                self.visit_block(block);
            }
            swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
                self.visit_expr(expr);
            }
        }

        self.current_scope = saved_scope;
        Some(self.graph.create_node(DfgNodeKind::Unknown, arrow.span))
    }

    fn visit_fn_expr(&mut self, fn_expr: &FnExpr) -> Option<DfgNodeId> {
        let saved_scope = self.current_scope;

        for (index, param) in fn_expr.function.params.iter().enumerate() {
            self.create_parameter_node(&param.pat, index);
        }

        if let Some(body) = &fn_expr.function.body {
            self.visit_block(body);
        }

        self.current_scope = saved_scope;
        Some(
            self.graph
                .create_node(DfgNodeKind::Unknown, fn_expr.function.span),
        )
    }

    fn visit_new_expr(&mut self, new_expr: &NewExpr) -> Option<DfgNodeId> {
        self.visit_expr(&new_expr.callee);

        if let Some(args) = &new_expr.args {
            for arg in args {
                self.visit_expr(&arg.expr);
            }
        }

        Some(self.graph.create_node(DfgNodeKind::Unknown, new_expr.span))
    }

    fn visit_cond_expr(&mut self, cond: &CondExpr) -> Option<DfgNodeId> {
        self.visit_expr(&cond.test);
        let cons = self.visit_expr(&cond.cons);
        let alt = self.visit_expr(&cond.alt);

        let cond_node = self.graph.create_node(DfgNodeKind::Unknown, cond.span);
        if let Some(c) = cons {
            self.graph.add_edge(c, cond_node);
        }
        if let Some(a) = alt {
            self.graph.add_edge(a, cond_node);
        }
        Some(cond_node)
    }

    fn visit_object_lit(&mut self, obj: &ObjectLit) -> Option<DfgNodeId> {
        for prop in &obj.props {
            match prop {
                PropOrSpread::Prop(prop) => {
                    if let Prop::KeyValue(kv) = prop.as_ref() {
                        self.visit_expr(&kv.value);
                    }
                }
                PropOrSpread::Spread(spread) => {
                    self.visit_expr(&spread.expr);
                }
            }
        }
        Some(self.graph.create_node(DfgNodeKind::Unknown, obj.span))
    }

    fn visit_template_literal(&mut self, tpl: &swc_ecma_ast::Tpl) -> Option<DfgNodeId> {
        let mut expr_nodes = Vec::new();
        for expr in &tpl.exprs {
            if let Some(node) = self.visit_expr(expr) {
                expr_nodes.push(node);
            }
        }

        if expr_nodes.is_empty() {
            return Some(self.graph.create_node(DfgNodeKind::Literal, tpl.span));
        }

        let tpl_node = self.graph.create_node(DfgNodeKind::Unknown, tpl.span);
        for node in expr_nodes {
            self.graph.add_edge(node, tpl_node);
        }
        Some(tpl_node)
    }

    fn visit_opt_chain_expr(&mut self, opt_chain: &OptChainExpr) -> Option<DfgNodeId> {
        match opt_chain.base.as_ref() {
            swc_ecma_ast::OptChainBase::Member(member) => self.visit_member_expr(member),
            swc_ecma_ast::OptChainBase::Call(call) => {
                let callee_name = match call.callee.as_ref() {
                    Expr::Ident(ident) => ident.sym.to_string(),
                    Expr::Member(member) => {
                        if let MemberProp::Ident(prop) = &member.prop {
                            prop.sym.to_string()
                        } else {
                            "unknown".to_string()
                        }
                    }
                    _ => "unknown".to_string(),
                };

                let call_node = self
                    .graph
                    .create_node(DfgNodeKind::Call { callee_name }, opt_chain.span);

                for arg in &call.args {
                    if let Some(arg_node) = self.visit_expr(&arg.expr) {
                        self.graph.add_edge(arg_node, call_node);
                    }
                }

                Some(call_node)
            }
        }
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) {
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
    }

    fn visit_for_in_stmt(&mut self, for_in: &ForInStmt) {
        match &for_in.left {
            swc_ecma_ast::ForHead::VarDecl(var_decl) => {
                self.visit_var_decl(var_decl);
            }
            swc_ecma_ast::ForHead::Pat(pat) => {
                self.create_node_for_pattern(pat);
            }
            _ => {}
        }
        self.visit_expr(&for_in.right);
        self.visit_stmt(&for_in.body);
    }

    fn visit_for_of_stmt(&mut self, for_of: &ForOfStmt) {
        match &for_of.left {
            swc_ecma_ast::ForHead::VarDecl(var_decl) => {
                self.visit_var_decl(var_decl);
            }
            swc_ecma_ast::ForHead::Pat(pat) => {
                self.create_node_for_pattern(pat);
            }
            _ => {}
        }
        self.visit_expr(&for_of.right);
        self.visit_stmt(&for_of.body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedFile;
    use crate::semantic::ScopeBuilder;

    fn build_dfg(code: &str) -> DataFlowGraph {
        let parsed = ParsedFile::from_source("test.js", code);
        let module = parsed.module().expect("parse failed");
        let semantic = ScopeBuilder::build(module);
        DataFlowGraph::build(module, &semantic)
    }

    #[test]
    fn dfg_tracks_simple_assignment() {
        let dfg = build_dfg("const source = 1; const x = source; const y = x;");

        assert!(dfg.node_count() >= 3);

        let source_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "source"));
        let x_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "x"));
        let y_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "y"));

        assert!(source_node.is_some(), "source node should exist");
        assert!(x_node.is_some(), "x node should exist");
        assert!(y_node.is_some(), "y node should exist");

        let x = x_node.unwrap();
        let y = y_node.unwrap();

        assert!(
            !x.flows_from.is_empty(),
            "x should have incoming flow from source"
        );

        assert!(
            !y.flows_from.is_empty(),
            "y should have incoming flow from x"
        );
    }

    #[test]
    fn dfg_tracks_function_call() {
        let dfg = build_dfg("const input = 1; const result = process(input);");

        let input_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "input"));
        let call_node = dfg.nodes().find(
            |n| matches!(&n.kind, DfgNodeKind::Call { callee_name } if callee_name == "process"),
        );
        let result_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "result"));

        assert!(input_node.is_some(), "input node should exist");
        assert!(call_node.is_some(), "process call node should exist");
        assert!(result_node.is_some(), "result node should exist");

        let call = call_node.unwrap();
        assert!(
            !call.flows_from.is_empty(),
            "call should have incoming flow from input"
        );

        let result = result_node.unwrap();
        assert!(
            !result.flows_from.is_empty(),
            "result should depend on call"
        );
    }

    #[test]
    fn dfg_tracks_property_access() {
        let dfg = build_dfg("const user = {}; const name = user.name;");

        let user_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "user"));
        let prop_node = dfg.nodes().find(|n| {
            matches!(&n.kind, DfgNodeKind::PropertyAccess { property, .. } if property == "name")
        });
        let name_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "name"));

        assert!(user_node.is_some(), "user node should exist");
        assert!(prop_node.is_some(), "property access node should exist");
        assert!(name_node.is_some(), "name node should exist");

        let prop = prop_node.unwrap();
        assert!(
            !prop.flows_from.is_empty(),
            "property access should depend on user"
        );

        let name_var = name_node.unwrap();
        assert!(
            !name_var.flows_from.is_empty(),
            "name should depend on property access"
        );
    }

    #[test]
    fn dfg_tracks_binary_expression() {
        let dfg = build_dfg("const column = 'id'; const query = 'SELECT ' + column;");

        let column_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "column"));
        let bin_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::BinaryOp { .. }));
        let query_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "query"));

        assert!(column_node.is_some(), "column node should exist");
        assert!(bin_node.is_some(), "binary operation node should exist");
        assert!(query_node.is_some(), "query node should exist");

        let bin = bin_node.unwrap();
        assert!(
            !bin.flows_from.is_empty(),
            "binary op should have incoming flows"
        );

        let query = query_node.unwrap();
        assert!(
            !query.flows_from.is_empty(),
            "query should depend on binary op"
        );
    }

    #[test]
    fn dfg_tracks_chained_property_access() {
        let dfg = build_dfg("const req = {}; const id = req.query.id;");

        let req_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "req"));
        let id_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "id"));

        assert!(req_node.is_some(), "req node should exist");
        assert!(id_node.is_some(), "id node should exist");

        let prop_nodes: Vec<_> = dfg
            .nodes()
            .filter(|n| matches!(&n.kind, DfgNodeKind::PropertyAccess { .. }))
            .collect();
        assert!(
            prop_nodes.len() >= 2,
            "should have at least 2 property access nodes"
        );
    }

    #[test]
    fn dfg_tracks_method_call() {
        let dfg = build_dfg("const db = {}; const data = db.query('SELECT * FROM users');");

        let call_node = dfg.nodes().find(
            |n| matches!(&n.kind, DfgNodeKind::Call { callee_name } if callee_name == "query"),
        );
        let data_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "data"));

        assert!(call_node.is_some(), "query call node should exist");
        assert!(data_node.is_some(), "data node should exist");

        let data = data_node.unwrap();
        assert!(
            !data.flows_from.is_empty(),
            "data should depend on query call"
        );
    }

    #[test]
    fn dfg_handles_template_literal() {
        let dfg = build_dfg("const name = 'test'; const msg = `Hello ${name}!`;");

        let name_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "name"));
        let msg_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "msg"));

        assert!(name_node.is_some(), "name node should exist");
        assert!(msg_node.is_some(), "msg node should exist");
    }

    #[test]
    fn dfg_get_sources_returns_leaf_nodes() {
        let dfg = build_dfg("const a = 1; const b = a; const c = b;");

        let c_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "c"))
            .unwrap();

        let sources = dfg.get_sources(c_node.id);
        assert!(!sources.is_empty(), "c should have sources");
    }

    #[test]
    fn dfg_depends_on_works() {
        let dfg = build_dfg("const source = 1; const x = source;");

        let source_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "source"))
            .unwrap();

        let x_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "x"))
            .unwrap();

        assert!(
            !x_node.flows_from.is_empty(),
            "x should have incoming flow from source"
        );

        assert!(
            x_node.flows_from.contains(&source_node.id),
            "x should directly depend on source"
        );
    }

    #[test]
    fn dfg_handles_reassignment() {
        let dfg = build_dfg("let x = 1; x = 2;");

        let x_nodes: Vec<_> = dfg
            .nodes()
            .filter(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "x"))
            .collect();

        assert!(!x_nodes.is_empty(), "x nodes should exist");
    }

    #[test]
    fn dfg_handles_destructuring() {
        let dfg = build_dfg("const obj = {}; const { a, b } = obj;");

        let a_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "a"));
        let b_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "b"));

        assert!(a_node.is_some(), "a node should exist from destructuring");
        assert!(b_node.is_some(), "b node should exist from destructuring");
    }

    #[test]
    fn dfg_handles_conditional_expression() {
        let dfg = build_dfg("const a = 1; const b = 2; const c = true ? a : b;");

        let c_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "c"));

        assert!(c_node.is_some(), "c node should exist");
        let c = c_node.unwrap();
        assert!(
            !c.flows_from.is_empty(),
            "c should have incoming flow from conditional"
        );
    }

    #[test]
    fn empty_module_creates_empty_dfg() {
        let dfg = build_dfg("");
        assert_eq!(dfg.node_count(), 0);
    }

    #[test]
    fn dfg_node_count_grows_with_declarations() {
        let dfg1 = build_dfg("const a = 1;");
        let dfg2 = build_dfg("const a = 1; const b = 2;");
        let dfg3 = build_dfg("const a = 1; const b = 2; const c = 3;");

        assert!(dfg2.node_count() > dfg1.node_count());
        assert!(dfg3.node_count() > dfg2.node_count());
    }
}
