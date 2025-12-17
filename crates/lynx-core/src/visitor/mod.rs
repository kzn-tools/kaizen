//! Visitor pattern for AST traversal.
//!
//! Provides a uniform way to traverse AST nodes with context information.

mod context;
mod traits;

pub use context::VisitorContext;
pub use traits::AstVisitor;

use std::ops::ControlFlow;

use swc_ecma_ast::Module;
use swc_ecma_visit::{Visit, VisitWith};

struct Walker<'a, V: AstVisitor> {
    visitor: &'a mut V,
    ctx: &'a VisitorContext<'a>,
    stopped: bool,
}

impl<V: AstVisitor> Visit for Walker<'_, V> {
    fn visit_function(&mut self, node: &swc_ecma_ast::Function) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_function(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, node: &swc_ecma_ast::FnDecl) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_fn_decl(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_arrow_expr(&mut self, node: &swc_ecma_ast::ArrowExpr) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_arrow_expr(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_var_decl(&mut self, node: &swc_ecma_ast::VarDecl) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_var_decl(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, node: &swc_ecma_ast::CallExpr) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_call_expr(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, node: &swc_ecma_ast::MemberExpr) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_member_expr(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_bin_expr(&mut self, node: &swc_ecma_ast::BinExpr) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_bin_expr(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_ident(&mut self, node: &swc_ecma_ast::Ident) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_ident(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_class_decl(&mut self, node: &swc_ecma_ast::ClassDecl) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.visitor.visit_class_decl(node, self.ctx) {
            self.stopped = true;
            return;
        }
        node.visit_children_with(self);
    }
}

pub fn walk_ast<V: AstVisitor>(module: &Module, visitor: &mut V, ctx: &VisitorContext) {
    let mut walker = Walker {
        visitor,
        ctx,
        stopped: false,
    };
    module.visit_with(&mut walker);
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;

    use swc_ecma_ast::{CallExpr, FnDecl};

    use super::*;
    use crate::parser::ParsedFile;

    #[test]
    fn visitor_counts_function_declarations() {
        let code = r#"
function foo() {}
function bar() {}
const baz = () => {};
"#;
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        struct FunctionCounter {
            count: usize,
        }

        impl AstVisitor for FunctionCounter {
            fn visit_fn_decl(&mut self, _node: &FnDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
                self.count += 1;
                ControlFlow::Continue(())
            }
        }

        let mut counter = FunctionCounter { count: 0 };
        walk_ast(parsed.module().unwrap(), &mut counter, &ctx);

        assert_eq!(counter.count, 2);
    }

    #[test]
    fn visitor_finds_all_call_expressions() {
        let code = r#"
foo();
bar(1, 2);
baz.qux();
"#;
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        struct CallCollector {
            calls: Vec<String>,
        }

        impl AstVisitor for CallCollector {
            fn visit_call_expr(
                &mut self,
                node: &CallExpr,
                ctx: &VisitorContext,
            ) -> ControlFlow<()> {
                if let Some(text) = ctx.get_source_text(node.span) {
                    self.calls.push(text.to_string());
                }
                ControlFlow::Continue(())
            }
        }

        let mut collector = CallCollector { calls: Vec::new() };
        walk_ast(parsed.module().unwrap(), &mut collector, &ctx);

        assert_eq!(collector.calls.len(), 3);
    }

    #[test]
    fn visitor_can_stop_early() {
        let code = r#"
function first() {}
function second() {}
function third() {}
"#;
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        struct StopAtSecond {
            visited: Vec<String>,
        }

        impl AstVisitor for StopAtSecond {
            fn visit_fn_decl(&mut self, node: &FnDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
                let name = node.ident.sym.to_string();
                self.visited.push(name.clone());
                if name == "second" {
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
        }

        let mut visitor = StopAtSecond {
            visited: Vec::new(),
        };
        walk_ast(parsed.module().unwrap(), &mut visitor, &ctx);

        assert_eq!(visitor.visited, vec!["first", "second"]);
    }

    #[test]
    fn visitor_traverses_nested_scopes() {
        let code = r#"
function outer() {
    function inner() {
        const x = 1;
    }
}
"#;
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        struct NestedCounter {
            fn_count: usize,
            var_count: usize,
        }

        impl AstVisitor for NestedCounter {
            fn visit_fn_decl(&mut self, _node: &FnDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
                self.fn_count += 1;
                ControlFlow::Continue(())
            }

            fn visit_var_decl(
                &mut self,
                _node: &swc_ecma_ast::VarDecl,
                _ctx: &VisitorContext,
            ) -> ControlFlow<()> {
                self.var_count += 1;
                ControlFlow::Continue(())
            }
        }

        let mut counter = NestedCounter {
            fn_count: 0,
            var_count: 0,
        };
        walk_ast(parsed.module().unwrap(), &mut counter, &ctx);

        assert_eq!(counter.fn_count, 2);
        assert_eq!(counter.var_count, 1);
    }
}
