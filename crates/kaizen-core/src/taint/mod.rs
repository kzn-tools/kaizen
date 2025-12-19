//! Taint analysis for tracking data flow
//!
//! Used for detecting security vulnerabilities by tracking untrusted data.

pub mod dfg;
pub mod propagation;
pub mod sanitizers;
pub mod sinks;
pub mod sources;

pub use dfg::{DataFlowGraph, DfgNode, DfgNodeId, DfgNodeKind};
pub use propagation::{TaintFinding, TaintPropagator, TaintState, TaintedNode};
pub use sanitizers::{
    SanitizerCategory, SanitizerKind, SanitizerMatch, SanitizerPattern, SanitizersRegistry,
};
pub use sinks::{
    TaintSinkCategory, TaintSinkKind, TaintSinkMatch, TaintSinkPattern, TaintSinksRegistry,
};
pub use sources::{
    PropertyMatcher, TaintCategory, TaintSourceKind, TaintSourceMatch, TaintSourcePattern,
    TaintSourcesRegistry,
};

use crate::parser::ParsedFile;
use crate::semantic::ScopeBuilder;

#[derive(Debug)]
pub struct TaintAnalyzer {
    sources_registry: TaintSourcesRegistry,
    sinks_registry: TaintSinksRegistry,
    sanitizers_registry: SanitizersRegistry,
}

impl Default for TaintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintAnalyzer {
    pub fn new() -> Self {
        Self {
            sources_registry: TaintSourcesRegistry::with_defaults(),
            sinks_registry: TaintSinksRegistry::with_defaults(),
            sanitizers_registry: SanitizersRegistry::with_defaults(),
        }
    }

    pub fn with_registries(
        sources_registry: TaintSourcesRegistry,
        sinks_registry: TaintSinksRegistry,
        sanitizers_registry: SanitizersRegistry,
    ) -> Self {
        Self {
            sources_registry,
            sinks_registry,
            sanitizers_registry,
        }
    }

    pub fn analyze(&self, parsed: &ParsedFile) -> Vec<TaintFinding> {
        let module = match parsed.module() {
            Some(m) => m,
            None => return Vec::new(),
        };

        let semantic = ScopeBuilder::build(module);
        let dfg = DataFlowGraph::build(module, &semantic);
        let mut propagator = TaintPropagator::new(
            &dfg,
            &self.sources_registry,
            &self.sinks_registry,
            &self.sanitizers_registry,
        );
        propagator.analyze()
    }

    pub fn sources_registry(&self) -> &TaintSourcesRegistry {
        &self.sources_registry
    }

    pub fn sinks_registry(&self) -> &TaintSinksRegistry {
        &self.sinks_registry
    }

    pub fn sanitizers_registry(&self) -> &SanitizersRegistry {
        &self.sanitizers_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzer_detects_code_execution() {
        let code = r#"
            function handler(req, res) {
                const code = req.body.code;
                eval(code);
            }
        "#;

        let parsed = ParsedFile::from_source("test.js", code);
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(&parsed);

        assert!(
            !findings.is_empty(),
            "should detect code execution vulnerability"
        );
        assert_eq!(findings[0].sink_category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn analyzer_handles_parse_errors() {
        let code = "this is not valid javascript {{{{";
        let parsed = ParsedFile::from_source("test.js", code);
        let analyzer = TaintAnalyzer::new();
        let findings = analyzer.analyze(&parsed);

        assert!(findings.is_empty(), "should return empty for parse errors");
    }
}
