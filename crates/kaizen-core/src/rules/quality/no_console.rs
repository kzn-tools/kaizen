//! no-console rule (Q032): Detects usage of console.* calls
//!
//! This rule is skipped in:
//! - Example files (*.example.js, example.js, examples/*)
//! - CLI tools (cli.js, bin/*, *-cli/*, *-cli.js)
//! - Scripts (scripts/*)
//! - Test files

use std::ops::ControlFlow;

use swc_ecma_ast::{CallExpr, Callee, Expr, MemberProp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

/// Check if the filename indicates a file where console.* is allowed
fn is_console_allowed_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();

    // Example files - console is expected for demonstration
    lower.contains(".example.")
        || lower.contains("/example.")
        || lower.ends_with("/example.js")
        || lower.ends_with("/example.ts")
        || lower.contains("/examples/")
        || lower.starts_with("examples/")
        // CLI tools - console output is expected
        || lower.contains("/cli.")
        || lower.ends_with("/cli.js")
        || lower.ends_with("/cli.ts")
        || lower.contains("-cli/")
        || lower.contains("-cli.js")
        || lower.contains("-cli.ts")
        || lower.contains("-cli.mjs")
        || lower.contains("/commands/")
        || lower.starts_with("commands/")
        || lower.contains("/bin/")
        || lower.starts_with("bin/")
        // Scripts - console output is expected
        || lower.contains("/scripts/")
        || lower.starts_with("scripts/")
        // Dev tools
        || lower.contains("/dev/")
        // Test files - console is often used for debugging
        || lower.contains(".test.")
        || lower.contains(".spec.")
        || lower.contains("_test.")
        || lower.contains("_spec.")
        || lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
        || lower.starts_with("test/")
        || lower.starts_with("tests/")
}

declare_rule!(
    NoConsole,
    id = "Q032",
    name = "no-console",
    description = "Disallow console.* calls in production code",
    category = Quality,
    severity = Info,
    examples = "// Bad\nconsole.log('debug');\nconsole.error('error');\n\n// Good\n// Use a proper logging library\nlogger.info('message');"
);

impl Rule for NoConsole {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        // Skip files where console.* is expected
        if is_console_allowed_file(&file.metadata().filename) {
            return Vec::new();
        }

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

                        let (line, column, end_line, end_column) = ctx.span_to_range(node.span);
                        let diagnostic = Diagnostic::new(
                            "Q032",
                            Severity::Info,
                            format!("Unexpected console.{} call", method_name),
                            &self.file_path,
                            line,
                            column,
                        )
                        .with_end(end_line, end_column)
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

    // === Exception tests for allowed file types ===

    fn run_no_console_with_filename(filename: &str, code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source(filename, code);
        let rule = NoConsole::new();
        rule.check(&file)
    }

    #[test]
    fn allows_console_in_example_file() {
        let code = "console.log('demo');";
        let diagnostics = run_no_console_with_filename("src/example.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in example.js files"
        );
    }

    #[test]
    fn allows_console_in_dot_example_file() {
        let code = "console.log('demo');";
        let diagnostics = run_no_console_with_filename("src/app.example.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in .example.js files"
        );
    }

    #[test]
    fn allows_console_in_examples_directory() {
        let code = "console.log('demo');";
        let diagnostics = run_no_console_with_filename("examples/basic/index.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in examples/ directory"
        );
    }

    #[test]
    fn allows_console_in_cli_file() {
        let code = "console.log('cli output');";
        let diagnostics = run_no_console_with_filename("src/cli.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in cli.js files"
        );
    }

    #[test]
    fn allows_console_in_bin_directory() {
        let code = "console.log('cli output');";
        let diagnostics = run_no_console_with_filename("bin/my-tool.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in bin/ directory"
        );
    }

    #[test]
    fn allows_console_in_scripts_directory() {
        let code = "console.log('build script');";
        let diagnostics = run_no_console_with_filename("scripts/build.js", code);

        assert!(
            diagnostics.is_empty(),
            "console.log should be allowed in scripts/ directory"
        );
    }

    #[test]
    fn still_detects_in_regular_source() {
        let code = "console.log('debug');";
        let diagnostics = run_no_console_with_filename("src/utils/helper.js", code);

        assert!(
            !diagnostics.is_empty(),
            "console.log should still be detected in regular source files"
        );
    }

    #[test]
    fn test_is_console_allowed_file_function() {
        // Files where console is allowed
        assert!(is_console_allowed_file("src/example.js"));
        assert!(is_console_allowed_file("app.example.ts"));
        assert!(is_console_allowed_file("examples/demo/index.js"));
        assert!(is_console_allowed_file("src/cli.js"));
        assert!(is_console_allowed_file("bin/tool"));
        assert!(is_console_allowed_file("scripts/build.js"));

        // Files where console should be flagged
        assert!(!is_console_allowed_file("src/utils.js"));
        assert!(!is_console_allowed_file("lib/helper.ts"));
        assert!(!is_console_allowed_file("components/Button.jsx"));
    }
}
