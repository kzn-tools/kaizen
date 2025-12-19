//! max-depth rule (Q011): Enforce maximum nesting depth threshold

use std::ops::ControlFlow;

use swc_common::Spanned;
use swc_ecma_ast::{
    ArrowExpr, BlockStmt, BlockStmtOrExpr, DoWhileStmt, Expr, ForInStmt, ForOfStmt, ForStmt,
    Function, IfStmt, Stmt, SwitchStmt, TryStmt, WhileStmt, WithStmt,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

const DEFAULT_THRESHOLD: usize = 4;

declare_rule!(
    MaxDepth,
    id = "Q011",
    name = "max-depth",
    description = "Enforce a maximum nesting depth threshold",
    category = Quality,
    severity = Warning,
    examples = "// Bad (depth > 4)\nfunction deep(x) {\n  if (a) {\n    if (b) {\n      if (c) {\n        if (d) {\n          if (e) {}\n        }\n      }\n    }\n  }\n}\n\n// Good (depth <= 4)\nfunction shallow(x) {\n  if (a) {\n    if (b) {\n      if (c) {\n        doSomething();\n      }\n    }\n  }\n}"
);

impl Rule for MaxDepth {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = MaxDepthVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            threshold: DEFAULT_THRESHOLD,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct MaxDepthVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    threshold: usize,
}

impl MaxDepthVisitor {
    fn check_function_depth(
        &mut self,
        name: Option<&str>,
        body: Option<&BlockStmt>,
        ctx: &VisitorContext,
        span: swc_common::Span,
    ) {
        let Some(body) = body else {
            return;
        };

        let (max_depth, deepest_span) = self.calculate_block_max_depth(body, 0);
        if max_depth > self.threshold {
            let report_span = deepest_span.unwrap_or(span);
            let (line, column) = ctx.span_to_location(report_span);
            let func_name = name.unwrap_or("anonymous function");
            let diagnostic = Diagnostic::new(
                "Q011",
                Severity::Warning,
                format!(
                    "Function '{}' has a nesting depth of {} (max: {})",
                    func_name, max_depth, self.threshold
                ),
                &self.file_path,
                line,
                column,
            )
            .with_suggestion(
                "Refactor code to reduce nesting levels by extracting logic into separate functions"
                    .to_string(),
            );

            self.diagnostics.push(diagnostic);
        }
    }

    fn check_arrow_depth(
        &mut self,
        body: &BlockStmtOrExpr,
        ctx: &VisitorContext,
        span: swc_common::Span,
    ) {
        let (max_depth, deepest_span) = match body {
            BlockStmtOrExpr::BlockStmt(block) => self.calculate_block_max_depth(block, 0),
            BlockStmtOrExpr::Expr(_) => (0, None),
        };

        if max_depth > self.threshold {
            let report_span = deepest_span.unwrap_or(span);
            let (line, column) = ctx.span_to_location(report_span);
            let diagnostic = Diagnostic::new(
                "Q011",
                Severity::Warning,
                format!(
                    "Arrow function has a nesting depth of {} (max: {})",
                    max_depth, self.threshold
                ),
                &self.file_path,
                line,
                column,
            )
            .with_suggestion(
                "Refactor code to reduce nesting levels by extracting logic into separate functions"
                    .to_string(),
            );

            self.diagnostics.push(diagnostic);
        }
    }

    fn calculate_block_max_depth(
        &self,
        block: &BlockStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let mut max_depth = current_depth;
        let mut deepest_span = None;

        for stmt in &block.stmts {
            let (stmt_depth, stmt_span) = self.calculate_stmt_max_depth(stmt, current_depth);
            if stmt_depth > max_depth {
                max_depth = stmt_depth;
                deepest_span = stmt_span;
            }
        }

        (max_depth, deepest_span)
    }

    fn calculate_stmt_max_depth(
        &self,
        stmt: &Stmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        match stmt {
            Stmt::If(if_stmt) => self.calculate_if_max_depth(if_stmt, current_depth),
            Stmt::While(while_stmt) => self.calculate_while_max_depth(while_stmt, current_depth),
            Stmt::DoWhile(do_while) => self.calculate_do_while_max_depth(do_while, current_depth),
            Stmt::For(for_stmt) => self.calculate_for_max_depth(for_stmt, current_depth),
            Stmt::ForIn(for_in) => self.calculate_for_in_max_depth(for_in, current_depth),
            Stmt::ForOf(for_of) => self.calculate_for_of_max_depth(for_of, current_depth),
            Stmt::Switch(switch_stmt) => {
                self.calculate_switch_max_depth(switch_stmt, current_depth)
            }
            Stmt::Try(try_stmt) => self.calculate_try_max_depth(try_stmt, current_depth),
            Stmt::With(with_stmt) => self.calculate_with_max_depth(with_stmt, current_depth),
            Stmt::Block(block) => self.calculate_block_max_depth(block, current_depth),
            Stmt::Labeled(labeled) => self.calculate_stmt_max_depth(&labeled.body, current_depth),
            Stmt::Expr(expr_stmt) => self.calculate_expr_max_depth(&expr_stmt.expr, current_depth),
            Stmt::Return(ret) => ret.arg.as_ref().map_or((current_depth, None), |e| {
                self.calculate_expr_max_depth(e, current_depth)
            }),
            Stmt::Throw(throw) => self.calculate_expr_max_depth(&throw.arg, current_depth),
            Stmt::Decl(decl) => self.calculate_decl_max_depth(decl, current_depth),
            _ => (current_depth, None),
        }
    }

    fn calculate_if_max_depth(
        &self,
        if_stmt: &IfStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let mut max_depth = new_depth;
        let mut deepest_span = Some(if_stmt.span());

        let (cons_depth, cons_span) = self.calculate_stmt_max_depth(&if_stmt.cons, new_depth);
        if cons_depth > max_depth {
            max_depth = cons_depth;
            deepest_span = cons_span;
        }

        if let Some(alt) = &if_stmt.alt {
            let (alt_depth, alt_span) = self.calculate_stmt_max_depth(alt, new_depth);
            if alt_depth > max_depth {
                max_depth = alt_depth;
                deepest_span = alt_span;
            }
        }

        (max_depth, deepest_span)
    }

    fn calculate_while_max_depth(
        &self,
        while_stmt: &WhileStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&while_stmt.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(while_stmt.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_do_while_max_depth(
        &self,
        do_while: &DoWhileStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&do_while.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(do_while.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_for_max_depth(
        &self,
        for_stmt: &ForStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&for_stmt.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(for_stmt.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_for_in_max_depth(
        &self,
        for_in: &ForInStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&for_in.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(for_in.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_for_of_max_depth(
        &self,
        for_of: &ForOfStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&for_of.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(for_of.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_switch_max_depth(
        &self,
        switch_stmt: &SwitchStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let mut max_depth = new_depth;
        let mut deepest_span = Some(switch_stmt.span());

        for case in &switch_stmt.cases {
            for stmt in &case.cons {
                let (stmt_depth, stmt_span) = self.calculate_stmt_max_depth(stmt, new_depth);
                if stmt_depth > max_depth {
                    max_depth = stmt_depth;
                    deepest_span = stmt_span;
                }
            }
        }

        (max_depth, deepest_span)
    }

    fn calculate_try_max_depth(
        &self,
        try_stmt: &TryStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let mut max_depth = new_depth;
        let mut deepest_span = Some(try_stmt.span());

        let (try_depth, try_span) = self.calculate_block_max_depth(&try_stmt.block, new_depth);
        if try_depth > max_depth {
            max_depth = try_depth;
            deepest_span = try_span;
        }

        if let Some(handler) = &try_stmt.handler {
            let (catch_depth, catch_span) =
                self.calculate_block_max_depth(&handler.body, new_depth);
            if catch_depth > max_depth {
                max_depth = catch_depth;
                deepest_span = catch_span;
            }
        }

        if let Some(finalizer) = &try_stmt.finalizer {
            let (finally_depth, finally_span) =
                self.calculate_block_max_depth(finalizer, new_depth);
            if finally_depth > max_depth {
                max_depth = finally_depth;
                deepest_span = finally_span;
            }
        }

        (max_depth, deepest_span)
    }

    fn calculate_with_max_depth(
        &self,
        with_stmt: &WithStmt,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        let new_depth = current_depth + 1;
        let (body_depth, body_span) = self.calculate_stmt_max_depth(&with_stmt.body, new_depth);
        let max_depth = body_depth.max(new_depth);
        let deepest_span = if body_depth > new_depth {
            body_span
        } else {
            Some(with_stmt.span())
        };
        (max_depth, deepest_span)
    }

    fn calculate_expr_max_depth(
        &self,
        _expr: &Expr,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        (current_depth, None)
    }

    fn calculate_decl_max_depth(
        &self,
        _decl: &swc_ecma_ast::Decl,
        current_depth: usize,
    ) -> (usize, Option<swc_common::Span>) {
        (current_depth, None)
    }
}

impl AstVisitor for MaxDepthVisitor {
    fn visit_function(&mut self, node: &Function, ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_function_depth(None, node.body.as_ref(), ctx, node.span());
        ControlFlow::Continue(())
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_arrow_depth(&node.body, ctx, node.span());
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = MaxDepth::new();
        rule.check(&file)
    }

    #[test]
    fn shallow_nesting_no_warning() {
        let diagnostics = run_rule("function simple() { if (x) { if (y) {} } }");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn depth_at_threshold_no_warning() {
        let code = r#"
function atThreshold(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {}
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert!(
            diagnostics.is_empty(),
            "Depth 4 should not warn (threshold is depth > 4)"
        );
    }

    #[test]
    fn exceeds_threshold_warns() {
        let code = r#"
function tooDeep(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {}
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q011");
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn deeply_nested_reports_correct_depth() {
        let code = r#"
function veryDeep(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {
                        if (f) {
                            if (g) {}
                        }
                    }
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("7"));
    }

    #[test]
    fn while_loop_adds_depth() {
        let code = r#"
function withLoop(x) {
    if (a) {
        while (b) {
            if (c) {
                for (;;) {
                    if (d) {}
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn for_loops_add_depth() {
        let code = r#"
function forLoops(arr) {
    for (let i = 0; i < 10; i++) {
        for (const x of arr) {
            for (const k in obj) {
                if (true) {
                    if (false) {}
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn switch_adds_depth() {
        let code = r#"
function withSwitch(x) {
    if (a) {
        switch (x) {
            case 1:
                if (b) {
                    if (c) {
                        if (d) {}
                    }
                }
                break;
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn try_catch_adds_depth() {
        let code = r#"
function withTryCatch() {
    try {
        if (a) {
            if (b) {
                if (c) {
                    if (d) {}
                }
            }
        }
    } catch (e) {}
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn do_while_adds_depth() {
        let code = r#"
function withDoWhile() {
    do {
        if (a) {
            if (b) {
                if (c) {
                    if (d) {}
                }
            }
        }
    } while (true);
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn arrow_function_warns() {
        let code = r#"
const fn = (x) => {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {}
                }
            }
        }
    }
};
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Arrow function"));
    }

    #[test]
    fn class_method_warns() {
        let code = r#"
class Test {
    deepMethod() {
        if (a) {
            if (b) {
                if (c) {
                    if (d) {
                        if (e) {}
                    }
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn nested_functions_each_checked_independently() {
        let code = r#"
function outer() {
    if (a) {
        if (b) {
            function inner() {
                if (c) {
                    if (d) {
                        if (e) {
                            if (f) {
                                if (g) {}
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(
            diagnostics.len(),
            1,
            "Only inner function should warn, outer is within threshold"
        );
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn multiple_functions_each_checked() {
        let code = r#"
function shallow() { if (x) {} }
function deep(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {}
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn else_branch_adds_same_depth_as_if() {
        let code = r#"
function withElse(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                } else {
                    if (e) {}
                }
            }
        }
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("5"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = MaxDepth::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q011");
        assert_eq!(metadata.name, "max-depth");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn empty_function_no_warning() {
        let diagnostics = run_rule("function empty() {}");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn function_expression_warns() {
        let code = r#"
const fn = function() {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {}
                }
            }
        }
    }
};
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }
}
