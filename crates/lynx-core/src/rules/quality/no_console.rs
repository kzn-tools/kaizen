//! no-console rule (Q032): Detects usage of console.* calls

use std::ops::ControlFlow;

use swc_ecma_ast::{CallExpr, Callee, Expr, MemberProp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    NoConsole,
    id = "Q032",
    name = "no-console",
    description = "Disallow console.* calls in production code",
    category = Quality,
    severity = Info
);

impl Rule for NoConsole {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = NoConsoleVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct NoConsoleVisitor {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
}

impl AstVisitor for NoConsoleVisitor {
    fn visit_call_expr(&mut self, node: &CallExpr, ctx: &VisitorContext) -> ControlFlow<()> {
        if let Callee::Expr(callee_expr) = &node.callee {
            if let Expr::Member(member_expr) = callee_expr.as_ref() {
                if let Expr::Ident(obj_ident) = member_expr.obj.as_ref() {
                    if obj_ident.sym.as_ref() == "console" {
                        let method_name = match &member_expr.prop {
                            MemberProp::Ident(ident) => ident.sym.to_string(),
                            MemberProp::Computed(_) => "method".to_string(),
                            MemberProp::PrivateName(_) => "method".to_string(),
                        };

                        let (line, column) = ctx.span_to_location(node.span);
                        let diagnostic = Diagnostic::new(
                            "Q032",
                            Severity::Info,
                            format!("Unexpected console.{} call", method_name),
                            &self.file_path,
                            line,
                            column,
                        )
                        .with_suggestion("Remove console call before production");

                        self.diagnostics.push(diagnostic);
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_no_console(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = NoConsole::new();
        rule.check(&file)
    }

    #[test]
    fn detects_console_log() {
        let diagnostics = run_no_console("console.log(\"debug\");");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q032");
        assert_eq!(diagnostics[0].message, "Unexpected console.log call");
        assert_eq!(diagnostics[0].line, 1);
    }

    #[test]
    fn detects_console_warn_error_info() {
        let code = r#"
console.warn("warning");
console.error("error");
console.info("info");
"#;
        let diagnostics = run_no_console(code);

        assert_eq!(diagnostics.len(), 3);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unexpected console.warn call")
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unexpected console.error call")
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unexpected console.info call")
        );
    }

    #[test]
    fn ignores_custom_console() {
        let code = r#"
const myConsole = { log: () => {} };
myConsole.log("test");
"#;
        let diagnostics = run_no_console(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn severity_is_info() {
        let rule = NoConsole::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.severity, Severity::Info);
    }

    #[test]
    fn detects_console_in_nested_scope() {
        let code = r#"
function test() {
    console.log("nested");
}
"#;
        let diagnostics = run_no_console(code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "Q032");
    }

    #[test]
    fn detects_multiple_console_calls() {
        let code = r#"
console.log("one");
console.log("two");
console.log("three");
"#;
        let diagnostics = run_no_console(code);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn detects_console_debug_and_trace() {
        let code = r#"
console.debug("debug");
console.trace("trace");
"#;
        let diagnostics = run_no_console(code);

        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unexpected console.debug call")
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "Unexpected console.trace call")
        );
    }

    #[test]
    fn ignores_other_object_methods() {
        let code = r#"
logger.log("test");
myObj.error("test");
"#;
        let diagnostics = run_no_console(code);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn metadata_is_correct() {
        let rule = NoConsole::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "Q032");
        assert_eq!(metadata.name, "no-console");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Info);
    }

    #[test]
    fn suggestion_provided() {
        let diagnostics = run_no_console("console.log(\"test\");");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].suggestion,
            Some("Remove console call before production".to_string())
        );
    }
}
