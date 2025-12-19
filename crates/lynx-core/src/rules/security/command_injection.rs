//! no-command-injection rule (S003): Detects command injection vulnerabilities via taint analysis

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::taint::{TaintAnalyzer, TaintSinkCategory};
use crate::visitor::VisitorContext;

declare_rule!(
    CommandInjection,
    id = "S003",
    name = "no-command-injection",
    description = "Disallow shell commands constructed with untrusted data",
    category = Security,
    severity = Error,
    examples = "// Bad\nconst cmd = req.body.command;\nexec(\"rm \" + cmd);\n\n// Good\nexecFile(\"rm\", [filename]);"
);

impl Rule for CommandInjection {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|finding| finding.sink_category == TaintSinkCategory::CommandInjection)
            .map(|finding| {
                let (sink_line, sink_column) = ctx.span_to_location(finding.sink_span);
                let (source_line, _) = ctx.span_to_location(finding.source_span);

                let message = format!(
                    "Potential command injection: untrusted data from line {} flows to {}",
                    source_line, finding.sink_description
                );

                Diagnostic::new(
                    "S003",
                    Severity::Error,
                    message,
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
                .with_suggestion(
                    "Use execFile with an arguments array or sanitize input with shell-escape",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_command_injection(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = CommandInjection::new();
        rule.check(&file)
    }

    #[test]
    fn detects_exec_with_string_concatenation() {
        let code = r#"
            function handler(req, res) {
                const filename = req.body.filename;
                exec("cat " + filename);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(!diagnostics.is_empty(), "should detect command injection");
        assert_eq!(diagnostics[0].rule_id, "S003");
        assert!(diagnostics[0].message.contains("command injection"));
    }

    #[test]
    fn detects_exec_with_template_literal() {
        let code = r#"
            function handler(req, res) {
                const pattern = req.query.pattern;
                exec(`grep ${pattern} /etc/passwd`);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection via template literal"
        );
        assert_eq!(diagnostics[0].rule_id, "S003");
    }

    #[test]
    fn detects_spawn_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const cmd = req.body.command;
                spawn(cmd);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection via spawn"
        );
        assert_eq!(diagnostics[0].rule_id, "S003");
    }

    #[test]
    fn detects_exec_sync_with_user_input() {
        let code = r#"
            function handler(req, res) {
                const args = req.body.args;
                execSync("npm " + args);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection via execSync"
        );
    }

    #[test]
    fn detects_child_process_exec() {
        let code = r#"
            function handler(req, res) {
                const file = req.params.file;
                child_process.exec("cat " + file);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection via child_process.exec"
        );
    }

    #[test]
    fn detects_child_process_spawn() {
        let code = r#"
            function handler(req, res) {
                const cmd = req.body.cmd;
                child_process.spawn(cmd);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection via child_process.spawn"
        );
    }

    #[test]
    fn detects_indirect_taint_flow() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const processed = input;
                const cmd = "ls " + processed;
                exec(cmd);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect taint flow through assignments"
        );
    }

    #[test]
    fn detects_environment_variable_taint() {
        let code = r#"
            const userCommand = process.env.USER_CMD;
            exec(userCommand);
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect command injection from env var"
        );
    }

    #[test]
    fn no_false_positive_for_static_command() {
        let code = r#"
            function deploy() {
                exec("npm install && npm run build");
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(diagnostics.is_empty(), "should not flag static commands");
    }

    #[test]
    fn no_false_positive_for_literal_only() {
        let code = r#"
            function backup() {
                const cmd = "tar -czf backup.tar.gz /data";
                exec(cmd);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag commands with only literals"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"
            function handler(req, res) {
                const file = req.body.file;
                exec("cat " + file);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("execFile")
        );
    }

    #[test]
    fn diagnostic_shows_source_line() {
        let code = r#"
            function handler(req, res) {
                const cmd = req.body.cmd;
                exec(cmd);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("line"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = CommandInjection::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S003");
        assert_eq!(metadata.name, "no-command-injection");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
    }

    #[test]
    fn no_false_positive_for_shell_escape_sanitized() {
        let code = r#"
            function handler(req, res) {
                const filename = req.body.filename;
                const safe = shellEscape(filename);
                exec("cat " + safe);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag shell-escaped input"
        );
    }

    #[test]
    fn no_false_positive_for_shlex_quote_sanitized() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.input;
                const safe = shlex.quote(input);
                exec("echo " + safe);
            }
        "#;

        let diagnostics = run_command_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag shlex.quote sanitized input"
        );
    }
}
