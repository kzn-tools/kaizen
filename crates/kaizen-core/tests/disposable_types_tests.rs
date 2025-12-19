//! Integration tests for DisposableTypesRegistry
//!
//! These tests verify that the registry correctly identifies disposable types
//! from various Node.js and web platform APIs.

use kaizen_core::semantic::{DisposableSource, DisposableTypeInfo, DisposableTypesRegistry};

#[test]
fn registry_with_defaults_covers_acceptance_criteria() {
    let registry = DisposableTypesRegistry::with_defaults();

    // AC1: FileHandle recognized as AsyncDisposable
    assert!(registry.is_disposable("FileHandle"));
    assert!(registry.is_async_disposable("FileHandle"));
    let info = registry.get_type_info("FileHandle").unwrap();
    assert_eq!(info.source, DisposableSource::BuiltIn);

    // AC1: ReadableStream recognized as AsyncDisposable
    assert!(registry.is_disposable("ReadableStream"));
    assert!(registry.is_async_disposable("ReadableStream"));

    // AC2: fs/promises.open return type recognized
    assert_eq!(
        registry.get_return_type("fs/promises.open"),
        Some("FileHandle")
    );

    // AC3: Heuristic patterns (acquire, connect) supported
    assert!(registry.matches_heuristic_pattern("acquireLock"));
    assert!(registry.matches_heuristic_pattern("connectToDatabase"));
}

#[test]
fn can_check_if_function_returns_disposable() {
    let registry = DisposableTypesRegistry::with_defaults();

    let check_function = |fn_name: &str| -> bool {
        if let Some(return_type) = registry.get_return_type(fn_name) {
            registry.is_disposable(return_type)
        } else {
            registry.matches_heuristic_pattern(fn_name)
        }
    };

    // Direct mappings
    assert!(check_function("fs/promises.open"));
    assert!(check_function("open"));

    // Heuristic patterns
    assert!(check_function("acquireConnection"));
    assert!(check_function("connectToServer"));
    assert!(check_function("openFile"));

    // Non-disposable functions
    assert!(!check_function("getData"));
    assert!(!check_function("parseJSON"));
}

#[test]
fn sync_vs_async_disposable_distinction() {
    let registry = DisposableTypesRegistry::with_defaults();

    // DisposableStack is synchronous
    assert!(registry.is_disposable("DisposableStack"));
    assert!(!registry.is_async_disposable("DisposableStack"));

    // AsyncDisposableStack is asynchronous
    assert!(registry.is_disposable("AsyncDisposableStack"));
    assert!(registry.is_async_disposable("AsyncDisposableStack"));
}

#[test]
fn custom_types_can_be_registered() {
    let mut registry = DisposableTypesRegistry::new();

    // Register a custom database connection type
    registry.register_type(DisposableTypeInfo::new(
        "DatabaseConnection",
        true,
        DisposableSource::Custom,
    ));

    // Register custom return type mapping
    registry
        .register_return_type_mapping("db.connect".to_string(), "DatabaseConnection".to_string());

    // Verify it works
    assert!(registry.is_disposable("DatabaseConnection"));
    assert!(registry.is_async_disposable("DatabaseConnection"));
    assert_eq!(
        registry.get_return_type("db.connect"),
        Some("DatabaseConnection")
    );
}

#[test]
fn stream_types_are_recognized() {
    let registry = DisposableTypesRegistry::with_defaults();

    // All stream-related types should be recognized
    let stream_types = [
        "ReadableStream",
        "ReadableStreamDefaultReader",
        "WritableStream",
        "WritableStreamDefaultWriter",
    ];

    for type_name in stream_types {
        assert!(
            registry.is_disposable(type_name),
            "{} should be disposable",
            type_name
        );
        assert!(
            registry.is_async_disposable(type_name),
            "{} should be async disposable",
            type_name
        );
    }
}

#[test]
fn heuristic_patterns_case_insensitive() {
    let registry = DisposableTypesRegistry::with_defaults();

    // Pattern matching should be case-insensitive
    assert!(registry.matches_heuristic_pattern("acquire"));
    assert!(registry.matches_heuristic_pattern("Acquire"));
    assert!(registry.matches_heuristic_pattern("ACQUIRE"));
    assert!(registry.matches_heuristic_pattern("acquireLock"));
    assert!(registry.matches_heuristic_pattern("AcquireLock"));
}
