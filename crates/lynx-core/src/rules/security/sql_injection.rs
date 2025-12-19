//! no-sql-injection rule (S001): Detects SQL injection vulnerabilities via taint analysis

use crate::declare_rule;
use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;
use crate::rules::{Rule, RuleMetadata, Severity};
use crate::taint::{TaintAnalyzer, TaintSinkCategory};
use crate::visitor::VisitorContext;

declare_rule!(
    SqlInjection,
    id = "S001",
    name = "no-sql-injection",
    description = "Disallow SQL queries constructed with untrusted data",
    category = Security,
    severity = Error,
    examples = "// Bad\nconst query = \"SELECT * FROM users WHERE id = \" + userId;\ndb.query(query);\n\n// Good\ndb.query(\"SELECT * FROM users WHERE id = ?\", [userId]);"
);

impl Rule for SqlInjection {
    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(file);
        let ctx = VisitorContext::new(file);

        findings
            .into_iter()
            .filter(|finding| finding.sink_category == TaintSinkCategory::SqlInjection)
            .map(|finding| {
                let (sink_line, sink_column) = ctx.span_to_location(finding.sink_span);
                let (source_line, _) = ctx.span_to_location(finding.source_span);

                let message = format!(
                    "Potential SQL injection: untrusted data from line {} flows to {}",
                    source_line, finding.sink_description
                );

                Diagnostic::new(
                    "S001",
                    Severity::Error,
                    message,
                    &file.metadata().filename,
                    sink_line,
                    sink_column,
                )
                .with_suggestion(
                    "Use parameterized queries or prepared statements instead of string concatenation",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_sql_injection(code: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::from_source("test.js", code);
        let rule = SqlInjection::new();
        rule.check(&file)
    }

    #[test]
    fn detects_string_concatenation_in_query() {
        let code = r#"
            function handler(req, res) {
                const username = req.body.username;
                const query = "SELECT * FROM users WHERE name = '" + username + "'";
                db.query(query);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(!diagnostics.is_empty(), "should detect SQL injection");
        assert_eq!(diagnostics[0].rule_id, "S001");
        assert!(diagnostics[0].message.contains("SQL injection"));
    }

    #[test]
    fn detects_template_literal_in_query() {
        let code = r#"
            function handler(req, res) {
                const searchTerm = req.query.search;
                const sql = `SELECT * FROM products WHERE description LIKE '%${searchTerm}%'`;
                db.query(sql);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via template literal"
        );
        assert_eq!(diagnostics[0].rule_id, "S001");
    }

    #[test]
    fn detects_execute_method() {
        let code = r#"
            function handler(req, res) {
                const id = req.params.id;
                db.execute("DELETE FROM records WHERE id = " + id);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via execute"
        );
    }

    #[test]
    fn detects_pool_query() {
        let code = r#"
            function handler(req, res) {
                const userId = req.body.userId;
                pool.query("SELECT * FROM users WHERE id = " + userId);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via pool.query"
        );
    }

    #[test]
    fn detects_connection_query() {
        let code = r#"
            function handler(req, res) {
                const name = req.body.name;
                connection.query("SELECT * FROM users WHERE name = '" + name + "'");
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via connection.query"
        );
    }

    #[test]
    fn detects_client_query() {
        let code = r#"
            function handler(req, res) {
                const email = req.body.email;
                client.query("SELECT * FROM users WHERE email = '" + email + "'");
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via client.query"
        );
    }

    #[test]
    fn detects_sequelize_query() {
        let code = r#"
            function handler(req, res) {
                const search = req.query.q;
                sequelize.query("SELECT * FROM items WHERE name LIKE '%" + search + "%'");
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via sequelize.query"
        );
    }

    #[test]
    fn detects_knex_raw() {
        let code = r#"
            function handler(req, res) {
                const id = req.params.id;
                knex.raw("SELECT * FROM users WHERE id = " + id);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via knex.raw"
        );
    }

    #[test]
    fn detects_prisma_query_raw() {
        let code = r#"
            function handler(req, res) {
                const userId = req.body.userId;
                prisma.$queryRaw("SELECT * FROM users WHERE id = " + userId);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via prisma.$queryRaw"
        );
    }

    #[test]
    fn detects_prisma_execute_raw() {
        let code = r#"
            function handler(req, res) {
                const email = req.body.email;
                prisma.$executeRaw("UPDATE users SET verified = true WHERE email = '" + email + "'");
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect SQL injection via prisma.$executeRaw"
        );
    }

    #[test]
    fn no_false_positive_for_safe_static_query() {
        let code = r#"
            function getUsers() {
                const query = "SELECT * FROM users WHERE active = true";
                db.query(query);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(diagnostics.is_empty(), "should not flag static queries");
    }

    #[test]
    fn no_false_positive_for_parameterized_query() {
        let code = r#"
            function handler(req, res) {
                const userId = req.body.userId;
                db.query("SELECT * FROM users WHERE id = ?", [userId]);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            diagnostics.is_empty(),
            "should not flag parameterized queries"
        );
    }

    #[test]
    fn detects_indirect_taint_flow() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const processed = input;
                const query = "SELECT * FROM t WHERE x = '" + processed + "'";
                db.query(query);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(
            !diagnostics.is_empty(),
            "should detect taint flow through assignments"
        );
    }

    #[test]
    fn diagnostic_has_suggestion() {
        let code = r#"
            function handler(req, res) {
                const id = req.body.id;
                db.query("SELECT * FROM users WHERE id = " + id);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].suggestion.is_some());
        assert!(
            diagnostics[0]
                .suggestion
                .as_ref()
                .unwrap()
                .contains("parameterized")
        );
    }

    #[test]
    fn diagnostic_shows_source_line() {
        let code = r#"
            function handler(req, res) {
                const userId = req.body.userId;
                db.query("SELECT * FROM users WHERE id = " + userId);
            }
        "#;

        let diagnostics = run_sql_injection(code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("line"));
    }

    #[test]
    fn metadata_is_correct() {
        let rule = SqlInjection::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "S001");
        assert_eq!(metadata.name, "no-sql-injection");
        assert_eq!(metadata.category, crate::rules::RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
    }
}
