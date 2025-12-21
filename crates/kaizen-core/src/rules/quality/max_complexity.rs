//! max-complexity rule (Q010): Enforce maximum cyclomatic complexity for functions

use std::ops::ControlFlow;

use swc_common::Spanned;
use swc_ecma_ast::{
    ArrowExpr, BinExpr, BinaryOp, BlockStmtOrExpr, CondExpr, DoWhileStmt, Expr, ForInStmt,
    ForOfStmt, ForStmt, Function, IfStmt, Stmt, SwitchStmt, TryStmt, WhileStmt,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

const DEFAULT_THRESHOLD: usize = 10;

declare_rule!(
    MaxComplexity,
    id = "Q010",
    name = "max-complexity",
    description = "Enforce a maximum cyclomatic complexity threshold for functions",
    category = Quality,
    severity = Warning,
    examples = "// Bad (complexity > 10)\nfunction complex(x) {\n  if (a) { if (b) { if (c) { if (d) { if (e) { if (f) { if (g) { if (h) { if (i) { if (j) { if (k) {} } } } } } } } } } }\n}\n\n// Good (complexity <= 10)\nfunction simple(x) {\n  if (x > 0) {\n    return x * 2;\n  }\n  return x;\n}"
);

impl Rule for MaxComplexity {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = MaxComplexityVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            threshold: DEFAULT_THRESHOLD,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct MaxComplexityVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    threshold: usize,
}

impl MaxComplexityVisitor {
    fn check_function_complexity(
        &mut self,
        name: Option<&str>,
        body: Option<&swc_ecma_ast::BlockStmt>,
        ctx: &VisitorContext,
        span: swc_common::Span,
    ) {
        let Some(body) = body else {
            return;
        };

        let complexity = self.calculate_block_complexity(body);
        if complexity > self.threshold {
            let (line, column, end_line, end_column) = ctx.span_to_range(span);
            let func_name = name.unwrap_or("anonymous function");
            let diagnostic = Diagnostic::new(
                "Q010",
                Severity::Warning,
                format!(
                    "Function '{}' has a cyclomatic complexity of {} (max: {})",
                    func_name, complexity, self.threshold
                ),
                &self.file_path,
                line,
                column,
            )
            .with_end(end_line, end_column)
            .with_suggestion(
                "Consider refactoring this function into smaller, more focused functions"
                    .to_string(),
            );

            self.diagnostics.push(diagnostic);
        }
    }

    fn check_arrow_complexity(
        &mut self,
        body: &BlockStmtOrExpr,
        ctx: &VisitorContext,
        span: swc_common::Span,
    ) {
        let complexity = match body {
            BlockStmtOrExpr::BlockStmt(block) => self.calculate_block_complexity(block),
            BlockStmtOrExpr::Expr(expr) => self.calculate_expr_complexity(expr),
        };

        if complexity > self.threshold {
            let (line, column, end_line, end_column) = ctx.span_to_range(span);
            let diagnostic = Diagnostic::new(
                "Q010",
                Severity::Warning,
                format!(
                    "Arrow function has a cyclomatic complexity of {} (max: {})",
                    complexity, self.threshold
                ),
                &self.file_path,
                line,
                column,
            )
            .with_end(end_line, end_column)
            .with_suggestion(
                "Consider refactoring this function into smaller, more focused functions"
                    .to_string(),
            );

            self.diagnostics.push(diagnostic);
        }
    }

    fn calculate_block_complexity(&self, block: &swc_ecma_ast::BlockStmt) -> usize {
        let mut complexity = 1;
        for stmt in &block.stmts {
            complexity += self.calculate_stmt_complexity(stmt);
        }
        complexity
    }

    fn calculate_stmt_complexity(&self, stmt: &Stmt) -> usize {
        match stmt {
            Stmt::If(if_stmt) => self.calculate_if_complexity(if_stmt),
            Stmt::While(while_stmt) => self.calculate_while_complexity(while_stmt),
            Stmt::DoWhile(do_while) => self.calculate_do_while_complexity(do_while),
            Stmt::For(for_stmt) => self.calculate_for_complexity(for_stmt),
            Stmt::ForIn(for_in) => self.calculate_for_in_complexity(for_in),
            Stmt::ForOf(for_of) => self.calculate_for_of_complexity(for_of),
            Stmt::Switch(switch_stmt) => self.calculate_switch_complexity(switch_stmt),
            Stmt::Try(try_stmt) => self.calculate_try_complexity(try_stmt),
            Stmt::Block(block) => {
                let mut complexity = 0;
                for s in &block.stmts {
                    complexity += self.calculate_stmt_complexity(s);
                }
                complexity
            }
            Stmt::Return(ret) => ret
                .arg
                .as_ref()
                .map_or(0, |e| self.calculate_expr_complexity(e)),
            Stmt::Throw(throw) => self.calculate_expr_complexity(&throw.arg),
            Stmt::Expr(expr_stmt) => self.calculate_expr_complexity(&expr_stmt.expr),
            Stmt::Decl(decl) => self.calculate_decl_complexity(decl),
            Stmt::Labeled(labeled) => self.calculate_stmt_complexity(&labeled.body),
            Stmt::With(with) => {
                self.calculate_expr_complexity(&with.obj)
                    + self.calculate_stmt_complexity(&with.body)
            }
            _ => 0,
        }
    }

    fn calculate_if_complexity(&self, if_stmt: &IfStmt) -> usize {
        let mut complexity = 1;
        complexity += self.calculate_expr_complexity(&if_stmt.test);
        complexity += self.calculate_stmt_complexity(&if_stmt.cons);
        if let Some(alt) = &if_stmt.alt {
            complexity += self.calculate_stmt_complexity(alt);
        }
        complexity
    }

    fn calculate_while_complexity(&self, while_stmt: &WhileStmt) -> usize {
        let mut complexity = 1;
        complexity += self.calculate_expr_complexity(&while_stmt.test);
        complexity += self.calculate_stmt_complexity(&while_stmt.body);
        complexity
    }

    fn calculate_do_while_complexity(&self, do_while: &DoWhileStmt) -> usize {
        let mut complexity = 1;
        complexity += self.calculate_stmt_complexity(&do_while.body);
        complexity += self.calculate_expr_complexity(&do_while.test);
        complexity
    }

    fn calculate_for_complexity(&self, for_stmt: &ForStmt) -> usize {
        let mut complexity = 1;
        if let Some(test) = &for_stmt.test {
            complexity += self.calculate_expr_complexity(test);
        }
        complexity += self.calculate_stmt_complexity(&for_stmt.body);
        complexity
    }

    fn calculate_for_in_complexity(&self, for_in: &ForInStmt) -> usize {
        1 + self.calculate_stmt_complexity(&for_in.body)
    }

    fn calculate_for_of_complexity(&self, for_of: &ForOfStmt) -> usize {
        1 + self.calculate_stmt_complexity(&for_of.body)
    }

    fn calculate_switch_complexity(&self, switch_stmt: &SwitchStmt) -> usize {
        let mut complexity = 0;
        complexity += self.calculate_expr_complexity(&switch_stmt.discriminant);
        for case in &switch_stmt.cases {
            if case.test.is_some() {
                complexity += 1;
            }
            for stmt in &case.cons {
                complexity += self.calculate_stmt_complexity(stmt);
            }
        }
        complexity
    }

    fn calculate_try_complexity(&self, try_stmt: &TryStmt) -> usize {
        let mut complexity = 0;
        for stmt in &try_stmt.block.stmts {
            complexity += self.calculate_stmt_complexity(stmt);
        }
        if let Some(handler) = &try_stmt.handler {
            complexity += 1;
            for stmt in &handler.body.stmts {
                complexity += self.calculate_stmt_complexity(stmt);
            }
        }
        if let Some(finalizer) = &try_stmt.finalizer {
            for stmt in &finalizer.stmts {
                complexity += self.calculate_stmt_complexity(stmt);
            }
        }
        complexity
    }

    fn calculate_expr_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Bin(bin) => self.calculate_bin_expr_complexity(bin),
            Expr::Cond(cond) => self.calculate_cond_complexity(cond),
            Expr::Paren(paren) => self.calculate_expr_complexity(&paren.expr),
            Expr::Seq(seq) => seq
                .exprs
                .iter()
                .map(|e| self.calculate_expr_complexity(e))
                .sum(),
            Expr::Assign(assign) => self.calculate_expr_complexity(&assign.right),
            Expr::Call(call) => call
                .args
                .iter()
                .map(|a| self.calculate_expr_complexity(&a.expr))
                .sum(),
            Expr::New(new) => new.args.as_ref().map_or(0, |args| {
                args.iter()
                    .map(|a| self.calculate_expr_complexity(&a.expr))
                    .sum()
            }),
            Expr::Array(arr) => arr
                .elems
                .iter()
                .flatten()
                .map(|e| self.calculate_expr_complexity(&e.expr))
                .sum(),
            Expr::Object(obj) => obj
                .props
                .iter()
                .map(|p| match p {
                    swc_ecma_ast::PropOrSpread::Prop(prop) => match prop.as_ref() {
                        swc_ecma_ast::Prop::KeyValue(kv) => {
                            self.calculate_expr_complexity(&kv.value)
                        }
                        swc_ecma_ast::Prop::Shorthand(_) => 0,
                        swc_ecma_ast::Prop::Assign(assign) => {
                            self.calculate_expr_complexity(&assign.value)
                        }
                        _ => 0,
                    },
                    swc_ecma_ast::PropOrSpread::Spread(spread) => {
                        self.calculate_expr_complexity(&spread.expr)
                    }
                })
                .sum(),
            _ => 0,
        }
    }

    fn calculate_bin_expr_complexity(&self, bin: &BinExpr) -> usize {
        let mut complexity = match bin.op {
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::NullishCoalescing => 1,
            _ => 0,
        };
        complexity += self.calculate_expr_complexity(&bin.left);
        complexity += self.calculate_expr_complexity(&bin.right);
        complexity
    }

    fn calculate_cond_complexity(&self, cond: &CondExpr) -> usize {
        let mut complexity = 1;
        complexity += self.calculate_expr_complexity(&cond.test);
        complexity += self.calculate_expr_complexity(&cond.cons);
        complexity += self.calculate_expr_complexity(&cond.alt);
        complexity
    }

    fn calculate_decl_complexity(&self, decl: &swc_ecma_ast::Decl) -> usize {
        match decl {
            swc_ecma_ast::Decl::Var(var) => var
                .decls
                .iter()
                .map(|d| {
                    d.init
                        .as_ref()
                        .map_or(0, |e| self.calculate_expr_complexity(e))
                })
                .sum(),
            _ => 0,
        }
    }
}

impl AstVisitor for MaxComplexityVisitor {
    fn visit_function(&mut self, node: &Function, ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_function_complexity(None, node.body.as_ref(), ctx, node.span());
        ControlFlow::Continue(())
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_arrow_complexity(&node.body, ctx, node.span());
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = MaxComplexity::new();
        rule.check(&file)
    }

    #[test]
    fn simple_function_no_warning() {
        let diagnostics = run_rule("function simple() { return 1; }");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn function_with_one_if_no_warning() {
        let diagnostics = run_rule("function test(x) { if (x) { return 1; } return 0; }");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn function_at_threshold_no_warning() {
        let code = r#"
function atThreshold(x) {
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
    if (f) {}
    if (g) {}
    if (h) {}
    if (i) {}
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty(), "Complexity 10 should not warn");
    }

    #[test]
    fn function_exceeding_threshold_warns() {
        let code = r#"
function tooComplex(x) {
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
    if (f) {}
    if (g) {}
    if (h) {}
    if (i) {}
    if (j) {}
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q010");
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn nested_ifs_add_complexity() {
        let code = r#"
function nested(x) {
    if (a) {
        if (b) {
            if (c) {
                if (d) {
                    if (e) {
                        if (f) {
                            if (g) {
                                if (h) {
                                    if (i) {
                                        if (j) {
                                        }
                                    }
                                }
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
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn loops_add_complexity() {
        let code = r#"
function manyLoops(arr) {
    for (let i = 0; i < 10; i++) {}
    for (const x of arr) {}
    for (const k in obj) {}
    while (true) {}
    do {} while (false);
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn switch_cases_add_complexity() {
        let code = r#"
function manySwitch(x) {
    switch (x) {
        case 1: break;
        case 2: break;
        case 3: break;
        case 4: break;
        case 5: break;
        case 6: break;
        case 7: break;
        case 8: break;
        case 9: break;
        case 10: break;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn logical_operators_add_complexity() {
        let code = r#"
function logicalOps(a, b, c, d, e, f, g, h, i, j, k) {
    return a && b && c && d && e && f && g && h && i && j && k;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ternary_adds_complexity() {
        let code = r#"
function manyTernary(x) {
    return a ? 1 : b ? 2 : c ? 3 : d ? 4 : e ? 5 : f ? 6 : g ? 7 : h ? 8 : i ? 9 : j ? 10 : 11;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn catch_adds_complexity() {
        let code = r#"
function tryCatch() {
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
    try {} catch (e) {}
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn arrow_function_warns() {
        let code = r#"
const fn = (x) => {
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
    if (f) {}
    if (g) {}
    if (h) {}
    if (i) {}
    if (j) {}
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
    complexMethod() {
        if (a) {}
        if (b) {}
        if (c) {}
        if (d) {}
        if (e) {}
        if (f) {}
        if (g) {}
        if (h) {}
        if (i) {}
        if (j) {}
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn object_method_warns() {
        let code = r#"
const obj = {
    complexMethod() {
        if (a) {}
        if (b) {}
        if (c) {}
        if (d) {}
        if (e) {}
        if (f) {}
        if (g) {}
        if (h) {}
        if (i) {}
        if (j) {}
    }
};
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn nullish_coalescing_adds_complexity() {
        let code = r#"
function nullishOps(a, b, c, d, e, f, g, h, i, j, k) {
    return a ?? b ?? c ?? d ?? e ?? f ?? g ?? h ?? i ?? j ?? k;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = MaxComplexity::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q010");
        assert_eq!(metadata.name, "max-complexity");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn multiple_functions_each_checked() {
        let code = r#"
function simple() { return 1; }
function complex(x) {
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
    if (f) {}
    if (g) {}
    if (h) {}
    if (i) {}
    if (j) {}
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("11"));
    }

    #[test]
    fn function_expression_warns() {
        let code = r#"
const fn = function() {
    if (a) {}
    if (b) {}
    if (c) {}
    if (d) {}
    if (e) {}
    if (f) {}
    if (g) {}
    if (h) {}
    if (i) {}
    if (j) {}
};
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }
}
