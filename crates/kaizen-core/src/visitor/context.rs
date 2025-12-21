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
        // swc's BytePos is 1-indexed, so subtract 1 to get actual byte index
        let lo = span.lo.0.saturating_sub(1) as usize;
        self.byte_pos_to_location(lo)
    }

    pub fn span_to_range(&self, span: Span) -> (usize, usize, usize, usize) {
        // swc's BytePos is 1-indexed, so subtract 1 to get actual byte index
        let lo = span.lo.0.saturating_sub(1) as usize;
        let hi = span.hi.0.saturating_sub(1) as usize;
        let (start_line, start_col) = self.byte_pos_to_location(lo);
        let (end_line, end_col) = self.byte_pos_to_location(hi);
        (start_line, start_col, end_line, end_col)
    }

    fn byte_pos_to_location(&self, pos: usize) -> (usize, usize) {
        let source = self.file.source();

        if source.is_empty() || pos == 0 {
            return (1, 1);
        }

        let prefix = &source[..pos.min(source.len())];
        let line = prefix.matches('\n').count() + 1;
        let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = pos - last_newline + 1;

        (line, column)
    }

    pub fn get_source_text(&self, span: Span) -> Option<&str> {
        let source = self.file.source();
        // swc's BytePos is 1-indexed, so subtract 1 to get actual byte index
        let lo = span.lo.0.saturating_sub(1) as usize;
        let hi = span.hi.0.saturating_sub(1) as usize;

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

        // BytePos is 1-indexed: BytePos(1) = byte index 0
        let (line, col) =
            ctx.span_to_location(Span::new(swc_common::BytePos(1), swc_common::BytePos(6)));

        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn span_to_location_second_line() {
        let code = "const x = 1;\nconst y = 2;";
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        // BytePos is 1-indexed: BytePos(14) = byte index 13 = start of line 2
        let (line, _col) =
            ctx.span_to_location(Span::new(swc_common::BytePos(14), swc_common::BytePos(19)));

        assert_eq!(line, 2);
    }

    #[test]
    fn get_source_text_returns_span_content() {
        let code = "const x = 1;";
        let parsed = ParsedFile::from_source("test.js", code);
        let ctx = VisitorContext::new(&parsed);

        // BytePos is 1-indexed: BytePos(7) = byte index 6 = 'x'
        let text = ctx.get_source_text(Span::new(swc_common::BytePos(7), swc_common::BytePos(8)));

        assert_eq!(text, Some("x"));
    }
}
