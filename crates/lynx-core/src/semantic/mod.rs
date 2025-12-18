//! Semantic analysis module
//!
//! Provides scope analysis and control flow graph construction.

pub mod cfg;
pub mod scope;

pub use scope::{AncestorIter, Scope, ScopeId, ScopeKind, ScopeTree};
