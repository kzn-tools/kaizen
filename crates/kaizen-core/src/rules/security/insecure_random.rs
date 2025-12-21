//! no-insecure-random rule (S012): Detects usage of Math.random() for security purposes
//!
//! This rule is skipped in test files (*.test.js, *.spec.js, etc.) where Math.random()
//! is commonly used for generating test data.

use std::ops::ControlFlow;

use swc_ecma_ast::{CallExpr, Callee, Expr, MemberProp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

/// Check if the filename indicates a test file
fn is_test_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();

    // Common test file patterns
    lower.contains(".test.")
        || lower.contains(".spec.")
        || lower.contains("_test.")
        || lower.contains("_spec.")
        || lower.ends_with(".test.js")
        || lower.ends_with(".test.ts")
        || lower.ends_with(".test.jsx")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".spec.js")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.jsx")
        || lower.ends_with(".spec.tsx")
        // test.js files (common pattern in some projects)
        || lower.ends_with("/test.js")
        || lower.ends_with("/test.mjs")
        || lower.ends_with("/test.ts")
        || lower == "test.js"
        || lower == "test.mjs"
        || lower == "test.ts"
        || lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
        || lower.contains("/__mocks__/")
        || lower.starts_with("test/")
        || lower.starts_with("tests/")
}

declare_rule!(
    InsecureRandom,
    id = "S012",
    name = "no-insecure-random",
    description = "Disallow Math.random() for security-sensitive operations",
    category = Security,
    severity = Warning,
    examples = "// Bad\nconst token = Math.random().toString(36);\nconst id = Math.random();\n\n// Good\nconst token = crypto.randomUUID();\nconst bytes = crypto.randomBytes(16);\nconst array = crypto.getRandomValues(new Uint8Array(16));"
);

impl Rule for InsecureRandom {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        // Skip test files - Math.random() is acceptable for test data generation
        if is_test_file(&file.metadata().filename) {
            return Vec::new();
        }

        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = InsecureRandomVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct InsecureRandomVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl InsecureRandomVisitor<'_> {
    fn check_math_random_call(&mut self, call: &CallExpr) {
        let Callee::Expr(callee_expr) = &call.callee else {
            return;
        };

        let Expr::Member(member) = callee_expr.as_ref() else {
            return;
        };

        let MemberProp::Ident(prop) = &member.prop else {
            return;
        };

        if prop.sym.as_ref() != "random" {
            return;
        }

        let Expr::Ident(obj_ident) = member.obj.as_ref() else {
            return;
        };

        if obj_ident.sym.as_ref() != "Math" {
            return;
        }

        let (line, column) = self.ctx.span_to_location(call.span);
        let diagnostic = Diagnostic::new(
            "S012",
            Severity::Warning,
            "Math.random() is not cryptographically secure",
            &self.file_path,
            line,
            column,
        )
        .with_suggestion(
            "Use crypto.randomBytes(), crypto.getRandomValues(), or crypto.randomUUID() instead",
        );
        self.diagnostics.push(diagnostic);
    }
}

impl AstVisitor for InsecureRandomVisitor<'_> {
    fn visit_call_expr(&mut self, node: &CallExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_math_random_call(node);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_insecure_random(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("source.js", code);
        let rule = InsecureRandom::new();
        rule.check(&file)
    }

    #[test]
    fn detects_math_random() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random(code);

        assert!(!diagnostics.is_empty(), "should detect Math.random()");
        assert_eq!(diagnostics[0].rule_id, "S012");
        assert!(diagnostics[0].message.contains("Math.random()"));
    }

    #[test]
    fn detects_math_random_in_expression() {
        let code = r#"const id = Math.random().toString(36).substring(2);"#;
        let diagnostics = run_insecure_random(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect Math.random() in expression"
        );
    }

    #[test]
    fn detects_math_random_in_template() {
        let code = r#"const token = `token_${Math.random()}`;"#;
        let diagnostics = run_insecure_random(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect Math.random() in template"
        );
    }

    #[test]
    fn detects_multiple_math_random() {
        let code = r#"
const a = Math.random();
const b = Math.random();
"#;
        let diagnostics = run_insecure_random(code);

        assert_eq!(
            diagnostics.len(),
            2,
            "should detect multiple Math.random() calls"
        );
    }

    #[test]
    fn allows_crypto_random_bytes() {
        let code = r#"const bytes = crypto.randomBytes(16);"#;
        let diagnostics = run_insecure_random(code);

        assert!(diagnostics.is_empty(), "should allow crypto.randomBytes()");
    }

    #[test]
    fn allows_crypto_get_random_values() {
        let code = r#"const array = crypto.getRandomValues(new Uint8Array(16));"#;
        let diagnostics = run_insecure_random(code);

        assert!(
            diagnostics.is_empty(),
            "should allow crypto.getRandomValues()"
        );
    }

    #[test]
    fn allows_crypto_random_uuid() {
        let code = r#"const uuid = crypto.randomUUID();"#;
        let diagnostics = run_insecure_random(code);

        assert!(diagnostics.is_empty(), "should allow crypto.randomUUID()");
    }

    #[test]
    fn allows_math_floor() {
        let code = r#"const value = Math.floor(100);"#;
        let diagnostics = run_insecure_random(code);

        assert!(diagnostics.is_empty(), "should allow other Math methods");
    }

    #[test]
    fn allows_other_random_implementations() {
        let code = r#"const value = myLib.random();"#;
        let diagnostics = run_insecure_random(code);

        assert!(
            diagnostics.is_empty(),
            "should allow other random implementations"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("crypto")
        );
    }

    #[test]
    fn metadata_is_correct() {
        let rule = InsecureRandom::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S012");
        assert_eq!(metadata.name, "no-insecure-random");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Warning);
    }

    // === Test file exception tests ===

    fn run_insecure_random_with_filename(filename: &str, code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source(filename, code);
        let rule = InsecureRandom::new();
        rule.check(&file)
    }

    #[test]
    fn allows_math_random_in_test_file() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("component.test.js", code);

        assert!(
            diagnostics.is_empty(),
            "Math.random() should be allowed in .test.js files"
        );
    }

    #[test]
    fn allows_math_random_in_spec_file() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("component.spec.ts", code);

        assert!(
            diagnostics.is_empty(),
            "Math.random() should be allowed in .spec.ts files"
        );
    }

    #[test]
    fn allows_math_random_in_tests_directory() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("src/__tests__/utils.js", code);

        assert!(
            diagnostics.is_empty(),
            "Math.random() should be allowed in __tests__ directories"
        );
    }

    #[test]
    fn allows_math_random_in_test_directory() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("test/helpers.js", code);

        assert!(
            diagnostics.is_empty(),
            "Math.random() should be allowed in test/ directories"
        );
    }

    #[test]
    fn allows_math_random_in_mocks_directory() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("src/__mocks__/api.js", code);

        assert!(
            diagnostics.is_empty(),
            "Math.random() should be allowed in __mocks__ directories"
        );
    }

    #[test]
    fn still_detects_in_production_code() {
        let code = r#"const value = Math.random();"#;
        let diagnostics = run_insecure_random_with_filename("src/utils/random.js", code);

        assert!(
            !diagnostics.is_empty(),
            "Math.random() should still be detected in production code"
        );
    }

    #[test]
    fn test_is_test_file_function() {
        // Test file patterns
        assert!(is_test_file("component.test.js"));
        assert!(is_test_file("component.spec.ts"));
        assert!(is_test_file("utils_test.js"));
        assert!(is_test_file("utils_spec.ts"));
        assert!(is_test_file("src/__tests__/file.js"));
        assert!(is_test_file("test/helpers.js"));
        assert!(is_test_file("tests/unit/file.js"));
        assert!(is_test_file("src/__mocks__/api.js"));

        // Non-test file patterns
        assert!(!is_test_file("component.js"));
        assert!(!is_test_file("utils.ts"));
        assert!(!is_test_file("src/test-utils.js")); // Contains 'test' but not a test file pattern
        assert!(!is_test_file("testing.js")); // Contains 'test' but not a test file pattern
    }
}
