//! Inline disable comment directives for suppressing diagnostics
//!
//! Supports ESLint-style disable comments:
//! - `// lynx-disable-next-line Q030` - disable Q030 for the next line
//! - `// lynx-disable-line Q030` - disable Q030 for the current line
//! - `// lynx-disable-next-line` - disable all rules for the next line
//! - `// lynx-disable-line` - disable all rules for the current line
//! - `// lynx-disable-next-line Q030, Q031` - disable multiple rules

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableDirective {
    pub line: usize,
    pub rule_ids: Vec<String>,
}

impl DisableDirective {
    pub fn new(line: usize, rule_ids: Vec<String>) -> Self {
        Self { line, rule_ids }
    }

    pub fn for_all_rules(line: usize) -> Self {
        Self {
            line,
            rule_ids: Vec::new(),
        }
    }

    pub fn disables_all(&self) -> bool {
        self.rule_ids.is_empty()
    }

    pub fn disables_rule(&self, rule_id: &str) -> bool {
        self.rule_ids.is_empty() || self.rule_ids.iter().any(|id| id == rule_id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisableDirectives {
    by_line: HashMap<usize, DisableDirective>,
}

impl DisableDirectives {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_source(source: &str) -> Self {
        let mut directives = Self::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;

            if let Some(comment_start) = line.find("//") {
                let comment = &line[comment_start + 2..].trim();

                if let Some(rest) = comment.strip_prefix("lynx-disable-next-line") {
                    let rule_ids = parse_rule_ids(rest);
                    let target_line = line_num + 1;
                    directives.add(DisableDirective::new(target_line, rule_ids));
                } else if let Some(rest) = comment.strip_prefix("lynx-disable-line") {
                    let rule_ids = parse_rule_ids(rest);
                    directives.add(DisableDirective::new(line_num, rule_ids));
                }
            }
        }

        directives
    }

    pub fn add(&mut self, directive: DisableDirective) {
        self.by_line.insert(directive.line, directive);
    }

    pub fn is_disabled(&self, line: usize, rule_id: &str) -> bool {
        self.by_line
            .get(&line)
            .is_some_and(|d| d.disables_rule(rule_id))
    }

    pub fn directives(&self) -> impl Iterator<Item = &DisableDirective> {
        self.by_line.values()
    }

    pub fn is_empty(&self) -> bool {
        self.by_line.is_empty()
    }

    pub fn len(&self) -> usize {
        self.by_line.len()
    }
}

fn parse_rule_ids(rest: &str) -> Vec<String> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable_next_line_with_specific_rule() {
        let source = r#"
// lynx-disable-next-line Q030
var x = 1;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(!directives.is_disabled(3, "Q033"));
        assert!(!directives.is_disabled(2, "Q030"));
    }

    #[test]
    fn disable_line_with_specific_rule() {
        let source = r#"
var x = 1; // lynx-disable-line Q030
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(2, "Q030"));
        assert!(!directives.is_disabled(2, "Q033"));
    }

    #[test]
    fn disable_next_line_all_rules() {
        let source = r#"
// lynx-disable-next-line
var x = 1;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(directives.is_disabled(3, "Q033"));
        assert!(directives.is_disabled(3, "ANY_RULE"));
    }

    #[test]
    fn disable_line_all_rules() {
        let source = r#"
var x = 1; // lynx-disable-line
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(2, "Q030"));
        assert!(directives.is_disabled(2, "Q033"));
        assert!(directives.is_disabled(2, "ANY_RULE"));
    }

    #[test]
    fn disable_next_line_multiple_rules() {
        let source = r#"
// lynx-disable-next-line Q030, Q033
var x = 1;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(directives.is_disabled(3, "Q033"));
        assert!(!directives.is_disabled(3, "Q035"));
    }

    #[test]
    fn disable_line_multiple_rules() {
        let source = r#"
var x = 1; // lynx-disable-line Q030, Q033
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(2, "Q030"));
        assert!(directives.is_disabled(2, "Q033"));
        assert!(!directives.is_disabled(2, "Q035"));
    }

    #[test]
    fn no_disable_comments() {
        let source = r#"
var x = 1;
const y = 2;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(!directives.is_disabled(2, "Q030"));
        assert!(!directives.is_disabled(3, "Q030"));
        assert!(directives.is_empty());
    }

    #[test]
    fn multiple_disable_comments() {
        let source = r#"
// lynx-disable-next-line Q030
var x = 1;
// lynx-disable-next-line Q033
if (x == 2) {}
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(!directives.is_disabled(3, "Q033"));
        assert!(directives.is_disabled(5, "Q033"));
        assert!(!directives.is_disabled(5, "Q030"));
    }

    #[test]
    fn directive_on_first_line() {
        let source = r#"// lynx-disable-next-line Q030
var x = 1;"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(2, "Q030"));
    }

    #[test]
    fn directive_on_last_line_next_line() {
        let source = r#"var x = 1;
// lynx-disable-next-line Q030"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
    }

    #[test]
    fn whitespace_handling_in_rule_ids() {
        let source = r#"
// lynx-disable-next-line   Q030  ,  Q033
var x = 1;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(directives.is_disabled(3, "Q033"));
    }

    #[test]
    fn indented_comment() {
        let source = r#"
function test() {
    // lynx-disable-next-line Q030
    var x = 1;
}
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(4, "Q030"));
    }

    #[test]
    fn comment_after_code() {
        let source = r#"
var x = 1; // lynx-disable-line Q030
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(2, "Q030"));
    }

    #[test]
    fn directive_does_not_affect_other_lines() {
        let source = r#"
// lynx-disable-next-line Q030
var x = 1;
var y = 2;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_disabled(3, "Q030"));
        assert!(!directives.is_disabled(4, "Q030"));
    }

    #[test]
    fn similar_but_not_directive() {
        let source = r#"
// lynx-disable Q030
// lynx-disable-block Q030
// some lynx-disable-next-line comment
var x = 1;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_empty());
    }

    #[test]
    fn directive_struct_disables_rule() {
        let directive = DisableDirective::new(5, vec!["Q030".to_string(), "Q033".to_string()]);

        assert!(directive.disables_rule("Q030"));
        assert!(directive.disables_rule("Q033"));
        assert!(!directive.disables_rule("Q035"));
        assert!(!directive.disables_all());
    }

    #[test]
    fn directive_struct_disables_all() {
        let directive = DisableDirective::for_all_rules(5);

        assert!(directive.disables_rule("Q030"));
        assert!(directive.disables_rule("ANY_RULE"));
        assert!(directive.disables_all());
    }

    #[test]
    fn empty_source() {
        let directives = DisableDirectives::from_source("");

        assert!(directives.is_empty());
        assert_eq!(directives.len(), 0);
    }

    #[test]
    fn directives_len() {
        let source = r#"
// lynx-disable-next-line Q030
var x = 1;
// lynx-disable-next-line Q033
var y = 2;
"#;
        let directives = DisableDirectives::from_source(source);

        assert_eq!(directives.len(), 2);
    }
}
