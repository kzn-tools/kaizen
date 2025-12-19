//! no-eval-injection rule (S005): Detects eval injection vulnerabilities via taint analysis

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::taint::{TaintAnalyzer, TaintSinkCategory};
use crate::visitor::VisitorContext;

declare_rule!(
    EvalInjection,
    id = "S005",
    name = "no-eval-injection",
    description = "Disallow code execution with untrusted data",
    category = Security,
    severity = Error,
    examples = "// Bad\nconst code = req.body.code;\neval(code);\nsetTimeout(userInput, 100);\n\n// Good\neval('safe-static-code');\nsetTimeout(() => alert(1), 100);"
);

impl Rule for EvalInjection {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|finding| finding.sink_category == TaintSinkCategory::CodeExecution)
            .map(|finding| {
                let (sink_line, sink_column) = ctx.span_to_location(finding.sink_span);
                let (source_line, _) = ctx.span_to_location(finding.source_span);

                let message = format!(
                    "Potential code injection: untrusted data from line {} flows to {}",
                    source_line, finding.sink_description
                );

                Diagnostic::new(
                    "S005",
                    Severity::Error,
                    message,
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
                .with_suggestion(
                    "Avoid executing untrusted data. Use static code or safe alternatives like function references instead of strings.",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_eval_injection(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = EvalInjection::new();
        rule.check(&file)
    }

    #[test]
    fn detects_eval_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const code = req.body.code;
                eval(code);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(!diagnostics.is_empty(), "should detect eval injection");
        assert_eq!(diagnostics[0].rule_id, "S005");
        assert!(diagnostics[0].message.contains("code injection"));
    }

    #[test]
    fn detects_new_function_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const userCode = req.query.code;
                const fn = new Function(userCode);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(!diagnostics.is_empty(), "should detect Function injection");
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_settimeout_string_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const userCode = req.body.callback;
                setTimeout(userCode, 100);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect setTimeout injection"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_setinterval_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const userCode = req.query.code;
                setInterval(userCode, 1000);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect setInterval injection"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_eval_with_template_literal() {
        let code = r#"
            function handler(req, res) {
                const userInput = req.query.input;
                eval(`return ${userInput}`);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect eval injection via template literal"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_eval_with_string_concatenation() {
        let code = r#"
            function handler(req, res) {
                const userInput = req.body.input;
                eval("const result = " + userInput);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect eval injection via concatenation"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_indirect_taint_flow() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const code = input;
                eval(code);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect taint flow through assignments"
        );
    }

    #[test]
    fn detects_vm_run_in_context_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const userCode = req.body.code;
                vm.runInContext(userCode, context);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect vm.runInContext injection"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_vm_run_in_new_context_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const userCode = req.query.script;
                vm.runInNewContext(userCode, {});
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect vm.runInNewContext injection"
        );
        assert_eq!(diagnostics[0].rule_id, "S005");
    }

    #[test]
    fn detects_process_env_in_eval() {
        let code = r#"
            const output = process.env.CODE;
            eval(output);
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect process.env as taint source in eval"
        );
    }

    #[test]
    fn detects_process_argv_in_eval() {
        let code = r#"
            const userArg = process.argv[2];
            eval(userArg);
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect process.argv as taint source in eval"
        );
    }

    #[test]
    fn no_false_positive_for_static_code() {
        let code = r#"
            function test() {
                eval("const x = 1;");
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(diagnostics.is_empty(), "should not flag static code");
    }

    #[test]
    fn no_false_positive_for_function_callback() {
        let code = r#"
            function handler(req, res) {
                setTimeout(() => console.log("test"), 100);
                setInterval(function() { console.log("test"); }, 1000);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(diagnostics.is_empty(), "should not flag function callbacks");
    }

    #[test]
    fn no_false_positive_for_literal_only() {
        let code = r#"
            function test() {
                const code = "console.log('hello')";
                eval(code);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag eval with literal string variable"
        );
    }

    #[test]
    fn no_false_positive_for_static_function_constructor() {
        let code = r#"
            const fn = new Function("return 42");
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag static Function constructor"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"
            function handler(req, res) {
                const code = req.body.code;
                eval(code);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("untrusted")
        );
    }

    #[test]
    fn diagnostic_shows_source_line() {
        let code = r#"
            function handler(req, res) {
                const code = req.body.code;
                eval(code);
            }
        "#;

        let diagnostics = run_eval_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("line"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = EvalInjection::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S005");
        assert_eq!(metadata.name, "no-eval-injection");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
    }
}
