//! Code action generation from diagnostics with fixes

use lynx_core::diagnostic::{Diagnostic as CoreDiagnostic, Fix, FixKind};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, Url, WorkspaceEdit,
};

pub fn generate_code_actions(
    uri: &Url,
    diagnostics: &[CoreDiagnostic],
    range: &Range,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    for diag in diagnostics {
        if !diagnostic_in_range(diag, range) {
            continue;
        }

        for fix in &diag.fixes {
            if let Some(action) = create_code_action(uri, diag, fix) {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }
    }

    actions
}

fn diagnostic_in_range(diag: &CoreDiagnostic, range: &Range) -> bool {
    let diag_start_line = diag.line.saturating_sub(1) as u32;
    let diag_end_line = diag.end_line.saturating_sub(1) as u32;

    range.start.line <= diag_end_line && range.end.line >= diag_start_line
}

fn create_code_action(uri: &Url, _diag: &CoreDiagnostic, fix: &Fix) -> Option<CodeAction> {
    let text_edit = match &fix.kind {
        FixKind::ReplaceWith { new_text } => TextEdit {
            range: Range {
                start: Position {
                    line: fix.line.saturating_sub(1) as u32,
                    character: fix.column.saturating_sub(1) as u32,
                },
                end: Position {
                    line: fix.end_line.saturating_sub(1) as u32,
                    character: fix.end_column.saturating_sub(1) as u32,
                },
            },
            new_text: new_text.clone(),
        },
        FixKind::InsertBefore { text } => TextEdit {
            range: Range {
                start: Position {
                    line: fix.line.saturating_sub(1) as u32,
                    character: fix.column.saturating_sub(1) as u32,
                },
                end: Position {
                    line: fix.line.saturating_sub(1) as u32,
                    character: fix.column.saturating_sub(1) as u32,
                },
            },
            new_text: text.clone(),
        },
    };

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![text_edit]);

    Some(CodeAction {
        title: fix.title.clone(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lynx_core::rules::Severity;

    fn make_diagnostic_with_fix(
        rule_id: &str,
        line: usize,
        column: usize,
        fix: Fix,
    ) -> CoreDiagnostic {
        CoreDiagnostic::new(
            rule_id,
            Severity::Warning,
            "Test message",
            "test.js",
            line,
            column,
        )
        .with_fix(fix)
    }

    #[test]
    fn generates_code_action_for_replace_fix() {
        let fix = Fix::replace("Replace const with await using", "await using ", 1, 1, 1, 7);
        let diag = make_diagnostic_with_fix("Q020", 1, 1, fix);
        let uri = Url::parse("file:///test.js").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        let actions = generate_code_actions(&uri, &[diag], &range);

        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.title, "Replace const with await using");
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        }
    }

    #[test]
    fn generates_code_action_for_insert_fix() {
        let fix = Fix::insert_before("Add await", "await ", 1, 1);
        let diag = make_diagnostic_with_fix("Q021", 1, 1, fix);
        let uri = Url::parse("file:///test.js").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        let actions = generate_code_actions(&uri, &[diag], &range);

        assert_eq!(actions.len(), 1);
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert_eq!(action.title, "Add await");
        }
    }

    #[test]
    fn generates_multiple_actions_for_multiple_fixes() {
        let diag = CoreDiagnostic::new(
            "Q021",
            Severity::Warning,
            "Floating promise",
            "test.js",
            1,
            1,
        )
        .with_fix(Fix::insert_before("Add await", "await ", 1, 1))
        .with_fix(Fix::insert_before("Add void", "void ", 1, 1));

        let uri = Url::parse("file:///test.js").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };

        let actions = generate_code_actions(&uri, &[diag], &range);

        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn filters_diagnostics_outside_range() {
        let fix = Fix::insert_before("Add await", "await ", 10, 1);
        let diag = make_diagnostic_with_fix("Q021", 10, 1, fix);
        let uri = Url::parse("file:///test.js").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 1,
                character: 0,
            },
        };

        let actions = generate_code_actions(&uri, &[diag], &range);

        assert!(actions.is_empty());
    }
}
