//! AstVisitor trait for uniform AST traversal.

use std::ops::ControlFlow;

use swc_ecma_ast::{
    ArrowExpr, AssignExpr, BinExpr, CallExpr, ClassDecl, FnDecl, Function, Ident, JSXElement,
    MemberExpr, NewExpr, Regex, UpdateExpr, VarDecl,
};

use super::context::VisitorContext;

pub trait AstVisitor {
    fn visit_function(&mut self, _node: &Function, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_fn_decl(&mut self, _node: &FnDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_arrow_expr(&mut self, _node: &ArrowExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_var_decl(&mut self, _node: &VarDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_call_expr(&mut self, _node: &CallExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_member_expr(&mut self, _node: &MemberExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_bin_expr(&mut self, _node: &BinExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_ident(&mut self, _node: &Ident, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_class_decl(&mut self, _node: &ClassDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_new_expr(&mut self, _node: &NewExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_assign_expr(&mut self, _node: &AssignExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_update_expr(&mut self, _node: &UpdateExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_regex(&mut self, _node: &Regex, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_jsx_element(&mut self, _node: &JSXElement, _ctx: &VisitorContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
