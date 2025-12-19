//! Disposable types registry for tracking types that implement Symbol.dispose/Symbol.asyncDispose
//!
//! This module provides a registry for identifying types that are disposable resources,
//! enabling rules to detect improper resource management patterns.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableSource {
    BuiltIn,
    HeuristicPattern,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DisposableTypeInfo {
    pub name: String,
    pub is_async: bool,
    pub source: DisposableSource,
}

impl DisposableTypeInfo {
    pub fn new(name: impl Into<String>, is_async: bool, source: DisposableSource) -> Self {
        Self {
            name: name.into(),
            is_async,
            source,
        }
    }

    pub fn builtin(name: impl Into<String>, is_async: bool) -> Self {
        Self::new(name, is_async, DisposableSource::BuiltIn)
    }

    pub fn heuristic(name: impl Into<String>, is_async: bool) -> Self {
        Self::new(name, is_async, DisposableSource::HeuristicPattern)
    }
}

pub struct DisposableTypesRegistry {
    types: HashMap<String, DisposableTypeInfo>,
    heuristic_patterns: Vec<String>,
    return_type_mappings: HashMap<String, String>,
}

impl Default for DisposableTypesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DisposableTypesRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            heuristic_patterns: Vec::new(),
            return_type_mappings: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_types();
        registry.register_default_heuristic_patterns();
        registry.register_default_return_type_mappings();
        registry
    }

    fn register_builtin_types(&mut self) {
        let async_disposable_types = [
            "FileHandle",
            "ReadableStream",
            "ReadableStreamDefaultReader",
            "WritableStream",
            "WritableStreamDefaultWriter",
            "AsyncDisposableStack",
        ];

        for type_name in async_disposable_types {
            self.register_type(DisposableTypeInfo::builtin(type_name, true));
        }

        let sync_disposable_types = ["DisposableStack"];

        for type_name in sync_disposable_types {
            self.register_type(DisposableTypeInfo::builtin(type_name, false));
        }
    }

    fn register_default_heuristic_patterns(&mut self) {
        let patterns = [
            "acquire",
            "connect",
            "open",
            "createPool",
            "createConnection",
        ];

        for pattern in patterns {
            self.register_heuristic_pattern(pattern.to_string());
        }
    }

    fn register_default_return_type_mappings(&mut self) {
        self.return_type_mappings
            .insert("fs/promises.open".to_string(), "FileHandle".to_string());
        self.return_type_mappings
            .insert("fsPromises.open".to_string(), "FileHandle".to_string());
        self.return_type_mappings
            .insert("open".to_string(), "FileHandle".to_string());
    }

    pub fn register_type(&mut self, info: DisposableTypeInfo) {
        self.types.insert(info.name.clone(), info);
    }

    pub fn register_heuristic_pattern(&mut self, pattern: String) {
        if !self.heuristic_patterns.contains(&pattern) {
            self.heuristic_patterns.push(pattern);
        }
    }

    pub fn register_return_type_mapping(&mut self, call_signature: String, return_type: String) {
        self.return_type_mappings
            .insert(call_signature, return_type);
    }

    pub fn is_disposable(&self, type_name: &str) -> bool {
        self.types.contains_key(type_name)
    }

    pub fn is_async_disposable(&self, type_name: &str) -> bool {
        self.types
            .get(type_name)
            .map(|info| info.is_async)
            .unwrap_or(false)
    }

    pub fn get_type_info(&self, type_name: &str) -> Option<&DisposableTypeInfo> {
        self.types.get(type_name)
    }

    pub fn matches_heuristic_pattern(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.heuristic_patterns
            .iter()
            .any(|pattern| name_lower.starts_with(&pattern.to_lowercase()))
    }

    pub fn get_return_type(&self, call_signature: &str) -> Option<&str> {
        self.return_type_mappings
            .get(call_signature)
            .map(|s| s.as_str())
    }

    pub fn all_types(&self) -> impl Iterator<Item = &DisposableTypeInfo> {
        self.types.values()
    }

    pub fn heuristic_patterns(&self) -> &[String] {
        &self.heuristic_patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_is_empty() {
        let registry = DisposableTypesRegistry::new();

        assert!(!registry.is_disposable("FileHandle"));
        assert!(!registry.is_async_disposable("FileHandle"));
        assert!(registry.get_type_info("FileHandle").is_none());
    }

    #[test]
    fn with_defaults_has_builtin_types() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.is_disposable("FileHandle"));
        assert!(registry.is_async_disposable("FileHandle"));

        assert!(registry.is_disposable("ReadableStream"));
        assert!(registry.is_async_disposable("ReadableStream"));

        assert!(registry.is_disposable("DisposableStack"));
        assert!(!registry.is_async_disposable("DisposableStack"));
    }

    #[test]
    fn file_handle_recognized() {
        let registry = DisposableTypesRegistry::with_defaults();

        let info = registry.get_type_info("FileHandle");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.name, "FileHandle");
        assert!(info.is_async);
        assert_eq!(info.source, DisposableSource::BuiltIn);
    }

    #[test]
    fn readable_stream_recognized() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.is_disposable("ReadableStream"));
        assert!(registry.is_async_disposable("ReadableStream"));

        assert!(registry.is_disposable("ReadableStreamDefaultReader"));
        assert!(registry.is_async_disposable("ReadableStreamDefaultReader"));
    }

    #[test]
    fn fs_promises_open_return_type() {
        let registry = DisposableTypesRegistry::with_defaults();

        let return_type = registry.get_return_type("fs/promises.open");
        assert_eq!(return_type, Some("FileHandle"));

        let return_type = registry.get_return_type("fsPromises.open");
        assert_eq!(return_type, Some("FileHandle"));

        let return_type = registry.get_return_type("open");
        assert_eq!(return_type, Some("FileHandle"));
    }

    #[test]
    fn heuristic_pattern_acquire() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.matches_heuristic_pattern("acquire"));
        assert!(registry.matches_heuristic_pattern("acquireLock"));
        assert!(registry.matches_heuristic_pattern("acquireConnection"));
        assert!(registry.matches_heuristic_pattern("ACQUIRE_RESOURCE"));
    }

    #[test]
    fn heuristic_pattern_connect() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.matches_heuristic_pattern("connect"));
        assert!(registry.matches_heuristic_pattern("connectToDatabase"));
        assert!(registry.matches_heuristic_pattern("connectTCP"));
    }

    #[test]
    fn heuristic_pattern_open() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.matches_heuristic_pattern("open"));
        assert!(registry.matches_heuristic_pattern("openFile"));
        assert!(registry.matches_heuristic_pattern("openStream"));
    }

    #[test]
    fn heuristic_pattern_create_pool() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(registry.matches_heuristic_pattern("createPool"));
        assert!(registry.matches_heuristic_pattern("createPoolConnection"));
    }

    #[test]
    fn heuristic_pattern_no_match() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(!registry.matches_heuristic_pattern("getData"));
        assert!(!registry.matches_heuristic_pattern("fetchUser"));
        assert!(!registry.matches_heuristic_pattern("readFile"));
    }

    #[test]
    fn register_custom_type() {
        let mut registry = DisposableTypesRegistry::new();

        registry.register_type(DisposableTypeInfo::new(
            "CustomResource",
            true,
            DisposableSource::Custom,
        ));

        assert!(registry.is_disposable("CustomResource"));
        assert!(registry.is_async_disposable("CustomResource"));

        let info = registry.get_type_info("CustomResource").unwrap();
        assert_eq!(info.source, DisposableSource::Custom);
    }

    #[test]
    fn register_custom_heuristic_pattern() {
        let mut registry = DisposableTypesRegistry::new();

        registry.register_heuristic_pattern("lease".to_string());

        assert!(registry.matches_heuristic_pattern("leaseResource"));
        assert!(registry.matches_heuristic_pattern("leaseConnection"));
    }

    #[test]
    fn register_custom_return_type_mapping() {
        let mut registry = DisposableTypesRegistry::new();

        registry.register_return_type_mapping(
            "db.getConnection".to_string(),
            "DatabaseConnection".to_string(),
        );

        assert_eq!(
            registry.get_return_type("db.getConnection"),
            Some("DatabaseConnection")
        );
    }

    #[test]
    fn all_types_iteration() {
        let registry = DisposableTypesRegistry::with_defaults();

        let types: Vec<_> = registry.all_types().collect();

        assert!(types.iter().any(|t| t.name == "FileHandle"));
        assert!(types.iter().any(|t| t.name == "ReadableStream"));
        assert!(types.iter().any(|t| t.name == "DisposableStack"));
    }

    #[test]
    fn heuristic_patterns_list() {
        let registry = DisposableTypesRegistry::with_defaults();

        let patterns = registry.heuristic_patterns();

        assert!(patterns.contains(&"acquire".to_string()));
        assert!(patterns.contains(&"connect".to_string()));
        assert!(patterns.contains(&"open".to_string()));
    }

    #[test]
    fn disposable_type_info_constructors() {
        let builtin = DisposableTypeInfo::builtin("Test", true);
        assert_eq!(builtin.source, DisposableSource::BuiltIn);

        let heuristic = DisposableTypeInfo::heuristic("Test", false);
        assert_eq!(heuristic.source, DisposableSource::HeuristicPattern);
    }

    #[test]
    fn unknown_type_returns_none() {
        let registry = DisposableTypesRegistry::with_defaults();

        assert!(!registry.is_disposable("UnknownType"));
        assert!(!registry.is_async_disposable("UnknownType"));
        assert!(registry.get_type_info("UnknownType").is_none());
        assert!(registry.get_return_type("unknownFunction").is_none());
    }

    #[test]
    fn duplicate_pattern_not_added() {
        let mut registry = DisposableTypesRegistry::new();

        registry.register_heuristic_pattern("acquire".to_string());
        registry.register_heuristic_pattern("acquire".to_string());

        assert_eq!(registry.heuristic_patterns().len(), 1);
    }
}
