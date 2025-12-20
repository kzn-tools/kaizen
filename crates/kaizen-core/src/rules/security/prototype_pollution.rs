//! no-prototype-pollution rule (S020): Detects prototype pollution vulnerabilities via taint analysis

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::taint::{TaintAnalyzer, TaintSinkCategory};
use crate::visitor::VisitorContext;

declare_rule!(
    PrototypePollution,
    id = "S020",
    name = "no-prototype-pollution",
    description =
        "Disallow merging untrusted data into objects which can lead to prototype pollution",
    category = Security,
    severity = Error,
    min_tier = Pro,
    examples = "// Bad\nconst config = Object.assign({}, req.body);\n_.merge(options, userInput);\n\n// Good\nconst config = { ...defaults, ...sanitizedInput };"
);

impl Rule for PrototypePollution {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|finding| finding.sink_category == TaintSinkCategory::PrototypePollution)
            .map(|finding| {
                let (sink_line, sink_column) = ctx.span_to_location(finding.sink_span);
                let (source_line, _) = ctx.span_to_location(finding.source_span);

                let message = format!(
                    "Potential prototype pollution: untrusted data from line {} flows to {}",
                    source_line, finding.sink_description
                );

                Diagnostic::new(
                    "S020",
                    Severity::Error,
                    message,
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
                .with_suggestion(
                    "Use object spread (...) instead, or validate keys against __proto__, constructor, and prototype before merging",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_prototype_pollution(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = PrototypePollution::new();
        rule.check(&file)
    }

    #[test]
    fn detects_object_assign_with_untrusted_data() {
        let code = r#"
            function handler(req, res) {
                const data = req.body;
                const config = Object.assign({}, data);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect prototype pollution via Object.assign"
        );
        assert_eq!(diagnostics[0].rule_id, "S020");
        assert!(diagnostics[0].message.contains("prototype pollution"));
    }

    #[test]
    fn detects_lodash_merge_with_untrusted_data() {
        let code = r#"
            function handler(req, res) {
                const userInput = req.body.options;
                _.merge(config, userInput);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect prototype pollution via _.merge"
        );
        assert_eq!(diagnostics[0].rule_id, "S020");
    }

    #[test]
    fn detects_lodash_defaults_deep_with_untrusted_data() {
        let code = r#"
            function handler(req, res) {
                const options = req.query;
                _.defaultsDeep(settings, options);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect prototype pollution via _.defaultsDeep"
        );
    }

    #[test]
    fn detects_lodash_merge_with_with_untrusted_data() {
        let code = r#"
            function handler(req, res) {
                const data = req.body;
                lodash.mergeWith(target, data, customizer);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect prototype pollution via lodash.mergeWith"
        );
    }

    #[test]
    fn detects_indirect_taint_flow() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const processed = input;
                Object.assign(target, processed);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect taint flow through assignments"
        );
    }

    #[test]
    fn no_false_positive_for_static_objects() {
        let code = r#"
            function getConfig() {
                const defaults = { foo: 1 };
                const overrides = { bar: 2 };
                Object.assign(defaults, overrides);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag static object literals"
        );
    }

    #[test]
    fn no_false_positive_for_object_spread() {
        let code = r#"
            function handler(req, res) {
                const data = req.body;
                const config = { ...defaults, ...data };
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag object spread (safe pattern)"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"
            function handler(req, res) {
                const data = req.body;
                Object.assign({}, data);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("object spread")
        );
    }

    #[test]
    fn diagnostic_shows_source_line() {
        let code = r#"
            function handler(req, res) {
                const data = req.body;
                Object.assign({}, data);
            }
        "#;

        let diagnostics = run_prototype_pollution(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("line"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = PrototypePollution::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S020");
        assert_eq!(metadata.name, "no-prototype-pollution");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
        assert_eq!(metadata.min_tier, crate::licensing::PremiumTier::Pro);
    }
}
