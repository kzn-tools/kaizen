//! no-hardcoded-secrets rule (S010): Detects hardcoded secrets in code

use std::ops::ControlFlow;

use regex::Regex;
use swc_ecma_ast::{Expr, Lit, MemberExpr, MemberProp, Pat, VarDecl};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    HardcodedSecrets,
    id = "S010",
    name = "no-hardcoded-secrets",
    description = "Disallow hardcoded secrets (API keys, tokens, passwords)",
    category = Security,
    severity = Error,
    examples = "// Bad\nconst API_KEY = \"AKIAIOSFODNN7EXAMPLE\";\nconst stripe_key = \"sk_live_abc123\";\n\n// Good\nconst API_KEY = process.env.API_KEY;\nconst stripe_key = process.env.STRIPE_KEY;"
);

#[derive(Debug)]
struct SecretPattern {
    name: &'static str,
    regex: Regex,
    description: &'static str,
}

impl SecretPattern {
    fn new(name: &'static str, pattern: &str, description: &'static str) -> Self {
        Self {
            name,
            regex: Regex::new(pattern).expect("Invalid regex pattern"),
            description,
        }
    }
}

fn get_secret_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern::new("AWS Access Key", r"^AKIA[0-9A-Z]{16}$", "AWS Access Key ID"),
        SecretPattern::new(
            "Stripe Live Key",
            r"^sk_live_[0-9a-zA-Z]{24,}$",
            "Stripe Live Secret Key",
        ),
        SecretPattern::new(
            "Stripe Test Key",
            r"^sk_test_[0-9a-zA-Z]{24,}$",
            "Stripe Test Secret Key",
        ),
        SecretPattern::new(
            "GitHub Personal Access Token",
            r"^ghp_[A-Za-z0-9]{36}$",
            "GitHub Personal Access Token",
        ),
        SecretPattern::new(
            "GitHub OAuth Token",
            r"^gho_[A-Za-z0-9]{36}$",
            "GitHub OAuth Access Token",
        ),
        SecretPattern::new(
            "GitHub User Token",
            r"^ghu_[A-Za-z0-9]{36}$",
            "GitHub User-to-Server Token",
        ),
        SecretPattern::new(
            "GitHub Server Token",
            r"^ghs_[A-Za-z0-9]{36}$",
            "GitHub Server-to-Server Token",
        ),
        SecretPattern::new(
            "GitHub Refresh Token",
            r"^ghr_[A-Za-z0-9]{36}$",
            "GitHub Refresh Token",
        ),
        SecretPattern::new(
            "Slack Token",
            r"^xox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*$",
            "Slack API Token",
        ),
        SecretPattern::new(
            "Google API Key",
            r"^AIza[0-9A-Za-z\-_]{35}$",
            "Google API Key",
        ),
    ]
}

fn is_sensitive_variable_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    let sensitive_keywords = [
        "password",
        "passwd",
        "pwd",
        "secret",
        "api_key",
        "apikey",
        "api-key",
        "token",
        "auth_token",
        "authtoken",
        "access_token",
        "accesstoken",
        "private_key",
        "privatekey",
        "credential",
        "credentials",
    ];

    sensitive_keywords
        .iter()
        .any(|keyword| lower.contains(keyword))
}

fn calculate_shannon_entropy(s: &str) -> f64 {
    use std::collections::HashMap;

    if s.is_empty() {
        return 0.0;
    }

    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    freq.values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn is_high_entropy_secret(value: &str, var_name: &str) -> bool {
    if value.len() < 16 {
        return false;
    }

    if !is_sensitive_variable_name(var_name) {
        return false;
    }

    let entropy = calculate_shannon_entropy(value);
    entropy > 3.5
}

fn is_placeholder_value(value: &str) -> bool {
    let lower = value.to_lowercase();
    let placeholders = [
        "your_",
        "your-",
        "xxx",
        "placeholder",
        "example",
        "replace_me",
        "change_me",
        "insert_",
        "todo",
        "fixme",
        "<",
        ">",
        "${",
        "{{",
    ];

    placeholders.iter().any(|p| lower.contains(p))
}

fn is_process_env_access(expr: &Expr) -> bool {
    if let Expr::Member(member) = expr {
        if let Expr::Member(inner) = member.obj.as_ref() {
            if let (Expr::Ident(obj_ident), MemberProp::Ident(prop_ident)) =
                (inner.obj.as_ref(), &inner.prop)
            {
                return obj_ident.sym.as_ref() == "process" && prop_ident.sym.as_ref() == "env";
            }
        }
    }
    false
}

fn get_variable_name(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.id.sym.to_string()),
        _ => None,
    }
}

fn check_string_value(value: &str, var_name: &str, patterns: &[SecretPattern]) -> Option<String> {
    if value.is_empty() || value.len() < 8 {
        return None;
    }

    for pattern in patterns {
        if pattern.regex.is_match(value) {
            return Some(format!(
                "Hardcoded {} detected: {}",
                pattern.name, pattern.description
            ));
        }
    }

    if is_placeholder_value(value) {
        return None;
    }

    if is_high_entropy_secret(value, var_name) {
        return Some(format!(
            "Potential hardcoded secret in variable '{}': high entropy string in sensitive context",
            var_name
        ));
    }

    None
}

impl Rule for HardcodedSecrets {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let patterns = get_secret_patterns();
        let mut visitor = HardcodedSecretsVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
            patterns: &patterns,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct HardcodedSecretsVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
    patterns: &'a [SecretPattern],
}

impl HardcodedSecretsVisitor<'_> {
    fn check_expr(&mut self, var_name: &str, init: &Expr, span: swc_common::Span) {
        if is_process_env_access(init) {
            return;
        }

        let value = match init {
            Expr::Lit(Lit::Str(s)) => s.value.to_string(),
            Expr::Tpl(tpl) if tpl.exprs.is_empty() => {
                tpl.quasis.iter().map(|q| q.raw.to_string()).collect()
            }
            _ => return,
        };

        if let Some(message) = check_string_value(&value, var_name, self.patterns) {
            let (line, column) = self.ctx.span_to_location(span);
            let diagnostic = Diagnostic::new(
                "S010",
                Severity::Error,
                message,
                &self.file_path,
                line,
                column,
            )
            .with_suggestion("Use environment variables instead: process.env.YOUR_SECRET_NAME");
            self.diagnostics.push(diagnostic);
        }
    }
}

impl AstVisitor for HardcodedSecretsVisitor<'_> {
    fn visit_var_decl(&mut self, node: &VarDecl, _ctx: &VisitorContext) -> ControlFlow<()> {
        for decl in &node.decls {
            let var_name = match get_variable_name(&decl.name) {
                Some(name) => name,
                None => continue,
            };

            if let Some(init) = &decl.init {
                self.check_expr(&var_name, init, decl.span);
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_member_expr(&mut self, node: &MemberExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        if let MemberProp::Ident(prop) = &node.prop {
            let prop_name = prop.sym.to_string();
            if is_sensitive_variable_name(&prop_name) {
                if let Expr::Object(obj) = node.obj.as_ref() {
                    for prop in &obj.props {
                        if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                            if let swc_ecma_ast::Prop::KeyValue(kv) = p.as_ref() {
                                if let Some(Lit::Str(s)) = kv.value.as_lit() {
                                    if let Some(message) =
                                        check_string_value(&s.value, &prop_name, self.patterns)
                                    {
                                        let (line, column) = self.ctx.span_to_location(node.span);
                                        let diagnostic = Diagnostic::new(
                                            "S010",
                                            Severity::Error,
                                            message,
                                            &self.file_path,
                                            line,
                                            column,
                                        )
                                        .with_suggestion(
                                            "Use environment variables instead: process.env.YOUR_SECRET_NAME",
                                        );
                                        self.diagnostics.push(diagnostic);
                                    }
                                }
                            }
                        }
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

    fn run_hardcoded_secrets(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = HardcodedSecrets::new();
        rule.check(&file)
    }

    #[test]
    fn detects_aws_access_key() {
        let code = r#"const AWS_KEY = "AKIAIOSFODNN7EXAMPLE";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(!diagnostics.is_empty(), "should detect AWS access key");
        assert_eq!(diagnostics[0].rule_id, "S010");
        assert!(diagnostics[0].message.contains("AWS Access Key"));
    }

    fn make_stripe_key(prefix: &str, length: usize) -> String {
        format!("{}{}", prefix, "a".repeat(length))
    }

    #[test]
    fn detects_stripe_live_key() {
        let patterns = get_secret_patterns();
        let stripe_pattern = patterns.iter().find(|p| p.name == "Stripe Live Key").unwrap();
        let test_key = make_stripe_key("sk_live_", 28);
        assert!(
            stripe_pattern.regex.is_match(&test_key),
            "pattern should match test key"
        );

        let code = format!(r#"const STRIPE_KEY = "{}";"#, test_key);
        let diagnostics = run_hardcoded_secrets(&code);

        assert!(!diagnostics.is_empty(), "should detect Stripe live key");
        assert_eq!(diagnostics[0].rule_id, "S010");
        assert!(diagnostics[0].message.contains("Stripe Live"));
    }

    #[test]
    fn detects_stripe_test_key() {
        let patterns = get_secret_patterns();
        let stripe_pattern = patterns.iter().find(|p| p.name == "Stripe Test Key").unwrap();
        let test_key = make_stripe_key("sk_test_", 28);
        assert!(
            stripe_pattern.regex.is_match(&test_key),
            "pattern should match test key"
        );

        let code = format!(r#"const STRIPE_TEST = "{}";"#, test_key);
        let diagnostics = run_hardcoded_secrets(&code);

        assert!(!diagnostics.is_empty(), "should detect Stripe test key");
        assert_eq!(diagnostics[0].rule_id, "S010");
        assert!(diagnostics[0].message.contains("Stripe Test"));
    }

    #[test]
    fn detects_github_personal_access_token() {
        let code = r#"const GITHUB_TOKEN = "ghp_1234567890abcdefghijklmnopqrstuvwxyz";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect GitHub personal access token"
        );
        assert_eq!(diagnostics[0].rule_id, "S010");
        assert!(diagnostics[0].message.contains("GitHub"));
    }

    #[test]
    fn detects_github_oauth_token() {
        let code = r#"const OAUTH = "gho_1234567890abcdefghijklmnopqrstuvwxyz";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(!diagnostics.is_empty(), "should detect GitHub OAuth token");
        assert_eq!(diagnostics[0].rule_id, "S010");
    }

    #[test]
    fn detects_high_entropy_password() {
        let code = r#"const password = "aB3$dE6@hI9#lM0&pQ3*tU6^";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect high entropy password"
        );
        assert_eq!(diagnostics[0].rule_id, "S010");
        assert!(diagnostics[0].message.contains("high entropy"));
    }

    #[test]
    fn detects_high_entropy_api_key() {
        let code = r#"const apiKey = "xK9mN2pL5qR8sT1uV4wX7yZ0aB3cD6eF";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect high entropy API key"
        );
        assert_eq!(diagnostics[0].rule_id, "S010");
    }

    #[test]
    fn detects_secret_in_sensitive_variable() {
        let code = r#"const access_token = "xK9mN2pL5qR8sT1uV4wX7yZ0";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect secret in sensitive variable"
        );
    }

    #[test]
    fn allows_process_env_access() {
        let code = r#"const API_KEY = process.env.API_KEY;"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(diagnostics.is_empty(), "should allow process.env access");
    }

    #[test]
    fn allows_placeholder_values() {
        let code = r#"const API_KEY = "your_api_key_here";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(diagnostics.is_empty(), "should allow placeholder values");
    }

    #[test]
    fn allows_example_values() {
        let code = r#"const API_KEY = "EXAMPLE_KEY_12345";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(diagnostics.is_empty(), "should allow example values");
    }

    #[test]
    fn allows_short_strings() {
        let code = r#"const token = "abc123";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(diagnostics.is_empty(), "should allow short strings");
    }

    #[test]
    fn allows_non_sensitive_variables() {
        let code = r#"const message = "Hello, World!";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            diagnostics.is_empty(),
            "should allow non-sensitive variables"
        );
    }

    #[test]
    fn allows_low_entropy_password() {
        let code = r#"const password = "password12345";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(
            diagnostics.is_empty(),
            "should allow low entropy passwords (not obviously secrets)"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"const AWS_KEY = "AKIAIOSFODNN7EXAMPLE";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("process.env")
        );
    }

    #[test]
    fn metadata_is_correct() {
        let rule = HardcodedSecrets::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S010");
        assert_eq!(metadata.name, "no-hardcoded-secrets");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
    }

    #[test]
    fn detects_google_api_key() {
        let code = r#"const GOOGLE_KEY = "AIzaSyA1234567890abcdefghijklmnopqrstuv";"#;
        let diagnostics = run_hardcoded_secrets(code);

        assert!(!diagnostics.is_empty(), "should detect Google API key");
        assert!(diagnostics[0].message.contains("Google API Key"));
    }

    #[test]
    fn shannon_entropy_calculation() {
        assert!(calculate_shannon_entropy("") == 0.0);
        assert!(calculate_shannon_entropy("aaaa") < 1.0);
        assert!(calculate_shannon_entropy("aB3$dE6@hI9#") > 3.0);
    }
}
