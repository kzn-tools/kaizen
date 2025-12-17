//! no-unused-vars rule (Q001): Detects variables declared but never used

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use swc_common::Span;
use swc_ecma_ast::{ArrowExpr, FnDecl, Ident, ModuleDecl, ModuleItem, Pat, VarDecl};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoUnusedVars,
    id = "Q001",
    name = "no-unused-vars",
    description = "Disallow unused variables",
    category = Quality,
    severity = Warning
);

struct Declaration {
    span: Span,
    is_exported: bool,
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

        let exported_names = collect_exported_names(module);

        let mut declaration_visitor = DeclarationCollector {
            declarations: HashMap::new(),
            exported_names: &exported_names,
        };
        walk_ast(module, &mut declaration_visitor, &ctx);

        let mut reference_visitor = ReferenceCollector {
            references: HashSet::new(),
            declaration_spans: declaration_visitor
                .declarations
                .values()
                .map(|d| d.span)
                .collect(),
        };
        walk_ast(module, &mut reference_visitor, &ctx);

        let mut diagnostics = Vec::new();
        let file_path = file.metadata().filename.clone();

        for (name, decl) in declaration_visitor.declarations {
            if decl.is_exported {
                continue;
            }

            if name.starts_with('_') {
                continue;
            }

            if !reference_visitor.references.contains(&name) {
                let (line, column) = ctx.span_to_location(decl.span);
                let diagnostic = Diagnostic::new(
                    "Q001",
                    Severity::Warning,
                    format!("'{}' is declared but never used", name),
                    &file_path,
                    line,
                    column,
                )
                .with_suggestion(format!(
                    "Remove unused variable '{}' or prefix with underscore if intentionally unused",
                    name
                ));

                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }
}

fn collect_exported_names(module: &swc_ecma_ast::Module) -> HashSet<String> {
    let mut exported = HashSet::new();

    for item in &module.body {
        if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) = item {
            if let swc_ecma_ast::Decl::Var(var_decl) = &export_decl.decl {
                for declarator in &var_decl.decls {
                    collect_names_from_pat(&declarator.name, &mut exported);
                }
            }
            if let swc_ecma_ast::Decl::Fn(fn_decl) = &export_decl.decl {
                exported.insert(fn_decl.ident.sym.to_string());
            }
        }
    }

    exported
}

fn collect_names_from_pat(pat: &Pat, names: &mut HashSet<String>) {
    match pat {
        Pat::Ident(binding_ident) => {
            names.insert(binding_ident.id.sym.to_string());
        }
        Pat::Array(array_pat) => {
            for elem in array_pat.elems.iter().flatten() {
                collect_names_from_pat(elem, names);
            }
        }
        Pat::Object(object_pat) => {
            for prop in &object_pat.props {
                match prop {
                    swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                        collect_names_from_pat(&kv.value, names);
                    }
                    swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                        names.insert(assign.key.sym.to_string());
                    }
                    swc_ecma_ast::ObjectPatProp::Rest(rest) => {
                        collect_names_from_pat(&rest.arg, names);
                    }
                }
            }
        }
        Pat::Rest(rest_pat) => {
            collect_names_from_pat(&rest_pat.arg, names);
        }
        Pat::Assign(assign_pat) => {
            collect_names_from_pat(&assign_pat.left, names);
        }
        Pat::Invalid(_) | Pat::Expr(_) => {}
    }
}

struct DeclarationCollector<'a> {
    declarations: HashMap<String, Declaration>,
    exported_names: &'a HashSet<String>,
}

impl AstVisitor for DeclarationCollector<'_> {
    fn visit_var_decl(&mut self, node: &VarDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        for declarator in &node.decls {
            self.collect_from_pat(&declarator.name);
        }
        ControlFlow::Continue(())
    }

    fn visit_fn_decl(&mut self, node: &FnDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        for param in &node.function.params {
            self.collect_from_pat(&param.pat);
        }
        ControlFlow::Continue(())
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        for param in &node.params {
            self.collect_from_pat(param);
        }
        ControlFlow::Continue(())
    }
}

impl DeclarationCollector<'_> {
    fn collect_from_pat(&mut self, pat: &Pat) {
        match pat {
            Pat::Ident(binding_ident) => {
                let name = binding_ident.id.sym.to_string();
                let is_exported = self.exported_names.contains(&name);
                let span = binding_ident.id.span;
                self.declarations
                    .insert(name, Declaration { span, is_exported });
            }
            Pat::Array(array_pat) => {
                for elem in array_pat.elems.iter().flatten() {
                    self.collect_from_pat(elem);
                }
            }
            Pat::Object(object_pat) => {
                for prop in &object_pat.props {
                    match prop {
                        swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                            self.collect_from_pat(&kv.value);
                        }
                        swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                            let name = assign.key.sym.to_string();
                            let is_exported = self.exported_names.contains(&name);
                            let span = assign.key.span;
                            self.declarations
                                .insert(name, Declaration { span, is_exported });
                        }
                        swc_ecma_ast::ObjectPatProp::Rest(rest) => {
                            self.collect_from_pat(&rest.arg);
                        }
                    }
                }
            }
            Pat::Rest(rest_pat) => {
                self.collect_from_pat(&rest_pat.arg);
            }
            Pat::Assign(assign_pat) => {
                self.collect_from_pat(&assign_pat.left);
            }
            Pat::Invalid(_) | Pat::Expr(_) => {}
        }
    }
}

struct ReferenceCollector {
    references: HashSet<String>,
    declaration_spans: HashSet<Span>,
}

impl AstVisitor for ReferenceCollector {
    fn visit_ident(&mut self, node: &Ident, _ctx: &VisitorContext) -> ControlFlow<()> {
        if !self.declaration_spans.contains(&node.span) {
            self.references.insert(node.sym.to_string());
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
}
