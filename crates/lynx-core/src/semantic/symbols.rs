//! Symbol table for tracking declarations and references
//!
//! This module provides a symbol table that stores all declarations
//! with their scope and supports lookup with scope chain traversal.

use std::collections::HashMap;

use id_arena::{Arena, Id};
use swc_common::Span;

use super::scope::{ScopeId, ScopeTree};

pub type SymbolId = Id<Symbol>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Class,
    Parameter,
    Import,
    TypeAlias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Var,
    Let,
    Const,
    Function,
    Class,
    Parameter,
    Import,
}

#[derive(Debug)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub declaration_kind: DeclarationKind,
    pub scope: ScopeId,
    pub span: Span,
    pub is_exported: bool,
    pub references: Vec<Span>,
}

#[derive(Debug, Clone)]
pub struct UnresolvedReference {
    pub name: String,
    pub span: Span,
    pub scope: ScopeId,
}

pub struct SymbolTable {
    arena: Arena<Symbol>,
    by_scope: HashMap<ScopeId, HashMap<String, SymbolId>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            by_scope: HashMap::new(),
        }
    }

    pub fn declare(
        &mut self,
        name: &str,
        kind: SymbolKind,
        declaration_kind: DeclarationKind,
        scope: ScopeId,
        span: Span,
        is_exported: bool,
    ) -> SymbolId {
        let id = self.arena.alloc_with_id(|id| Symbol {
            id,
            name: name.to_string(),
            kind,
            declaration_kind,
            scope,
            span,
            is_exported,
            references: Vec::new(),
        });

        self.by_scope
            .entry(scope)
            .or_default()
            .insert(name.to_string(), id);

        id
    }

    pub fn lookup(&self, name: &str, scope: ScopeId, scope_tree: &ScopeTree) -> Option<SymbolId> {
        if let Some(scope_symbols) = self.by_scope.get(&scope) {
            if let Some(&id) = scope_symbols.get(name) {
                return Some(id);
            }
        }

        if let Some(parent) = scope_tree.get(scope).parent {
            return self.lookup(name, parent, scope_tree);
        }

        None
    }

    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.arena[id]
    }

    pub fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.arena[id]
    }

    pub fn add_reference(&mut self, symbol_id: SymbolId, reference_span: Span) {
        self.arena[symbol_id].references.push(reference_span);
    }

    pub fn symbols_in_scope(&self, scope: ScopeId) -> impl Iterator<Item = &Symbol> {
        self.by_scope
            .get(&scope)
            .into_iter()
            .flat_map(|symbols| symbols.values().map(|&id| &self.arena[id]))
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.arena.iter().map(|(_, s)| s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::scope::{ScopeKind, ScopeTree};
    use swc_common::DUMMY_SP;

    fn dummy_span() -> Span {
        DUMMY_SP
    }

    #[test]
    fn register_symbol() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let symbol_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );

        let symbol = symbols.get(symbol_id);
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.kind, SymbolKind::Constant);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Const);
        assert_eq!(symbol.scope, global);
        assert!(!symbol.is_exported);
        assert!(symbol.references.is_empty());
    }

    #[test]
    fn lookup_in_current_scope() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let declared_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );

        let found_id = symbols.lookup("x", global, &scope_tree);
        assert_eq!(found_id, Some(declared_id));
    }

    #[test]
    fn lookup_in_parent_scope() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = scope_tree.create_scope(ScopeKind::Function, Some(global), dummy_span());
        let block = scope_tree.create_scope(ScopeKind::Block, Some(func), dummy_span());

        let mut symbols = SymbolTable::new();
        let declared_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );

        let found_id = symbols.lookup("x", block, &scope_tree);
        assert_eq!(found_id, Some(declared_id));

        let found_from_func = symbols.lookup("x", func, &scope_tree);
        assert_eq!(found_from_func, Some(declared_id));
    }

    #[test]
    fn shadowing_returns_local() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());
        let block = scope_tree.create_scope(ScopeKind::Block, Some(global), dummy_span());

        let mut symbols = SymbolTable::new();
        let outer_x = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );
        let inner_x = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            block,
            dummy_span(),
            false,
        );

        let found_in_block = symbols.lookup("x", block, &scope_tree);
        assert_eq!(found_in_block, Some(inner_x));

        let found_in_global = symbols.lookup("x", global, &scope_tree);
        assert_eq!(found_in_global, Some(outer_x));
    }

    #[test]
    fn lookup_nonexistent_returns_none() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let symbols = SymbolTable::new();
        let found = symbols.lookup("undeclared", global, &scope_tree);
        assert!(found.is_none());
    }

    #[test]
    fn add_reference_tracks_usage() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let symbol_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );

        let ref_span1 = swc_common::Span::new(swc_common::BytePos(10), swc_common::BytePos(11));
        let ref_span2 = swc_common::Span::new(swc_common::BytePos(20), swc_common::BytePos(21));

        symbols.add_reference(symbol_id, ref_span1);
        symbols.add_reference(symbol_id, ref_span2);

        let symbol = symbols.get(symbol_id);
        assert_eq!(symbol.references.len(), 2);
        assert_eq!(symbol.references[0], ref_span1);
        assert_eq!(symbol.references[1], ref_span2);
    }

    #[test]
    fn multiple_symbols_in_same_scope() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let x_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );
        let y_id = symbols.declare(
            "y",
            SymbolKind::Variable,
            DeclarationKind::Let,
            global,
            dummy_span(),
            false,
        );
        let z_id = symbols.declare(
            "z",
            SymbolKind::Variable,
            DeclarationKind::Var,
            global,
            dummy_span(),
            false,
        );

        assert_eq!(symbols.lookup("x", global, &scope_tree), Some(x_id));
        assert_eq!(symbols.lookup("y", global, &scope_tree), Some(y_id));
        assert_eq!(symbols.lookup("z", global, &scope_tree), Some(z_id));
    }

    #[test]
    fn function_declaration_symbol() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let func_id = symbols.declare(
            "foo",
            SymbolKind::Function,
            DeclarationKind::Function,
            global,
            dummy_span(),
            false,
        );

        let symbol = symbols.get(func_id);
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert_eq!(symbol.declaration_kind, DeclarationKind::Function);
    }

    #[test]
    fn exported_symbol() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();
        let symbol_id = symbols.declare(
            "exportedFn",
            SymbolKind::Function,
            DeclarationKind::Function,
            global,
            dummy_span(),
            true,
        );

        let symbol = symbols.get(symbol_id);
        assert!(symbol.is_exported);
    }

    #[test]
    fn symbols_in_scope_iteration() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = scope_tree.create_scope(ScopeKind::Function, Some(global), dummy_span());

        let mut symbols = SymbolTable::new();
        symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );
        symbols.declare(
            "y",
            SymbolKind::Variable,
            DeclarationKind::Let,
            global,
            dummy_span(),
            false,
        );
        symbols.declare(
            "a",
            SymbolKind::Parameter,
            DeclarationKind::Parameter,
            func,
            dummy_span(),
            false,
        );

        let global_symbols: Vec<&str> = symbols
            .symbols_in_scope(global)
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(global_symbols.len(), 2);
        assert!(global_symbols.contains(&"x"));
        assert!(global_symbols.contains(&"y"));

        let func_symbols: Vec<&str> = symbols
            .symbols_in_scope(func)
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(func_symbols.len(), 1);
        assert!(func_symbols.contains(&"a"));
    }

    #[test]
    fn deep_scope_lookup() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());
        let level1 = scope_tree.create_scope(ScopeKind::Function, Some(global), dummy_span());
        let level2 = scope_tree.create_scope(ScopeKind::Block, Some(level1), dummy_span());
        let level3 = scope_tree.create_scope(ScopeKind::Block, Some(level2), dummy_span());
        let level4 = scope_tree.create_scope(ScopeKind::Block, Some(level3), dummy_span());

        let mut symbols = SymbolTable::new();
        let x_id = symbols.declare(
            "x",
            SymbolKind::Constant,
            DeclarationKind::Const,
            global,
            dummy_span(),
            false,
        );

        let found = symbols.lookup("x", level4, &scope_tree);
        assert_eq!(found, Some(x_id));
    }

    #[test]
    fn all_symbol_kinds() {
        let mut scope_tree = ScopeTree::new();
        let global = scope_tree.create_scope(ScopeKind::Global, None, dummy_span());

        let mut symbols = SymbolTable::new();

        let kinds = [
            (SymbolKind::Variable, DeclarationKind::Let),
            (SymbolKind::Constant, DeclarationKind::Const),
            (SymbolKind::Function, DeclarationKind::Function),
            (SymbolKind::Class, DeclarationKind::Class),
            (SymbolKind::Parameter, DeclarationKind::Parameter),
            (SymbolKind::Import, DeclarationKind::Import),
            (SymbolKind::TypeAlias, DeclarationKind::Const),
        ];

        for (i, (kind, decl_kind)) in kinds.iter().enumerate() {
            let id = symbols.declare(
                &format!("sym{}", i),
                *kind,
                *decl_kind,
                global,
                dummy_span(),
                false,
            );
            assert_eq!(symbols.get(id).kind, *kind);
        }
    }
}
