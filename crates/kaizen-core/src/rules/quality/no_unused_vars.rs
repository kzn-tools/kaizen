//! no-unused-vars rule (Q001): Detects variables declared but never used
//!
//! This rule uses SemanticModel for scope-aware unused variable detection.
//! It correctly handles:
//! - Variables used in closures (cross-scope references)
//! - Underscore-prefixed variables (intentionally unused)
//! - Write-only variables (assigned but never read)
//! - React import in files containing JSX (legacy JSX transform)
//! - Function parameters before a used parameter ("args: after-used" pattern)
//! - Catch clause parameters - often intentionally unused

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use swc_common::Span;
use swc_ecma_ast::{AssignTarget, JSXElement, SimpleAssignTarget};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::semantic::scope::{ScopeId, ScopeKind};
use crate::semantic::symbols::{Symbol, SymbolKind};
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

/// Collect parameters that should be ignored because they precede a used parameter.
/// This implements the ESLint "args: after-used" pattern.
fn collect_ignored_params(
    symbols: impl Iterator<Item = impl std::borrow::Borrow<Symbol>>,
) -> HashSet<Span> {
    // Group parameters by scope (function)
    let mut params_by_scope: HashMap<ScopeId, Vec<(Span, bool)>> = HashMap::new();

    for symbol in symbols {
        let symbol = symbol.borrow();
        if symbol.kind == SymbolKind::Parameter {
            let is_used = !symbol.references.is_empty();
            params_by_scope
                .entry(symbol.scope)
                .or_default()
                .push((symbol.span, is_used));
        }
    }

    let mut ignored = HashSet::new();

    // For each function scope, find parameters that precede any used parameter
    for (_scope_id, mut params) in params_by_scope {
        // Sort by span position (byte offset) to get declaration order
        params.sort_by_key(|(span, _)| span.lo.0);

        // Find the position of the last used parameter
        let last_used_idx = params.iter().rposition(|(_, is_used)| *is_used);

        if let Some(last_idx) = last_used_idx {
            // All parameters before the last used one should be ignored
            for (span, is_used) in params.iter().take(last_idx) {
                if !is_used {
                    ignored.insert(*span);
                }
            }
        }
    }

    ignored
}

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

        // In files containing JSX, React import is allowed even if not explicitly used
        // (required for JSX transformation in React < 17)
        // We detect JSX presence in the AST rather than relying on file extension
        let contains_jsx = file_contains_jsx(module, &ctx);

        // Collect parameters to ignore (those preceding a used parameter)
        let ignored_params = collect_ignored_params(semantic.symbol_table.all_symbols());

        let mut diagnostics = Vec::new();
        let file_path = file.metadata().filename.clone();

        for symbol in semantic.symbol_table.all_symbols() {
            if symbol.is_exported {
                continue;
            }

            if symbol.name.starts_with('_') {
                continue;
            }

            // Allow React import in files containing JSX (legacy JSX transform requirement)
            if contains_jsx && symbol.name == "React" && symbol.kind == SymbolKind::Import {
                continue;
            }

            // Skip parameters that precede a used parameter (API-imposed unused params)
            if symbol.kind == SymbolKind::Parameter && ignored_params.contains(&symbol.span) {
                continue;
            }

            // Skip catch clause parameters - often intentionally unused for empty catch blocks
            // e.g., catch (e) {} or catch (error) {}
            let scope = semantic.scope_tree.get(symbol.scope);
            if symbol.kind == SymbolKind::Parameter && scope.kind == ScopeKind::Catch {
                continue;
            }

            let is_unused = symbol.references.is_empty();
            let is_write_only = !symbol.references.is_empty()
                && symbol
                    .references
                    .iter()
                    .all(|span| write_only_collector.write_only_spans.contains(span));

            if is_unused || is_write_only {
                let (line, column, end_line, end_column) = ctx.span_to_range(symbol.span);

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
                .with_end(end_line, end_column)
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

/// Check if the module contains any JSX elements
fn file_contains_jsx(module: &swc_ecma_ast::Module, ctx: &VisitorContext) -> bool {
    let mut visitor = JsxDetector { found_jsx: false };
    walk_ast(module, &mut visitor, ctx);
    visitor.found_jsx
}

struct JsxDetector {
    found_jsx: bool,
}

impl AstVisitor for JsxDetector {
    fn visit_jsx_element(&mut self, _node: &JSXElement, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.found_jsx = true;
        // Stop early - we only need to find one JSX element
        ControlFlow::Break(())
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

    // Note: We intentionally don't mark UpdateExpr (++x, --x) as write-only.
    // Update expressions always READ the current value before modifying it,
    // and often the result is used (e.g., `if (--n)` or `arr[i++]`).
    // Only pure assignments (x = value) should be considered write-only.
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
        // Note: We intentionally don't flag update expressions (++, --) as write-only
        // because they always read the current value before modifying it, and the
        // result is often used (e.g., `if (--n)` or `arr[i++]`).
        // This is a trade-off to avoid false positives on patterns like `if (--n)`.
        let code = r#"
let counter = 0;
counter++;
counter++;
"#;
        let diagnostics = run_no_unused_vars(code);

        // counter++ reads the value internally, so it's not purely write-only
        assert!(diagnostics.is_empty());
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

    #[test]
    fn variable_used_in_condition_and_throw() {
        let code = r#"
const code = 1
if (code !== 0) {
  throw code
}
"#;
        let diagnostics = run_no_unused_vars(code);

        println!(
            "Diagnostics: {:?}",
            diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
        );

        // code IS used in condition and throw - should NOT be flagged
        assert!(
            diagnostics.is_empty(),
            "code should not be flagged as unused"
        );
    }

    // === React import exception tests ===

    fn run_no_unused_vars_jsx(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("component.jsx", code);
        let rule = NoUnusedVars::new();
        rule.check(&file)
    }

    fn run_no_unused_vars_tsx(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("component.tsx", code);
        let rule = NoUnusedVars::new();
        rule.check(&file)
    }

    #[test]
    fn allows_react_import_in_jsx() {
        let code = r#"
import React from 'react';
const element = <div>Hello</div>;
"#;
        let diagnostics = run_no_unused_vars_jsx(code);

        // Filter to only React-related diagnostics
        let react_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("React"))
            .collect();

        assert!(
            react_diagnostics.is_empty(),
            "React import should be allowed in JSX files (legacy JSX transform)"
        );
    }

    #[test]
    fn allows_react_import_in_tsx() {
        let code = r#"
import React from 'react';
const element: JSX.Element = <div>Hello</div>;
"#;
        let diagnostics = run_no_unused_vars_tsx(code);

        let react_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("React"))
            .collect();

        assert!(
            react_diagnostics.is_empty(),
            "React import should be allowed in TSX files (legacy JSX transform)"
        );
    }

    #[test]
    fn still_detects_other_unused_vars_in_jsx() {
        let code = r#"
import React from 'react';
const unused = 1;
const element = <div>Hello</div>;
console.log(element);
"#;
        let diagnostics = run_no_unused_vars_jsx(code);

        // Should detect 'unused' but not 'React' or 'element'
        assert_eq!(
            diagnostics.len(),
            1,
            "Should detect unused variable but not React"
        );
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn detects_react_import_in_non_jsx_file() {
        let code = r#"
import React from 'react';
console.log('no jsx here');
"#;
        let diagnostics = run_no_unused_vars(code);

        let react_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("React"))
            .collect();

        assert_eq!(
            react_diagnostics.len(),
            1,
            "React import should be flagged in non-JSX files"
        );
    }

    // === Callback parameter exception tests (args: after-used) ===

    #[test]
    fn ignores_unused_param_before_used_param() {
        // ESLint "args: after-used" pattern
        // item is unused but index is used, so item should be ignored
        let code = r#"
const items = [1, 2, 3];
items.forEach((item, index) => {
    console.log(index);
});
"#;
        let diagnostics = run_no_unused_vars(code);

        println!(
            "All diagnostics: {:?}",
            diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
        );

        let item_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("item"))
            .collect();

        assert!(
            item_diagnostics.is_empty(),
            "Unused param before used param should be ignored (args: after-used)"
        );
    }

    #[test]
    fn ignores_multiple_unused_params_before_used() {
        // Similar to Passport.js strategy callback pattern
        let code = r#"
new Strategy(conf, async (accessToken, refreshToken, profile, done) => {
    console.log(profile);
    done(null, profile);
});
"#;
        let diagnostics = run_no_unused_vars(code);

        // accessToken and refreshToken are unused but precede profile which is used
        let token_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("accessToken") || d.message.contains("refreshToken"))
            .collect();

        assert!(
            token_diagnostics.is_empty(),
            "Unused params before used param should be ignored"
        );
    }

    #[test]
    fn still_detects_unused_params_after_last_used() {
        let code = r#"
function test(a, b, c) {
    console.log(a);
}
"#;
        let diagnostics = run_no_unused_vars(code);

        // b and c are after a (the last used), so they should be flagged
        let bc_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("'b'") || d.message.contains("'c'"))
            .collect();

        assert_eq!(
            bc_diagnostics.len(),
            2,
            "Unused params after last used should be detected"
        );
    }

    #[test]
    fn detects_all_unused_params_when_none_used() {
        let code = r#"
function test(a, b, c) {
    console.log("no params used");
}
"#;
        let diagnostics = run_no_unused_vars(code);

        // All parameters should be flagged
        let param_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                d.message.contains("'a'") || d.message.contains("'b'") || d.message.contains("'c'")
            })
            .collect();

        assert_eq!(
            param_diagnostics.len(),
            3,
            "All unused params should be detected when none are used"
        );
    }

    #[test]
    fn handles_arrow_function_callback() {
        let code = r#"
const arr = [1, 2, 3];
const result = arr.map((item, idx, array) => {
    return array.length;
});
console.log(result);
"#;
        let diagnostics = run_no_unused_vars(code);

        // item and idx are before array (which is used), so should be ignored
        let ignored: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("item") || d.message.contains("idx"))
            .collect();

        assert!(
            ignored.is_empty(),
            "Unused params before used param in arrow function should be ignored"
        );
    }

    #[test]
    fn all_params_flagged_when_none_used_in_callback() {
        // When no parameter is used, all should be flagged
        let code = r#"
const items = [1, 2, 3];
items.forEach((item, index) => {
    console.log("nothing used");
});
"#;
        let diagnostics = run_no_unused_vars(code);

        // Both should be flagged since neither is used
        let param_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("item") || d.message.contains("index"))
            .collect();

        assert_eq!(
            param_diagnostics.len(),
            2,
            "All params should be flagged when none are used"
        );
    }

    #[test]
    fn preserves_underscore_prefix_rule() {
        // Underscore prefix should still work for params
        let code = r#"
items.forEach((_item, index) => {
    console.log(index);
});
"#;
        let diagnostics = run_no_unused_vars(code);

        assert!(
            diagnostics.is_empty(),
            "Underscore prefix should still allow unused params"
        );
    }

    // === Catch clause parameter exception tests ===

    #[test]
    fn ignores_unused_catch_param() {
        let code = r#"
try {
    doSomething();
} catch (error) {
    // Intentionally ignoring the error
}
"#;
        let diagnostics = run_no_unused_vars(code);

        let catch_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("error"))
            .collect();

        assert!(
            catch_diagnostics.is_empty(),
            "Unused catch parameter should be ignored"
        );
    }

    #[test]
    fn ignores_unused_catch_param_e() {
        let code = r#"
try {
    riskyOperation();
} catch (e) {}
"#;
        let diagnostics = run_no_unused_vars(code);

        let catch_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("'e'"))
            .collect();

        assert!(
            catch_diagnostics.is_empty(),
            "Unused catch parameter 'e' should be ignored"
        );
    }

    #[test]
    fn ignores_unused_catch_param_err() {
        let code = r#"
try {
    doWork();
} catch (err) {
    handleDefault();
}
"#;
        let diagnostics = run_no_unused_vars(code);

        let catch_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("err"))
            .collect();

        assert!(
            catch_diagnostics.is_empty(),
            "Unused catch parameter 'err' should be ignored"
        );
    }
}
