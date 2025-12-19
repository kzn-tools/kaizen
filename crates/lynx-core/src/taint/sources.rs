//! Taint sources registry for tracking untrusted data entry points
//!
//! This module provides a registry for identifying taint sources - places where
//! untrusted data can enter the application, such as HTTP request parameters,
//! environment variables, and DOM inputs.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintSourceKind {
    BuiltIn,
    HeuristicPattern,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintCategory {
    HttpRequest,
    Environment,
    UserInput,
    FileSystem,
    Network,
    Database,
}

impl TaintCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaintCategory::HttpRequest => "http_request",
            TaintCategory::Environment => "environment",
            TaintCategory::UserInput => "user_input",
            TaintCategory::FileSystem => "file_system",
            TaintCategory::Network => "network",
            TaintCategory::Database => "database",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintSourcePattern {
    pub object_path: Vec<String>,
    pub property: PropertyMatcher,
    pub category: TaintCategory,
    pub kind: TaintSourceKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyMatcher {
    Exact(String),
    Any,
    None,
}

impl TaintSourcePattern {
    pub fn new(
        object_path: Vec<&str>,
        property: PropertyMatcher,
        category: TaintCategory,
        kind: TaintSourceKind,
        description: &str,
    ) -> Self {
        Self {
            object_path: object_path.into_iter().map(|s| s.to_string()).collect(),
            property,
            category,
            kind,
            description: description.to_string(),
        }
    }

    pub fn builtin(
        object_path: Vec<&str>,
        property: PropertyMatcher,
        category: TaintCategory,
        description: &str,
    ) -> Self {
        Self::new(
            object_path,
            property,
            category,
            TaintSourceKind::BuiltIn,
            description,
        )
    }

    pub fn custom(
        object_path: Vec<&str>,
        property: PropertyMatcher,
        category: TaintCategory,
        description: &str,
    ) -> Self {
        Self::new(
            object_path,
            property,
            category,
            TaintSourceKind::Custom,
            description,
        )
    }

    pub fn matches(&self, object_chain: &[String], property: Option<&str>) -> bool {
        if object_chain.len() != self.object_path.len() {
            return false;
        }

        for (actual, expected) in object_chain.iter().zip(self.object_path.iter()) {
            if actual != expected {
                return false;
            }
        }

        match (&self.property, property) {
            (PropertyMatcher::None, None) => true,
            (PropertyMatcher::None, Some(_)) => false,
            (PropertyMatcher::Any, Some(_)) => true,
            (PropertyMatcher::Any, None) => true,
            (PropertyMatcher::Exact(expected), Some(actual)) => expected == actual,
            (PropertyMatcher::Exact(_), None) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintSourceMatch {
    pub pattern: TaintSourcePattern,
    pub matched_path: Vec<String>,
    pub matched_property: Option<String>,
}

#[derive(Debug)]
pub struct TaintSourcesRegistry {
    patterns: Vec<TaintSourcePattern>,
    object_index: HashMap<String, Vec<usize>>,
    parameter_names: HashMap<String, TaintCategory>,
}

impl Default for TaintSourcesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintSourcesRegistry {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            object_index: HashMap::new(),
            parameter_names: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_express_sources();
        registry.register_node_sources();
        registry.register_dom_sources();
        registry.register_default_parameter_names();
        registry
    }

    fn register_express_sources(&mut self) {
        let sources = [
            (
                vec!["req", "body"],
                PropertyMatcher::Any,
                "HTTP request body",
            ),
            (
                vec!["req", "query"],
                PropertyMatcher::Any,
                "URL query parameters",
            ),
            (
                vec!["req", "params"],
                PropertyMatcher::Any,
                "Route parameters",
            ),
            (vec!["req", "headers"], PropertyMatcher::Any, "HTTP headers"),
            (vec!["req", "cookies"], PropertyMatcher::Any, "HTTP cookies"),
            (
                vec!["request", "body"],
                PropertyMatcher::Any,
                "HTTP request body",
            ),
            (
                vec!["request", "query"],
                PropertyMatcher::Any,
                "URL query parameters",
            ),
            (
                vec!["request", "params"],
                PropertyMatcher::Any,
                "Route parameters",
            ),
            (
                vec!["request", "headers"],
                PropertyMatcher::Any,
                "HTTP headers",
            ),
            (
                vec!["request", "cookies"],
                PropertyMatcher::Any,
                "HTTP cookies",
            ),
            (vec!["ctx", "request"], PropertyMatcher::Any, "Koa request"),
            (
                vec!["ctx", "query"],
                PropertyMatcher::Any,
                "Koa query parameters",
            ),
            (
                vec!["ctx", "params"],
                PropertyMatcher::Any,
                "Koa route params",
            ),
        ];

        for (path, property, desc) in sources {
            self.register_pattern(TaintSourcePattern::builtin(
                path,
                property,
                TaintCategory::HttpRequest,
                desc,
            ));
        }

        let direct_sources = [
            (vec!["req"], "body", "HTTP request body"),
            (vec!["req"], "query", "URL query parameters"),
            (vec!["req"], "params", "Route parameters"),
            (vec!["req"], "headers", "HTTP headers"),
            (vec!["req"], "cookies", "HTTP cookies"),
            (vec!["request"], "body", "HTTP request body"),
            (vec!["request"], "query", "URL query parameters"),
            (vec!["request"], "params", "Route parameters"),
            (vec!["request"], "headers", "HTTP headers"),
            (vec!["request"], "cookies", "HTTP cookies"),
        ];

        for (path, prop, desc) in direct_sources {
            self.register_pattern(TaintSourcePattern::builtin(
                path,
                PropertyMatcher::Exact(prop.to_string()),
                TaintCategory::HttpRequest,
                desc,
            ));
        }
    }

    fn register_node_sources(&mut self) {
        self.register_pattern(TaintSourcePattern::builtin(
            vec!["process", "env"],
            PropertyMatcher::Any,
            TaintCategory::Environment,
            "Environment variable",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["process"],
            PropertyMatcher::Exact("env".to_string()),
            TaintCategory::Environment,
            "Environment variables object",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["process", "argv"],
            PropertyMatcher::Any,
            TaintCategory::Environment,
            "Command line argument",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["process"],
            PropertyMatcher::Exact("argv".to_string()),
            TaintCategory::Environment,
            "Command line arguments array",
        ));
    }

    fn register_dom_sources(&mut self) {
        let location_sources = [
            (vec!["document", "location"], "href", "Document URL"),
            (vec!["document", "location"], "search", "URL query string"),
            (vec!["document", "location"], "hash", "URL hash"),
            (vec!["document", "location"], "pathname", "URL pathname"),
            (vec!["window", "location"], "href", "Window URL"),
            (vec!["window", "location"], "search", "URL query string"),
            (vec!["window", "location"], "hash", "URL hash"),
            (vec!["window", "location"], "pathname", "URL pathname"),
            (vec!["location"], "href", "Current URL"),
            (vec!["location"], "search", "URL query string"),
            (vec!["location"], "hash", "URL hash"),
            (vec!["location"], "pathname", "URL pathname"),
        ];

        for (path, prop, desc) in location_sources {
            self.register_pattern(TaintSourcePattern::builtin(
                path,
                PropertyMatcher::Exact(prop.to_string()),
                TaintCategory::UserInput,
                desc,
            ));
        }

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["document"],
            PropertyMatcher::Exact("cookie".to_string()),
            TaintCategory::UserInput,
            "Browser cookies",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["document"],
            PropertyMatcher::Exact("referrer".to_string()),
            TaintCategory::UserInput,
            "Document referrer",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["document"],
            PropertyMatcher::Exact("URL".to_string()),
            TaintCategory::UserInput,
            "Document URL",
        ));

        self.register_pattern(TaintSourcePattern::builtin(
            vec!["document"],
            PropertyMatcher::Exact("documentURI".to_string()),
            TaintCategory::UserInput,
            "Document URI",
        ));
    }

    fn register_default_parameter_names(&mut self) {
        self.parameter_names
            .insert("req".to_string(), TaintCategory::HttpRequest);
        self.parameter_names
            .insert("request".to_string(), TaintCategory::HttpRequest);
        self.parameter_names
            .insert("ctx".to_string(), TaintCategory::HttpRequest);
    }

    pub fn register_pattern(&mut self, pattern: TaintSourcePattern) {
        let index = self.patterns.len();

        if let Some(first_object) = pattern.object_path.first() {
            self.object_index
                .entry(first_object.clone())
                .or_default()
                .push(index);
        }

        self.patterns.push(pattern);
    }

    pub fn register_parameter_name(&mut self, name: String, category: TaintCategory) {
        self.parameter_names.insert(name, category);
    }

    pub fn is_taint_source(
        &self,
        object_chain: &[String],
        property: Option<&str>,
    ) -> Option<TaintSourceMatch> {
        if object_chain.is_empty() {
            return None;
        }

        let first = &object_chain[0];
        if let Some(indices) = self.object_index.get(first) {
            for &idx in indices {
                let pattern = &self.patterns[idx];
                if pattern.matches(object_chain, property) {
                    return Some(TaintSourceMatch {
                        pattern: pattern.clone(),
                        matched_path: object_chain.to_vec(),
                        matched_property: property.map(|s| s.to_string()),
                    });
                }
            }
        }

        None
    }

    pub fn is_tainted_parameter(&self, name: &str) -> Option<TaintCategory> {
        self.parameter_names.get(name).copied()
    }

    pub fn patterns(&self) -> &[TaintSourcePattern] {
        &self.patterns
    }

    pub fn parameter_names(&self) -> &HashMap<String, TaintCategory> {
        &self.parameter_names
    }

    pub fn patterns_for_category(&self, category: TaintCategory) -> Vec<&TaintSourcePattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> TaintSourcesRegistry {
        TaintSourcesRegistry::with_defaults()
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = TaintSourcesRegistry::new();
        assert!(registry.patterns().is_empty());
    }

    #[test]
    fn with_defaults_has_patterns() {
        let registry = registry();
        assert!(!registry.patterns().is_empty());
    }

    #[test]
    fn req_body_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["req".into(), "body".into()], Some("username"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn req_body_direct_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["req".into()], Some("body"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn req_query_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["req".into(), "query".into()], Some("id"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn req_params_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["req".into(), "params".into()], Some("userId"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn req_headers_is_taint_source() {
        let registry = registry();
        let result =
            registry.is_taint_source(&["req".into(), "headers".into()], Some("authorization"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn req_cookies_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["req".into(), "cookies".into()], Some("session"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn request_body_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["request".into(), "body".into()], Some("username"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn process_env_is_taint_source() {
        let registry = registry();
        let result =
            registry.is_taint_source(&["process".into(), "env".into()], Some("DATABASE_URL"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::Environment);
    }

    #[test]
    fn process_env_direct_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["process".into()], Some("env"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::Environment);
    }

    #[test]
    fn process_argv_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["process".into(), "argv".into()], Some("2"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::Environment);
    }

    #[test]
    fn process_argv_direct_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["process".into()], Some("argv"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::Environment);
    }

    #[test]
    fn document_location_href_is_taint_source() {
        let registry = registry();
        let result =
            registry.is_taint_source(&["document".into(), "location".into()], Some("href"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn document_location_search_is_taint_source() {
        let registry = registry();
        let result =
            registry.is_taint_source(&["document".into(), "location".into()], Some("search"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn window_location_hash_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["window".into(), "location".into()], Some("hash"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn location_href_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["location".into()], Some("href"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn document_cookie_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["document".into()], Some("cookie"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn document_referrer_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["document".into()], Some("referrer"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::UserInput);
    }

    #[test]
    fn random_property_is_not_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["foo".into(), "bar".into()], Some("baz"));
        assert!(result.is_none());
    }

    #[test]
    fn req_is_tainted_parameter() {
        let registry = registry();
        let result = registry.is_tainted_parameter("req");
        assert_eq!(result, Some(TaintCategory::HttpRequest));
    }

    #[test]
    fn request_is_tainted_parameter() {
        let registry = registry();
        let result = registry.is_tainted_parameter("request");
        assert_eq!(result, Some(TaintCategory::HttpRequest));
    }

    #[test]
    fn ctx_is_tainted_parameter() {
        let registry = registry();
        let result = registry.is_tainted_parameter("ctx");
        assert_eq!(result, Some(TaintCategory::HttpRequest));
    }

    #[test]
    fn random_is_not_tainted_parameter() {
        let registry = registry();
        let result = registry.is_tainted_parameter("foo");
        assert!(result.is_none());
    }

    #[test]
    fn custom_pattern_registration() {
        let mut registry = TaintSourcesRegistry::new();
        registry.register_pattern(TaintSourcePattern::custom(
            vec!["myApp", "input"],
            PropertyMatcher::Any,
            TaintCategory::UserInput,
            "Custom input source",
        ));

        let result = registry.is_taint_source(&["myApp".into(), "input".into()], Some("data"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.kind, TaintSourceKind::Custom);
    }

    #[test]
    fn custom_parameter_name_registration() {
        let mut registry = TaintSourcesRegistry::new();
        registry.register_parameter_name("customReq".to_string(), TaintCategory::HttpRequest);

        let result = registry.is_tainted_parameter("customReq");
        assert_eq!(result, Some(TaintCategory::HttpRequest));
    }

    #[test]
    fn patterns_for_category() {
        let registry = registry();
        let http_patterns = registry.patterns_for_category(TaintCategory::HttpRequest);
        assert!(!http_patterns.is_empty());

        for pattern in http_patterns {
            assert_eq!(pattern.category, TaintCategory::HttpRequest);
        }
    }

    #[test]
    fn taint_category_as_str() {
        assert_eq!(TaintCategory::HttpRequest.as_str(), "http_request");
        assert_eq!(TaintCategory::Environment.as_str(), "environment");
        assert_eq!(TaintCategory::UserInput.as_str(), "user_input");
        assert_eq!(TaintCategory::FileSystem.as_str(), "file_system");
        assert_eq!(TaintCategory::Network.as_str(), "network");
        assert_eq!(TaintCategory::Database.as_str(), "database");
    }

    #[test]
    fn pattern_matches_exact_property() {
        let pattern = TaintSourcePattern::builtin(
            vec!["obj"],
            PropertyMatcher::Exact("prop".to_string()),
            TaintCategory::UserInput,
            "test",
        );

        assert!(pattern.matches(&["obj".to_string()], Some("prop")));
        assert!(!pattern.matches(&["obj".to_string()], Some("other")));
        assert!(!pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn pattern_matches_any_property() {
        let pattern = TaintSourcePattern::builtin(
            vec!["obj"],
            PropertyMatcher::Any,
            TaintCategory::UserInput,
            "test",
        );

        assert!(pattern.matches(&["obj".to_string()], Some("anything")));
        assert!(pattern.matches(&["obj".to_string()], Some("other")));
        assert!(pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn pattern_matches_no_property() {
        let pattern = TaintSourcePattern::builtin(
            vec!["obj"],
            PropertyMatcher::None,
            TaintCategory::UserInput,
            "test",
        );

        assert!(pattern.matches(&["obj".to_string()], None));
        assert!(!pattern.matches(&["obj".to_string()], Some("prop")));
    }

    #[test]
    fn pattern_requires_matching_path() {
        let pattern = TaintSourcePattern::builtin(
            vec!["obj", "nested"],
            PropertyMatcher::Any,
            TaintCategory::UserInput,
            "test",
        );

        assert!(pattern.matches(&["obj".to_string(), "nested".to_string()], Some("prop")));
        assert!(!pattern.matches(&["obj".to_string()], Some("prop")));
        assert!(!pattern.matches(&["obj".to_string(), "other".to_string()], Some("prop")));
        assert!(!pattern.matches(&["other".to_string(), "nested".to_string()], Some("prop")));
    }

    #[test]
    fn ctx_request_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["ctx".into(), "request".into()], Some("body"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn ctx_query_is_taint_source() {
        let registry = registry();
        let result = registry.is_taint_source(&["ctx".into(), "query".into()], Some("id"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintCategory::HttpRequest);
    }

    #[test]
    fn empty_chain_returns_none() {
        let registry = registry();
        let result = registry.is_taint_source(&[], Some("prop"));
        assert!(result.is_none());
    }
}
