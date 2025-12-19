//! no-weak-hashing rule (S011): Detects usage of weak cryptographic hash algorithms

use std::ops::ControlFlow;

use swc_ecma_ast::{CallExpr, Callee, Expr, Lit, MemberProp};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    WeakHashing,
    id = "S011",
    name = "no-weak-hashing",
    description = "Disallow weak cryptographic hash algorithms (MD5, SHA1)",
    category = Security,
    severity = Warning,
    examples = "// Bad\nconst hash = crypto.createHash('md5');\nconst hash = crypto.createHash('sha1');\n\n// Good\nconst hash = crypto.createHash('sha256');\nconst hash = crypto.createHash('sha512');"
);

const WEAK_ALGORITHMS: &[&str] = &["md5", "sha1"];

fn is_weak_algorithm(value: &str) -> bool {
    WEAK_ALGORITHMS.contains(&value.to_lowercase().as_str())
}

fn get_algorithm_recommendation(algorithm: &str) -> &'static str {
    match algorithm.to_lowercase().as_str() {
        "md5" => "Use SHA-256 or SHA-512 instead of MD5",
        "sha1" => "Use SHA-256 or SHA-512 instead of SHA1",
        _ => "Use a strong hashing algorithm like SHA-256 or SHA-512",
    }
}

impl Rule for WeakHashing {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = WeakHashingVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct WeakHashingVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl WeakHashingVisitor<'_> {
    fn check_create_hash_call(&mut self, call: &CallExpr) {
        let Callee::Expr(callee_expr) = &call.callee else {
            return;
        };

        let Expr::Member(member) = callee_expr.as_ref() else {
            return;
        };

        let MemberProp::Ident(prop) = &member.prop else {
            return;
        };

        if prop.sym.as_ref() != "createHash" {
            return;
        }

        let Expr::Ident(obj_ident) = member.obj.as_ref() else {
            return;
        };

        if obj_ident.sym.as_ref() != "crypto" {
            return;
        }

        if let Some(first_arg) = call.args.first() {
            let algorithm = match first_arg.expr.as_ref() {
                Expr::Lit(Lit::Str(s)) => s.value.to_string(),
                Expr::Tpl(tpl) if tpl.exprs.is_empty() => tpl
                    .quasis
                    .first()
                    .map(|q| q.raw.to_string())
                    .unwrap_or_default(),
                _ => return,
            };

            if is_weak_algorithm(&algorithm) {
                let (line, column) = self.ctx.span_to_location(call.span);
                let diagnostic = Diagnostic::new(
                    "S011",
                    Severity::Warning,
                    format!(
                        "Weak cryptographic hash algorithm '{}' detected",
                        algorithm.to_uppercase()
                    ),
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion(get_algorithm_recommendation(&algorithm));
                self.diagnostics.push(diagnostic);
            }
        }
    }

    fn check_require_call(&mut self, call: &CallExpr) {
        let Callee::Expr(callee_expr) = &call.callee else {
            return;
        };

        let Expr::Ident(ident) = callee_expr.as_ref() else {
            return;
        };

        if ident.sym.as_ref() != "require" {
            return;
        }

        if let Some(first_arg) = call.args.first() {
            let module_name = match first_arg.expr.as_ref() {
                Expr::Lit(Lit::Str(s)) => s.value.to_string(),
                _ => return,
            };

            if is_weak_algorithm(&module_name) {
                let (line, column) = self.ctx.span_to_location(call.span);
                let diagnostic = Diagnostic::new(
                    "S011",
                    Severity::Warning,
                    format!(
                        "Import of weak cryptographic hash library '{}' detected",
                        module_name
                    ),
                    &self.file_path,
                    line,
                    column,
                )
                .with_suggestion(format!(
                    "Use Node.js built-in crypto module with a strong algorithm instead of '{}'",
                    module_name
                ));
                self.diagnostics.push(diagnostic);
            }
        }
    }
}

impl AstVisitor for WeakHashingVisitor<'_> {
    fn visit_call_expr(&mut self, node: &CallExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_create_hash_call(node);
        self.check_require_call(node);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_weak_hashing(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = WeakHashing::new();
        rule.check(&file)
    }

    #[test]
    fn detects_md5_create_hash() {
        let code = r#"const hash = crypto.createHash('md5');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(!diagnostics.is_empty(), "should detect MD5 usage");
        assert_eq!(diagnostics[0].rule_id, "S011");
        assert!(diagnostics[0].message.contains("MD5"));
    }

    #[test]
    fn detects_sha1_create_hash() {
        let code = r#"const hash = crypto.createHash('sha1');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(!diagnostics.is_empty(), "should detect SHA1 usage");
        assert_eq!(diagnostics[0].rule_id, "S011");
        assert!(diagnostics[0].message.contains("SHA1"));
    }

    #[test]
    fn detects_md5_case_insensitive() {
        let code = r#"const hash = crypto.createHash('MD5');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect MD5 case insensitive"
        );
    }

    #[test]
    fn detects_sha1_double_quotes() {
        let code = r#"const hash = crypto.createHash("sha1");"#;
        let diagnostics = run_weak_hashing(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SHA1 with double quotes"
        );
    }

    #[test]
    fn detects_md5_require() {
        let code = r#"const md5 = require('md5');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(!diagnostics.is_empty(), "should detect md5 require");
        assert!(diagnostics[0].message.contains("md5"));
    }

    #[test]
    fn detects_sha1_require() {
        let code = r#"const sha1 = require('sha1');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(!diagnostics.is_empty(), "should detect sha1 require");
    }

    #[test]
    fn allows_sha256() {
        let code = r#"const hash = crypto.createHash('sha256');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(diagnostics.is_empty(), "should allow SHA256");
    }

    #[test]
    fn allows_sha512() {
        let code = r#"const hash = crypto.createHash('sha512');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(diagnostics.is_empty(), "should allow SHA512");
    }

    #[test]
    fn allows_sha3() {
        let code = r#"const hash = crypto.createHash('sha3-256');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(diagnostics.is_empty(), "should allow SHA3");
    }

    #[test]
    fn ignores_non_crypto_create_hash() {
        let code = r#"const hash = myModule.createHash('md5');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(
            diagnostics.is_empty(),
            "should ignore non-crypto createHash"
        );
    }

    #[test]
    fn ignores_crypto_other_methods() {
        let code = r#"const cipher = crypto.createCipher('aes192', 'password');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(diagnostics.is_empty(), "should ignore other crypto methods");
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"const hash = crypto.createHash('md5');"#;
        let diagnostics = run_weak_hashing(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("SHA-256")
        );
    }

    #[test]
    fn metadata_is_correct() {
        let rule = WeakHashing::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S011");
        assert_eq!(metadata.name, "no-weak-hashing");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Warning);
    }
}
