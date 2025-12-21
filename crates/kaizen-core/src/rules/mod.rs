//! Rule system for code analysis
//!
//! Provides quality and security rules for analyzing JavaScript/TypeScript code.

pub mod helpers;
pub mod quality;
pub mod security;

use crate::config::RulesConfig;
use crate::diagnostic::Diagnostic;
use crate::licensing::PremiumTier;
use crate::parser::ParsedFile;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Confidence {
    #[default]
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn level(&self) -> u8 {
        match self {
            Confidence::High => 3,
            Confidence::Medium => 2,
            Confidence::Low => 1,
        }
    }
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
    pub min_tier: PremiumTier,
    pub docs_url: Option<&'static str>,
    pub examples: Option<&'static str>,
}

pub trait Rule: Send + Sync {
    fn metadata(&self) -> &RuleMetadata;
    fn check(&self, file: &ParsedFile) -> Vec<Diagnostic>;
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    disabled_rules: HashSet<String>,
    severity_overrides: HashMap<String, Severity>,
    quality_enabled: bool,
    security_enabled: bool,
    current_tier: PremiumTier,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            disabled_rules: HashSet::new(),
            severity_overrides: HashMap::new(),
            quality_enabled: true,
            security_enabled: true,
            current_tier: PremiumTier::Free,
        }
    }

    pub fn set_tier(&mut self, tier: PremiumTier) {
        self.current_tier = tier;
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn configure(&mut self, config: &RulesConfig) {
        self.disabled_rules.clear();
        self.severity_overrides.clear();

        for rule_ref in &config.disabled {
            self.disabled_rules.insert(rule_ref.clone());
        }

        for (rule_ref, severity_value) in &config.severity {
            self.severity_overrides
                .insert(rule_ref.clone(), (*severity_value).into());
        }

        self.quality_enabled = config.quality.unwrap_or(true);
        self.security_enabled = config.security.unwrap_or(true);
    }

    pub fn rules(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.iter().map(|r| r.as_ref())
    }

    pub fn run_all(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .filter(|rule| self.should_run_rule(rule.as_ref()))
            .flat_map(|rule| {
                let mut diagnostics = rule.check(file);
                self.apply_severity_overrides(rule.as_ref(), &mut diagnostics);
                diagnostics
            })
            .collect()
    }

    fn should_run_rule(&self, rule: &dyn Rule) -> bool {
        let metadata = rule.metadata();

        if metadata.min_tier.level() > self.current_tier.level() {
            return false;
        }

        if !self.quality_enabled && metadata.category == RuleCategory::Quality {
            return false;
        }
        if !self.security_enabled && metadata.category == RuleCategory::Security {
            return false;
        }

        !self.is_rule_disabled(metadata)
    }

    fn is_rule_disabled(&self, metadata: &RuleMetadata) -> bool {
        self.disabled_rules.contains(metadata.id) || self.disabled_rules.contains(metadata.name)
    }

    fn apply_severity_overrides(&self, rule: &dyn Rule, diagnostics: &mut [Diagnostic]) {
        let metadata = rule.metadata();

        let override_severity = self
            .severity_overrides
            .get(metadata.id)
            .or_else(|| self.severity_overrides.get(metadata.name));

        if let Some(severity) = override_severity {
            for diag in diagnostics.iter_mut() {
                diag.severity = *severity;
            }
        }
    }

    pub fn is_rule_enabled(&self, id_or_name: &str) -> bool {
        if let Some(rule) = self
            .get_rule(id_or_name)
            .or_else(|| self.get_rule_by_name(id_or_name))
        {
            self.should_run_rule(rule)
        } else {
            false
        }
    }

    pub fn get_rule(&self, id: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|r| r.metadata().id == id)
            .map(|r| r.as_ref())
    }

    pub fn get_rule_by_name(&self, name: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|r| r.metadata().name == name)
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
        $(, min_tier = $tier:ident)?
        $(, docs_url = $url:literal)?
        $(, examples = $examples:literal)?
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
                        min_tier: declare_rule!(@min_tier $($tier)?),
                        docs_url: declare_rule!(@docs_url $($url)?),
                        examples: declare_rule!(@examples $($examples)?),
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
    (@min_tier $tier:ident) => { $crate::licensing::PremiumTier::$tier };
    (@min_tier) => { $crate::licensing::PremiumTier::Free };
    (@docs_url $url:literal) => { Some($url) };
    (@docs_url) => { None };
    (@examples $examples:literal) => { Some($examples) };
    (@examples) => { None };
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
                    min_tier: PremiumTier::Free,
                    docs_url: None,
                    examples: None,
                },
                diagnostics_to_return: Vec::new(),
            }
        }

        fn with_name(mut self, name: &'static str) -> Self {
            self.metadata.name = name;
            self
        }

        fn with_category(mut self, category: RuleCategory) -> Self {
            self.metadata.category = category;
            self
        }

        fn with_min_tier(mut self, tier: PremiumTier) -> Self {
            self.metadata.min_tier = tier;
            self
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
        assert!(metadata.examples.is_none());
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
    fn confidence_values_exist() {
        let _high = Confidence::High;
        let _medium = Confidence::Medium;
        let _low = Confidence::Low;
    }

    #[test]
    fn confidence_default_is_high() {
        assert_eq!(Confidence::default(), Confidence::High);
    }

    #[test]
    fn confidence_level_ordering() {
        assert!(Confidence::High.level() > Confidence::Medium.level());
        assert!(Confidence::Medium.level() > Confidence::Low.level());
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
        assert!(metadata.examples.is_none());
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
        assert!(metadata.examples.is_none());
    }

    declare_rule!(
        MacroTestRuleWithExamples,
        id = "M003",
        name = "macro-test-examples",
        description = "Tests the declare_rule! macro with examples",
        category = Quality,
        severity = Warning,
        examples = "// Bad\nvar x = 1;\n\n// Good\nlet x = 1;"
    );

    impl Rule for MacroTestRuleWithExamples {
        fn metadata(&self) -> &RuleMetadata {
            &self.metadata
        }

        fn check(&self, _file: &ParsedFile) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    #[test]
    fn declare_rule_macro_with_examples() {
        let rule = MacroTestRuleWithExamples::new();
        let metadata = rule.metadata();

        assert_eq!(metadata.id, "M003");
        assert_eq!(metadata.category, RuleCategory::Quality);
        assert_eq!(metadata.severity, Severity::Warning);
        assert!(metadata.docs_url.is_none());
        assert!(metadata.examples.is_some());
        assert!(metadata.examples.unwrap().contains("var x = 1"));
    }

    // ==================== Configuration Tests ====================

    #[test]
    fn disabled_rule_not_executed() {
        use crate::config::RulesConfig;

        let mut registry = RuleRegistry::new();
        let diag = Diagnostic::new("Q032", Severity::Info, "console detected", "test.js", 1, 0);
        registry.register(Box::new(
            TestRule::new("Q032")
                .with_name("no-console")
                .with_diagnostic(diag),
        ));

        let config = RulesConfig {
            disabled: vec!["Q032".to_string()],
            ..Default::default()
        };
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "console.log('test')");
        let diagnostics = registry.run_all(&file);

        assert!(
            diagnostics.is_empty(),
            "Disabled rule should not produce diagnostics"
        );
    }

    #[test]
    fn disabled_rule_by_name_not_executed() {
        use crate::config::RulesConfig;

        let mut registry = RuleRegistry::new();
        let diag = Diagnostic::new("Q032", Severity::Info, "console detected", "test.js", 1, 0);
        registry.register(Box::new(
            TestRule::new("Q032")
                .with_name("no-console")
                .with_diagnostic(diag),
        ));

        let config = RulesConfig {
            disabled: vec!["no-console".to_string()],
            ..Default::default()
        };
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "console.log('test')");
        let diagnostics = registry.run_all(&file);

        assert!(
            diagnostics.is_empty(),
            "Rule disabled by name should not produce diagnostics"
        );
    }

    #[test]
    fn all_rules_active_by_default() {
        use crate::config::RulesConfig;

        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Issue 1", "test.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Warning, "Issue 2", "test.js", 2, 0);
        registry.register(Box::new(TestRule::new("T001").with_diagnostic(diag1)));
        registry.register(Box::new(TestRule::new("T002").with_diagnostic(diag2)));

        let config = RulesConfig::default();
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(
            diagnostics.len(),
            2,
            "All rules should be active by default"
        );
    }

    #[test]
    fn disable_category() {
        use crate::config::RulesConfig;

        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("Q001", Severity::Warning, "Quality issue", "test.js", 1, 0);
        let diag2 = Diagnostic::new("S001", Severity::Warning, "Security issue", "test.js", 2, 0);
        registry.register(Box::new(
            TestRule::new("Q001")
                .with_category(RuleCategory::Quality)
                .with_diagnostic(diag1),
        ));
        registry.register(Box::new(
            TestRule::new("S001")
                .with_category(RuleCategory::Security)
                .with_diagnostic(diag2),
        ));

        let config = RulesConfig {
            quality: Some(false),
            ..Default::default()
        };
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 1, "Only security rule should run");
        assert_eq!(diagnostics[0].rule_id, "S001");
    }

    #[test]
    fn override_severity() {
        use crate::config::{RulesConfig, SeverityValue};
        use std::collections::HashMap;

        let mut registry = RuleRegistry::new();
        let diag = Diagnostic::new("Q030", Severity::Warning, "var detected", "test.js", 1, 0);
        registry.register(Box::new(
            TestRule::new("Q030")
                .with_name("no-var")
                .with_diagnostic(diag),
        ));

        let mut severity_overrides = HashMap::new();
        severity_overrides.insert("Q030".to_string(), SeverityValue::Error);

        let config = RulesConfig {
            severity: severity_overrides,
            ..Default::default()
        };
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "var x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].severity,
            Severity::Error,
            "Severity should be overridden to Error"
        );
    }

    #[test]
    fn override_severity_by_name() {
        use crate::config::{RulesConfig, SeverityValue};
        use std::collections::HashMap;

        let mut registry = RuleRegistry::new();
        let diag = Diagnostic::new("Q030", Severity::Warning, "var detected", "test.js", 1, 0);
        registry.register(Box::new(
            TestRule::new("Q030")
                .with_name("no-var")
                .with_diagnostic(diag),
        ));

        let mut severity_overrides = HashMap::new();
        severity_overrides.insert("no-var".to_string(), SeverityValue::Error);

        let config = RulesConfig {
            severity: severity_overrides,
            ..Default::default()
        };
        registry.configure(&config);

        let file = ParsedFile::from_source("test.js", "var x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].severity,
            Severity::Error,
            "Severity should be overridden by name"
        );
    }

    #[test]
    fn is_rule_enabled_returns_true_for_active_rules() {
        use crate::config::RulesConfig;

        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("T001")));
        registry.register(Box::new(TestRule::new("T002")));

        let config = RulesConfig {
            disabled: vec!["T002".to_string()],
            ..Default::default()
        };
        registry.configure(&config);

        assert!(registry.is_rule_enabled("T001"));
        assert!(!registry.is_rule_enabled("T002"));
    }

    #[test]
    fn get_rule_by_name_finds_rule() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("Q030").with_name("no-var")));
        registry.register(Box::new(TestRule::new("Q032").with_name("no-console")));

        let rule = registry.get_rule_by_name("no-console");

        assert!(rule.is_some());
        assert_eq!(rule.unwrap().metadata().id, "Q032");
    }

    #[test]
    fn tier_filtering_blocks_premium_rules_for_free_tier() {
        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Free rule", "test.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Warning, "Pro rule", "test.js", 2, 0);

        registry.register(Box::new(
            TestRule::new("T001")
                .with_min_tier(PremiumTier::Free)
                .with_diagnostic(diag1),
        ));
        registry.register(Box::new(
            TestRule::new("T002")
                .with_min_tier(PremiumTier::Pro)
                .with_diagnostic(diag2),
        ));

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "T001");
    }

    #[test]
    fn tier_filtering_allows_premium_rules_for_pro_tier() {
        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Free rule", "test.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Warning, "Pro rule", "test.js", 2, 0);

        registry.register(Box::new(
            TestRule::new("T001")
                .with_min_tier(PremiumTier::Free)
                .with_diagnostic(diag1),
        ));
        registry.register(Box::new(
            TestRule::new("T002")
                .with_min_tier(PremiumTier::Pro)
                .with_diagnostic(diag2),
        ));
        registry.set_tier(PremiumTier::Pro);

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn tier_filtering_blocks_enterprise_rules_for_pro_tier() {
        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Pro rule", "test.js", 1, 0);
        let diag2 = Diagnostic::new(
            "T002",
            Severity::Warning,
            "Enterprise rule",
            "test.js",
            2,
            0,
        );

        registry.register(Box::new(
            TestRule::new("T001")
                .with_min_tier(PremiumTier::Pro)
                .with_diagnostic(diag1),
        ));
        registry.register(Box::new(
            TestRule::new("T002")
                .with_min_tier(PremiumTier::Enterprise)
                .with_diagnostic(diag2),
        ));
        registry.set_tier(PremiumTier::Pro);

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "T001");
    }

    #[test]
    fn enterprise_tier_allows_all_rules() {
        let mut registry = RuleRegistry::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Free rule", "test.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Warning, "Pro rule", "test.js", 2, 0);
        let diag3 = Diagnostic::new(
            "T003",
            Severity::Warning,
            "Enterprise rule",
            "test.js",
            3,
            0,
        );

        registry.register(Box::new(
            TestRule::new("T001")
                .with_min_tier(PremiumTier::Free)
                .with_diagnostic(diag1),
        ));
        registry.register(Box::new(
            TestRule::new("T002")
                .with_min_tier(PremiumTier::Pro)
                .with_diagnostic(diag2),
        ));
        registry.register(Box::new(
            TestRule::new("T003")
                .with_min_tier(PremiumTier::Enterprise)
                .with_diagnostic(diag3),
        ));
        registry.set_tier(PremiumTier::Enterprise);

        let file = ParsedFile::from_source("test.js", "const x = 1;");
        let diagnostics = registry.run_all(&file);

        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn is_rule_enabled_respects_tier_filtering() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(
            TestRule::new("T001")
                .with_name("free-rule")
                .with_min_tier(PremiumTier::Free),
        ));
        registry.register(Box::new(
            TestRule::new("T002")
                .with_name("pro-rule")
                .with_min_tier(PremiumTier::Pro),
        ));

        // Default tier is Free
        assert!(
            registry.is_rule_enabled("T001"),
            "Free rule should be enabled for Free tier"
        );
        assert!(
            !registry.is_rule_enabled("T002"),
            "Pro rule should NOT be enabled for Free tier"
        );

        // Upgrade to Pro
        registry.set_tier(PremiumTier::Pro);
        assert!(
            registry.is_rule_enabled("T001"),
            "Free rule should be enabled for Pro tier"
        );
        assert!(
            registry.is_rule_enabled("T002"),
            "Pro rule should be enabled for Pro tier"
        );
    }
}
