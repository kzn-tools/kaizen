//! Taint analysis for tracking data flow
//!
//! Used for detecting security vulnerabilities by tracking untrusted data.

pub mod dfg;

pub use dfg::{DataFlowGraph, DfgNode, DfgNodeId, DfgNodeKind};

#[derive(Debug)]
pub struct TaintAnalyzer;
