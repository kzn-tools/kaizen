//! Visitor context providing file information during AST traversal.

use swc_common::Span;

use crate::parser::ParsedFile;

pub struct VisitorContext<'a> {
    file: &'a ParsedFile,
}

impl<'a> VisitorContext<'a> {
    pub fn new(file: &'a ParsedFile) -> Self {
        Self { file }
    }

    pub fn file(&self) -> &ParsedFile {
        self.file
    }

    pub fn span_to_location(&self, span: Span) -> (usize, usize) {
        let source = self.file.source();
        let lo = span.lo.0 as usize;

        if source.is_empty() || lo == 0 {
            return (1, 1);
        }

        let prefix = &source[..lo.min(source.len())];
        let line = prefix.matches('\n').count() + 1;
        let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = lo - last_newline + 1;

        (line, column)
    }

    pub fn get_source_text(&self, span: Span) -> Option<&str> {
        let source = self.file.source();
        let lo = span.lo.0 as usize;
        let hi = span.hi.0 as usize;

        if lo <= hi && hi <= source.len() {
            Some(&source[lo..hi])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_provides_file_reference() {
        let parsed = ParsedFile::from_source("test.js", "const x = 1;");
        let ctx = VisitorContext::new(&parsed);

        assert_eq!(ctx.file().metadata().filename, "test.js");
    }

    #[test]
    fn span_to_location_returns_line_and_column() {
        let code = "const x = 1;\nconst y = 2;";
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        let (line, col) =
            ctx.span_to_location(Span::new(swc_common::BytePos(0), swc_common::BytePos(5)));

        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn span_to_location_second_line() {
        let code = "const x = 1;\nconst y = 2;";
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        let (line, _col) =
            ctx.span_to_location(Span::new(swc_common::BytePos(13), swc_common::BytePos(18)));

        assert_eq!(line, 2);
    }

    #[test]
    fn get_source_text_returns_span_content() {
        let code = "const x = 1;";
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        let text = ctx.get_source_text(Span::new(swc_common::BytePos(6), swc_common::BytePos(7)));

        assert_eq!(text, Some("x"));
    }
}
