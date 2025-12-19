//! no-unused-vars rule (Q001): Detects variables declared but never used
//!
//! This rule uses SemanticModel for scope-aware unused variable detection.
//! It correctly handles:
//! - Variables used in closures (cross-scope references)
//! - Underscore-prefixed variables (intentionally unused)
//! - Write-only variables (assigned but never read)

use std::collections::HashSet;
use std::ops::ControlFlow;

use swc_common::Span;
use swc_ecma_ast::{AssignTarget, Expr, SimpleAssignTarget};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::semantic::visitor::ScopeBuilder;
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoUnusedVars,
    id = "Q001",
    name = "no-unused-vars",
    description = "Disallow unused variables",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nconst unused = 1;\n\n// Good\nconst used = 1;\nconsole.log(used);\n\n// Allowed (underscore prefix)\nconst _intentionallyUnused = 1;"
);

impl Rule for NoUnusedVars {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let semantic = ScopeBuilder::build(module);

        let mut write_only_collector = WriteOnlyCollector {
            write_only_spans: HashSet::new(),
        };
        walk_ast(module, &mut write_only_collector, &ctx);

        let mut diagnostics = Vec::new();
        let file_path = file.metadata().filename.clone();

        for symbol in semantic.symbol_table.all_symbols() {
            if symbol.is_exported {
                continue;
            }

            if symbol.name.starts_with('_') {
                continue;
            }

            let is_unused = symbol.references.is_empty();
            let is_write_only = !symbol.references.is_empty()
                && symbol
                    .references
                    .iter()
                    .all(|span| write_only_collector.write_only_spans.contains(span));

            if is_unused || is_write_only {
                let (line, column) = ctx.span_to_location(symbol.span);

                let message = if is_write_only {
                    format!("'{}' is assigned a value but never read", symbol.name)
                } else {
                    format!("'{}' is declared but never used", symbol.name)
                };

                let diagnostic = Diagnostic::new(
                    "Q001",
                    Severity::Warning,
                    message,
                    &file_path,
                    line,
                    column,
                )
                .with_suggestion(format!(
                    "Remove unused variable '{}' or prefix with underscore if intentionally unused",
                    symbol.name
                ));

                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }
}

struct WriteOnlyCollector {
    write_only_spans: HashSet<Span>,
}

impl AstVisitor for WriteOnlyCollector {
    fn visit_assign_expr(
        &mut self,
        node: &swc_ecma_ast::AssignExpr,
        _ctx: &VisitorContext,
    ) -> ControlFlow<()> {
        if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = &node.left {
            self.write_only_spans.insert(ident.span);
        }
        ControlFlow::Continue(())
    }

    fn visit_update_expr(
        &mut self,
        node: &swc_ecma_ast::UpdateExpr,
        _ctx: &VisitorContext,
    ) -> ControlFlow<()> {
        if let Expr::Ident(ident) = node.arg.as_ref() {
            self.write_only_spans.insert(ident.span);
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_no_unused_vars(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoUnusedVars::new();
        rule.check(&file)
    }

    #[test]
    fn detects_unused_const() {
        let diagnostics = run_no_unused_vars("const x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q001");
        assert!(diagnostics[0].message.contains("x"));
        assert!(
            diagnostics[0].message.contains("unused")
                || diagnostics[0].message.contains("never used")
        );
    }

    #[test]
    fn ignores_used_variable() {
        let diagnostics = run_no_unused_vars("const x = 1; console.log(x);");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unused_param() {
        let code = r#"
function foo(unusedParam) {
    return 42;
}
foo();
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unusedParam"));
    }

    #[test]
    fn ignores_exported_variable() {
        let diagnostics = run_no_unused_vars("export const x = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_underscore_prefix() {
        let diagnostics = run_no_unused_vars("const _unused = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unused_let() {
        let diagnostics = run_no_unused_vars("let y = 2;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q001");
        assert!(diagnostics[0].message.contains("y"));
    }

    #[test]
    fn detects_unused_var() {
        let diagnostics = run_no_unused_vars("var z = 3;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q001");
        assert!(diagnostics[0].message.contains("z"));
    }

    #[test]
    fn detects_multiple_unused_variables() {
        let code = r#"
const a = 1;
let b = 2;
var c = 3;
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn ignores_used_in_expression() {
        let code = r#"
const x = 10;
const y = x + 5;
console.log(y);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_used_arrow_function_param() {
        let code = r#"
const add = (a, b) => a + b;
add(1, 2);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_unused_arrow_function_param() {
        let code = r#"
const greet = (name, unused) => console.log(name);
greet("hello", "world");
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoUnusedVars::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q001");
        assert_eq!(metadata.name, "no-unused-vars");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn suggestion_provided() {
        let diagnostics = run_no_unused_vars("const x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].suggestion.is_some());
    }

    // === Acceptance Criteria Tests ===

    #[test]
    fn closure_variable_not_flagged() {
        let code = r#"
function createCounter() {
    let count = 0;
    return function() {
        count++;
        return count;
    };
}
createCounter();
"#;
        let diagnostics = run_no_unused_vars(code);

        let count_unused = diagnostics.iter().any(|d| d.message.contains("count"));
        assert!(
            !count_unused,
            "Variable used in closure should not be flagged"
        );
    }

    #[test]
    fn closure_with_arrow_function() {
        let code = r#"
function outer() {
    const value = 42;
    const inner = () => value * 2;
    return inner;
}
outer();
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Variables used in arrow function closures should not be flagged"
        );
    }

    #[test]
    fn nested_closure_variable_not_flagged() {
        let code = r#"
function outer(a) {
    function middle(b) {
        function inner(c) {
            return a + b + c;
        }
        return inner;
    }
    return middle;
}
outer(1);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Variables used across nested closures should not be flagged"
        );
    }

    #[test]
    fn underscore_prefix_respected() {
        let code = r#"
const _unused1 = 1;
let _unused2 = 2;
var _unused3 = 3;
function foo(_unusedParam) {
    return 42;
}
foo(1);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Variables with underscore prefix should not be flagged"
        );
    }

    #[test]
    fn write_only_variable_detected() {
        let code = r#"
let x = 1;
x = 2;
x = 3;
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("x"));
        assert!(
            diagnostics[0].message.contains("assigned")
                && diagnostics[0].message.contains("never read"),
            "Write-only variable should have specific message"
        );
    }

    #[test]
    fn write_only_with_increment() {
        let code = r#"
let counter = 0;
counter++;
counter++;
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("counter"));
    }

    #[test]
    fn variable_read_and_written_not_flagged() {
        let code = r#"
let x = 1;
x = x + 1;
console.log(x);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Variable that is both read and written should not be flagged"
        );
    }

    #[test]
    fn shadowed_variable_in_closure() {
        let code = r#"
const x = 1;
function foo() {
    const x = 2;
    return x;
}
console.log(x);
foo();
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Shadowed variables should be tracked separately"
        );
    }

    #[test]
    fn function_expression_in_closure() {
        let code = r#"
const data = [1, 2, 3];
const result = data.map(function(item) {
    return item * 2;
});
console.log(result);
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Parameters in function expressions should be tracked"
        );
    }

    #[test]
    fn callback_parameter_used() {
        let code = r#"
const items = [1, 2, 3];
items.forEach((item, index) => {
    console.log(index, item);
});
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Used callback parameters should not be flagged"
        );
    }

    #[test]
    fn callback_parameter_unused() {
        let code = r#"
const items = [1, 2, 3];
items.forEach((item, index) => {
    console.log(item);
});
"#;
        let diagnostics = run_no_unused_vars(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("index"));
    }
}
