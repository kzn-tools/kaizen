//! Taint sanitizers registry for tracking sanitization operations
//!
//! This module provides a registry for identifying taint sanitizers - functions
//! that clean or escape untrusted data, making it safe for specific operations.
//! For example, `shellEscape()` sanitizes data for shell command execution.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SanitizerKind {
    BuiltIn,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SanitizerCategory {
    CommandInjection,
    SqlInjection,
    Xss,
    PathTraversal,
    UrlEncoding,
    General,
}

impl SanitizerCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SanitizerCategory::CommandInjection => "command_injection",
            SanitizerCategory::SqlInjection => "sql_injection",
            SanitizerCategory::Xss => "xss",
            SanitizerCategory::PathTraversal => "path_traversal",
            SanitizerCategory::UrlEncoding => "url_encoding",
            SanitizerCategory::General => "general",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomSanitizerConfig {
    pub callee_path: Vec<String>,
    pub method: Option<String>,
    pub category: SanitizerCategory,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizerPattern {
    pub callee_path: Vec<String>,
    pub method: Option<String>,
    pub category: SanitizerCategory,
    pub kind: SanitizerKind,
    pub description: String,
}

impl SanitizerPattern {
    pub fn new(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: SanitizerCategory,
        kind: SanitizerKind,
        description: &str,
    ) -> Self {
        Self {
            callee_path: callee_path.into_iter().map(|s| s.to_string()).collect(),
            method: method.map(|s| s.to_string()),
            category,
            kind,
            description: description.to_string(),
        }
    }

    pub fn builtin(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: SanitizerCategory,
        description: &str,
    ) -> Self {
        Self::new(
            callee_path,
            method,
            category,
            SanitizerKind::BuiltIn,
            description,
        )
    }

    pub fn custom(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: SanitizerCategory,
        description: &str,
    ) -> Self {
        Self::new(
            callee_path,
            method,
            category,
            SanitizerKind::Custom,
            description,
        )
    }

    pub fn matches(&self, callee_chain: &[String], method: Option<&str>) -> bool {
        if callee_chain.len() != self.callee_path.len() {
            return false;
        }

        for (actual, expected) in callee_chain.iter().zip(self.callee_path.iter()) {
            if actual != expected {
                return false;
            }
        }

        match (&self.method, method) {
            (None, _) => true,
            (Some(expected), Some(actual)) => expected == actual,
            (Some(_), None) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizerMatch {
    pub pattern: SanitizerPattern,
    pub matched_callee: Vec<String>,
    pub matched_method: Option<String>,
}

#[derive(Debug)]
pub struct SanitizersRegistry {
    patterns: Vec<SanitizerPattern>,
    callee_index: HashMap<String, Vec<usize>>,
}

impl Default for SanitizersRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SanitizersRegistry {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            callee_index: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_command_injection_sanitizers();
        registry.register_sql_sanitizers();
        registry.register_xss_sanitizers();
        registry.register_path_sanitizers();
        registry.register_url_encoding_sanitizers();
        registry
    }

    fn register_command_injection_sanitizers(&mut self) {
        self.register_pattern(SanitizerPattern::builtin(
            vec!["shellEscape"],
            None,
            SanitizerCategory::CommandInjection,
            "Shell escape function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["shell-escape"],
            None,
            SanitizerCategory::CommandInjection,
            "shell-escape module",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["shellQuote"],
            None,
            SanitizerCategory::CommandInjection,
            "Shell quote function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["shlex"],
            Some("quote"),
            SanitizerCategory::CommandInjection,
            "shlex.quote function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["shlex"],
            Some("split"),
            SanitizerCategory::CommandInjection,
            "shlex.split function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["escapeShellArg"],
            None,
            SanitizerCategory::CommandInjection,
            "Shell argument escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["escapeShellCmd"],
            None,
            SanitizerCategory::CommandInjection,
            "Shell command escape",
        ));
    }

    fn register_sql_sanitizers(&mut self) {
        self.register_pattern(SanitizerPattern::builtin(
            vec!["escape"],
            None,
            SanitizerCategory::SqlInjection,
            "SQL escape function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["mysql"],
            Some("escape"),
            SanitizerCategory::SqlInjection,
            "MySQL escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["mysql"],
            Some("escapeId"),
            SanitizerCategory::SqlInjection,
            "MySQL identifier escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["sqlstring"],
            Some("escape"),
            SanitizerCategory::SqlInjection,
            "sqlstring escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["pg"],
            Some("escapeLiteral"),
            SanitizerCategory::SqlInjection,
            "PostgreSQL literal escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["pg"],
            Some("escapeIdentifier"),
            SanitizerCategory::SqlInjection,
            "PostgreSQL identifier escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["mysql"],
            Some("format"),
            SanitizerCategory::SqlInjection,
            "MySQL parameterized format",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["sqlstring"],
            Some("format"),
            SanitizerCategory::SqlInjection,
            "sqlstring parameterized format",
        ));

        let db_objects = ["db", "database", "connection", "conn", "pool", "client"];
        for obj in db_objects {
            self.register_pattern(SanitizerPattern::builtin(
                vec![obj],
                Some("prepare"),
                SanitizerCategory::SqlInjection,
                "Prepared statement",
            ));
        }

        self.register_pattern(SanitizerPattern::builtin(
            vec!["statement"],
            Some("bind"),
            SanitizerCategory::SqlInjection,
            "Statement parameter binding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["stmt"],
            Some("bind"),
            SanitizerCategory::SqlInjection,
            "Statement parameter binding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["knex"],
            Some("bind"),
            SanitizerCategory::SqlInjection,
            "Knex parameter binding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["sql"],
            None,
            SanitizerCategory::SqlInjection,
            "Tagged template SQL literal",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["Prisma"],
            Some("sql"),
            SanitizerCategory::SqlInjection,
            "Prisma SQL template tag",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["prisma"],
            Some("sql"),
            SanitizerCategory::SqlInjection,
            "Prisma SQL template tag",
        ));
    }

    fn register_xss_sanitizers(&mut self) {
        self.register_pattern(SanitizerPattern::builtin(
            vec!["escapeHtml"],
            None,
            SanitizerCategory::Xss,
            "HTML escape function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["escape-html"],
            None,
            SanitizerCategory::Xss,
            "escape-html module",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["sanitizeHtml"],
            None,
            SanitizerCategory::Xss,
            "HTML sanitize function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["DOMPurify"],
            Some("sanitize"),
            SanitizerCategory::Xss,
            "DOMPurify sanitize",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["xss"],
            None,
            SanitizerCategory::Xss,
            "xss filter function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["validator"],
            Some("escape"),
            SanitizerCategory::Xss,
            "validator.js escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["he"],
            Some("encode"),
            SanitizerCategory::Xss,
            "he.encode HTML entities",
        ));
    }

    fn register_path_sanitizers(&mut self) {
        self.register_pattern(SanitizerPattern::builtin(
            vec!["path"],
            Some("normalize"),
            SanitizerCategory::PathTraversal,
            "Path normalization",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["path"],
            Some("resolve"),
            SanitizerCategory::PathTraversal,
            "Path resolution",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["path"],
            Some("basename"),
            SanitizerCategory::PathTraversal,
            "Path basename extraction",
        ));
    }

    fn register_url_encoding_sanitizers(&mut self) {
        self.register_pattern(SanitizerPattern::builtin(
            vec!["encodeURIComponent"],
            None,
            SanitizerCategory::UrlEncoding,
            "URL component encoding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["encodeURI"],
            None,
            SanitizerCategory::UrlEncoding,
            "URL encoding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["escape"],
            None,
            SanitizerCategory::UrlEncoding,
            "Legacy URL escape function",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["URLSearchParams"],
            Some("toString"),
            SanitizerCategory::UrlEncoding,
            "URLSearchParams encoding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["url"],
            Some("format"),
            SanitizerCategory::UrlEncoding,
            "Node.js url.format",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["querystring"],
            Some("stringify"),
            SanitizerCategory::UrlEncoding,
            "Node.js querystring encoding",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["querystring"],
            Some("escape"),
            SanitizerCategory::UrlEncoding,
            "Node.js querystring escape",
        ));

        self.register_pattern(SanitizerPattern::builtin(
            vec!["qs"],
            Some("stringify"),
            SanitizerCategory::UrlEncoding,
            "qs library encoding",
        ));
    }

    pub fn register_pattern(&mut self, pattern: SanitizerPattern) {
        let index = self.patterns.len();

        if let Some(first_callee) = pattern.callee_path.first() {
            self.callee_index
                .entry(first_callee.clone())
                .or_default()
                .push(index);
        }

        self.patterns.push(pattern);
    }

    pub fn is_sanitizer(
        &self,
        callee_chain: &[String],
        method: Option<&str>,
    ) -> Option<SanitizerMatch> {
        if callee_chain.is_empty() {
            return None;
        }

        let first = &callee_chain[0];
        if let Some(indices) = self.callee_index.get(first) {
            for &idx in indices {
                let pattern = &self.patterns[idx];
                if pattern.matches(callee_chain, method) {
                    return Some(SanitizerMatch {
                        pattern: pattern.clone(),
                        matched_callee: callee_chain.to_vec(),
                        matched_method: method.map(|s| s.to_string()),
                    });
                }
            }
        }

        None
    }

    pub fn is_sanitizer_for_category(
        &self,
        callee_chain: &[String],
        method: Option<&str>,
        category: SanitizerCategory,
    ) -> Option<SanitizerMatch> {
        self.is_sanitizer(callee_chain, method)
            .filter(|m| m.pattern.category == category)
    }

    pub fn patterns(&self) -> &[SanitizerPattern] {
        &self.patterns
    }

    pub fn patterns_for_category(&self, category: SanitizerCategory) -> Vec<&SanitizerPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn register_custom_sanitizers(&mut self, configs: &[CustomSanitizerConfig]) {
        for config in configs {
            let callee_path: Vec<&str> = config.callee_path.iter().map(|s| s.as_str()).collect();
            self.register_pattern(SanitizerPattern::custom(
                callee_path,
                config.method.as_deref(),
                config.category,
                &config.description,
            ));
        }
    }

    pub fn with_custom_sanitizers(configs: &[CustomSanitizerConfig]) -> Self {
        let mut registry = Self::with_defaults();
        registry.register_custom_sanitizers(configs);
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> SanitizersRegistry {
        SanitizersRegistry::with_defaults()
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = SanitizersRegistry::new();
        assert!(registry.patterns().is_empty());
    }

    #[test]
    fn with_defaults_has_patterns() {
        let registry = registry();
        assert!(!registry.patterns().is_empty());
    }

    #[test]
    fn shell_escape_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["shellEscape".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::CommandInjection);
    }

    #[test]
    fn shell_escape_module_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["shell-escape".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::CommandInjection);
    }

    #[test]
    fn shell_quote_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["shellQuote".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::CommandInjection);
    }

    #[test]
    fn shlex_quote_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["shlex".into()], Some("quote"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::CommandInjection);
    }

    #[test]
    fn escape_shell_arg_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["escapeShellArg".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::CommandInjection);
    }

    #[test]
    fn mysql_escape_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["mysql".into()], Some("escape"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn pg_escape_literal_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["pg".into()], Some("escapeLiteral"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn escape_html_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["escapeHtml".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::Xss);
    }

    #[test]
    fn dompurify_sanitize_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["DOMPurify".into()], Some("sanitize"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::Xss);
    }

    #[test]
    fn path_normalize_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["path".into()], Some("normalize"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::PathTraversal);
    }

    #[test]
    fn path_basename_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["path".into()], Some("basename"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::PathTraversal);
    }

    #[test]
    fn random_function_is_not_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["randomFunction".into()], None);
        assert!(result.is_none());
    }

    #[test]
    fn custom_pattern_registration() {
        let mut registry = SanitizersRegistry::new();
        registry.register_pattern(SanitizerPattern::custom(
            vec!["mySanitizer"],
            None,
            SanitizerCategory::General,
            "Custom sanitizer",
        ));

        let result = registry.is_sanitizer(&["mySanitizer".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.kind, SanitizerKind::Custom);
    }

    #[test]
    fn patterns_for_category() {
        let registry = registry();
        let cmd_patterns = registry.patterns_for_category(SanitizerCategory::CommandInjection);
        assert!(!cmd_patterns.is_empty());

        for pattern in cmd_patterns {
            assert_eq!(pattern.category, SanitizerCategory::CommandInjection);
        }
    }

    #[test]
    fn sanitizer_category_as_str() {
        assert_eq!(
            SanitizerCategory::CommandInjection.as_str(),
            "command_injection"
        );
        assert_eq!(SanitizerCategory::SqlInjection.as_str(), "sql_injection");
        assert_eq!(SanitizerCategory::Xss.as_str(), "xss");
        assert_eq!(SanitizerCategory::PathTraversal.as_str(), "path_traversal");
        assert_eq!(SanitizerCategory::UrlEncoding.as_str(), "url_encoding");
        assert_eq!(SanitizerCategory::General.as_str(), "general");
    }

    #[test]
    fn pattern_matches_exact_method() {
        let pattern = SanitizerPattern::builtin(
            vec!["obj"],
            Some("method"),
            SanitizerCategory::General,
            "test",
        );

        assert!(pattern.matches(&["obj".to_string()], Some("method")));
        assert!(!pattern.matches(&["obj".to_string()], Some("other")));
        assert!(!pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn pattern_matches_any_method() {
        let pattern =
            SanitizerPattern::builtin(vec!["obj"], None, SanitizerCategory::General, "test");

        assert!(pattern.matches(&["obj".to_string()], Some("anything")));
        assert!(pattern.matches(&["obj".to_string()], Some("other")));
        assert!(pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn empty_chain_returns_none() {
        let registry = registry();
        let result = registry.is_sanitizer(&[], Some("method"));
        assert!(result.is_none());
    }

    #[test]
    fn is_sanitizer_for_category_filters_correctly() {
        let registry = registry();

        let result = registry.is_sanitizer_for_category(
            &["shellEscape".into()],
            None,
            SanitizerCategory::CommandInjection,
        );
        assert!(result.is_some());

        let result = registry.is_sanitizer_for_category(
            &["shellEscape".into()],
            None,
            SanitizerCategory::SqlInjection,
        );
        assert!(result.is_none());
    }

    #[test]
    fn encode_uri_component_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["encodeURIComponent".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::UrlEncoding);
    }

    #[test]
    fn encode_uri_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["encodeURI".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::UrlEncoding);
    }

    #[test]
    fn url_search_params_to_string_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["URLSearchParams".into()], Some("toString"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::UrlEncoding);
    }

    #[test]
    fn querystring_stringify_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["querystring".into()], Some("stringify"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::UrlEncoding);
    }

    #[test]
    fn qs_stringify_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["qs".into()], Some("stringify"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::UrlEncoding);
    }

    #[test]
    fn mysql_format_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["mysql".into()], Some("format"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn db_prepare_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["db".into()], Some("prepare"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn connection_prepare_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["connection".into()], Some("prepare"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn statement_bind_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["statement".into()], Some("bind"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn sql_tagged_template_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["sql".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn prisma_sql_is_sanitizer() {
        let registry = registry();
        let result = registry.is_sanitizer(&["prisma".into()], Some("sql"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
    }

    #[test]
    fn custom_sanitizer_config_registration() {
        let configs = vec![
            CustomSanitizerConfig {
                callee_path: vec!["myCompany".to_string(), "sanitize".to_string()],
                method: Some("html".to_string()),
                category: SanitizerCategory::Xss,
                description: "Company HTML sanitizer".to_string(),
            },
            CustomSanitizerConfig {
                callee_path: vec!["customEscape".to_string()],
                method: None,
                category: SanitizerCategory::SqlInjection,
                description: "Custom SQL escape".to_string(),
            },
        ];

        let registry = SanitizersRegistry::with_custom_sanitizers(&configs);

        let result = registry.is_sanitizer(&["myCompany".into(), "sanitize".into()], Some("html"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::Xss);
        assert_eq!(m.pattern.kind, SanitizerKind::Custom);

        let result = registry.is_sanitizer(&["customEscape".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, SanitizerCategory::SqlInjection);
        assert_eq!(m.pattern.kind, SanitizerKind::Custom);
    }

    #[test]
    fn url_encoding_patterns_exist() {
        let registry = registry();
        let url_patterns = registry.patterns_for_category(SanitizerCategory::UrlEncoding);
        assert!(!url_patterns.is_empty());
        assert!(url_patterns.len() >= 5);
    }

    #[test]
    fn parameterized_query_patterns_exist() {
        let registry = registry();
        let sql_patterns = registry.patterns_for_category(SanitizerCategory::SqlInjection);
        let prepare_patterns: Vec<_> = sql_patterns
            .iter()
            .filter(|p| p.description.contains("Prepared") || p.description.contains("format"))
            .collect();
        assert!(!prepare_patterns.is_empty());
    }
}
