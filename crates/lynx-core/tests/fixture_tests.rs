//! Integration tests for parsing fixtures from tests/fixtures/

use std::fs;
use std::path::Path;

use lynx_core::parser::{Language, ParsedFile, Parser};

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");

fn read_fixture(relative_path: &str) -> String {
    let path = Path::new(FIXTURES_DIR).join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
}

fn collect_fixtures(subdir: &str, extensions: &[&str]) -> Vec<(String, String)> {
    let dir_path = Path::new(FIXTURES_DIR).join(subdir);
    if !dir_path.exists() {
        return vec![];
    }

    let mut fixtures = vec![];
    for entry in fs::read_dir(&dir_path).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if extensions.iter().any(|e| ext == *e) {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).expect("Failed to read fixture file");
                fixtures.push((name, content));
            }
        }
    }
    fixtures.sort_by(|a, b| a.0.cmp(&b.0));
    fixtures
}

#[test]
fn parse_all_js_fixtures() {
    let js_extensions = ["js", "mjs", "cjs", "jsx"];
    let fixtures = collect_fixtures("javascript", &js_extensions);

    assert!(
        !fixtures.is_empty(),
        "No JavaScript fixtures found in tests/fixtures/javascript/"
    );

    for (filename, content) in &fixtures {
        let parser = Parser::for_file(filename);
        let result = parser.parse_module_recovering(content);

        assert!(
            result.is_ok(),
            "JavaScript fixture {} failed to parse: {:?}",
            filename,
            result.errors
        );
        assert!(
            result.module.is_some(),
            "JavaScript fixture {} produced no AST",
            filename
        );
    }
}

#[test]
fn parse_all_ts_fixtures() {
    let ts_extensions = ["ts", "mts", "cts", "tsx"];
    let fixtures_ts = collect_fixtures("typescript", &ts_extensions);
    let fixtures_valid = collect_fixtures("valid", &ts_extensions);
    let fixtures_quality = collect_fixtures("quality", &ts_extensions);
    let fixtures_security = collect_fixtures("security", &ts_extensions);

    let mut all_fixtures = vec![];
    all_fixtures.extend(fixtures_ts);
    all_fixtures.extend(fixtures_valid);
    all_fixtures.extend(fixtures_quality);
    all_fixtures.extend(fixtures_security);

    assert!(!all_fixtures.is_empty(), "No TypeScript fixtures found");

    for (filename, content) in &all_fixtures {
        let parser = Parser::for_file(filename);
        let result = parser.parse_module_recovering(content);

        assert!(
            result.is_ok(),
            "TypeScript fixture {} failed to parse: {:?}",
            filename,
            result.errors
        );
        assert!(
            result.module.is_some(),
            "TypeScript fixture {} produced no AST",
            filename
        );
    }
}

#[test]
fn parsed_file_detects_correct_language() {
    let js_fixture = read_fixture("javascript/simple.js");
    let ts_fixture = read_fixture("typescript/simple.ts");
    let jsx_fixture = read_fixture("javascript/jsx-component.jsx");
    let tsx_fixture = read_fixture("typescript/tsx-component.tsx");

    let js_parsed = ParsedFile::from_source("simple.js", &js_fixture);
    let ts_parsed = ParsedFile::from_source("simple.ts", &ts_fixture);
    let jsx_parsed = ParsedFile::from_source("component.jsx", &jsx_fixture);
    let tsx_parsed = ParsedFile::from_source("component.tsx", &tsx_fixture);

    assert_eq!(js_parsed.metadata().language, Language::JavaScript);
    assert_eq!(ts_parsed.metadata().language, Language::TypeScript);
    assert_eq!(jsx_parsed.metadata().language, Language::Jsx);
    assert_eq!(tsx_parsed.metadata().language, Language::Tsx);
}

#[test]
fn all_fixtures_produce_valid_metadata() {
    let all_extensions = ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"];
    let dirs = ["javascript", "typescript", "valid", "quality", "security"];

    for dir in dirs {
        let fixtures = collect_fixtures(dir, &all_extensions);
        for (filename, content) in &fixtures {
            let parsed = ParsedFile::from_source(filename, content);
            let metadata = parsed.metadata();

            assert_eq!(metadata.filename, *filename);
            assert!(metadata.line_count > 0, "Fixture {} has no lines", filename);
            assert!(
                !metadata.has_errors,
                "Fixture {} has parse errors: {:?}",
                filename,
                parsed.errors()
            );
        }
    }
}

mod snapshots {
    use super::*;
    use insta::assert_json_snapshot;
    use serde::Serialize;
    use swc_ecma_ast::{FnDecl, ModuleItem, Stmt};

    #[derive(Serialize)]
    struct SimpleFunctionSnapshot {
        name: String,
        params_count: usize,
        is_async: bool,
        is_generator: bool,
    }

    fn extract_function_info(module: &swc_ecma_ast::Module) -> Vec<SimpleFunctionSnapshot> {
        let mut functions = vec![];
        for item in &module.body {
            if let ModuleItem::Stmt(Stmt::Decl(swc_ecma_ast::Decl::Fn(FnDecl {
                ident,
                function,
                ..
            }))) = item
            {
                functions.push(SimpleFunctionSnapshot {
                    name: ident.sym.to_string(),
                    params_count: function.params.len(),
                    is_async: function.is_async,
                    is_generator: function.is_generator,
                });
            }
        }
        functions
    }

    #[test]
    fn ast_snapshot_simple_function() {
        let code = r#"
function calculateArea(radius) {
    return Math.PI * radius * radius;
}

async function fetchData(url) {
    const response = await fetch(url);
    return response.json();
}

function* generateNumbers(n) {
    for (let i = 0; i < n; i++) {
        yield i;
    }
}
"#;

        let parser = Parser::new();
        let result = parser.parse_module_recovering(code);
        let module = result.module.expect("Failed to parse simple function");

        let functions = extract_function_info(&module);
        assert_json_snapshot!(functions);
    }

    #[test]
    fn ast_snapshot_typescript_interfaces() {
        let fixture = read_fixture("typescript/interfaces.ts");
        let parsed = ParsedFile::from_source("interfaces.ts", &fixture);

        assert!(
            parsed.module().is_some(),
            "Failed to parse TypeScript interfaces fixture"
        );
        assert!(!parsed.metadata().has_errors);

        let module = parsed.module().unwrap();
        let functions = extract_function_info(module);
        assert_json_snapshot!(functions);
    }
}
