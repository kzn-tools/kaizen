//! Integration tests for SemanticModel with complex fixtures
//!
//! Tests closures, classes, modules, and edge cases using snapshot testing.

use std::fs;
use std::path::Path;

use insta::assert_json_snapshot;
use kaizen_core::parser::ParsedFile;
use kaizen_core::semantic::{DeclarationKind, ScopeBuilder, ScopeKind, SemanticModel, SymbolKind};
use serde::Serialize;

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");

fn read_fixture(relative_path: &str) -> String {
    let path = Path::new(FIXTURES_DIR).join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
}

fn build_semantic_model(code: &str, filename: &str) -> SemanticModel {
    let parsed = ParsedFile::from_source(filename, code);
    let module = parsed.module().expect("parse failed");
    ScopeBuilder::build(module)
}

#[derive(Serialize)]
struct ScopeSnapshot {
    kind: String,
    children_count: usize,
    has_parent: bool,
}

#[derive(Serialize)]
struct SymbolSnapshot {
    name: String,
    kind: String,
    declaration_kind: String,
    is_exported: bool,
    references_count: usize,
}

#[derive(Serialize)]
struct SemanticModelSnapshot {
    scope_count: usize,
    scopes: Vec<ScopeSnapshot>,
    symbol_count: usize,
    symbols: Vec<SymbolSnapshot>,
    unresolved_count: usize,
}

fn scope_kind_to_string(kind: ScopeKind) -> String {
    match kind {
        ScopeKind::Global => "Global",
        ScopeKind::Module => "Module",
        ScopeKind::Function => "Function",
        ScopeKind::ArrowFunction => "ArrowFunction",
        ScopeKind::Block => "Block",
        ScopeKind::For => "For",
        ScopeKind::While => "While",
        ScopeKind::Switch => "Switch",
        ScopeKind::Try => "Try",
        ScopeKind::Catch => "Catch",
        ScopeKind::Class => "Class",
    }
    .to_string()
}

fn symbol_kind_to_string(kind: SymbolKind) -> String {
    match kind {
        SymbolKind::Variable => "Variable",
        SymbolKind::Constant => "Constant",
        SymbolKind::Function => "Function",
        SymbolKind::Class => "Class",
        SymbolKind::Parameter => "Parameter",
        SymbolKind::Import => "Import",
        SymbolKind::TypeAlias => "TypeAlias",
        SymbolKind::Enum => "Enum",
    }
    .to_string()
}

fn declaration_kind_to_string(kind: DeclarationKind) -> String {
    match kind {
        DeclarationKind::Var => "Var",
        DeclarationKind::Let => "Let",
        DeclarationKind::Const => "Const",
        DeclarationKind::Function => "Function",
        DeclarationKind::Class => "Class",
        DeclarationKind::Parameter => "Parameter",
        DeclarationKind::Import => "Import",
        DeclarationKind::TypeAlias => "TypeAlias",
        DeclarationKind::Enum => "Enum",
    }
    .to_string()
}

fn create_snapshot(model: &SemanticModel) -> SemanticModelSnapshot {
    let root = model.scope_tree.root().expect("no root scope");

    let mut scopes = Vec::new();
    let mut scope_stack = vec![root];
    let mut scope_count = 0;

    while let Some(scope_id) = scope_stack.pop() {
        scope_count += 1;
        let scope = model.scope_tree.get(scope_id);
        scopes.push(ScopeSnapshot {
            kind: scope_kind_to_string(scope.kind),
            children_count: scope.children.len(),
            has_parent: scope.parent.is_some(),
        });
        for &child in scope.children.iter().rev() {
            scope_stack.push(child);
        }
    }

    let mut symbols: Vec<SymbolSnapshot> = model
        .symbol_table
        .all_symbols()
        .map(|s| SymbolSnapshot {
            name: s.name.clone(),
            kind: symbol_kind_to_string(s.kind),
            declaration_kind: declaration_kind_to_string(s.declaration_kind),
            is_exported: s.is_exported,
            references_count: s.references.len(),
        })
        .collect();

    symbols.sort_by(|a, b| a.name.cmp(&b.name));

    SemanticModelSnapshot {
        scope_count,
        scopes,
        symbol_count: symbols.len(),
        symbols,
        unresolved_count: model.unresolved_references.len(),
    }
}

mod closures {
    use super::*;

    #[test]
    fn closures_fixture_snapshot() {
        let code = read_fixture("semantic/closures.mts");
        let model = build_semantic_model(&code, "closures.mts");
        let snapshot = create_snapshot(&model);
        assert_json_snapshot!(snapshot);
    }

    #[test]
    fn nested_closure_captures_outer_variable() {
        let code = r#"
function outer(a) {
    function middle(b) {
        function inner(c) {
            return a + b + c;
        }
        return inner;
    }
    return middle;
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let outer_scope = model.scope_tree.get(global).children[0];
        let middle_scope = model.scope_tree.get(outer_scope).children[0];
        let inner_scope = model.scope_tree.get(middle_scope).children[0];

        let a_from_inner = model
            .symbol_table
            .lookup("a", inner_scope, &model.scope_tree);
        assert!(a_from_inner.is_some());
        let a_symbol = model.symbol_table.get(a_from_inner.unwrap());
        assert_eq!(a_symbol.scope, outer_scope);
        assert!(!a_symbol.references.is_empty());

        let b_from_inner = model
            .symbol_table
            .lookup("b", inner_scope, &model.scope_tree);
        assert!(b_from_inner.is_some());
        let b_symbol = model.symbol_table.get(b_from_inner.unwrap());
        assert_eq!(b_symbol.scope, middle_scope);
        assert!(!b_symbol.references.is_empty());
    }

    #[test]
    fn iife_creates_isolated_scope() {
        let code = r#"
const result = (function(x) {
    const privateValue = x * 2;
    return privateValue + 1;
})(5);
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let result_symbol = model
            .symbol_table
            .lookup("result", global, &model.scope_tree);
        assert!(result_symbol.is_some());

        let iife_scope = model.scope_tree.get(global).children[0];
        assert_eq!(model.scope_tree.get(iife_scope).kind, ScopeKind::Function);

        let private_value =
            model
                .symbol_table
                .lookup("privateValue", iife_scope, &model.scope_tree);
        assert!(private_value.is_some());

        let not_in_global = model
            .symbol_table
            .symbols_in_scope(global)
            .find(|s| s.name == "privateValue");
        assert!(not_in_global.is_none());
    }

    #[test]
    fn arrow_function_closure() {
        let code = r#"
const multiplier = 2;
const double = (x) => x * multiplier;
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let arrow_scope = model.scope_tree.get(global).children[0];
        assert_eq!(
            model.scope_tree.get(arrow_scope).kind,
            ScopeKind::ArrowFunction
        );

        let multiplier_id = model
            .symbol_table
            .lookup("multiplier", arrow_scope, &model.scope_tree);
        assert!(multiplier_id.is_some());
        let multiplier = model.symbol_table.get(multiplier_id.unwrap());
        assert_eq!(multiplier.scope, global);
        assert!(!multiplier.references.is_empty());
    }

    #[test]
    fn closure_in_loop() {
        let code = r#"
const closures = [];
for (let i = 0; i < 3; i++) {
    closures.push(() => i);
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let for_scope = model.scope_tree.get(global).children[0];
        assert_eq!(model.scope_tree.get(for_scope).kind, ScopeKind::For);

        let i_symbol = model
            .symbol_table
            .lookup("i", for_scope, &model.scope_tree)
            .unwrap();
        let i = model.symbol_table.get(i_symbol);
        assert_eq!(i.scope, for_scope);
        assert!(
            !i.references.is_empty(),
            "i should be referenced at least once"
        );
    }
}

mod classes {
    use super::*;

    #[test]
    fn classes_fixture_snapshot() {
        let code = read_fixture("semantic/classes.mts");
        let model = build_semantic_model(&code, "classes.mts");
        let snapshot = create_snapshot(&model);
        assert_json_snapshot!(snapshot);
    }

    #[test]
    fn class_creates_class_scope() {
        let code = r#"
class MyClass {
    constructor(value) {
        this.value = value;
    }

    getValue() {
        return this.value;
    }
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let class_symbol = model
            .symbol_table
            .lookup("MyClass", global, &model.scope_tree);
        assert!(class_symbol.is_some());
        assert_eq!(
            model.symbol_table.get(class_symbol.unwrap()).kind,
            SymbolKind::Class
        );

        let class_scope = model.scope_tree.get(global).children[0];
        assert_eq!(model.scope_tree.get(class_scope).kind, ScopeKind::Class);
    }

    #[test]
    fn class_with_inheritance() {
        let code = r#"
class Animal {
    constructor(name) {
        this.name = name;
    }
}

class Dog extends Animal {
    constructor(name, breed) {
        super(name);
        this.breed = breed;
    }
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        let animal = model
            .symbol_table
            .lookup("Animal", global, &model.scope_tree);
        let dog = model.symbol_table.lookup("Dog", global, &model.scope_tree);

        assert!(animal.is_some());
        assert!(dog.is_some());
        assert_eq!(
            model.symbol_table.get(animal.unwrap()).kind,
            SymbolKind::Class
        );
        assert_eq!(model.symbol_table.get(dog.unwrap()).kind, SymbolKind::Class);
    }

    #[test]
    fn class_expression() {
        let code = r#"
const Rectangle = class {
    constructor(width, height) {
        this.width = width;
        this.height = height;
    }
};
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let rectangle = model
            .symbol_table
            .lookup("Rectangle", global, &model.scope_tree);
        assert!(rectangle.is_some());
        assert_eq!(
            model.symbol_table.get(rectangle.unwrap()).kind,
            SymbolKind::Constant
        );

        let class_scope = model.scope_tree.get(global).children[0];
        assert_eq!(model.scope_tree.get(class_scope).kind, ScopeKind::Class);
    }

    #[test]
    fn static_members() {
        let code = r#"
class MathUtils {
    static PI = 3.14159;

    static square(x) {
        return x * x;
    }
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let math_utils = model
            .symbol_table
            .lookup("MathUtils", global, &model.scope_tree);
        assert!(math_utils.is_some());
    }
}

mod modules {
    use super::*;

    #[test]
    fn modules_fixture_snapshot() {
        let code = read_fixture("semantic/modules.mts");
        let model = build_semantic_model(&code, "modules.mts");
        let snapshot = create_snapshot(&model);
        assert_json_snapshot!(snapshot);
    }

    #[test]
    fn named_imports_registered() {
        let code = r#"
import { readFile, writeFile } from 'fs';
import { join, resolve } from 'path';
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        for name in &["readFile", "writeFile", "join", "resolve"] {
            let symbol = model
                .symbol_table
                .lookup(name, global, &model.scope_tree)
                .unwrap();
            assert_eq!(
                model.symbol_table.get(symbol).kind,
                SymbolKind::Import,
                "{} should be Import",
                name
            );
        }
    }

    #[test]
    fn namespace_import() {
        let code = r#"
import * as utils from './utils';
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let utils = model
            .symbol_table
            .lookup("utils", global, &model.scope_tree);
        assert!(utils.is_some());
        assert_eq!(
            model.symbol_table.get(utils.unwrap()).kind,
            SymbolKind::Import
        );
    }

    #[test]
    fn default_import() {
        let code = r#"
import DefaultClass from './module';
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let default_import = model
            .symbol_table
            .lookup("DefaultClass", global, &model.scope_tree);
        assert!(default_import.is_some());
        assert_eq!(
            model.symbol_table.get(default_import.unwrap()).kind,
            SymbolKind::Import
        );
    }

    #[test]
    fn exported_symbols_marked() {
        let code = r#"
export const VERSION = '1.0.0';
export function helper() {}
export class Service {}
const private_var = 1;
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        let version = model
            .symbol_table
            .lookup("VERSION", global, &model.scope_tree)
            .unwrap();
        let helper = model
            .symbol_table
            .lookup("helper", global, &model.scope_tree)
            .unwrap();
        let service = model
            .symbol_table
            .lookup("Service", global, &model.scope_tree)
            .unwrap();
        let private_var = model
            .symbol_table
            .lookup("private_var", global, &model.scope_tree)
            .unwrap();

        assert!(model.symbol_table.get(version).is_exported);
        assert!(model.symbol_table.get(helper).is_exported);
        assert!(model.symbol_table.get(service).is_exported);
        assert!(!model.symbol_table.get(private_var).is_exported);
    }

    #[test]
    fn mixed_import() {
        let code = r#"
import React, { useState, useEffect } from 'react';
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        for name in &["React", "useState", "useEffect"] {
            let symbol = model.symbol_table.lookup(name, global, &model.scope_tree);
            assert!(symbol.is_some(), "{} should be imported", name);
            assert_eq!(
                model.symbol_table.get(symbol.unwrap()).kind,
                SymbolKind::Import
            );
        }
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn edge_cases_fixture_snapshot() {
        let code = read_fixture("semantic/edge-cases.mts");
        let model = build_semantic_model(&code, "edge-cases.mts");
        let snapshot = create_snapshot(&model);
        assert_json_snapshot!(snapshot);
    }

    #[test]
    fn variable_shadowing() {
        let code = r#"
const x = 1;
{
    const x = 2;
    console.log(x);
}
console.log(x);
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let block_scope = model.scope_tree.get(global).children[0];

        let outer_x = model
            .symbol_table
            .symbols_in_scope(global)
            .find(|s| s.name == "x")
            .unwrap();
        let inner_x = model
            .symbol_table
            .symbols_in_scope(block_scope)
            .find(|s| s.name == "x")
            .unwrap();

        assert_ne!(outer_x.id, inner_x.id);
        assert_eq!(outer_x.scope, global);
        assert_eq!(inner_x.scope, block_scope);
    }

    #[test]
    fn var_hoisting() {
        let code = r#"
function test() {
    if (true) {
        var hoisted = 1;
    }
    console.log(hoisted);
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let func_scope = model.scope_tree.get(global).children[0];

        let hoisted = model
            .symbol_table
            .lookup("hoisted", func_scope, &model.scope_tree);
        assert!(hoisted.is_some());
        assert_eq!(
            model.symbol_table.get(hoisted.unwrap()).scope,
            func_scope,
            "var should be hoisted to function scope"
        );
    }

    #[test]
    fn complex_destructuring() {
        let code = r#"
const { a, b: { c, d: { e } }, f: [first, ...rest] } = obj;
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        for name in &["a", "c", "e", "first", "rest"] {
            let symbol = model.symbol_table.lookup(name, global, &model.scope_tree);
            assert!(symbol.is_some(), "{} should be declared", name);
        }

        let b = model.symbol_table.lookup("b", global, &model.scope_tree);
        assert!(b.is_none(), "b should not be declared (it's a pattern)");

        let d = model.symbol_table.lookup("d", global, &model.scope_tree);
        assert!(d.is_none(), "d should not be declared (it's a pattern)");
    }

    #[test]
    fn for_of_destructuring() {
        let code = r#"
const items = [{ key: 'a', value: 1 }];
for (const { key, value } of items) {
    console.log(key, value);
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let for_scope = model.scope_tree.get(global).children[0];

        let key = model
            .symbol_table
            .lookup("key", for_scope, &model.scope_tree);
        let value = model
            .symbol_table
            .lookup("value", for_scope, &model.scope_tree);

        assert!(key.is_some());
        assert!(value.is_some());
    }

    #[test]
    fn try_catch_finally_scoping() {
        let code = r#"
try {
    throw new Error('test');
} catch (error) {
    const catchScoped = 'only in catch';
} finally {
    const finallyScoped = 'only in finally';
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();

        let try_scope = model
            .scope_tree
            .children(global)
            .find(|s| s.kind == ScopeKind::Try);
        let catch_scope = model
            .scope_tree
            .children(global)
            .find(|s| s.kind == ScopeKind::Catch);

        assert!(try_scope.is_some());
        assert!(catch_scope.is_some());

        let catch_id = catch_scope.unwrap().id;
        let error = model
            .symbol_table
            .lookup("error", catch_id, &model.scope_tree);
        assert!(error.is_some());
    }

    #[test]
    fn generator_function() {
        let code = r#"
function* generator() {
    const a = yield 1;
    const b = yield 2;
    return a + b;
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let generator = model
            .symbol_table
            .lookup("generator", global, &model.scope_tree);
        assert!(generator.is_some());
        assert_eq!(
            model.symbol_table.get(generator.unwrap()).kind,
            SymbolKind::Function
        );

        let func_scope = model.scope_tree.get(global).children[0];
        let a = model
            .symbol_table
            .lookup("a", func_scope, &model.scope_tree);
        let b = model
            .symbol_table
            .lookup("b", func_scope, &model.scope_tree);
        assert!(a.is_some());
        assert!(b.is_some());
    }

    #[test]
    fn async_function() {
        let code = r#"
async function asyncExample() {
    const result = await Promise.resolve(42);
    return result;
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let async_func = model
            .symbol_table
            .lookup("asyncExample", global, &model.scope_tree);
        assert!(async_func.is_some());

        let func_scope = model.scope_tree.get(global).children[0];
        let result = model
            .symbol_table
            .lookup("result", func_scope, &model.scope_tree);
        assert!(result.is_some());
    }

    #[test]
    fn labeled_statement_loop_variables() {
        let code = r#"
outer: for (let i = 0; i < 3; i++) {
    inner: for (let j = 0; j < 3; j++) {
        console.log(i, j);
    }
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let outer_for = model.scope_tree.get(global).children[0];

        let i = model.symbol_table.lookup("i", outer_for, &model.scope_tree);
        assert!(i.is_some(), "i should be declared in outer for loop");

        let j = model.symbol_table.all_symbols().find(|s| s.name == "j");
        assert!(
            j.is_some(),
            "j should be declared somewhere in the nested structure"
        );
    }

    #[test]
    fn var_redeclaration() {
        let code = r#"
var redeclared = 1;
var redeclared = 2;
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let redeclared = model
            .symbol_table
            .lookup("redeclared", global, &model.scope_tree);
        assert!(redeclared.is_some());
    }

    #[test]
    fn default_parameter_referencing_previous() {
        let code = r#"
function defaults(a, b = a * 2) {
    return a + b;
}
"#;
        let model = build_semantic_model(code, "test.js");

        let global = model.scope_tree.root().unwrap();
        let func_scope = model.scope_tree.get(global).children[0];

        let a = model
            .symbol_table
            .lookup("a", func_scope, &model.scope_tree);
        let b = model
            .symbol_table
            .lookup("b", func_scope, &model.scope_tree);

        assert!(a.is_some());
        assert!(b.is_some());

        let a_symbol = model.symbol_table.get(a.unwrap());
        assert!(
            !a_symbol.references.is_empty(),
            "a should be referenced in default parameter"
        );
    }
}

mod typescript {
    use super::*;

    #[test]
    fn typescript_interfaces_fixture_snapshot() {
        let code = read_fixture("typescript/interfaces.ts");
        let model = build_semantic_model(&code, "interfaces.ts");
        let snapshot = create_snapshot(&model);
        assert_json_snapshot!(snapshot);
    }

    #[test]
    fn interface_registered_as_symbol() {
        let code = r#"
interface User {
    id: number;
    name: string;
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();
        let user = model.symbol_table.lookup("User", global, &model.scope_tree);
        assert!(user.is_some(), "Interface User should be registered");
        assert_eq!(
            model.symbol_table.get(user.unwrap()).kind,
            SymbolKind::TypeAlias
        );
    }

    #[test]
    fn type_alias_registered_as_symbol() {
        let code = r#"
type UserId = number;
type UserRole = 'admin' | 'user' | 'guest';
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let user_id = model
            .symbol_table
            .lookup("UserId", global, &model.scope_tree);
        let user_role = model
            .symbol_table
            .lookup("UserRole", global, &model.scope_tree);

        assert!(user_id.is_some(), "Type alias UserId should be registered");
        assert!(
            user_role.is_some(),
            "Type alias UserRole should be registered"
        );

        assert_eq!(
            model.symbol_table.get(user_id.unwrap()).kind,
            SymbolKind::TypeAlias
        );
        assert_eq!(
            model.symbol_table.get(user_role.unwrap()).kind,
            SymbolKind::TypeAlias
        );
    }

    #[test]
    fn enum_registered_as_symbol() {
        let code = r#"
enum Status {
    Pending,
    Active,
    Completed
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();
        let status = model
            .symbol_table
            .lookup("Status", global, &model.scope_tree);

        assert!(status.is_some(), "Enum Status should be registered");
        assert_eq!(
            model.symbol_table.get(status.unwrap()).kind,
            SymbolKind::Enum
        );
    }

    #[test]
    fn exported_interface_marked_as_exported() {
        let code = r#"
export interface PublicApi {
    endpoint: string;
}

interface PrivateApi {
    secret: string;
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let public_api = model
            .symbol_table
            .lookup("PublicApi", global, &model.scope_tree)
            .unwrap();
        let private_api = model
            .symbol_table
            .lookup("PrivateApi", global, &model.scope_tree)
            .unwrap();

        assert!(
            model.symbol_table.get(public_api).is_exported,
            "PublicApi should be exported"
        );
        assert!(
            !model.symbol_table.get(private_api).is_exported,
            "PrivateApi should not be exported"
        );
    }

    #[test]
    fn exported_type_alias_marked_as_exported() {
        let code = r#"
export type UserId = string;
type InternalId = number;
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let user_id = model
            .symbol_table
            .lookup("UserId", global, &model.scope_tree)
            .unwrap();
        let internal_id = model
            .symbol_table
            .lookup("InternalId", global, &model.scope_tree)
            .unwrap();

        assert!(
            model.symbol_table.get(user_id).is_exported,
            "UserId should be exported"
        );
        assert!(
            !model.symbol_table.get(internal_id).is_exported,
            "InternalId should not be exported"
        );
    }

    #[test]
    fn exported_enum_marked_as_exported() {
        let code = r#"
export enum PublicStatus {
    Open,
    Closed
}

enum PrivateStatus {
    Internal
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let public_status = model
            .symbol_table
            .lookup("PublicStatus", global, &model.scope_tree)
            .unwrap();
        let private_status = model
            .symbol_table
            .lookup("PrivateStatus", global, &model.scope_tree)
            .unwrap();

        assert!(
            model.symbol_table.get(public_status).is_exported,
            "PublicStatus should be exported"
        );
        assert!(
            !model.symbol_table.get(private_status).is_exported,
            "PrivateStatus should not be exported"
        );
    }

    #[test]
    fn type_only_import_registered() {
        let code = r#"
import type { User, Role } from './types';
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let user = model.symbol_table.lookup("User", global, &model.scope_tree);
        let role = model.symbol_table.lookup("Role", global, &model.scope_tree);

        assert!(user.is_some(), "Type import User should be registered");
        assert!(role.is_some(), "Type import Role should be registered");

        assert_eq!(
            model.symbol_table.get(user.unwrap()).kind,
            SymbolKind::Import
        );
        assert_eq!(
            model.symbol_table.get(role.unwrap()).kind,
            SymbolKind::Import
        );
    }

    #[test]
    fn generic_interface_registered() {
        let code = r#"
interface Repository<T> {
    find(id: number): Promise<T | null>;
    save(entity: T): Promise<T>;
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();
        let repo = model
            .symbol_table
            .lookup("Repository", global, &model.scope_tree);

        assert!(
            repo.is_some(),
            "Generic interface Repository should be registered"
        );
        assert_eq!(
            model.symbol_table.get(repo.unwrap()).kind,
            SymbolKind::TypeAlias
        );
    }

    #[test]
    fn function_with_type_annotations() {
        let code = r#"
function add(a: number, b: number): number {
    return a + b;
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();
        let add_fn = model.symbol_table.lookup("add", global, &model.scope_tree);

        assert!(add_fn.is_some(), "Function add should be registered");
        assert_eq!(
            model.symbol_table.get(add_fn.unwrap()).kind,
            SymbolKind::Function
        );

        let func_scope = model.scope_tree.get(global).children[0];
        let a = model
            .symbol_table
            .lookup("a", func_scope, &model.scope_tree);
        let b = model
            .symbol_table
            .lookup("b", func_scope, &model.scope_tree);

        assert!(a.is_some(), "Parameter a should be registered");
        assert!(b.is_some(), "Parameter b should be registered");
    }

    #[test]
    fn class_implementing_interface() {
        let code = r#"
interface Comparable {
    compare(other: unknown): number;
}

class NumberWrapper implements Comparable {
    constructor(private value: number) {}

    compare(other: unknown): number {
        return this.value - (other as number);
    }
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        let comparable = model
            .symbol_table
            .lookup("Comparable", global, &model.scope_tree);
        let number_wrapper = model
            .symbol_table
            .lookup("NumberWrapper", global, &model.scope_tree);

        assert!(
            comparable.is_some(),
            "Interface Comparable should be registered"
        );
        assert!(
            number_wrapper.is_some(),
            "Class NumberWrapper should be registered"
        );

        assert_eq!(
            model.symbol_table.get(comparable.unwrap()).kind,
            SymbolKind::TypeAlias
        );
        assert_eq!(
            model.symbol_table.get(number_wrapper.unwrap()).kind,
            SymbolKind::Class
        );
    }

    #[test]
    fn tsx_component_analysis() {
        let code = read_fixture("typescript/tsx-component.tsx");
        let model = build_semantic_model(&code, "component.tsx");

        let global = model.scope_tree.root().unwrap();

        let button_props = model
            .symbol_table
            .lookup("ButtonProps", global, &model.scope_tree);
        let todo_item = model
            .symbol_table
            .lookup("TodoItem", global, &model.scope_tree);
        let button = model
            .symbol_table
            .lookup("Button", global, &model.scope_tree);

        assert!(
            button_props.is_some(),
            "Interface ButtonProps should be registered"
        );
        assert!(
            todo_item.is_some(),
            "Interface TodoItem should be registered"
        );
        assert!(button.is_some(), "Component Button should be registered");

        assert_eq!(
            model.symbol_table.get(button_props.unwrap()).kind,
            SymbolKind::TypeAlias
        );
        assert_eq!(
            model.symbol_table.get(button.unwrap()).kind,
            SymbolKind::Constant
        );
    }

    #[test]
    fn const_enum() {
        let code = r#"
const enum Direction {
    Up,
    Down,
    Left,
    Right
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();
        let direction = model
            .symbol_table
            .lookup("Direction", global, &model.scope_tree);

        assert!(
            direction.is_some(),
            "Const enum Direction should be registered"
        );
        assert_eq!(
            model.symbol_table.get(direction.unwrap()).kind,
            SymbolKind::Enum
        );
    }

    #[test]
    fn multiple_types_in_same_file() {
        let code = r#"
interface User {
    id: number;
    name: string;
}

type UserRole = 'admin' | 'user';

enum Status {
    Active,
    Inactive
}

class UserService {
    private users: User[] = [];
}
"#;
        let model = build_semantic_model(code, "test.ts");

        let global = model.scope_tree.root().unwrap();

        assert!(
            model
                .symbol_table
                .lookup("User", global, &model.scope_tree)
                .is_some(),
            "Interface User should be registered"
        );
        assert!(
            model
                .symbol_table
                .lookup("UserRole", global, &model.scope_tree)
                .is_some(),
            "Type alias UserRole should be registered"
        );
        assert!(
            model
                .symbol_table
                .lookup("Status", global, &model.scope_tree)
                .is_some(),
            "Enum Status should be registered"
        );
        assert!(
            model
                .symbol_table
                .lookup("UserService", global, &model.scope_tree)
                .is_some(),
            "Class UserService should be registered"
        );
    }
}
