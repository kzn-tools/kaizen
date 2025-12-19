//! Code action generation from diagnostics with fixes

use kaizen_core::diagnostic::{Diagnostic as CoreDiagnostic, Fix, FixKind};
use kaizen_core::rules::Severity;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic as LspDiagnostic,
    DiagnosticSeverity, NumberOrString, Position, Range, TextEdit, Url, WorkspaceEdit,
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

        let lsp_diagnostic = core_to_lsp_diagnostic(diag);

        for (index, fix) in diag.fixes.iter().enumerate() {
            let is_preferred = index == 0;
            if let Some(action) = create_code_action(uri, &lsp_diagnostic, fix, is_preferred) {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }
    }

    actions
}

fn convert_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
        Severity::Hint => DiagnosticSeverity::HINT,
    }
}

fn core_to_lsp_diagnostic(diag: &CoreDiagnostic) -> LspDiagnostic {
    LspDiagnostic {
        range: Range {
            start: Position {
                line: diag.line.saturating_sub(1) as u32,
                character: diag.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: diag.end_line.saturating_sub(1) as u32,
                character: diag.end_column.saturating_sub(1) as u32,
            },
        },
        severity: Some(convert_severity(diag.severity)),
        code: Some(NumberOrString::String(diag.rule_id.clone())),
        code_description: None,
        source: Some("kaizen".to_string()),
        message: diag.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn diagnostic_in_range(diag: &CoreDiagnostic, range: &Range) -> bool {
    let diag_start_line = diag.line.saturating_sub(1) as u32;
    let diag_end_line = diag.end_line.saturating_sub(1) as u32;

    range.start.line <= diag_end_line && range.end.line >= diag_start_line
}

fn create_code_action(
    uri: &Url,
    lsp_diag: &LspDiagnostic,
    fix: &Fix,
    is_preferred: bool,
) -> Option<CodeAction> {
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
        diagnostics: Some(vec![lsp_diag.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(is_preferred),
        disabled: None,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn code_action_includes_linked_diagnostic() {
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
            assert!(action.diagnostics.is_some());
            let linked_diags = action.diagnostics.as_ref().unwrap();
            assert_eq!(linked_diags.len(), 1);
            assert_eq!(
                linked_diags[0].code,
                Some(NumberOrString::String("Q021".to_string()))
            );
            assert_eq!(linked_diags[0].source, Some("kaizen".to_string()));
        }
    }

    #[test]
    fn first_fix_is_marked_as_preferred() {
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
        if let CodeActionOrCommand::CodeAction(first) = &actions[0] {
            assert_eq!(first.is_preferred, Some(true));
        }
        if let CodeActionOrCommand::CodeAction(second) = &actions[1] {
            assert_eq!(second.is_preferred, Some(false));
        }
    }

    #[test]
    fn convert_severity_maps_correctly() {
        assert_eq!(convert_severity(Severity::Error), DiagnosticSeverity::ERROR);
        assert_eq!(
            convert_severity(Severity::Warning),
            DiagnosticSeverity::WARNING
        );
        assert_eq!(
            convert_severity(Severity::Info),
            DiagnosticSeverity::INFORMATION
        );
        assert_eq!(convert_severity(Severity::Hint), DiagnosticSeverity::HINT);
    }
}
