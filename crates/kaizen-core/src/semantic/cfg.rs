//! Control Flow Graph construction and analysis
//!
//! This module provides a CFG data structure for representing execution paths
//! through JavaScript/TypeScript code. It supports:
//! - Linear code: sequential nodes
//! - Branching: if/else with merge points
//! - Loops: for/while/do-while with back edges

use id_arena::{Arena, Id};
use swc_common::{Span, Spanned};
use swc_ecma_ast::{DoWhileStmt, ForInStmt, ForOfStmt, ForStmt, IfStmt, Module, Stmt, WhileStmt};

pub type BasicBlockId = Id<BasicBlock>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicBlockKind {
    Entry,
    Exit,
    Normal,
    Condition,
    LoopHeader,
}

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub kind: BasicBlockKind,
    pub predecessors: Vec<BasicBlockId>,
    pub successors: Vec<BasicBlockId>,
    pub span: Option<Span>,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    blocks: Arena<BasicBlock>,
    entry: Option<BasicBlockId>,
    exit: Option<BasicBlockId>,
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self {
            blocks: Arena::new(),
            entry: None,
            exit: None,
        }
    }

    pub fn build(module: &Module) -> Self {
        let mut builder = CfgBuilder::new();
        builder.build_module(module);
        builder.graph
    }

    fn create_block(&mut self, kind: BasicBlockKind, span: Option<Span>) -> BasicBlockId {
        self.blocks.alloc_with_id(|id| BasicBlock {
            id,
            kind,
            predecessors: Vec::new(),
            successors: Vec::new(),
            span,
        })
    }

    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        if !self.blocks[from].successors.contains(&to) {
            self.blocks[from].successors.push(to);
        }
        if !self.blocks[to].predecessors.contains(&from) {
            self.blocks[to].predecessors.push(from);
        }
    }

    pub fn entry(&self) -> Option<BasicBlockId> {
        self.entry
    }

    pub fn exit(&self) -> Option<BasicBlockId> {
        self.exit
    }

    pub fn get(&self, id: BasicBlockId) -> &BasicBlock {
        &self.blocks[id]
    }

    pub fn blocks(&self) -> impl Iterator<Item = &BasicBlock> {
        self.blocks.iter().map(|(_, block)| block)
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn successors(&self, id: BasicBlockId) -> impl Iterator<Item = &BasicBlock> {
        self.blocks[id]
            .successors
            .iter()
            .map(|&succ| &self.blocks[succ])
    }

    pub fn predecessors(&self, id: BasicBlockId) -> impl Iterator<Item = &BasicBlock> {
        self.blocks[id]
            .predecessors
            .iter()
            .map(|&pred| &self.blocks[pred])
    }

    pub fn has_back_edge(&self, from: BasicBlockId, to: BasicBlockId) -> bool {
        self.blocks[to].kind == BasicBlockKind::LoopHeader
            && self.blocks[from].successors.contains(&to)
    }
}

struct CfgBuilder {
    graph: ControlFlowGraph,
}

impl CfgBuilder {
    fn new() -> Self {
        Self {
            graph: ControlFlowGraph::new(),
        }
    }

    fn build_module(&mut self, module: &Module) {
        let entry = self.graph.create_block(BasicBlockKind::Entry, None);
        let exit = self.graph.create_block(BasicBlockKind::Exit, None);
        self.graph.entry = Some(entry);
        self.graph.exit = Some(exit);

        let mut current = entry;

        for item in &module.body {
            if let swc_ecma_ast::ModuleItem::Stmt(stmt) = item {
                current = self.build_stmt(stmt, current, exit);
            }
        }

        self.graph.add_edge(current, exit);
    }

    fn build_stmt(
        &mut self,
        stmt: &Stmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        match stmt {
            Stmt::Block(block) => {
                let mut curr = current;
                for s in &block.stmts {
                    curr = self.build_stmt(s, curr, exit);
                }
                curr
            }
            Stmt::If(if_stmt) => self.build_if_stmt(if_stmt, current, exit),
            Stmt::For(for_stmt) => self.build_for_stmt(for_stmt, current, exit),
            Stmt::ForIn(for_in) => self.build_for_in_stmt(for_in, current, exit),
            Stmt::ForOf(for_of) => self.build_for_of_stmt(for_of, current, exit),
            Stmt::While(while_stmt) => self.build_while_stmt(while_stmt, current, exit),
            Stmt::DoWhile(do_while) => self.build_do_while_stmt(do_while, current, exit),
            Stmt::Return(_) => {
                self.graph.add_edge(current, exit);
                current
            }
            Stmt::Break(_) | Stmt::Continue(_) => current,
            Stmt::Switch(switch_stmt) => self.build_switch_stmt(switch_stmt, current, exit),
            Stmt::Try(try_stmt) => self.build_try_stmt(try_stmt, current, exit),
            Stmt::Throw(_) => {
                self.graph.add_edge(current, exit);
                current
            }
            _ => {
                let block = self
                    .graph
                    .create_block(BasicBlockKind::Normal, Some(stmt.span()));
                self.graph.add_edge(current, block);
                block
            }
        }
    }

    fn build_if_stmt(
        &mut self,
        if_stmt: &IfStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let condition = self
            .graph
            .create_block(BasicBlockKind::Condition, Some(if_stmt.test.span()));
        self.graph.add_edge(current, condition);

        let then_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(if_stmt.cons.span()));
        self.graph.add_edge(condition, then_start);
        let then_end = self.build_stmt(&if_stmt.cons, then_start, exit);

        let merge = self.graph.create_block(BasicBlockKind::Normal, None);

        if let Some(alt) = &if_stmt.alt {
            let else_start = self
                .graph
                .create_block(BasicBlockKind::Normal, Some(alt.span()));
            self.graph.add_edge(condition, else_start);
            let else_end = self.build_stmt(alt, else_start, exit);
            self.graph.add_edge(else_end, merge);
        } else {
            self.graph.add_edge(condition, merge);
        }

        self.graph.add_edge(then_end, merge);
        merge
    }

    fn build_for_stmt(
        &mut self,
        for_stmt: &ForStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let init = if for_stmt.init.is_some() {
            let init_block = self.graph.create_block(BasicBlockKind::Normal, None);
            self.graph.add_edge(current, init_block);
            init_block
        } else {
            current
        };

        let header = self
            .graph
            .create_block(BasicBlockKind::LoopHeader, Some(for_stmt.span));
        self.graph.add_edge(init, header);

        let condition = if for_stmt.test.is_some() {
            let cond = self.graph.create_block(
                BasicBlockKind::Condition,
                for_stmt.test.as_ref().map(|t| t.span()),
            );
            self.graph.add_edge(header, cond);
            cond
        } else {
            header
        };

        let body_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(for_stmt.body.span()));
        self.graph.add_edge(condition, body_start);

        let body_end = self.build_stmt(&for_stmt.body, body_start, exit);

        let update = if for_stmt.update.is_some() {
            let update_block = self.graph.create_block(BasicBlockKind::Normal, None);
            self.graph.add_edge(body_end, update_block);
            update_block
        } else {
            body_end
        };

        self.graph.add_edge(update, header);

        let after_loop = self.graph.create_block(BasicBlockKind::Normal, None);
        self.graph.add_edge(condition, after_loop);

        after_loop
    }

    fn build_for_in_stmt(
        &mut self,
        for_in: &ForInStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let header = self
            .graph
            .create_block(BasicBlockKind::LoopHeader, Some(for_in.span));
        self.graph.add_edge(current, header);

        let condition = self
            .graph
            .create_block(BasicBlockKind::Condition, Some(for_in.right.span()));
        self.graph.add_edge(header, condition);

        let body_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(for_in.body.span()));
        self.graph.add_edge(condition, body_start);

        let body_end = self.build_stmt(&for_in.body, body_start, exit);
        self.graph.add_edge(body_end, header);

        let after_loop = self.graph.create_block(BasicBlockKind::Normal, None);
        self.graph.add_edge(condition, after_loop);

        after_loop
    }

    fn build_for_of_stmt(
        &mut self,
        for_of: &ForOfStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let header = self
            .graph
            .create_block(BasicBlockKind::LoopHeader, Some(for_of.span));
        self.graph.add_edge(current, header);

        let condition = self
            .graph
            .create_block(BasicBlockKind::Condition, Some(for_of.right.span()));
        self.graph.add_edge(header, condition);

        let body_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(for_of.body.span()));
        self.graph.add_edge(condition, body_start);

        let body_end = self.build_stmt(&for_of.body, body_start, exit);
        self.graph.add_edge(body_end, header);

        let after_loop = self.graph.create_block(BasicBlockKind::Normal, None);
        self.graph.add_edge(condition, after_loop);

        after_loop
    }

    fn build_while_stmt(
        &mut self,
        while_stmt: &WhileStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let header = self
            .graph
            .create_block(BasicBlockKind::LoopHeader, Some(while_stmt.span));
        self.graph.add_edge(current, header);

        let condition = self
            .graph
            .create_block(BasicBlockKind::Condition, Some(while_stmt.test.span()));
        self.graph.add_edge(header, condition);

        let body_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(while_stmt.body.span()));
        self.graph.add_edge(condition, body_start);

        let body_end = self.build_stmt(&while_stmt.body, body_start, exit);
        self.graph.add_edge(body_end, header);

        let after_loop = self.graph.create_block(BasicBlockKind::Normal, None);
        self.graph.add_edge(condition, after_loop);

        after_loop
    }

    fn build_do_while_stmt(
        &mut self,
        do_while: &DoWhileStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let header = self
            .graph
            .create_block(BasicBlockKind::LoopHeader, Some(do_while.span));
        self.graph.add_edge(current, header);

        let body_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(do_while.body.span()));
        self.graph.add_edge(header, body_start);

        let body_end = self.build_stmt(&do_while.body, body_start, exit);

        let condition = self
            .graph
            .create_block(BasicBlockKind::Condition, Some(do_while.test.span()));
        self.graph.add_edge(body_end, condition);

        self.graph.add_edge(condition, header);

        let after_loop = self.graph.create_block(BasicBlockKind::Normal, None);
        self.graph.add_edge(condition, after_loop);

        after_loop
    }

    fn build_switch_stmt(
        &mut self,
        switch_stmt: &swc_ecma_ast::SwitchStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let discriminant = self.graph.create_block(
            BasicBlockKind::Condition,
            Some(switch_stmt.discriminant.span()),
        );
        self.graph.add_edge(current, discriminant);

        let merge = self.graph.create_block(BasicBlockKind::Normal, None);
        let mut last_case_end: Option<BasicBlockId> = None;

        for case in &switch_stmt.cases {
            let case_start = self
                .graph
                .create_block(BasicBlockKind::Normal, Some(case.span));
            self.graph.add_edge(discriminant, case_start);

            if let Some(prev_end) = last_case_end {
                self.graph.add_edge(prev_end, case_start);
            }

            let mut case_current = case_start;
            for stmt in &case.cons {
                case_current = self.build_stmt(stmt, case_current, exit);
            }

            last_case_end = Some(case_current);
        }

        if let Some(last) = last_case_end {
            self.graph.add_edge(last, merge);
        } else {
            self.graph.add_edge(discriminant, merge);
        }

        merge
    }

    fn build_try_stmt(
        &mut self,
        try_stmt: &swc_ecma_ast::TryStmt,
        current: BasicBlockId,
        exit: BasicBlockId,
    ) -> BasicBlockId {
        let try_start = self
            .graph
            .create_block(BasicBlockKind::Normal, Some(try_stmt.block.span));
        self.graph.add_edge(current, try_start);

        let mut try_current = try_start;
        for stmt in &try_stmt.block.stmts {
            try_current = self.build_stmt(stmt, try_current, exit);
        }

        let merge = self.graph.create_block(BasicBlockKind::Normal, None);

        if let Some(handler) = &try_stmt.handler {
            let catch_start = self
                .graph
                .create_block(BasicBlockKind::Normal, Some(handler.span));
            self.graph.add_edge(try_start, catch_start);

            let mut catch_current = catch_start;
            for stmt in &handler.body.stmts {
                catch_current = self.build_stmt(stmt, catch_current, exit);
            }

            if let Some(finalizer) = &try_stmt.finalizer {
                let finally_start = self
                    .graph
                    .create_block(BasicBlockKind::Normal, Some(finalizer.span));
                self.graph.add_edge(try_current, finally_start);
                self.graph.add_edge(catch_current, finally_start);

                let mut finally_current = finally_start;
                for stmt in &finalizer.stmts {
                    finally_current = self.build_stmt(stmt, finally_current, exit);
                }
                self.graph.add_edge(finally_current, merge);
            } else {
                self.graph.add_edge(try_current, merge);
                self.graph.add_edge(catch_current, merge);
            }
        } else if let Some(finalizer) = &try_stmt.finalizer {
            let finally_start = self
                .graph
                .create_block(BasicBlockKind::Normal, Some(finalizer.span));
            self.graph.add_edge(try_current, finally_start);

            let mut finally_current = finally_start;
            for stmt in &finalizer.stmts {
                finally_current = self.build_stmt(stmt, finally_current, exit);
            }
            self.graph.add_edge(finally_current, merge);
        } else {
            self.graph.add_edge(try_current, merge);
        }

        merge
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedFile;

    fn build_cfg(code: &str) -> ControlFlowGraph {
        let parsed = ParsedFile::from_source("test.js", code);
        let module = parsed.module().expect("parse failed");
        ControlFlowGraph::build(module)
    }

    #[test]
    fn empty_module_has_entry_and_exit() {
        let cfg = build_cfg("");

        assert!(cfg.entry().is_some());
        assert!(cfg.exit().is_some());
        assert_eq!(cfg.block_count(), 2);

        let entry = cfg.get(cfg.entry().unwrap());
        assert_eq!(entry.kind, BasicBlockKind::Entry);

        let exit = cfg.get(cfg.exit().unwrap());
        assert_eq!(exit.kind, BasicBlockKind::Exit);
    }

    #[test]
    fn sequential_statements_create_chain() {
        let cfg = build_cfg("const x = 1; const y = 2; const z = 3;");

        assert!(cfg.block_count() >= 4);

        let entry = cfg.entry().unwrap();
        assert!(!cfg.get(entry).successors.is_empty());
    }

    #[test]
    fn if_statement_creates_branches() {
        let cfg = build_cfg(
            r#"
            if (condition) {
                const x = 1;
            }
        "#,
        );

        let condition_blocks: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        assert!(!condition_blocks.is_empty());

        let cond = condition_blocks[0];
        assert!(cond.successors.len() >= 2);
    }

    #[test]
    fn if_else_creates_two_branches() {
        let cfg = build_cfg(
            r#"
            if (condition) {
                const x = 1;
            } else {
                const y = 2;
            }
        "#,
        );

        let condition_blocks: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        assert!(!condition_blocks.is_empty());

        let cond = condition_blocks[0];
        assert_eq!(cond.successors.len(), 2);
    }

    #[test]
    fn while_loop_creates_back_edge() {
        let cfg = build_cfg(
            r#"
            while (condition) {
                const x = 1;
            }
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!loop_headers.is_empty());

        let header = loop_headers[0];
        assert!(header.predecessors.len() >= 2);
    }

    #[test]
    fn for_loop_creates_back_edge() {
        let cfg = build_cfg(
            r#"
            for (let i = 0; i < 10; i++) {
                console.log(i);
            }
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!loop_headers.is_empty());

        let header = loop_headers[0];
        assert!(header.predecessors.len() >= 2);
    }

    #[test]
    fn do_while_loop_creates_back_edge() {
        let cfg = build_cfg(
            r#"
            do {
                const x = 1;
            } while (condition);
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!loop_headers.is_empty());

        let header = loop_headers[0];
        assert!(header.predecessors.len() >= 2);
    }

    #[test]
    fn for_in_loop_creates_back_edge() {
        let cfg = build_cfg(
            r#"
            for (const key in obj) {
                console.log(key);
            }
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!loop_headers.is_empty());
    }

    #[test]
    fn for_of_loop_creates_back_edge() {
        let cfg = build_cfg(
            r#"
            for (const item of arr) {
                console.log(item);
            }
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!loop_headers.is_empty());
    }

    #[test]
    fn nested_if_creates_multiple_branches() {
        let cfg = build_cfg(
            r#"
            if (a) {
                if (b) {
                    const x = 1;
                }
            }
        "#,
        );

        let condition_blocks: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        assert!(condition_blocks.len() >= 2);
    }

    #[test]
    fn nested_loops_create_multiple_headers() {
        let cfg = build_cfg(
            r#"
            for (let i = 0; i < 10; i++) {
                for (let j = 0; j < 10; j++) {
                    console.log(i, j);
                }
            }
        "#,
        );

        let loop_headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(loop_headers.len() >= 2);
    }

    #[test]
    fn return_statement_connects_to_exit() {
        let cfg = build_cfg(
            r#"
            const x = 1;
            return x;
            const y = 2;
        "#,
        );

        let exit = cfg.exit().unwrap();
        let exit_preds = cfg.get(exit).predecessors.len();
        assert!(exit_preds >= 2);
    }

    #[test]
    fn switch_statement_creates_case_branches() {
        let cfg = build_cfg(
            r#"
            switch (x) {
                case 1:
                    const a = 1;
                    break;
                case 2:
                    const b = 2;
                    break;
                default:
                    const c = 3;
            }
        "#,
        );

        let condition_blocks: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        assert!(!condition_blocks.is_empty());

        let discriminant = condition_blocks[0];
        assert!(discriminant.successors.len() >= 3);
    }

    #[test]
    fn try_catch_creates_exception_paths() {
        let cfg = build_cfg(
            r#"
            try {
                const x = 1;
            } catch (e) {
                const y = 2;
            }
        "#,
        );

        assert!(cfg.block_count() >= 4);
    }

    #[test]
    fn try_finally_creates_paths() {
        let cfg = build_cfg(
            r#"
            try {
                const x = 1;
            } finally {
                const z = 3;
            }
        "#,
        );

        assert!(cfg.block_count() >= 4);
    }

    #[test]
    fn try_catch_finally_creates_all_paths() {
        let cfg = build_cfg(
            r#"
            try {
                const x = 1;
            } catch (e) {
                const y = 2;
            } finally {
                const z = 3;
            }
        "#,
        );

        assert!(cfg.block_count() >= 5);
    }

    #[test]
    fn has_back_edge_detects_loop_edges() {
        let cfg = build_cfg(
            r#"
            while (true) {
                const x = 1;
            }
        "#,
        );

        let headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();
        assert!(!headers.is_empty());

        let header = headers[0];
        let has_back = header
            .predecessors
            .iter()
            .any(|&pred| cfg.has_back_edge(pred, header.id));
        assert!(has_back);
    }

    #[test]
    fn block_statement_processes_contents() {
        let cfg = build_cfg(
            r#"
            {
                const x = 1;
                const y = 2;
            }
        "#,
        );

        assert!(cfg.block_count() >= 3);
    }

    #[test]
    fn loop_inside_if() {
        let cfg = build_cfg(
            r#"
            if (condition) {
                while (true) {
                    const x = 1;
                }
            }
        "#,
        );

        let conditions: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        let headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();

        assert!(!conditions.is_empty());
        assert!(!headers.is_empty());
    }

    #[test]
    fn if_inside_loop() {
        let cfg = build_cfg(
            r#"
            while (condition) {
                if (x) {
                    const a = 1;
                } else {
                    const b = 2;
                }
            }
        "#,
        );

        let conditions: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::Condition)
            .collect();
        let headers: Vec<_> = cfg
            .blocks()
            .filter(|b| b.kind == BasicBlockKind::LoopHeader)
            .collect();

        assert!(conditions.len() >= 2);
        assert!(!headers.is_empty());
    }

    #[test]
    fn throw_statement_connects_to_exit() {
        let cfg = build_cfg(
            r#"
            throw new Error("error");
            const x = 1;
        "#,
        );

        let exit = cfg.exit().unwrap();
        let exit_preds = cfg.get(exit).predecessors.len();
        assert!(exit_preds >= 2);
    }

    #[test]
    fn successors_iterator_works() {
        let cfg = build_cfg("if (x) { const a = 1; }");

        let entry = cfg.entry().unwrap();
        let succs: Vec<_> = cfg.successors(entry).collect();
        assert!(!succs.is_empty());
    }

    #[test]
    fn predecessors_iterator_works() {
        let cfg = build_cfg("const x = 1;");

        let exit = cfg.exit().unwrap();
        let preds: Vec<_> = cfg.predecessors(exit).collect();
        assert!(!preds.is_empty());
    }
}
