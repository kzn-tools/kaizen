//! Shared helper functions for rule implementations.
//!
//! This module provides common utilities that are used across multiple rules.

use std::ops::ControlFlow;

use swc_ecma_ast::JSXElement;

use crate::visitor::{AstVisitor, VisitorContext, walk_ast};

/// Check if the module contains any JSX elements.
///
/// This is used to detect files that contain JSX syntax, regardless of their
/// file extension. This allows us to correctly handle `.js` files that contain
/// JSX (common in legacy projects using Babel).
pub fn file_contains_jsx(module: &swc_ecma_ast::Module, ctx: &VisitorContext) -> bool {
    let mut visitor = JsxDetector { found_jsx: false };
    walk_ast(module, &mut visitor, ctx);
    visitor.found_jsx
}

struct JsxDetector {
    found_jsx: bool,
}

impl AstVisitor for JsxDetector {
    fn visit_jsx_element(&mut self, _node: &JSXElement, _ctx: &VisitorContext) -> ControlFlow<()> {
        self.found_jsx = true;
        // Stop early - we only need to find one JSX element
        ControlFlow::Break(())
    }
}

/// Check if the filename indicates a test file.
///
/// This function recognizes common test file patterns used in JavaScript/TypeScript
/// projects, including:
/// - Files with `.test.` or `.spec.` in their name
/// - Files with `_test.` or `_spec.` in their name
/// - Files named `test.js`, `test.ts`, etc.
/// - Files in directories named `test/`, `tests/`, `__tests__/`, or `__mocks__/`
pub fn is_test_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();

    // Common test file patterns
    lower.contains(".test.")
        || lower.contains(".spec.")
        || lower.contains("_test.")
        || lower.contains("_spec.")
        || lower.ends_with(".test.js")
        || lower.ends_with(".test.ts")
        || lower.ends_with(".test.jsx")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".spec.js")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.jsx")
        || lower.ends_with(".spec.tsx")
        // test.js files (common pattern in some projects)
        || lower.ends_with("/test.js")
        || lower.ends_with("/test.mjs")
        || lower.ends_with("/test.ts")
        || lower == "test.js"
        || lower == "test.mjs"
        || lower == "test.ts"
        || lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
        || lower.contains("/__mocks__/")
        || lower.starts_with("test/")
        || lower.starts_with("tests/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_file_with_test_suffix() {
        assert!(is_test_file("component.test.js"));
        assert!(is_test_file("component.test.ts"));
        assert!(is_test_file("component.test.jsx"));
        assert!(is_test_file("component.test.tsx"));
    }

    #[test]
    fn test_is_test_file_with_spec_suffix() {
        assert!(is_test_file("component.spec.js"));
        assert!(is_test_file("component.spec.ts"));
        assert!(is_test_file("utils_spec.js"));
    }

    #[test]
    fn test_is_test_file_in_test_directories() {
        assert!(is_test_file("src/__tests__/file.js"));
        assert!(is_test_file("test/helpers.js"));
        assert!(is_test_file("tests/unit/file.js"));
        assert!(is_test_file("src/__mocks__/api.js"));
    }

    #[test]
    fn test_is_test_file_standalone_test_files() {
        assert!(is_test_file("test.js"));
        assert!(is_test_file("test.mjs"));
        assert!(is_test_file("test.ts"));
        assert!(is_test_file("/path/to/test.js"));
    }

    #[test]
    fn test_is_not_test_file() {
        assert!(!is_test_file("component.js"));
        assert!(!is_test_file("utils.ts"));
        assert!(!is_test_file("src/test-utils.js")); // Contains 'test' but not a test file pattern
        assert!(!is_test_file("testing.js")); // Contains 'test' but not a test file pattern
    }
}
