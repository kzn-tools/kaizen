//! Scope analysis for variable bindings and references
//!
//! This module provides a scope tree data structure for representing
//! nested program scopes (global, function, block).

use id_arena::{Arena, Id};
use swc_common::Span;

pub type ScopeId = Id<Scope>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    ArrowFunction,
    Block,
    For,
    While,
    Switch,
    Try,
    Catch,
    Class,
}

#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub span: Span,
}

pub struct ScopeTree {
    arena: Arena<Scope>,
    root: Option<ScopeId>,
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeTree {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            root: None,
        }
    }

    pub fn create_scope(
        &mut self,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        span: Span,
    ) -> ScopeId {
        let id = self.arena.alloc_with_id(|id| Scope {
            id,
            kind,
            parent,
            children: Vec::new(),
            span,
        });

        if let Some(parent_id) = parent {
            self.arena[parent_id].children.push(id);
        }

        if self.root.is_none() {
            self.root = Some(id);
        }

        id
    }

    pub fn root(&self) -> Option<ScopeId> {
        self.root
    }

    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.arena[id]
    }

    pub fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.arena[id]
    }

    pub fn parent(&self, id: ScopeId) -> Option<&Scope> {
        self.arena[id].parent.map(|p| &self.arena[p])
    }

    pub fn children(&self, id: ScopeId) -> impl Iterator<Item = &Scope> {
        self.arena[id].children.iter().map(|&c| &self.arena[c])
    }

    pub fn ancestors(&self, id: ScopeId) -> AncestorIter<'_> {
        AncestorIter {
            tree: self,
            current: Some(id),
        }
    }

    pub fn is_descendant_of(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        self.ancestors(scope).any(|s| s.id == ancestor)
    }
}

pub struct AncestorIter<'a> {
    tree: &'a ScopeTree,
    current: Option<ScopeId>,
}

impl<'a> Iterator for AncestorIter<'a> {
    type Item = &'a Scope;

    fn next(&mut self) -> Option<Self::Item> {
        let current_id = self.current?;
        let scope = &self.tree.arena[current_id];
        self.current = scope.parent;
        Some(scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_common::{BytePos, DUMMY_SP};

    fn dummy_span() -> Span {
        DUMMY_SP
    }

    fn span_at(lo: u32, hi: u32) -> Span {
        Span::new(BytePos(lo), BytePos(hi))
    }

    #[test]
    fn creates_global_scope() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());

        assert!(tree.root().is_some());
        assert_eq!(tree.root(), Some(global));

        let scope = tree.get(global);
        assert_eq!(scope.kind, ScopeKind::Global);
        assert!(scope.parent.is_none());
        assert!(scope.children.is_empty());
    }

    #[test]
    fn creates_function_scope() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = tree.create_scope(ScopeKind::Function, Some(global), span_at(10, 50));

        let func_scope = tree.get(func);
        assert_eq!(func_scope.kind, ScopeKind::Function);
        assert_eq!(func_scope.parent, Some(global));

        let global_scope = tree.get(global);
        assert_eq!(global_scope.children.len(), 1);
        assert_eq!(global_scope.children[0], func);
    }

    #[test]
    fn creates_block_scope() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let block = tree.create_scope(ScopeKind::Block, Some(global), span_at(20, 30));

        let block_scope = tree.get(block);
        assert_eq!(block_scope.kind, ScopeKind::Block);
        assert_eq!(block_scope.parent, Some(global));
    }

    #[test]
    fn nested_scopes_have_correct_parent() {
        let mut tree = ScopeTree::new();

        // Create 5-level nesting: Global -> Function -> Block -> Block -> Block
        let global = tree.create_scope(ScopeKind::Global, None, span_at(0, 100));
        let func = tree.create_scope(ScopeKind::Function, Some(global), span_at(10, 90));
        let block1 = tree.create_scope(ScopeKind::Block, Some(func), span_at(20, 80));
        let block2 = tree.create_scope(ScopeKind::Block, Some(block1), span_at(30, 70));
        let block3 = tree.create_scope(ScopeKind::Block, Some(block2), span_at(40, 60));

        // Verify parent chain
        assert_eq!(tree.get(block3).parent, Some(block2));
        assert_eq!(tree.get(block2).parent, Some(block1));
        assert_eq!(tree.get(block1).parent, Some(func));
        assert_eq!(tree.get(func).parent, Some(global));
        assert!(tree.get(global).parent.is_none());

        // Verify children
        assert_eq!(tree.get(global).children, vec![func]);
        assert_eq!(tree.get(func).children, vec![block1]);
        assert_eq!(tree.get(block1).children, vec![block2]);
        assert_eq!(tree.get(block2).children, vec![block3]);
        assert!(tree.get(block3).children.is_empty());
    }

    #[test]
    fn ancestors_iterator_traverses_parent_chain() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = tree.create_scope(ScopeKind::Function, Some(global), dummy_span());
        let block = tree.create_scope(ScopeKind::Block, Some(func), dummy_span());

        let ancestors: Vec<ScopeKind> = tree.ancestors(block).map(|s| s.kind).collect();

        assert_eq!(
            ancestors,
            vec![ScopeKind::Block, ScopeKind::Function, ScopeKind::Global]
        );
    }

    #[test]
    fn parent_returns_parent_scope() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = tree.create_scope(ScopeKind::Function, Some(global), dummy_span());

        let parent = tree.parent(func);
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().kind, ScopeKind::Global);

        let no_parent = tree.parent(global);
        assert!(no_parent.is_none());
    }

    #[test]
    fn children_returns_child_scopes() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func1 = tree.create_scope(ScopeKind::Function, Some(global), dummy_span());
        let func2 = tree.create_scope(ScopeKind::ArrowFunction, Some(global), dummy_span());

        let children: Vec<ScopeKind> = tree.children(global).map(|s| s.kind).collect();

        assert_eq!(children.len(), 2);
        assert!(children.contains(&ScopeKind::Function));
        assert!(children.contains(&ScopeKind::ArrowFunction));

        let _ = func1;
        let _ = func2;
    }

    #[test]
    fn is_descendant_of_checks_ancestry() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());
        let func = tree.create_scope(ScopeKind::Function, Some(global), dummy_span());
        let block = tree.create_scope(ScopeKind::Block, Some(func), dummy_span());

        assert!(tree.is_descendant_of(block, block));
        assert!(tree.is_descendant_of(block, func));
        assert!(tree.is_descendant_of(block, global));
        assert!(tree.is_descendant_of(func, global));
        assert!(!tree.is_descendant_of(global, func));
        assert!(!tree.is_descendant_of(func, block));
    }

    #[test]
    fn multiple_children_at_same_level() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());

        let if_block = tree.create_scope(ScopeKind::Block, Some(global), span_at(10, 20));
        let else_block = tree.create_scope(ScopeKind::Block, Some(global), span_at(25, 35));
        let for_loop = tree.create_scope(ScopeKind::For, Some(global), span_at(40, 60));

        let global_scope = tree.get(global);
        assert_eq!(global_scope.children.len(), 3);
        assert!(global_scope.children.contains(&if_block));
        assert!(global_scope.children.contains(&else_block));
        assert!(global_scope.children.contains(&for_loop));
    }

    #[test]
    fn all_scope_kinds_can_be_created() {
        let mut tree = ScopeTree::new();
        let global = tree.create_scope(ScopeKind::Global, None, dummy_span());

        let kinds = vec![
            ScopeKind::Module,
            ScopeKind::Function,
            ScopeKind::ArrowFunction,
            ScopeKind::Block,
            ScopeKind::For,
            ScopeKind::While,
            ScopeKind::Switch,
            ScopeKind::Try,
            ScopeKind::Catch,
            ScopeKind::Class,
        ];

        for kind in kinds {
            let scope_id = tree.create_scope(kind, Some(global), dummy_span());
            assert_eq!(tree.get(scope_id).kind, kind);
        }
    }
}
