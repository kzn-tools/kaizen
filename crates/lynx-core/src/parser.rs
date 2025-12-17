//! Parser module for JavaScript/TypeScript source code
//!
//! Integrates with SWC for parsing source files into AST.

use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap, Spanned};
use swc_ecma_parser::{
    EsSyntax, StringInput, Syntax, TsSyntax, lexer::Lexer, parse_file_as_module,
};

pub use swc_ecma_ast::{EsVersion, Module, Script};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
}

pub fn detect_language(filename: &str) -> Language {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "ts" | "mts" | "cts" => Language::TypeScript,
        "tsx" => Language::Tsx,
        "jsx" => Language::Jsx,
        _ => Language::JavaScript,
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message} at {line}:{column}")]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub span_lo: u32,
    pub span_hi: u32,
    pub message: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub module: Option<Module>,
    pub errors: Vec<ParseError>,
}

impl ParseResult {
    pub fn is_ok(&self) -> bool {
        self.module.is_some()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParserBuilder {
    jsx: bool,
    typescript: bool,
    decorators: bool,
}

impl ParserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jsx(mut self, enabled: bool) -> Self {
        self.jsx = enabled;
        self
    }

    pub fn typescript(mut self, enabled: bool) -> Self {
        self.typescript = enabled;
        self
    }

    pub fn decorators(mut self, enabled: bool) -> Self {
        self.decorators = enabled;
        self
    }

    pub fn build(self) -> Parser {
        let syntax = if self.typescript {
            Syntax::Typescript(TsSyntax {
                tsx: self.jsx,
                decorators: self.decorators,
                ..Default::default()
            })
        } else {
            Syntax::Es(EsSyntax {
                jsx: self.jsx,
                decorators: self.decorators,
                ..Default::default()
            })
        };

        Parser { syntax }
    }
}

#[derive(Debug, Clone)]
pub struct Parser {
    syntax: Syntax,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            syntax: Syntax::Es(Default::default()),
        }
    }

    pub fn for_file(filename: &str) -> Self {
        let language = detect_language(filename);
        match language {
            Language::JavaScript => Self::new(),
            Language::TypeScript => Self::builder().typescript(true).build(),
            Language::Jsx => Self::builder().jsx(true).build(),
            Language::Tsx => Self::builder().typescript(true).jsx(true).build(),
        }
    }

    pub fn builder() -> ParserBuilder {
        ParserBuilder::new()
    }

    pub fn parse_script(&self, code: &str) -> Result<Script, ParseError> {
        let source_map: Lrc<SourceMap> = Default::default();
        let fm = source_map
            .new_source_file(FileName::Custom("input.js".into()).into(), code.to_string());

        let lexer = Lexer::new(
            self.syntax,
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = swc_ecma_parser::Parser::new_from(lexer);

        parser.parse_script().map_err(|e| {
            let span = e.span();
            let loc = source_map.lookup_char_pos(span.lo);
            ParseError {
                line: loc.line,
                column: loc.col_display,
                span_lo: span.lo.0,
                span_hi: span.hi.0,
                message: e.kind().msg().to_string(),
            }
        })
    }

    pub fn parse_module(&self, code: &str) -> Result<Module, ParseError> {
        let source_map: Lrc<SourceMap> = Default::default();
        let fm = source_map
            .new_source_file(FileName::Custom("input.js".into()).into(), code.to_string());

        let lexer = Lexer::new(
            self.syntax,
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = swc_ecma_parser::Parser::new_from(lexer);

        parser.parse_module().map_err(|e| {
            let span = e.span();
            let loc = source_map.lookup_char_pos(span.lo);
            ParseError {
                line: loc.line,
                column: loc.col_display,
                span_lo: span.lo.0,
                span_hi: span.hi.0,
                message: e.kind().msg().to_string(),
            }
        })
    }

    pub fn parse_module_recovering(&self, code: &str) -> ParseResult {
        let source_map: Lrc<SourceMap> = Default::default();
        let fm = source_map
            .new_source_file(FileName::Custom("input.js".into()).into(), code.to_string());

        let mut recovered_errors = Vec::new();

        let result = parse_file_as_module(
            &fm,
            self.syntax,
            EsVersion::latest(),
            None,
            &mut recovered_errors,
        );

        let errors: Vec<ParseError> = recovered_errors
            .into_iter()
            .map(|e| {
                let span = e.span();
                let loc = source_map.lookup_char_pos(span.lo);
                ParseError {
                    line: loc.line,
                    column: loc.col_display,
                    span_lo: span.lo.0,
                    span_hi: span.hi.0,
                    message: e.kind().msg().to_string(),
                }
            })
            .collect();

        match result {
            Ok(module) => ParseResult {
                module: Some(module),
                errors,
            },
            Err(e) => {
                let span = e.span();
                let loc = source_map.lookup_char_pos(span.lo);
                let fatal_error = ParseError {
                    line: loc.line,
                    column: loc.col_display,
                    span_lo: span.lo.0,
                    span_hi: span.hi.0,
                    message: e.kind().msg().to_string(),
                };
                let mut all_errors = errors;
                all_errors.push(fatal_error);
                ParseResult {
                    module: None,
                    errors: all_errors,
                }
            }
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_variable_declaration() {
        let parser = Parser::new();
        let code = "const x = 1;";

        let result = parser.parse_script(code);

        assert!(result.is_ok());
        let script = result.unwrap();
        assert_eq!(script.body.len(), 1);
    }

    #[test]
    fn parse_function_declaration() {
        let parser = Parser::new();
        let code = "function foo() { return 42; }";

        let result = parser.parse_script(code);

        assert!(result.is_ok());
        let script = result.unwrap();
        assert_eq!(script.body.len(), 1);
    }

    #[test]
    fn parse_invalid_syntax_returns_error() {
        let parser = Parser::new();
        let code = "const = ;";

        let result = parser.parse_script(code);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.line, 1);
        assert!(error.column > 0);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn parse_module_with_imports() {
        let parser = Parser::new();
        let code = "import x from 'y';";

        let result = parser.parse_module(code);

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.body.len(), 1);
    }

    #[test]
    fn builder_creates_parser_with_jsx() {
        let parser = Parser::builder().jsx(true).build();
        let code = "const element = <div>Hello</div>;";

        let result = parser.parse_script(code);

        assert!(result.is_ok());
    }

    #[test]
    fn builder_creates_typescript_parser() {
        let parser = Parser::builder().typescript(true).build();
        let code = "const x: number = 1;";

        let result = parser.parse_module(code);

        assert!(result.is_ok());
    }

    #[test]
    fn builder_creates_tsx_parser() {
        let parser = Parser::builder().typescript(true).jsx(true).build();
        let code = "const element: JSX.Element = <div>Hello</div>;";

        let result = parser.parse_module(code);

        assert!(result.is_ok());
    }

    #[test]
    fn parse_typescript_type_annotations() {
        let parser = Parser::for_file("example.ts");
        let code = "const x: number = 1;";

        let result = parser.parse_module(code);

        assert!(result.is_ok());
    }

    #[test]
    fn parse_tsx_jsx_element() {
        let parser = Parser::for_file("component.tsx");
        let code = "const App = () => <div />;";

        let result = parser.parse_module(code);

        assert!(result.is_ok());
    }

    #[test]
    fn detect_language_from_extension() {
        assert_eq!(detect_language("file.js"), Language::JavaScript);
        assert_eq!(detect_language("file.mjs"), Language::JavaScript);
        assert_eq!(detect_language("file.cjs"), Language::JavaScript);
        assert_eq!(detect_language("file.jsx"), Language::Jsx);
        assert_eq!(detect_language("file.ts"), Language::TypeScript);
        assert_eq!(detect_language("file.mts"), Language::TypeScript);
        assert_eq!(detect_language("file.cts"), Language::TypeScript);
        assert_eq!(detect_language("file.tsx"), Language::Tsx);
        assert_eq!(detect_language("unknown"), Language::JavaScript);
    }

    #[test]
    fn parse_typescript_interface() {
        let parser = Parser::for_file("types.ts");
        let code = r#"
            interface User {
                id: number;
                name: string;
                email?: string;
            }
        "#;

        let result = parser.parse_module(code);

        assert!(result.is_ok());
    }

    #[test]
    fn for_file_creates_correct_parser_for_js() {
        let parser = Parser::for_file("script.js");
        let code = "const x = 1;";

        let result = parser.parse_script(code);

        assert!(result.is_ok());
    }

    #[test]
    fn for_file_creates_correct_parser_for_jsx() {
        let parser = Parser::for_file("component.jsx");
        let code = "const element = <div>Hello</div>;";

        let result = parser.parse_script(code);

        assert!(result.is_ok());
    }

    #[test]
    fn parse_recovers_from_missing_semicolon() {
        let parser = Parser::new();
        let code = r#"
const a = 1
const b = 2
function foo() { return a + b }
"#;

        let result = parser.parse_module_recovering(code);

        assert!(result.is_ok());
        assert!(result.module.is_some());
        let module = result.module.unwrap();
        assert_eq!(module.body.len(), 3);
    }

    #[test]
    fn parse_recovers_from_unclosed_brace() {
        let parser = Parser::new();
        let code = r#"
function foo() {
    const x = 1;
// missing closing brace

const y = 2;
"#;

        let result = parser.parse_module_recovering(code);

        assert!(result.has_errors());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn parse_incomplete_code() {
        let parser = Parser::new();
        let code = "const x =";

        let result = parser.parse_module_recovering(code);

        assert!(result.has_errors());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn errors_have_correct_positions() {
        let parser = Parser::new();
        let code = "const = ;";

        let result = parser.parse_module_recovering(code);

        assert!(result.has_errors());
        let error = &result.errors[0];
        assert_eq!(error.line, 1);
        assert!(error.column > 0);
        assert!(error.span_lo > 0);
        assert!(error.span_hi >= error.span_lo);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn parse_recovering_returns_partial_ast_with_errors() {
        let parser = Parser::new();
        let code = r#"
const valid = 1;
const = invalid;
const alsoValid = 2;
"#;

        let result = parser.parse_module_recovering(code);

        assert!(result.has_errors());
    }

    #[test]
    fn parse_recovering_valid_code_has_no_errors() {
        let parser = Parser::new();
        let code = r#"
const x = 1;
const y = 2;
function add(a, b) { return a + b; }
"#;

        let result = parser.parse_module_recovering(code);

        assert!(result.is_ok());
        assert!(!result.has_errors());
        assert!(result.module.is_some());
    }

    #[test]
    fn parse_recovering_typescript_with_errors() {
        let parser = Parser::builder().typescript(true).build();
        let code = r#"
const x: number = 1;
const y: = 2;
interface User { name: string; }
"#;

        let result = parser.parse_module_recovering(code);

        assert!(result.has_errors());
    }
}
