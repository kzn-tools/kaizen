//! Rule system for code analysis
//!
//! Provides quality and security rules for analyzing JavaScript/TypeScript code.

pub mod quality;
pub mod security;

use crate::diagnostic::Diagnostic;
use crate::parser::ParsedFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    Quality,
    Security,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: RuleCategory,
    pub severity: Severity,
    pub docs_url: Option<&'static str>,
}

pub trait Rule: Send + Sync {
    fn metadata(&self) -> &RuleMetadata;
    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic>;
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.iter().map(|r| r.as_ref())
    }

    pub fn run_all(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check(file))
            .collect()
    }

    pub fn get_rule(&self, id: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|r| r.metadata().id == id)
            .map(|r| r.as_ref())
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! declare_rule {
    (
        $name:ident,
        id = $id:literal,
        name = $rule_name:literal,
        description = $desc:literal,
        category = $cat:ident,
        severity = $sev:ident
        $(, docs_url = $url:literal)?
    ) => {
        pub struct $name {
            metadata: $crate::rules::RuleMetadata,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    metadata: $crate::rules::RuleMetadata {
                        id: $id,
                        name: $rule_name,
                        description: $desc,
                        category: $crate::rules::RuleCategory::$cat,
                        severity: $crate::rules::Severity::$sev,
                        docs_url: declare_rule!(@docs_url $($url)?),
                    },
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
    (@docs_url $url:literal) => { Some($url) };
    (@docs_url) => { None };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule {
        metadata: RuleMetadata,
        diagnostics_to_return: Vec<Diagnostic>,
    }

    impl TestRule {
        fn new(id: &'static str) -> Self {
            Self {
                metadata: RuleMetadata {
                    id,
                    name: "test-rule",
                    description: "A test rule",
                    category: RuleCategory::Quality,
                    severity: Severity::Warning,
                    docs_url: None,
                },
                diagnostics_to_return: Vec::new(),
            }
        }

        fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
            self.diagnostics_to_return.push(diagnostic);
            self
        }
    }

    impl Rule for TestRule {
        fn metadata(&self) -> &RuleMetadata {
            &self.metadata
        }

        fn check(&self, _file: &ParsedFile) -> Vec<Diagnostic> {
            self.diagnostics_to_return.clone()
        }
    }

    #[test]
    fn rule_has_required_metadata() {
        let rule = TestRule::new("T001");
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "T001");
        assert_eq!(metadata.name, "test-rule");
        assert_eq!(metadata.description, "A test rule");
        assert_eq!(metadata.category, RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
        assert!(metadata.docs_url.is_none());
    }

    #[test]
    fn registry_contains_all_rules() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("T001")));
        registry.register(Box::new(TestRule::new("T002")));
        registry.register(Box::new(TestRule::new("T003")));

        let rules: Vec<_> = registry.rules().collect();

        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].metadata().id, "T001");
        assert_eq!(rules[1].metadata().id, "T002");
        assert_eq!(rules[2].metadata().id, "T003");
    }

    #[test]
    fn run_all_collects_diagnostics() {
        let mut registry = RuleRegistry::new();

        let diag1 = Diagnostic::new("T001", Severity::Warning, "Issue 1", "test.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Error, "Issue 2", "test.js", 2, 0);

        registry.register(Box::new(
            TestRule::new("T001").with_diagnostic(diag1.clone()),
        ));
        registry.register(Box::new(
            TestRule::new("T002").with_diagnostic(diag2.clone()),
        ));

        let file = ParsedFile::from_source("test.js", "const x = 1;\nconst y = 2;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].rule_id, "T001");
        assert_eq!(diagnostics[1].rule_id, "T002");
    }

    #[test]
    fn registry_get_rule_finds_by_id() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("T001")));
        registry.register(Box::new(TestRule::new("T002")));

        let rule = registry.get_rule("T002");

        assert!(rule.is_some());
        assert_eq!(rule.unwrap().metadata().id, "T002");
    }

    #[test]
    fn registry_get_rule_returns_none_for_unknown() {
        let registry = RuleRegistry::new();

        let rule = registry.get_rule("UNKNOWN");

        assert!(rule.is_none());
    }

    #[test]
    fn registry_len_returns_count() {
        let mut registry = RuleRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(TestRule::new("T001")));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn severity_values_exist() {
        let _error = Severity::Error;
        let _warning = Severity::Warning;
        let _info = Severity::Info;
        let _hint = Severity::Hint;
    }

    #[test]
    fn rule_category_values_exist() {
        let _quality = RuleCategory::Quality;
        let _security = RuleCategory::Security;
    }

    declare_rule!(
        MacroTestRule,
        id = "M001",
        name = "macro-test",
        description = "Tests the declare_rule! macro",
        category = Quality,
        severity = Info
    );

    impl Rule for MacroTestRule {
        fn metadata(&self) -> &RuleMetadata {
            &self.metadata
        }

        fn check(&self, _file: &ParsedFile) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    #[test]
    fn declare_rule_macro_creates_rule() {
        let rule = MacroTestRule::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "M001");
        assert_eq!(metadata.name, "macro-test");
        assert_eq!(metadata.description, "Tests the declare_rule! macro");
        assert_eq!(metadata.category, RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Info);
        assert!(metadata.docs_url.is_none());
    }

    declare_rule!(
        MacroTestRuleWithDocs,
        id = "M002",
        name = "macro-test-docs",
        description = "Tests the declare_rule! macro with docs",
        category = Security,
        severity = Error,
        docs_url = "https://example.com/rules/M002"
    );

    impl Rule for MacroTestRuleWithDocs {
        fn metadata(&self) -> &RuleMetadata {
            &self.metadata
        }

        fn check(&self, _file: &ParsedFile) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    #[test]
    fn declare_rule_macro_with_docs_url() {
        let rule = MacroTestRuleWithDocs::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "M002");
        assert_eq!(metadata.category, RuleCategory::Security);
        assert_eq!(metadata.severity, Severity::Error);
        assert_eq!(metadata.docs_url, Some("https://example.com/rules/M002"));
    }
}
