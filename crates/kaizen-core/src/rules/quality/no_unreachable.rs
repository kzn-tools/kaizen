//! no-unreachable rule (Q004): Detect code after return/throw/break/continue that can never execute

use std::ops::ControlFlow;

use swc_common::Spanned;
use swc_ecma_ast::{
    ArrowExpr, BlockStmt, BlockStmtOrExpr, DoWhileStmt, ForInStmt, ForOfStmt, ForStmt, Function,
    IfStmt, LabeledStmt, Stmt, SwitchCase, SwitchStmt, TryStmt, WhileStmt,
};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoUnreachable,
    id = "Q004",
    name = "no-unreachable",
    description = "Disallow unreachable code after return, throw, break, or continue statements",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nfunction foo() {\n  return 1;\n  const x = 2; // unreachable\n}\n\n// Good\nfunction foo() {\n  if (condition) {\n    return 1;\n  }\n  const x = 2; // reachable\n}"
);

impl Rule for NoUnreachable {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoUnreachableVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct NoUnreachableVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl<'a> NoUnreachableVisitor<'a> {
    fn check_block(&mut self, block: &BlockStmt) {
        self.check_statements(&block.stmts);
    }

    fn check_statements(&mut self, stmts: &[Stmt]) {
        let mut terminated = false;

        for stmt in stmts {
            if terminated {
                if Self::is_function_declaration(stmt) {
                    continue;
                }

                let (line, column) = self.ctx.span_to_location(stmt.span());
                let diagnostic = Diagnostic::new(
                    "Q004",
                    Severity::Warning,
                    "Unreachable code detected",
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion("Remove unreachable code or check the control flow".to_string());

                self.diagnostics.push(diagnostic);
                continue;
            }

            if self.is_terminating_statement(stmt) {
                terminated = true;
            }

            self.check_nested_scopes(stmt);
        }
    }

    fn is_function_declaration(stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Decl(swc_ecma_ast::Decl::Fn(_)))
    }

    fn is_terminating_statement(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) | Stmt::Throw(_) | Stmt::Break(_) | Stmt::Continue(_) => true,
            Stmt::If(if_stmt) => self.if_always_terminates(if_stmt),
            Stmt::Switch(switch_stmt) => self.switch_always_terminates(switch_stmt),
            Stmt::Try(try_stmt) => self.try_always_terminates(try_stmt),
            Stmt::Block(block) => self.block_always_terminates(block),
            Stmt::Labeled(labeled) => self.is_terminating_statement(&labeled.body),
            _ => false,
        }
    }

    fn if_always_terminates(&self, if_stmt: &IfStmt) -> bool {
        let cons_terminates = self.stmt_always_terminates(&if_stmt.cons);

        if let Some(alt) = &if_stmt.alt {
            let alt_terminates = self.stmt_always_terminates(alt);
            cons_terminates && alt_terminates
        } else {
            false
        }
    }

    fn switch_always_terminates(&self, switch_stmt: &SwitchStmt) -> bool {
        if switch_stmt.cases.is_empty() {
            return false;
        }

        let has_default = switch_stmt.cases.iter().any(|c| c.test.is_none());
        if !has_default {
            return false;
        }

        switch_stmt
            .cases
            .iter()
            .all(|case| self.case_terminates(case))
    }

    fn case_terminates(&self, case: &SwitchCase) -> bool {
        for stmt in &case.cons {
            if self.is_terminating_statement(stmt) {
                return true;
            }
        }
        false
    }

    fn try_always_terminates(&self, try_stmt: &TryStmt) -> bool {
        if let Some(finalizer) = &try_stmt.finalizer {
            if self.block_always_terminates(finalizer) {
                return true;
            }
        }

        let try_terminates = self.block_always_terminates(&try_stmt.block);

        if let Some(handler) = &try_stmt.handler {
            let catch_terminates = self.block_always_terminates(&handler.body);
            try_terminates && catch_terminates
        } else {
            try_terminates
        }
    }

    fn block_always_terminates(&self, block: &BlockStmt) -> bool {
        block
            .stmts
            .iter()
            .any(|stmt| self.is_terminating_statement(stmt))
    }

    fn stmt_always_terminates(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Block(block) => self.block_always_terminates(block),
            _ => self.is_terminating_statement(stmt),
        }
    }

    fn check_nested_scopes(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => self.check_block(block),
            Stmt::If(if_stmt) => self.check_if_stmt(if_stmt),
            Stmt::While(while_stmt) => self.check_while_stmt(while_stmt),
            Stmt::DoWhile(do_while) => self.check_do_while_stmt(do_while),
            Stmt::For(for_stmt) => self.check_for_stmt(for_stmt),
            Stmt::ForIn(for_in) => self.check_for_in_stmt(for_in),
            Stmt::ForOf(for_of) => self.check_for_of_stmt(for_of),
            Stmt::Switch(switch_stmt) => self.check_switch_stmt(switch_stmt),
            Stmt::Try(try_stmt) => self.check_try_stmt(try_stmt),
            Stmt::Labeled(labeled) => self.check_labeled_stmt(labeled),
            _ => {}
        }
    }

    fn check_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.check_stmt(&if_stmt.cons);
        if let Some(alt) = &if_stmt.alt {
            self.check_stmt(alt);
        }
    }

    fn check_while_stmt(&mut self, while_stmt: &WhileStmt) {
        self.check_stmt(&while_stmt.body);
    }

    fn check_do_while_stmt(&mut self, do_while: &DoWhileStmt) {
        self.check_stmt(&do_while.body);
    }

    fn check_for_stmt(&mut self, for_stmt: &ForStmt) {
        self.check_stmt(&for_stmt.body);
    }

    fn check_for_in_stmt(&mut self, for_in: &ForInStmt) {
        self.check_stmt(&for_in.body);
    }

    fn check_for_of_stmt(&mut self, for_of: &ForOfStmt) {
        self.check_stmt(&for_of.body);
    }

    fn check_switch_stmt(&mut self, switch_stmt: &SwitchStmt) {
        for case in &switch_stmt.cases {
            self.check_case(case);
        }
    }

    fn check_case(&mut self, case: &SwitchCase) {
        let mut terminated = false;

        for stmt in &case.cons {
            if terminated {
                if Self::is_function_declaration(stmt) {
                    continue;
                }

                let (line, column) = self.ctx.span_to_location(stmt.span());
                let diagnostic = Diagnostic::new(
                    "Q004",
                    Severity::Warning,
                    "Unreachable code detected",
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion("Remove unreachable code or check the control flow".to_string());

                self.diagnostics.push(diagnostic);
                continue;
            }

            if self.is_terminating_statement(stmt) {
                terminated = true;
            }

            self.check_nested_scopes(stmt);
        }
    }

    fn check_try_stmt(&mut self, try_stmt: &TryStmt) {
        self.check_block(&try_stmt.block);
        if let Some(handler) = &try_stmt.handler {
            self.check_block(&handler.body);
        }
        if let Some(finalizer) = &try_stmt.finalizer {
            self.check_block(finalizer);
        }
    }

    fn check_labeled_stmt(&mut self, labeled: &LabeledStmt) {
        self.check_stmt(&labeled.body);
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Block(block) = stmt {
            self.check_block(block);
        } else {
            self.check_nested_scopes(stmt);
        }
    }

    fn check_function_body(&mut self, body: Option<&BlockStmt>) {
        if let Some(block) = body {
            self.check_block(block);
        }
    }

    fn check_arrow_body(&mut self, body: &BlockStmtOrExpr) {
        if let BlockStmtOrExpr::BlockStmt(block) = body {
            self.check_block(block);
        }
    }
}

impl AstVisitor for NoUnreachableVisitor<'_> {
    fn visit_function(&mut self, node: &Function, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_function_body(node.body.as_ref());
        ControlFlow::Continue(())
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_arrow_body(&node.body);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_rule(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoUnreachable::new();
        rule.check(&file)
    }

    #[test]
    fn no_unreachable_code_no_warning() {
        let code = r#"
function foo() {
    const x = 1;
    return x;
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn code_after_return_warns() {
        let code = r#"
function foo() {
    return 1;
    const x = 2;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q004");
        assert!(diagnostics[0].message.contains("Unreachable"));
    }

    #[test]
    fn code_after_throw_warns() {
        let code = r#"
function foo() {
    throw new Error("error");
    const x = 2;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q004");
    }

    #[test]
    fn code_after_conditional_return_no_warning() {
        let code = r#"
function foo(x) {
    if (x) {
        return 1;
    }
    const y = 2;
    return y;
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn code_after_if_else_both_return_warns() {
        let code = r#"
function foo(x) {
    if (x) {
        return 1;
    } else {
        return 2;
    }
    const y = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q004");
    }

    #[test]
    fn unreachable_in_nested_block() {
        let code = r#"
function foo() {
    if (true) {
        return 1;
        const x = 2;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn code_after_break_in_loop_warns() {
        let code = r#"
function foo() {
    for (let i = 0; i < 10; i++) {
        break;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn code_after_continue_in_loop_warns() {
        let code = r#"
function foo() {
    for (let i = 0; i < 10; i++) {
        continue;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn switch_case_with_break() {
        let code = r#"
function foo(x) {
    switch (x) {
        case 1:
            return 1;
            const y = 2;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn switch_with_default_all_return_warns() {
        let code = r#"
function foo(x) {
    switch (x) {
        case 1:
            return 1;
        default:
            return 2;
    }
    const y = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn switch_without_default_no_warning() {
        let code = r#"
function foo(x) {
    switch (x) {
        case 1:
            return 1;
        case 2:
            return 2;
    }
    const y = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn try_catch_both_return_warns() {
        let code = r#"
function foo() {
    try {
        return 1;
    } catch (e) {
        return 2;
    }
    const x = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn try_without_catch_return_warns() {
        let code = r#"
function foo() {
    try {
        return 1;
    } finally {
        console.log("cleanup");
    }
    const x = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn finally_return_warns() {
        let code = r#"
function foo() {
    try {
        console.log("try");
    } finally {
        return 1;
    }
    const x = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn unreachable_inside_try_block() {
        let code = r#"
function foo() {
    try {
        return 1;
        const x = 2;
    } catch (e) {
        console.log(e);
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn function_declaration_after_return_allowed() {
        let code = r#"
function foo() {
    return 1;
    function bar() {
        return 2;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn while_loop_unreachable() {
        let code = r#"
function foo() {
    while (true) {
        break;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn do_while_loop_unreachable() {
        let code = r#"
function foo() {
    do {
        break;
        const x = 1;
    } while (true);
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn for_in_loop_unreachable() {
        let code = r#"
function foo(obj) {
    for (const key in obj) {
        break;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn for_of_loop_unreachable() {
        let code = r#"
function foo(arr) {
    for (const item of arr) {
        break;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn arrow_function_unreachable() {
        let code = r#"
const foo = () => {
    return 1;
    const x = 2;
};
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn arrow_function_expression_body_no_warning() {
        let code = r#"
const foo = () => 1;
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn multiple_unreachable_statements() {
        let code = r#"
function foo() {
    return 1;
    const x = 2;
    const y = 3;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn labeled_statement_unreachable() {
        let code = r#"
function foo() {
    label: {
        break label;
        const x = 1;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn nested_if_all_branches_return() {
        let code = r#"
function foo(a, b) {
    if (a) {
        if (b) {
            return 1;
        } else {
            return 2;
        }
    } else {
        return 3;
    }
    const x = 4;
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn nested_if_partial_return_no_warning() {
        let code = r#"
function foo(a, b) {
    if (a) {
        if (b) {
            return 1;
        }
    } else {
        return 3;
    }
    const x = 4;
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn class_method_unreachable() {
        let code = r#"
class Test {
    method() {
        return 1;
        const x = 2;
    }
}
"#;
        let diagnostics = run_rule(code);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoUnreachable::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q004");
        assert_eq!(metadata.name, "no-unreachable");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn empty_function_no_warning() {
        let diagnostics = run_rule("function empty() {}");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn return_at_end_no_warning() {
        let code = r#"
function foo() {
    const x = 1;
    const y = 2;
    return x + y;
}
"#;
        let diagnostics = run_rule(code);
        assert!(diagnostics.is_empty());
    }
}
