//! no-redos rule (S021): Detects regular expressions vulnerable to ReDoS attacks

use std::ops::ControlFlow;
use std::sync::LazyLock;

use regex::Regex as RustRegex;
use swc_ecma_ast::{Expr, Lit, NewExpr, Regex};

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

declare_rule!(
    ReDoS,
    id = "S021",
    name = "no-redos",
    description = "Disallow regular expressions vulnerable to ReDoS attacks",
    category = Security,
    severity = Warning,
    min_tier = Pro,
    examples = "// Bad\nconst re = /(a+)+$/;\nconst re = new RegExp('(.*)+b');\n\n// Good\nconst re = /^[a-z]+$/;\nconst re = /\\d{2,4}/;"
);

static NESTED_QUANTIFIERS: LazyLock<RustRegex> = LazyLock::new(|| {
    RustRegex::new(r"\([^)]*[+*][^)]*\)[+*?]|\([^)]*[+*][^)]*\)\{").expect("Invalid regex pattern")
});

static OVERLAPPING_ALTERNATION: LazyLock<RustRegex> = LazyLock::new(|| {
    RustRegex::new(r"\(\.\|[\[\]\\sS]+\)[+*]|\(\[\^?\]?\][^]]*\]\|\[\^?\]?\][^]]*\]\)[+*]")
        .expect("Invalid regex pattern")
});

static STAR_PLUS_GROUPS: LazyLock<RustRegex> =
    LazyLock::new(|| RustRegex::new(r"\(\.\*\)[+*]|\(\.\+\)[+*]").expect("Invalid regex pattern"));

static NESTED_GROUPS_WITH_QUANTIFIERS: LazyLock<RustRegex> = LazyLock::new(|| {
    RustRegex::new(r"\([^)]*\([^)]*[+*]\)[^)]*\)[+*]").expect("Invalid regex pattern")
});

fn is_redos_vulnerable(pattern: &str) -> Option<&'static str> {
    if NESTED_QUANTIFIERS.is_match(pattern) {
        return Some("nested quantifiers can cause exponential backtracking");
    }

    if OVERLAPPING_ALTERNATION.is_match(pattern) {
        return Some("overlapping alternations can cause exponential backtracking");
    }

    if STAR_PLUS_GROUPS.is_match(pattern) {
        return Some("quantified wildcard groups can cause exponential backtracking");
    }

    if NESTED_GROUPS_WITH_QUANTIFIERS.is_match(pattern) {
        return Some("deeply nested quantified groups can cause exponential backtracking");
    }

    None
}

impl Rule for ReDoS {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let Some(module) = file.module() else {
            return Vec::new();
        };

        let ctx = VisitorContext::new(file);
        let mut visitor = ReDoSVisitor {
            diagnostics: Vec::new(),
            file_path: file.metadata().filename.clone(),
            ctx: &ctx,
        };

        walk_ast(module, &mut visitor, &ctx);
        visitor.diagnostics
    }
}

struct ReDoSVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    file_path: String,
    ctx: &'a VisitorContext<'a>,
}

impl ReDoSVisitor<'_> {
    fn check_pattern(&mut self, pattern: &str, span: swc_common::Span) {
        if let Some(reason) = is_redos_vulnerable(pattern) {
            let (line, column) = self.ctx.span_to_location(span);
            let diagnostic = Diagnostic::new(
                "S021",
                Severity::Warning,
                format!("Potential ReDoS vulnerability: {}", reason),
                &self.file_path,
                line,
                column,
            )
            .with_suggestion(
                "Simplify the regex pattern or use atomic groups/possessive quantifiers if supported",
            );
            self.diagnostics.push(diagnostic);
        }
    }

    fn check_new_regexp(&mut self, node: &NewExpr) {
        let Expr::Ident(ident) = node.callee.as_ref() else {
            return;
        };

        if ident.sym.as_ref() != "RegExp" {
            return;
        }

        let Some(args) = &node.args else {
            return;
        };

        if let Some(first_arg) = args.first() {
            let pattern = match first_arg.expr.as_ref() {
                Expr::Lit(Lit::Str(s)) => s.value.to_string(),
                Expr::Tpl(tpl) if tpl.exprs.is_empty() => tpl
                    .quasis
                    .first()
                    .map(|q| q.raw.to_string())
                    .unwrap_or_default(),
                _ => return,
            };

            self.check_pattern(&pattern, node.span);
        }
    }
}

impl AstVisitor for ReDoSVisitor<'_> {
    fn visit_regex(&mut self, node: &Regex, _ctx: &VisitorContext) -> ControlFlow<()> {
        let pattern = node.exp.as_ref();
        self.check_pattern(pattern, node.span);
        ControlFlow::Continue(())
    }

    fn visit_new_expr(&mut self, node: &NewExpr, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.check_new_regexp(node);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_redos(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = ReDoS::new();
        rule.check(&file)
    }

    #[test]
    fn detects_nested_plus_quantifier() {
        let code = r#"const re = /(a+)+$/;"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect nested plus quantifier"
        );
        assert_eq!(diagnostics[0].rule_id, "S021");
        assert!(diagnostics[0].message.contains("nested quantifiers"));
    }

    #[test]
    fn detects_nested_star_quantifier() {
        let code = r#"const re = /(a*)*$/;"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect nested star quantifier"
        );
        assert_eq!(diagnostics[0].rule_id, "S021");
    }

    #[test]
    fn detects_mixed_nested_quantifiers() {
        let code = r#"const re = /(a+)*$/;"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect mixed nested quantifiers"
        );
    }

    #[test]
    fn detects_wildcard_group_with_quantifier() {
        let code = r#"const re = /(.*)+b/;"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect quantified wildcard group"
        );
    }

    #[test]
    fn detects_regexp_constructor() {
        let code = r#"const re = new RegExp('(a+)+');"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect ReDoS in RegExp constructor"
        );
        assert_eq!(diagnostics[0].rule_id, "S021");
    }

    #[test]
    fn detects_regexp_constructor_template() {
        let code = r#"const re = new RegExp(`(a+)+`);"#;
        let diagnostics = run_redos(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect ReDoS in RegExp with template literal"
        );
    }

    #[test]
    fn no_false_positive_for_simple_pattern() {
        let code = r#"const re = /^[a-z]+$/;"#;
        let diagnostics = run_redos(code);

        assert!(diagnostics.is_empty(), "should not flag simple patterns");
    }

    #[test]
    fn no_false_positive_for_bounded_quantifier() {
        let code = r#"const re = /\d{2,4}/;"#;
        let diagnostics = run_redos(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag bounded quantifiers"
        );
    }

    #[test]
    fn no_false_positive_for_alternation_without_overlap() {
        let code = r#"const re = /(cat|dog)+/;"#;
        let diagnostics = run_redos(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag non-overlapping alternation"
        );
    }

    #[test]
    fn no_false_positive_for_simple_group() {
        let code = r#"const re = /(abc)+/;"#;
        let diagnostics = run_redos(code);

        assert!(diagnostics.is_empty(), "should not flag simple groups");
    }

    #[test]
    fn no_false_positive_for_email_like_pattern() {
        let code = r#"const re = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;"#;
        let diagnostics = run_redos(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag email-like patterns"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"const re = /(a+)+$/;"#;
        let diagnostics = run_redos(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("Simplify")
        );
    }

    #[test]
    fn metadata_is_correct() {
        let rule = ReDoS::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S021");
        assert_eq!(metadata.name, "no-redos");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Warning);
        assert_eq!(metadata.min_tier, crate::licensing::PremiumTier::Pro);
    }
}
