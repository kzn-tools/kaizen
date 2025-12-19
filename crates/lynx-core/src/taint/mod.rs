//! Taint analysis for tracking data flow
//!
//! Used for detecting security vulnerabilities by tracking untrusted data.

pub mod dfg;
pub mod sinks;
pub mod sources;

pub use dfg::{DataFlowGraph, DfgNode, DfgNodeId, DfgNodeKind};
pub use sinks::{
    TaintSinkCategory, TaintSinkKind, TaintSinkMatch, TaintSinkPattern, TaintSinksRegistry,
};
pub use sources::{
    PropertyMatcher, TaintCategory, TaintSourceKind, TaintSourceMatch, TaintSourcePattern,
    TaintSourcesRegistry,
};

#[derive(Debug)]
pub struct TaintAnalyzer;
