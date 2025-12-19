//! no-var rule (Q030): Detects usage of `var` and suggests `let` or `const`

use std::ops::ControlFlow;

use swc_ecma_ast::{VarDecl, VarDeclKind};

use crate::declare_rule;
use crate::diagnostic::{Diagnostic, Fix};
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoVar,
    id = "Q030",
    name = "no-var",
    description = "Disallow var declarations, use let or const instead",
    category = Quality,
    severity = Warning,
    examples =
        "// Bad\nvar x = 1;\nvar name = 'test';\n\n// Good\nlet x = 1;\nconst name = 'test';"
);

impl Rule for NoVar {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoVarVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct NoVarVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl AstVisitor for NoVarVisitor<'_> {
    fn visit_var_decl(&mut self, node: &VarDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        if node.kind == VarDeclKind::Var {
            let (line, column) = self.ctx.span_to_location(node.span);

            let fix = Fix::replace(
                "Replace 'var' with 'let'",
                "let",
                line,
                column,
                line,
                column + 2,
            );

            let diagnostic = Diagnostic::new(
                "Q030",
                Severity::Warning,
                "Unexpected var, use let or const instead",
                &self.file_path,
                line,
                column,
            )
            .with_end(line, column + 2)
            .with_suggestion("Replace 'var' with 'let' or 'const'")
            .with_fix(fix);

            self.diagnostics.push(diagnostic);
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_no_var(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoVar::new();
        rule.check(&file)
    }

    #[test]
    fn detects_var_declaration() {
        let diagnostics = run_no_var("var x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q030");
        assert_eq!(
            diagnostics[0].message,
            "Unexpected var, use let or const instead"
        );
        assert_eq!(diagnostics[0].line, 1);
        assert!(diagnostics[0].suggestion.is_some());
    }

    #[test]
    fn ignores_let_declaration() {
        let diagnostics = run_no_var("let x = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_const_declaration() {
        let diagnostics = run_no_var("const x = 1;");

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_let_and_const() {
        let code = r#"
let a = 1;
const b = 2;
let c = 3;
"#;
        let diagnostics = run_no_var(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_var_in_nested_scope() {
        let code = r#"
function test() {
    var x = 1;
}
"#;
        let diagnostics = run_no_var(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q030");
    }

    #[test]
    fn detects_multiple_var_declarations() {
        let code = r#"
var a = 1;
var b = 2;
var c = 3;
"#;
        let diagnostics = run_no_var(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn detects_var_in_for_loop() {
        let code = "for (var i = 0; i < 10; i++) {}";
        let diagnostics = run_no_var(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q030");
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoVar::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q030");
        assert_eq!(metadata.name, "no-var");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    #[test]
    fn suggestion_provided() {
        let diagnostics = run_no_var("var x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].suggestion,
            Some("Replace 'var' with 'let' or 'const'".to_string())
        );
    }

    #[test]
    fn fix_provided() {
        let diagnostics = run_no_var("var x = 1;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].fixes.len(), 1);

        let fix = &diagnostics[0].fixes[0];
        assert_eq!(fix.title, "Replace 'var' with 'let'");
        assert!(matches!(
            &fix.kind,
            crate::diagnostic::FixKind::ReplaceWith { new_text } if new_text == "let"
        ));
        assert_eq!(fix.line, 1);
        assert_eq!(fix.column, diagnostics[0].column);
        assert_eq!(fix.end_line, 1);
        assert_eq!(fix.end_column, diagnostics[0].column + 2);
    }
}
