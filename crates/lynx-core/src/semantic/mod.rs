//! Semantic analysis module
//!
//! Provides scope analysis, symbol tables, and control flow graph construction.

pub mod cfg;
pub mod scope;
pub mod symbols;
pub mod types;
pub mod visitor;

pub use cfg::{BasicBlock, BasicBlockId, BasicBlockKind, ControlFlowGraph};
pub use scope::{AncestorIter, Scope, ScopeId, ScopeKind, ScopeTree};
pub use symbols::{
    DeclarationKind, Symbol, SymbolId, SymbolKind, SymbolTable, UnresolvedReference,
};
pub use types::{DisposableSource, DisposableTypeInfo, DisposableTypesRegistry};
pub use visitor::{ScopeBuilder, SemanticModel};
