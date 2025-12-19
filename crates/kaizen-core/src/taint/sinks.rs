//! Taint sinks registry for tracking dangerous operations
//!
//! This module provides a registry for identifying taint sinks - places where
//! untrusted data can cause security vulnerabilities, such as SQL injection,
//! command injection, XSS, and file system attacks.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintSinkKind {
    BuiltIn,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintSinkCategory {
    SqlInjection,
    CommandInjection,
    CodeExecution,
    XssSink,
    FileSystem,
    PathTraversal,
    NetworkRequest,
}

impl TaintSinkCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaintSinkCategory::SqlInjection => "sql_injection",
            TaintSinkCategory::CommandInjection => "command_injection",
            TaintSinkCategory::CodeExecution => "code_execution",
            TaintSinkCategory::XssSink => "xss_sink",
            TaintSinkCategory::FileSystem => "file_system",
            TaintSinkCategory::PathTraversal => "path_traversal",
            TaintSinkCategory::NetworkRequest => "network_request",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintSinkPattern {
    pub callee_path: Vec<String>,
    pub method: Option<String>,
    pub category: TaintSinkCategory,
    pub kind: TaintSinkKind,
    pub description: String,
    pub arg_positions: Vec<usize>,
}

impl TaintSinkPattern {
    pub fn new(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: TaintSinkCategory,
        kind: TaintSinkKind,
        description: &str,
        arg_positions: Vec<usize>,
    ) -> Self {
        Self {
            callee_path: callee_path.into_iter().map(|s| s.to_string()).collect(),
            method: method.map(|s| s.to_string()),
            category,
            kind,
            description: description.to_string(),
            arg_positions,
        }
    }

    pub fn builtin(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: TaintSinkCategory,
        description: &str,
        arg_positions: Vec<usize>,
    ) -> Self {
        Self::new(
            callee_path,
            method,
            category,
            TaintSinkKind::BuiltIn,
            description,
            arg_positions,
        )
    }

    pub fn custom(
        callee_path: Vec<&str>,
        method: Option<&str>,
        category: TaintSinkCategory,
        description: &str,
        arg_positions: Vec<usize>,
    ) -> Self {
        Self::new(
            callee_path,
            method,
            category,
            TaintSinkKind::Custom,
            description,
            arg_positions,
        )
    }

    pub fn matches(&self, callee_chain: &[String], method: Option<&str>) -> bool {
        if callee_chain.len() != self.callee_path.len() {
            return false;
        }

        for (actual, expected) in callee_chain.iter().zip(self.callee_path.iter()) {
            if actual != expected {
                return false;
            }
        }

        match (&self.method, method) {
            (None, _) => true,
            (Some(expected), Some(actual)) => expected == actual,
            (Some(_), None) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintSinkMatch {
    pub pattern: TaintSinkPattern,
    pub matched_callee: Vec<String>,
    pub matched_method: Option<String>,
}

#[derive(Debug)]
pub struct TaintSinksRegistry {
    patterns: Vec<TaintSinkPattern>,
    callee_index: HashMap<String, Vec<usize>>,
}

impl Default for TaintSinksRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintSinksRegistry {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            callee_index: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_code_execution_sinks();
        registry.register_command_injection_sinks();
        registry.register_sql_injection_sinks();
        registry.register_xss_sinks();
        registry.register_file_system_sinks();
        registry.register_network_sinks();
        registry
    }

    fn register_code_execution_sinks(&mut self) {
        self.register_pattern(TaintSinkPattern::builtin(
            vec!["eval"],
            None,
            TaintSinkCategory::CodeExecution,
            "Dynamic code evaluation",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["Function"],
            None,
            TaintSinkCategory::CodeExecution,
            "Dynamic function construction",
            vec![0],
        ));

        let vm_methods = ["runInContext", "runInNewContext", "runInThisContext"];
        for method in vm_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["vm"],
                Some(method),
                TaintSinkCategory::CodeExecution,
                "VM code execution",
                vec![0],
            ));
        }

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["setTimeout"],
            None,
            TaintSinkCategory::CodeExecution,
            "Delayed code execution",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["setInterval"],
            None,
            TaintSinkCategory::CodeExecution,
            "Repeated code execution",
            vec![0],
        ));
    }

    fn register_command_injection_sinks(&mut self) {
        let exec_methods = [
            "exec",
            "execSync",
            "spawn",
            "spawnSync",
            "execFile",
            "execFileSync",
        ];
        for method in exec_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["child_process"],
                Some(method),
                TaintSinkCategory::CommandInjection,
                "Shell command execution",
                vec![0],
            ));
        }

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["exec"],
            None,
            TaintSinkCategory::CommandInjection,
            "Shell command execution",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["execSync"],
            None,
            TaintSinkCategory::CommandInjection,
            "Synchronous shell command execution",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["spawn"],
            None,
            TaintSinkCategory::CommandInjection,
            "Process spawn",
            vec![0],
        ));
    }

    fn register_sql_injection_sinks(&mut self) {
        let db_objects = ["db", "database", "connection", "conn", "pool", "client"];
        for obj in db_objects {
            self.register_pattern(TaintSinkPattern::builtin(
                vec![obj],
                Some("query"),
                TaintSinkCategory::SqlInjection,
                "Database query execution",
                vec![0],
            ));

            self.register_pattern(TaintSinkPattern::builtin(
                vec![obj],
                Some("execute"),
                TaintSinkCategory::SqlInjection,
                "Database query execution",
                vec![0],
            ));
        }

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["sequelize"],
            Some("query"),
            TaintSinkCategory::SqlInjection,
            "Sequelize raw query",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["knex"],
            Some("raw"),
            TaintSinkCategory::SqlInjection,
            "Knex raw query",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["prisma"],
            Some("$queryRaw"),
            TaintSinkCategory::SqlInjection,
            "Prisma raw query",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["prisma"],
            Some("$executeRaw"),
            TaintSinkCategory::SqlInjection,
            "Prisma raw execute",
            vec![0],
        ));
    }

    fn register_xss_sinks(&mut self) {
        self.register_pattern(TaintSinkPattern::builtin(
            vec!["document"],
            Some("write"),
            TaintSinkCategory::XssSink,
            "Document write",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["document"],
            Some("writeln"),
            TaintSinkCategory::XssSink,
            "Document writeln",
            vec![0],
        ));

        let element_sinks = ["innerHTML", "outerHTML"];
        for sink in element_sinks {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["element"],
                Some(sink),
                TaintSinkCategory::XssSink,
                "DOM element HTML injection",
                vec![],
            ));
        }

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["element"],
            Some("insertAdjacentHTML"),
            TaintSinkCategory::XssSink,
            "Adjacent HTML insertion",
            vec![1],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["$"],
            Some("html"),
            TaintSinkCategory::XssSink,
            "jQuery HTML injection",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["jQuery"],
            Some("html"),
            TaintSinkCategory::XssSink,
            "jQuery HTML injection",
            vec![0],
        ));
    }

    fn register_file_system_sinks(&mut self) {
        let read_methods = ["readFile", "readFileSync", "createReadStream"];
        for method in read_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["fs"],
                Some(method),
                TaintSinkCategory::FileSystem,
                "File read operation",
                vec![0],
            ));
        }

        let write_methods = [
            "writeFile",
            "writeFileSync",
            "appendFile",
            "appendFileSync",
            "createWriteStream",
        ];
        for method in write_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["fs"],
                Some(method),
                TaintSinkCategory::FileSystem,
                "File write operation",
                vec![0],
            ));
        }

        let delete_methods = ["unlink", "unlinkSync", "rmdir", "rmdirSync", "rm", "rmSync"];
        for method in delete_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["fs"],
                Some(method),
                TaintSinkCategory::FileSystem,
                "File delete operation",
                vec![0],
            ));
        }

        let dir_methods = ["mkdir", "mkdirSync"];
        for method in dir_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["fs"],
                Some(method),
                TaintSinkCategory::FileSystem,
                "Directory creation",
                vec![0],
            ));
        }

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["fs"],
            Some("rename"),
            TaintSinkCategory::FileSystem,
            "File rename operation",
            vec![0, 1],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["fs"],
            Some("renameSync"),
            TaintSinkCategory::FileSystem,
            "Synchronous file rename",
            vec![0, 1],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["fs"],
            Some("copyFile"),
            TaintSinkCategory::FileSystem,
            "File copy operation",
            vec![0, 1],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["fs"],
            Some("copyFileSync"),
            TaintSinkCategory::FileSystem,
            "Synchronous file copy",
            vec![0, 1],
        ));
    }

    fn register_network_sinks(&mut self) {
        self.register_pattern(TaintSinkPattern::builtin(
            vec!["fetch"],
            None,
            TaintSinkCategory::NetworkRequest,
            "Fetch API request",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["axios"],
            Some("get"),
            TaintSinkCategory::NetworkRequest,
            "Axios GET request",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["axios"],
            Some("post"),
            TaintSinkCategory::NetworkRequest,
            "Axios POST request",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["axios"],
            Some("request"),
            TaintSinkCategory::NetworkRequest,
            "Axios request",
            vec![0],
        ));

        self.register_pattern(TaintSinkPattern::builtin(
            vec!["XMLHttpRequest"],
            Some("open"),
            TaintSinkCategory::NetworkRequest,
            "XHR open",
            vec![1],
        ));

        let http_methods = ["get", "post", "request"];
        for method in http_methods {
            self.register_pattern(TaintSinkPattern::builtin(
                vec!["http"],
                Some(method),
                TaintSinkCategory::NetworkRequest,
                "HTTP request",
                vec![0],
            ));

            self.register_pattern(TaintSinkPattern::builtin(
                vec!["https"],
                Some(method),
                TaintSinkCategory::NetworkRequest,
                "HTTPS request",
                vec![0],
            ));
        }
    }

    pub fn register_pattern(&mut self, pattern: TaintSinkPattern) {
        let index = self.patterns.len();

        if let Some(first_callee) = pattern.callee_path.first() {
            self.callee_index
                .entry(first_callee.clone())
                .or_default()
                .push(index);
        }

        self.patterns.push(pattern);
    }

    pub fn is_taint_sink(
        &self,
        callee_chain: &[String],
        method: Option<&str>,
    ) -> Option<TaintSinkMatch> {
        if callee_chain.is_empty() {
            return None;
        }

        let first = &callee_chain[0];
        if let Some(indices) = self.callee_index.get(first) {
            for &idx in indices {
                let pattern = &self.patterns[idx];
                if pattern.matches(callee_chain, method) {
                    return Some(TaintSinkMatch {
                        pattern: pattern.clone(),
                        matched_callee: callee_chain.to_vec(),
                        matched_method: method.map(|s| s.to_string()),
                    });
                }
            }
        }

        None
    }

    pub fn patterns(&self) -> &[TaintSinkPattern] {
        &self.patterns
    }

    pub fn patterns_for_category(&self, category: TaintSinkCategory) -> Vec<&TaintSinkPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> TaintSinksRegistry {
        TaintSinksRegistry::with_defaults()
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = TaintSinksRegistry::new();
        assert!(registry.patterns().is_empty());
    }

    #[test]
    fn with_defaults_has_patterns() {
        let registry = registry();
        assert!(!registry.patterns().is_empty());
    }

    #[test]
    fn eval_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["eval".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn function_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["Function".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn vm_run_in_context_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["vm".into()], Some("runInContext"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn vm_run_in_new_context_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["vm".into()], Some("runInNewContext"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn set_timeout_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["setTimeout".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CodeExecution);
    }

    #[test]
    fn child_process_exec_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["child_process".into()], Some("exec"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn child_process_spawn_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["child_process".into()], Some("spawn"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn exec_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["exec".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn spawn_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["spawn".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::CommandInjection);
    }

    #[test]
    fn db_query_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["db".into()], Some("query"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn connection_query_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["connection".into()], Some("query"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn pool_query_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["pool".into()], Some("query"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn sequelize_query_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["sequelize".into()], Some("query"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn knex_raw_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["knex".into()], Some("raw"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn prisma_query_raw_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["prisma".into()], Some("$queryRaw"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn document_write_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["document".into()], Some("write"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn document_writeln_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["document".into()], Some("writeln"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn element_inner_html_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["element".into()], Some("innerHTML"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn element_outer_html_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["element".into()], Some("outerHTML"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn element_insert_adjacent_html_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["element".into()], Some("insertAdjacentHTML"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn jquery_html_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["$".into()], Some("html"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::XssSink);
    }

    #[test]
    fn fs_read_file_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("readFile"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_read_file_sync_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("readFileSync"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_write_file_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("writeFile"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_write_file_sync_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("writeFileSync"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_unlink_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("unlink"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_mkdir_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("mkdir"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fs_rename_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fs".into()], Some("rename"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::FileSystem);
    }

    #[test]
    fn fetch_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["fetch".into()], None);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::NetworkRequest);
    }

    #[test]
    fn axios_get_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["axios".into()], Some("get"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::NetworkRequest);
    }

    #[test]
    fn axios_post_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["axios".into()], Some("post"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::NetworkRequest);
    }

    #[test]
    fn xhr_open_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["XMLHttpRequest".into()], Some("open"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::NetworkRequest);
    }

    #[test]
    fn http_request_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["http".into()], Some("request"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::NetworkRequest);
    }

    #[test]
    fn random_property_is_not_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["foo".into(), "bar".into()], Some("baz"));
        assert!(result.is_none());
    }

    #[test]
    fn custom_pattern_registration() {
        let mut registry = TaintSinksRegistry::new();
        registry.register_pattern(TaintSinkPattern::custom(
            vec!["myDb"],
            Some("exec"),
            TaintSinkCategory::SqlInjection,
            "Custom database execution",
            vec![0],
        ));

        let result = registry.is_taint_sink(&["myDb".into()], Some("exec"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.kind, TaintSinkKind::Custom);
    }

    #[test]
    fn patterns_for_category() {
        let registry = registry();
        let sql_patterns = registry.patterns_for_category(TaintSinkCategory::SqlInjection);
        assert!(!sql_patterns.is_empty());

        for pattern in sql_patterns {
            assert_eq!(pattern.category, TaintSinkCategory::SqlInjection);
        }
    }

    #[test]
    fn taint_sink_category_as_str() {
        assert_eq!(TaintSinkCategory::SqlInjection.as_str(), "sql_injection");
        assert_eq!(
            TaintSinkCategory::CommandInjection.as_str(),
            "command_injection"
        );
        assert_eq!(TaintSinkCategory::CodeExecution.as_str(), "code_execution");
        assert_eq!(TaintSinkCategory::XssSink.as_str(), "xss_sink");
        assert_eq!(TaintSinkCategory::FileSystem.as_str(), "file_system");
        assert_eq!(TaintSinkCategory::PathTraversal.as_str(), "path_traversal");
        assert_eq!(
            TaintSinkCategory::NetworkRequest.as_str(),
            "network_request"
        );
    }

    #[test]
    fn pattern_matches_exact_method() {
        let pattern = TaintSinkPattern::builtin(
            vec!["obj"],
            Some("method"),
            TaintSinkCategory::CodeExecution,
            "test",
            vec![0],
        );

        assert!(pattern.matches(&["obj".to_string()], Some("method")));
        assert!(!pattern.matches(&["obj".to_string()], Some("other")));
        assert!(!pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn pattern_matches_any_method() {
        let pattern = TaintSinkPattern::builtin(
            vec!["obj"],
            None,
            TaintSinkCategory::CodeExecution,
            "test",
            vec![0],
        );

        assert!(pattern.matches(&["obj".to_string()], Some("anything")));
        assert!(pattern.matches(&["obj".to_string()], Some("other")));
        assert!(pattern.matches(&["obj".to_string()], None));
    }

    #[test]
    fn pattern_requires_matching_callee() {
        let pattern = TaintSinkPattern::builtin(
            vec!["obj", "nested"],
            None,
            TaintSinkCategory::CodeExecution,
            "test",
            vec![0],
        );

        assert!(pattern.matches(&["obj".to_string(), "nested".to_string()], Some("method")));
        assert!(!pattern.matches(&["obj".to_string()], Some("method")));
        assert!(!pattern.matches(&["obj".to_string(), "other".to_string()], Some("method")));
        assert!(!pattern.matches(&["other".to_string(), "nested".to_string()], Some("method")));
    }

    #[test]
    fn empty_chain_returns_none() {
        let registry = registry();
        let result = registry.is_taint_sink(&[], Some("method"));
        assert!(result.is_none());
    }

    #[test]
    fn db_execute_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["db".into()], Some("execute"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }

    #[test]
    fn client_query_is_taint_sink() {
        let registry = registry();
        let result = registry.is_taint_sink(&["client".into()], Some("query"));
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.pattern.category, TaintSinkCategory::SqlInjection);
    }
}
