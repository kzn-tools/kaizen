//! Diagnostic reporting for analysis results
//!
//! Provides structured diagnostic information for issues found during analysis.

use crate::rules::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixKind {
    ReplaceWith { new_text: String },
    InsertBefore { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub title: String,
    pub kind: FixKind,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Fix {
    pub fn replace(
        title: impl Into<String>,
        new_text: impl Into<String>,
        line: usize,
        column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self {
            title: title.into(),
            kind: FixKind::ReplaceWith {
                new_text: new_text.into(),
            },
            line,
            column,
            end_line,
            end_column,
        }
    }

    pub fn insert_before(
        title: impl Into<String>,
        text: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            title: title.into(),
            kind: FixKind::InsertBefore { text: text.into() },
            line,
            column,
            end_line: line,
            end_column: column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub suggestion: Option<String>,
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    pub fn new(
        rule_id: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            message: message.into(),
            file: file.into(),
            line,
            column,
            end_line: line,
            end_column: column,
            suggestion: None,
            fixes: Vec::new(),
        }
    }

    pub fn with_end(mut self, end_line: usize, end_column: usize) -> Self {
        self.end_line = end_line;
        self.end_column = end_column;
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fixes.push(fix);
        self
    }

    pub fn with_fixes(mut self, fixes: Vec<Fix>) -> Self {
        self.fixes = fixes;
        self
    }
}
