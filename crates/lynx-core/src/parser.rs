//! Parser module for JavaScript/TypeScript source code
//!
//! Integrates with SWC for parsing source files into AST.

use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap, Spanned};
use swc_ecma_parser::{EsSyntax, StringInput, Syntax, TsSyntax, lexer::Lexer};

pub use swc_ecma_ast::{Module, Script};

#[derive(Debug, thiserror::Error)]
#[error("{message} at {line}:{column}")]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
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
                message: e.kind().msg().to_string(),
            }
        })
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
}
