//! Taint propagation algorithm for tracking untrusted data flow
//!
//! This module implements the core taint propagation that tracks how
//! tainted values flow through assignments, string concatenation,
//! and function returns using the Data Flow Graph.

use std::collections::{HashMap, HashSet, VecDeque};

use swc_common::Span;

use super::{
    DataFlowGraph, DfgNode, DfgNodeId, DfgNodeKind, TaintCategory, TaintSinkCategory,
    TaintSinkMatch, TaintSinksRegistry, TaintSourceMatch, TaintSourcesRegistry,
};

#[derive(Debug, Clone)]
pub struct TaintedNode {
    pub node_id: DfgNodeId,
    pub categories: HashSet<TaintCategory>,
    pub source_spans: Vec<Span>,
}

#[derive(Debug)]
pub struct TaintState {
    tainted: HashMap<DfgNodeId, TaintedNode>,
}

impl Default for TaintState {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintState {
    pub fn new() -> Self {
        Self {
            tainted: HashMap::new(),
        }
    }

    pub fn mark_tainted(&mut self, node_id: DfgNodeId, category: TaintCategory, source_span: Span) {
        self.tainted
            .entry(node_id)
            .and_modify(|t| {
                t.categories.insert(category);
                if !t.source_spans.contains(&source_span) {
                    t.source_spans.push(source_span);
                }
            })
            .or_insert_with(|| {
                let mut categories = HashSet::new();
                categories.insert(category);
                TaintedNode {
                    node_id,
                    categories,
                    source_spans: vec![source_span],
                }
            });
    }

    pub fn merge_taint(&mut self, target: DfgNodeId, source: DfgNodeId) {
        if let Some(source_taint) = self.tainted.get(&source).cloned() {
            self.tainted
                .entry(target)
                .and_modify(|t| {
                    t.categories.extend(source_taint.categories.iter().copied());
                    for span in &source_taint.source_spans {
                        if !t.source_spans.contains(span) {
                            t.source_spans.push(*span);
                        }
                    }
                })
                .or_insert_with(|| TaintedNode {
                    node_id: target,
                    categories: source_taint.categories.clone(),
                    source_spans: source_taint.source_spans.clone(),
                });
        }
    }

    pub fn is_tainted(&self, node_id: DfgNodeId) -> bool {
        self.tainted.contains_key(&node_id)
    }

    pub fn get_taint(&self, node_id: DfgNodeId) -> Option<&TaintedNode> {
        self.tainted.get(&node_id)
    }

    pub fn tainted_nodes(&self) -> impl Iterator<Item = &TaintedNode> {
        self.tainted.values()
    }
}

#[derive(Debug, Clone)]
pub struct TaintFinding {
    pub source_span: Span,
    pub sink_span: Span,
    pub source_category: TaintCategory,
    pub sink_category: TaintSinkCategory,
    pub sink_description: String,
    pub path: Vec<DfgNodeId>,
}

pub struct TaintPropagator<'a> {
    dfg: &'a DataFlowGraph,
    sources_registry: &'a TaintSourcesRegistry,
    sinks_registry: &'a TaintSinksRegistry,
    state: TaintState,
}

impl<'a> TaintPropagator<'a> {
    pub fn new(
        dfg: &'a DataFlowGraph,
        sources_registry: &'a TaintSourcesRegistry,
        sinks_registry: &'a TaintSinksRegistry,
    ) -> Self {
        Self {
            dfg,
            sources_registry,
            sinks_registry,
            state: TaintState::new(),
        }
    }

    pub fn analyze(&mut self) -> Vec<TaintFinding> {
        self.identify_initial_taint();
        self.propagate();
        self.find_vulnerabilities()
    }

    fn identify_initial_taint(&mut self) {
        for node in self.dfg.nodes() {
            if let Some(source_match) = self.is_taint_source_node(node) {
                self.state
                    .mark_tainted(node.id, source_match.pattern.category, node.span);
            }
        }
    }

    fn is_taint_source_node(&self, node: &DfgNode) -> Option<TaintSourceMatch> {
        match &node.kind {
            DfgNodeKind::PropertyAccess { object, property } => {
                let chain = self.build_property_chain(*object, Some(property.clone()));
                if chain.len() >= 2 {
                    let (object_chain, prop) = chain.split_at(chain.len() - 1);
                    self.sources_registry
                        .is_taint_source(object_chain, Some(&prop[0]))
                } else if chain.len() == 1 {
                    self.sources_registry.is_taint_source(&chain, None)
                } else {
                    None
                }
            }
            DfgNodeKind::Parameter { name, .. } => self
                .sources_registry
                .is_tainted_parameter(name)
                .map(|category| TaintSourceMatch {
                    pattern: super::TaintSourcePattern::builtin(
                        vec![name.as_str()],
                        super::PropertyMatcher::None,
                        category,
                        "Tainted parameter",
                    ),
                    matched_path: vec![name.clone()],
                    matched_property: None,
                }),
            DfgNodeKind::Variable { name, .. } => self
                .sources_registry
                .is_tainted_parameter(name)
                .map(|category| TaintSourceMatch {
                    pattern: super::TaintSourcePattern::builtin(
                        vec![name.as_str()],
                        super::PropertyMatcher::None,
                        category,
                        "Tainted variable",
                    ),
                    matched_path: vec![name.clone()],
                    matched_property: None,
                }),
            _ => None,
        }
    }

    fn build_property_chain(&self, node_id: DfgNodeId, property: Option<String>) -> Vec<String> {
        let mut chain = Vec::new();
        self.collect_chain(node_id, &mut chain);
        if let Some(prop) = property {
            chain.push(prop);
        }
        chain
    }

    fn collect_chain(&self, node_id: DfgNodeId, chain: &mut Vec<String>) {
        let node = self.dfg.get(node_id);
        match &node.kind {
            DfgNodeKind::Variable { name, .. } => {
                chain.push(name.clone());
            }
            DfgNodeKind::PropertyAccess { object, property } => {
                self.collect_chain(*object, chain);
                chain.push(property.clone());
            }
            _ => {}
        }
    }

    fn propagate(&mut self) {
        let mut worklist: VecDeque<DfgNodeId> = self.state.tainted.keys().copied().collect();

        let mut visited = HashSet::new();

        while let Some(node_id) = worklist.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);

            let node = self.dfg.get(node_id);
            for &dependent in &node.flows_to {
                if !self.state.is_tainted(dependent) {
                    self.state.merge_taint(dependent, node_id);
                    worklist.push_back(dependent);
                } else {
                    self.state.merge_taint(dependent, node_id);
                }
            }
        }
    }

    fn find_vulnerabilities(&self) -> Vec<TaintFinding> {
        let mut findings = Vec::new();

        for node in self.dfg.nodes() {
            if let DfgNodeKind::Call { callee_name } = &node.kind {
                if let Some(sink_match) = self.check_sink_call(node, callee_name) {
                    for &from_id in &node.flows_from {
                        if let Some(taint) = self.state.get_taint(from_id) {
                            for &category in &taint.categories {
                                for &source_span in &taint.source_spans {
                                    let path = self.build_path(from_id, node.id);
                                    findings.push(TaintFinding {
                                        source_span,
                                        sink_span: node.span,
                                        source_category: category,
                                        sink_category: sink_match.pattern.category,
                                        sink_description: sink_match.pattern.description.clone(),
                                        path,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    fn check_sink_call(&self, node: &DfgNode, callee_name: &str) -> Option<TaintSinkMatch> {
        if let Some(result) = self
            .sinks_registry
            .is_taint_sink(&[callee_name.to_string()], None)
        {
            return Some(result);
        }

        for &from_id in &node.flows_from {
            let from_node = self.dfg.get(from_id);
            if let DfgNodeKind::Variable { name, .. } = &from_node.kind {
                if let Some(result) = self
                    .sinks_registry
                    .is_taint_sink(std::slice::from_ref(name), Some(callee_name))
                {
                    return Some(result);
                }
            } else if let DfgNodeKind::PropertyAccess { property, .. } = &from_node.kind {
                if let Some(result) = self
                    .sinks_registry
                    .is_taint_sink(std::slice::from_ref(property), Some(callee_name))
                {
                    return Some(result);
                }
            }
        }

        None
    }

    fn build_path(&self, source: DfgNodeId, sink: DfgNodeId) -> Vec<DfgNodeId> {
        let mut path = vec![source];
        let mut current = source;
        let mut visited = HashSet::new();
        visited.insert(current);

        while current != sink {
            let node = self.dfg.get(current);
            let next = node.flows_to.iter().find(|&&id| {
                !visited.contains(&id) && self.reaches_target(id, sink, &mut HashSet::new())
            });

            match next {
                Some(&next_id) => {
                    path.push(next_id);
                    visited.insert(next_id);
                    current = next_id;
                }
                None => break,
            }
        }

        path
    }

    fn reaches_target(
        &self,
        from: DfgNodeId,
        target: DfgNodeId,
        visited: &mut HashSet<DfgNodeId>,
    ) -> bool {
        if from == target {
            return true;
        }
        if visited.contains(&from) {
            return false;
        }
        visited.insert(from);

        let node = self.dfg.get(from);
        for &next in &node.flows_to {
            if self.reaches_target(next, target, visited) {
                return true;
            }
        }
        false
    }

    pub fn state(&self) -> &TaintState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedFile;
    use crate::semantic::ScopeBuilder;

    fn analyze_code(code: &str) -> (DataFlowGraph, Vec<TaintFinding>) {
        let parsed = ParsedFile::from_source("test.js", code);
        let module = parsed.module().expect("parse failed");
        let semantic = ScopeBuilder::build(module);
        let dfg = DataFlowGraph::build(module, &semantic);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        let findings = propagator.analyze();
        (dfg, findings)
    }

    #[test]
    fn taint_flows_through_assignment() {
        let code = r#"
            function handler(req, res) {
                const userInput = req.body.username;
                const x = userInput;
                const y = x;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let y_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "y"));

        assert!(y_node.is_some(), "y node should exist");
        let y = y_node.unwrap();
        assert!(
            propagator.state().is_tainted(y.id),
            "y should be tainted through assignment chain"
        );
    }

    #[test]
    fn string_concatenation_propagates_taint() {
        let code = r#"
            function handler(req, res) {
                const table = req.body.table;
                const query = "SELECT * FROM " + table;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let query_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "query"));

        assert!(query_node.is_some(), "query node should exist");
        let query = query_node.unwrap();
        assert!(
            propagator.state().is_tainted(query.id),
            "query should be tainted through concatenation"
        );
    }

    #[test]
    fn template_literal_propagates_taint() {
        let code = r#"
            function handler(req, res) {
                const name = req.body.name;
                const greeting = `Hello ${name}!`;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let greeting_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "greeting"));

        assert!(greeting_node.is_some(), "greeting node should exist");
        let greeting = greeting_node.unwrap();
        assert!(
            propagator.state().is_tainted(greeting.id),
            "greeting should be tainted through template literal"
        );
    }

    #[test]
    fn function_return_propagates_taint() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const result = process(input);
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let result_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "result"));

        assert!(result_node.is_some(), "result node should exist");
        let result = result_node.unwrap();
        assert!(
            propagator.state().is_tainted(result.id),
            "result should be tainted (call depends on tainted input)"
        );
    }

    #[test]
    fn detects_sql_injection_with_exec() {
        let code = r#"
            function handler(req, res) {
                const userId = req.body.userId;
                exec("SELECT * FROM users WHERE id = " + userId);
            }
        "#;

        let (_, findings) = analyze_code(code);

        assert!(
            !findings.is_empty(),
            "should detect command injection via exec"
        );
        let finding = &findings[0];
        assert_eq!(finding.sink_category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn detects_command_injection() {
        let code = r#"
            function handler(req, res) {
                const cmd = req.body.command;
                exec(cmd);
            }
        "#;

        let (_, findings) = analyze_code(code);

        assert!(!findings.is_empty(), "should detect command injection");
        let finding = &findings[0];
        assert_eq!(finding.sink_category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn detects_code_execution() {
        let code = r#"
            function handler(req, res) {
                const code = req.body.code;
                eval(code);
            }
        "#;

        let (_, findings) = analyze_code(code);

        assert!(
            !findings.is_empty(),
            "should detect code execution vulnerability"
        );
        let finding = &findings[0];
        assert_eq!(finding.sink_category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn no_false_positives_for_safe_code() {
        let code = r#"
            function handler(req, res) {
                const safeQuery = "SELECT * FROM users WHERE id = 1";
                db.query(safeQuery);
            }
        "#;

        let (_, findings) = analyze_code(code);

        assert!(
            findings.is_empty(),
            "should not report false positives for safe code"
        );
    }

    #[test]
    fn tainted_parameter_is_identified() {
        let code = r#"
            function handler(req, res) {
                const body = req.body;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let req_param = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Parameter { name, .. } if name == "req"));

        assert!(req_param.is_some(), "req parameter should exist");
        let req = req_param.unwrap();
        assert!(
            propagator.state().is_tainted(req.id),
            "req parameter should be tainted"
        );
    }

    #[test]
    fn process_env_is_tainted() {
        let code = r#"
            const apiKey = process.env.API_KEY;
            eval(apiKey);
        "#;

        let (_, findings) = analyze_code(code);

        assert!(
            !findings.is_empty(),
            "should detect code execution with env var"
        );
        let finding = &findings[0];
        assert_eq!(finding.source_category, TaintCategory::Environment);
        assert_eq!(finding.sink_category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn multiple_taint_sources_merged() {
        let code = r#"
            function handler(req, res) {
                const userInput = req.body.data;
                const queryInput = req.query.id;
                const combined = userInput + queryInput;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let combined_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "combined"));

        assert!(combined_node.is_some(), "combined node should exist");
        let combined = combined_node.unwrap();
        let taint = propagator.state().get_taint(combined.id);
        assert!(
            taint.is_some(),
            "combined should be tainted from multiple sources"
        );

        let user_input_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "userInput"));
        let query_input_node = dfg.nodes().find(
            |n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "queryInput"),
        );

        assert!(user_input_node.is_some(), "userInput node should exist");
        assert!(query_input_node.is_some(), "queryInput node should exist");
        assert!(
            propagator.state().is_tainted(user_input_node.unwrap().id),
            "userInput should be tainted"
        );
        assert!(
            propagator.state().is_tainted(query_input_node.unwrap().id),
            "queryInput should be tainted"
        );
    }

    #[test]
    fn property_access_chain_detected() {
        let code = r#"
            function handler(req, res) {
                const value = req.query.id;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let value_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "value"));

        assert!(value_node.is_some(), "value node should exist");
        let value = value_node.unwrap();
        assert!(
            propagator.state().is_tainted(value.id),
            "value should be tainted from req.query.id"
        );
    }

    #[test]
    fn conditional_expression_propagates_taint() {
        let code = r#"
            function handler(req, res) {
                const input = req.body.data;
                const value = true ? input : "default";
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let value_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "value"));

        assert!(value_node.is_some(), "value node should exist");
        let value = value_node.unwrap();
        assert!(
            propagator.state().is_tainted(value.id),
            "value should be tainted through conditional"
        );
    }

    #[test]
    fn binary_operation_preserves_taint() {
        let code = r#"
            function handler(req, res) {
                const num = req.body.count;
                const doubled = num * 2;
            }
        "#;

        let (dfg, _) = analyze_code(code);
        let sources = TaintSourcesRegistry::with_defaults();
        let sinks = TaintSinksRegistry::with_defaults();
        let mut propagator = TaintPropagator::new(&dfg, &sources, &sinks);
        propagator.analyze();

        let doubled_node = dfg
            .nodes()
            .find(|n| matches!(&n.kind, DfgNodeKind::Variable { name, .. } if name == "doubled"));

        assert!(doubled_node.is_some(), "doubled node should exist");
        let doubled = doubled_node.unwrap();
        assert!(
            propagator.state().is_tainted(doubled.id),
            "doubled should be tainted through binary operation"
        );
    }
}
