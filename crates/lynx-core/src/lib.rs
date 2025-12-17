//! Lynx Core - Static analysis engine for JavaScript/TypeScript
//!
//! This crate provides the core analysis functionality including:
//! - Parser integration with SWC
//! - Semantic analysis (scope, control flow)
//! - Rule system (quality, security)
//! - Taint analysis for security vulnerabilities
//! - Diagnostic reporting

pub mod diagnostic;
pub mod parser;
pub mod rules;
pub mod semantic;
pub mod taint;
pub mod visitor;

#[cfg(test)]
mod tests {
    #[test]
    fn parser_module_accessible() {
        let _ = crate::parser::Parser::new();
    }

    #[test]
    fn rules_quality_module_accessible() {
        let _ = crate::rules::quality::QualityRule;
    }

    #[test]
    fn semantic_module_accessible() {
        let _ = crate::semantic::scope::Scope;
        let _ = crate::semantic::cfg::ControlFlowGraph;
    }

    #[test]
    fn taint_module_accessible() {
        let _ = crate::taint::TaintAnalyzer;
    }

    #[test]
    fn diagnostic_module_accessible() {
        let _ = crate::diagnostic::Diagnostic;
    }

    #[test]
    fn swc_parser_is_available() {
        use swc_ecma_parser::{Parser, StringInput, Syntax};

        let src = "const x = 1;";
        let input = StringInput::new(src, Default::default(), Default::default());
        let syntax = Syntax::Es(Default::default());
        let _ = Parser::new(syntax, input, None);
    }
}
