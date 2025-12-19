//! JSON output formatter for diagnostic display
//!
//! Provides structured JSON and NDJSON output formats for programmatic integration.

use kaizen_core::diagnostic::{Diagnostic, Fix, FixKind};
use kaizen_core::rules::{RuleCategory, RuleRegistry, Severity};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Serialize)]
pub struct JsonOutput {
    pub version: &'static str,
    pub metadata: JsonMetadata,
    pub summary: JsonSummary,
    pub diagnostics: Vec<JsonDiagnostic>,
}

#[derive(Serialize)]
pub struct JsonMetadata {
    pub kaizen_version: &'static str,
    pub working_directory: String,
    pub analyzed_path: String,
}

#[derive(Serialize)]
pub struct JsonSummary {
    pub total_files: usize,
    pub files_with_issues: usize,
    pub total_diagnostics: usize,
    pub by_severity: SeverityCounts,
    pub by_category: CategoryCounts,
}

#[derive(Serialize)]
pub struct SeverityCounts {
    pub error: usize,
    pub warning: usize,
    pub info: usize,
    pub hint: usize,
}

#[derive(Serialize)]
pub struct CategoryCounts {
    pub quality: usize,
    pub security: usize,
}

#[derive(Serialize)]
pub struct JsonDiagnostic {
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub severity: String,
    pub confidence: String,
    pub message: String,
    pub location: JsonLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<JsonFix>,
}

#[derive(Serialize)]
pub struct JsonLocation {
    pub file: String,
    pub start: JsonPosition,
    pub end: JsonPosition,
}

#[derive(Serialize)]
pub struct JsonPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct JsonFix {
    pub title: String,
    pub kind: String,
    pub start: JsonPosition,
    pub end: JsonPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum NdjsonRecord {
    #[serde(rename = "metadata")]
    Metadata(JsonMetadata),
    #[serde(rename = "diagnostic")]
    Diagnostic(JsonDiagnostic),
    #[serde(rename = "summary")]
    Summary(JsonSummary),
}

pub struct JsonFormatter<'a> {
    registry: Option<&'a RuleRegistry>,
}

impl<'a> JsonFormatter<'a> {
    pub fn new() -> Self {
        Self { registry: None }
    }

    pub fn with_registry(registry: &'a RuleRegistry) -> Self {
        Self {
            registry: Some(registry),
        }
    }

    pub fn format(
        &self,
        diagnostics: &[Diagnostic],
        total_files: usize,
        analyzed_path: &str,
    ) -> String {
        let output = self.build_output(diagnostics, total_files, analyzed_path);
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn format_ndjson<W: Write>(
        &self,
        diagnostics: &[Diagnostic],
        total_files: usize,
        analyzed_path: &str,
        writer: &mut W,
    ) -> io::Result<()> {
        let metadata = self.build_metadata(analyzed_path);
        writeln!(
            writer,
            "{}",
            serde_json::to_string(&NdjsonRecord::Metadata(metadata))?
        )?;

        for diag in diagnostics {
            let json_diag = self.convert_diagnostic(diag);
            writeln!(
                writer,
                "{}",
                serde_json::to_string(&NdjsonRecord::Diagnostic(json_diag))?
            )?;
        }

        let summary = self.build_summary(diagnostics, total_files);
        writeln!(
            writer,
            "{}",
            serde_json::to_string(&NdjsonRecord::Summary(summary))?
        )?;

        Ok(())
    }

    fn build_output(
        &self,
        diagnostics: &[Diagnostic],
        total_files: usize,
        analyzed_path: &str,
    ) -> JsonOutput {
        JsonOutput {
            version: "1.0",
            metadata: self.build_metadata(analyzed_path),
            summary: self.build_summary(diagnostics, total_files),
            diagnostics: diagnostics
                .iter()
                .map(|d| self.convert_diagnostic(d))
                .collect(),
        }
    }

    fn build_metadata(&self, analyzed_path: &str) -> JsonMetadata {
        JsonMetadata {
            kaizen_version: env!("CARGO_PKG_VERSION"),
            working_directory: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            analyzed_path: analyzed_path.to_string(),
        }
    }

    fn build_summary(&self, diagnostics: &[Diagnostic], total_files: usize) -> JsonSummary {
        let mut by_severity = SeverityCounts {
            error: 0,
            warning: 0,
            info: 0,
            hint: 0,
        };
        let mut by_category = CategoryCounts {
            quality: 0,
            security: 0,
        };
        let mut files_with_issues: HashMap<&str, bool> = HashMap::new();

        for diag in diagnostics {
            match diag.severity {
                Severity::Error => by_severity.error += 1,
                Severity::Warning => by_severity.warning += 1,
                Severity::Info => by_severity.info += 1,
                Severity::Hint => by_severity.hint += 1,
            }

            if let Some(category) = self.get_category(&diag.rule_id) {
                match category {
                    RuleCategory::Quality => by_category.quality += 1,
                    RuleCategory::Security => by_category.security += 1,
                }
            }

            files_with_issues.insert(&diag.file, true);
        }

        JsonSummary {
            total_files,
            files_with_issues: files_with_issues.len(),
            total_diagnostics: diagnostics.len(),
            by_severity,
            by_category,
        }
    }

    fn convert_diagnostic(&self, diag: &Diagnostic) -> JsonDiagnostic {
        let (rule_name, category) = self.get_rule_info(&diag.rule_id);

        JsonDiagnostic {
            rule_id: diag.rule_id.clone(),
            rule_name,
            category,
            severity: format!("{:?}", diag.severity).to_lowercase(),
            confidence: format!("{:?}", diag.confidence).to_lowercase(),
            message: diag.message.clone(),
            location: JsonLocation {
                file: diag.file.clone(),
                start: JsonPosition {
                    line: diag.line,
                    column: diag.column,
                },
                end: JsonPosition {
                    line: diag.end_line,
                    column: diag.end_column,
                },
            },
            suggestion: diag.suggestion.clone(),
            fixes: diag.fixes.iter().map(convert_fix).collect(),
        }
    }

    fn get_rule_info(&self, rule_id: &str) -> (Option<String>, Option<String>) {
        if let Some(registry) = self.registry {
            if let Some(rule) = registry.get_rule(rule_id) {
                let metadata = rule.metadata();
                let category = match metadata.category {
                    RuleCategory::Quality => "quality",
                    RuleCategory::Security => "security",
                };
                return (Some(metadata.name.to_string()), Some(category.to_string()));
            }
        }
        (None, None)
    }

    fn get_category(&self, rule_id: &str) -> Option<RuleCategory> {
        self.registry
            .and_then(|r| r.get_rule(rule_id))
            .map(|rule| rule.metadata().category)
    }
}

impl Default for JsonFormatter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn convert_fix(fix: &Fix) -> JsonFix {
    let (kind, new_text, insert_text) = match &fix.kind {
        FixKind::ReplaceWith { new_text } => ("replace", Some(new_text.clone()), None),
        FixKind::InsertBefore { text } => ("insert_before", None, Some(text.clone())),
    };

    JsonFix {
        title: fix.title.clone(),
        kind: kind.to_string(),
        start: JsonPosition {
            line: fix.line,
            column: fix.column,
        },
        end: JsonPosition {
            line: fix.end_line,
            column: fix.end_column,
        },
        new_text,
        insert_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kaizen_core::rules::Confidence;

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic::new(
            "Q030",
            Severity::Warning,
            "Avoid using 'var'",
            "test.js",
            10,
            1,
        )
        .with_end(10, 8)
        .with_confidence(Confidence::High)
        .with_suggestion("Use 'let' or 'const' instead")
    }

    #[test]
    fn format_produces_valid_json() {
        let formatter = JsonFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics, 5, "./src");

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["metadata"].is_object());
        assert!(parsed["summary"].is_object());
        assert!(parsed["diagnostics"].is_array());
    }

    #[test]
    fn format_includes_metadata() {
        let formatter = JsonFormatter::new();
        let diagnostics = vec![];

        let output = formatter.format(&diagnostics, 10, "./test");

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["metadata"]["kaizen_version"].is_string());
        assert_eq!(parsed["metadata"]["analyzed_path"], "./test");
    }

    #[test]
    fn format_includes_summary() {
        let formatter = JsonFormatter::new();
        let diagnostics = vec![
            Diagnostic::new("Q001", Severity::Error, "Error 1", "a.js", 1, 0),
            Diagnostic::new("Q002", Severity::Warning, "Warning 1", "a.js", 2, 0),
            Diagnostic::new("Q003", Severity::Warning, "Warning 2", "b.js", 1, 0),
        ];

        let output = formatter.format(&diagnostics, 10, "./src");

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["summary"]["total_files"], 10);
        assert_eq!(parsed["summary"]["files_with_issues"], 2);
        assert_eq!(parsed["summary"]["total_diagnostics"], 3);
        assert_eq!(parsed["summary"]["by_severity"]["error"], 1);
        assert_eq!(parsed["summary"]["by_severity"]["warning"], 2);
    }

    #[test]
    fn format_includes_diagnostic_details() {
        let formatter = JsonFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics, 1, "./src");

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let diag = &parsed["diagnostics"][0];
        assert_eq!(diag["rule_id"], "Q030");
        assert_eq!(diag["severity"], "warning");
        assert_eq!(diag["confidence"], "high");
        assert_eq!(diag["message"], "Avoid using 'var'");
        assert_eq!(diag["location"]["file"], "test.js");
        assert_eq!(diag["location"]["start"]["line"], 10);
        assert_eq!(diag["location"]["start"]["column"], 1);
        assert_eq!(diag["suggestion"], "Use 'let' or 'const' instead");
    }

    #[test]
    fn ndjson_format_produces_lines() {
        let formatter = JsonFormatter::new();
        let diagnostics = vec![sample_diagnostic()];
        let mut output = Vec::new();

        formatter
            .format_ndjson(&diagnostics, 5, "./src", &mut output)
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 3);

        let metadata: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(metadata["type"], "metadata");

        let diagnostic: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(diagnostic["type"], "diagnostic");

        let summary: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(summary["type"], "summary");
    }

    #[test]
    fn empty_diagnostics_produces_valid_output() {
        let formatter = JsonFormatter::new();
        let diagnostics: Vec<Diagnostic> = vec![];

        let output = formatter.format(&diagnostics, 0, ".");

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["summary"]["total_diagnostics"], 0);
        assert!(parsed["diagnostics"].as_array().unwrap().is_empty());
    }
}
