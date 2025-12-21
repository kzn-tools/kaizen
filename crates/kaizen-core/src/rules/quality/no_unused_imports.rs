//! no-unused-imports rule (Q003): Detects imports that are declared but never used
//!
//! Exception: In files containing JSX, the `React` import is allowed even if not explicitly
//! used, as it was required for JSX transformation in React versions before 17.

use std::collections::HashSet;

use swc_ecma_ast::{ExportSpecifier, ModuleDecl, ModuleItem};

use crate::declare_rule;
use crate::diagnostic::{Diagnostic, Fix};
use crate::parser::ParsedFile;
use crate::rules::helpers::file_contains_jsx;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::semantic::symbols::SymbolKind;
use crate::semantic::visitor::ScopeBuilder;
use crate::visitor::VisitorContext;

declare_rule!(
    NoUnusedImports,
    id = "Q003",
    name = "no-unused-imports",
    description = "Disallow unused imports",
    category = Quality,
    severity = Warning,
    examples = "// Bad\nimport { unused } from 'module';\n\n// Good\nimport { used } from 'module';\nconsole.log(used);\n\n// Allowed (re-export)\nimport { foo } from 'module';\nexport { foo };"
);

impl Rule for NoUnusedImports {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let semantic = ScopeBuilder::build(module);

        let re_exported_names = collect_re_exported_names(module);

        // In files containing JSX, React import is allowed even if not explicitly used
        // (required for JSX transformation in React < 17)
        // We detect JSX presence in the AST rather than relying on file extension
        let contains_jsx = file_contains_jsx(module, &ctx);

        let mut diagnostics = Vec::new();
        let file_path = file.metadata().filename.clone();

        for symbol in semantic.symbol_table.all_symbols() {
            if symbol.kind != SymbolKind::Import {
                continue;
            }

            if symbol.name.starts_with('_') {
                continue;
            }

            if re_exported_names.contains(&symbol.name) {
                continue;
            }

            // Allow React import in files containing JSX (legacy JSX transform requirement)
            if contains_jsx && symbol.name == "React" {
                continue;
            }

            let is_unused = symbol.references.is_empty();

            if is_unused {
                let (line, column) = ctx.span_to_location(symbol.span);
                let end_column = column + symbol.name.len() - 1;

                let fix = Fix::replace(
                    format!("Remove unused import '{}'", symbol.name),
                    "",
                    line,
                    column,
                    line,
                    end_column,
                );

                let diagnostic = Diagnostic::new(
                    "Q003",
                    Severity::Warning,
                    format!("'{}' is imported but never used", symbol.name),
                    &file_path,
                    line,
                    column,
                )
                .with_end(line, end_column)
                .with_suggestion(format!(
                    "Remove unused import '{}' or prefix with underscore if intentionally unused",
                    symbol.name
                ))
                .with_fix(fix);

                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }
}

fn collect_re_exported_names(module: &swc_ecma_ast::Module) -> HashSet<String> {
    let mut re_exported = HashSet::new();

    for item in &module.body {
        if let ModuleItem::ModuleDecl(decl) = item {
            match decl {
                ModuleDecl::ExportNamed(export) => {
                    for specifier in &export.specifiers {
                        match specifier {
                            ExportSpecifier::Named(named) => {
                                let name = named.orig.atom().to_string();
                                if !name.is_empty() {
                                    re_exported.insert(name);
                                }
                            }
                            ExportSpecifier::Namespace(ns) => {
                                let name = ns.name.atom().to_string();
                                re_exported.insert(name);
                            }
                            ExportSpecifier::Default(_) => {}
                        }
                    }
                }
                ModuleDecl::ExportDefaultExpr(default_export) => {
                    if let Some(ident) = default_export.expr.as_ident() {
                        re_exported.insert(ident.sym.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    re_exported
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_no_unused_imports(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoUnusedImports::new();
        rule.check(&file)
    }

    fn run_no_unused_imports_ts(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.ts", code);
        let rule = NoUnusedImports::new();
        rule.check(&file)
    }

    #[test]
    fn detects_unused_named_import() {
        let diagnostics = run_no_unused_imports("import { unused } from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q003");
        assert!(diagnostics[0].message.contains("unused"));
        assert!(diagnostics[0].message.contains("never used"));
    }

    #[test]
    fn detects_unused_default_import() {
        let diagnostics = run_no_unused_imports("import unused from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q003");
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn detects_unused_namespace_import() {
        let diagnostics = run_no_unused_imports("import * as unused from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q003");
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn ignores_used_named_import() {
        let code = r#"
import { used } from 'module';
console.log(used);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_used_default_import() {
        let code = r#"
import used from 'module';
console.log(used);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_used_namespace_import() {
        let code = r#"
import * as utils from 'module';
console.log(utils.foo);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_underscore_prefixed_import() {
        let diagnostics = run_no_unused_imports("import { _unused } from 'module';");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_unused_imports() {
        let code = r#"
import defaultExport from 'module1';
import { named1, named2 } from 'module2';
import * as namespace from 'module3';
"#;
        let diagnostics = run_no_unused_imports(code);

        assert_eq!(diagnostics.len(), 4);
    }

    #[test]
    fn ignores_re_exported_named_import() {
        let code = r#"
import { foo } from 'module';
export { foo };
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_re_exported_with_rename() {
        let code = r#"
import { foo } from 'module';
export { foo as bar };
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_re_exported_default() {
        let code = r#"
import foo from 'module';
export default foo;
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn mixed_used_and_unused_imports() {
        let code = r#"
import { used, unused } from 'module';
console.log(used);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn import_used_in_function() {
        let code = r#"
import { helper } from 'module';

function foo() {
    return helper();
}
foo();
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn import_used_in_class_method() {
        let code = r#"
import { helper } from 'module';

class MyClass {
    method() {
        return helper();
    }
}
new MyClass();
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn import_used_in_jsx_like_call() {
        let code = r#"
import { Component } from 'react';
const element = Component();
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoUnusedImports::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q003");
        assert_eq!(metadata.name, "no-unused-imports");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn suggestion_provided() {
        let diagnostics = run_no_unused_imports("import { unused } from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].suggestion.is_some());
    }

    #[test]
    fn type_only_import_unused_flagged() {
        let code = "import type { UnusedType } from 'module';";
        let diagnostics = run_no_unused_imports_ts(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("UnusedType"));
    }

    #[test]
    fn type_only_import_used_as_value() {
        let code = r#"
import type { SomeType } from 'module';
const x = SomeType;
"#;
        let diagnostics = run_no_unused_imports_ts(code);

        assert!(
            diagnostics.is_empty(),
            "Type-only import used as value should not be flagged"
        );
    }

    #[test]
    fn import_with_alias_unused() {
        let diagnostics = run_no_unused_imports("import { foo as bar } from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("bar"));
    }

    #[test]
    fn import_with_alias_used() {
        let code = r#"
import { foo as bar } from 'module';
console.log(bar);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn side_effect_import_not_flagged() {
        let diagnostics = run_no_unused_imports("import 'module';");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn import_used_in_object_shorthand() {
        let code = r#"
import { value } from 'module';
const obj = { value };
console.log(obj);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn import_used_in_destructuring() {
        let code = r#"
import { config } from 'module';
const { option } = config;
console.log(option);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn import_used_as_argument() {
        let code = r#"
import { data } from 'module';
process(data);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn re_export_all_from_module() {
        let code = r#"
import { helper } from 'module';
export * from 'module';
"#;
        let diagnostics = run_no_unused_imports(code);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("helper"));
    }

    #[test]
    fn multiple_re_exports() {
        let code = r#"
import { foo, bar, baz } from 'module';
export { foo, bar };
console.log(baz);
"#;
        let diagnostics = run_no_unused_imports(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn fix_provided() {
        let diagnostics = run_no_unused_imports("import { unused } from 'module';");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].fixes.len(), 1);

        let fix = &diagnostics[0].fixes[0];
        assert!(fix.title.contains("Remove unused import"));
        assert!(matches!(
            &fix.kind,
            crate::diagnostic::FixKind::ReplaceWith { new_text } if new_text.is_empty()
        ));
    }

    // === React import exception tests ===

    fn run_no_unused_imports_jsx(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("component.jsx", code);
        let rule = NoUnusedImports::new();
        rule.check(&file)
    }

    fn run_no_unused_imports_tsx(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("component.tsx", code);
        let rule = NoUnusedImports::new();
        rule.check(&file)
    }

    #[test]
    fn allows_react_import_in_jsx() {
        let code = r#"
import React from 'react';
const element = <div>Hello</div>;
"#;
        let diagnostics = run_no_unused_imports_jsx(code);

        assert!(
            diagnostics.is_empty(),
            "React import should be allowed in JSX files (legacy JSX transform)"
        );
    }

    #[test]
    fn allows_react_import_in_tsx() {
        let code = r#"
import React from 'react';
const element: JSX.Element = <div>Hello</div>;
"#;
        let diagnostics = run_no_unused_imports_tsx(code);

        assert!(
            diagnostics.is_empty(),
            "React import should be allowed in TSX files (legacy JSX transform)"
        );
    }

    #[test]
    fn still_detects_other_unused_imports_in_jsx() {
        let code = r#"
import React from 'react';
import { unused } from 'module';
const element = <div>Hello</div>;
"#;
        let diagnostics = run_no_unused_imports_jsx(code);

        assert_eq!(
            diagnostics.len(),
            1,
            "Other unused imports should still be detected in JSX files"
        );
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn detects_react_import_in_non_jsx_file() {
        let code = r#"
import React from 'react';
console.log('no jsx here');
"#;
        let diagnostics = run_no_unused_imports(code);

        assert_eq!(
            diagnostics.len(),
            1,
            "React import should be flagged in non-JSX files"
        );
        assert!(diagnostics[0].message.contains("React"));
    }
}
