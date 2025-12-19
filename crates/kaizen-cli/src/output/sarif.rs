//! SARIF output formatter for GitHub Code Scanning
//!
//! Provides SARIF 2.1.0 output format for integration with GitHub Code Scanning
//! and other static analysis tools that support the SARIF standard.

use kaizen_core::diagnostic::{Diagnostic, Fix, FixKind};
use kaizen_core::rules::{RuleCategory, RuleRegistry, Severity};
use serde::Serialize;
use std::collections::HashSet;

const SARIF_VERSION: &str = "2.1.0";
const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifOutput {
    #[serde(rename = "$schema")]
    pub schema: &'static str,
    pub version: &'static str,
    pub runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<SarifArtifact>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    pub name: &'static str,
    pub semantic_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<&'static str>,
    pub rules: Vec<SarifRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub short_description: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    pub default_configuration: SarifRuleConfiguration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<SarifRuleProperties>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRuleConfiguration {
    pub level: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRuleProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "security-severity")]
    pub security_severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
    pub partial_fingerprints: SarifPartialFingerprints,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<SarifFix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<SarifResultProperties>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: usize,
    pub start_column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPartialFingerprints {
    #[serde(rename = "primaryLocationLineHash")]
    pub primary_location_line_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifFix {
    pub description: SarifMessage,
    pub artifact_changes: Vec<SarifArtifactChange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactChange {
    pub artifact_location: SarifArtifactLocation,
    pub replacements: Vec<SarifReplacement>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifReplacement {
    pub deleted_region: SarifRegion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_content: Option<SarifArtifactContent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactContent {
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifact {
    pub location: SarifArtifactLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResultProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

pub struct SarifFormatter<'a> {
    registry: Option<&'a RuleRegistry>,
}

impl<'a> SarifFormatter<'a> {
    pub fn new() -> Self {
        Self { registry: None }
    }

    pub fn with_registry(registry: &'a RuleRegistry) -> Self {
        Self {
            registry: Some(registry),
        }
    }

    pub fn format(&self, diagnostics: &[Diagnostic]) -> String {
        let output = self.build_output(diagnostics);
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    fn build_output(&self, diagnostics: &[Diagnostic]) -> SarifOutput {
        let rule_ids: HashSet<&str> = diagnostics.iter().map(|d| d.rule_id.as_str()).collect();
        let rules = self.build_rules(&rule_ids);
        let results = diagnostics.iter().map(|d| self.convert_result(d)).collect();
        let artifacts = self.build_artifacts(diagnostics);

        SarifOutput {
            schema: SARIF_SCHEMA,
            version: SARIF_VERSION,
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "Kaizen",
                        semantic_version: env!("CARGO_PKG_VERSION"),
                        information_uri: Some("https://github.com/mpiton/kaizen"),
                        rules,
                    },
                },
                results,
                artifacts,
            }],
        }
    }

    fn build_rules(&self, rule_ids: &HashSet<&str>) -> Vec<SarifRule> {
        rule_ids.iter().map(|&id| self.build_rule(id)).collect()
    }

    fn build_rule(&self, rule_id: &str) -> SarifRule {
        if let Some(registry) = self.registry {
            if let Some(rule) = registry.get_rule(rule_id) {
                let metadata = rule.metadata();
                let (level, security_severity) =
                    self.severity_to_sarif(&metadata.severity, &metadata.category);

                let tags = match metadata.category {
                    RuleCategory::Quality => vec!["quality".to_string()],
                    RuleCategory::Security => {
                        vec!["security".to_string(), "external/cwe".to_string()]
                    }
                };

                return SarifRule {
                    id: metadata.id.to_string(),
                    name: Some(metadata.name.to_string()),
                    short_description: SarifMessage {
                        text: metadata.name.to_string(),
                    },
                    full_description: Some(SarifMessage {
                        text: metadata.description.to_string(),
                    }),
                    help: Some(SarifMessage {
                        text: metadata.description.to_string(),
                    }),
                    help_uri: metadata.docs_url.map(|u| u.to_string()),
                    default_configuration: SarifRuleConfiguration { level },
                    properties: Some(SarifRuleProperties {
                        security_severity,
                        precision: Some("high".to_string()),
                        tags,
                    }),
                };
            }
        }

        SarifRule {
            id: rule_id.to_string(),
            name: None,
            short_description: SarifMessage {
                text: rule_id.to_string(),
            },
            full_description: None,
            help: None,
            help_uri: None,
            default_configuration: SarifRuleConfiguration {
                level: "warning".to_string(),
            },
            properties: None,
        }
    }

    fn convert_result(&self, diag: &Diagnostic) -> SarifResult {
        let (level, _) = self.get_diagnostic_level(diag);

        let locations = vec![SarifLocation {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: normalize_path(&diag.file),
                    uri_base_id: Some("%SRCROOT%".to_string()),
                },
                region: SarifRegion {
                    start_line: diag.line,
                    start_column: diag.column,
                    end_line: if diag.end_line != diag.line {
                        Some(diag.end_line)
                    } else {
                        None
                    },
                    end_column: if diag.end_column != diag.column || diag.end_line != diag.line {
                        Some(diag.end_column)
                    } else {
                        None
                    },
                },
            },
        }];

        let fixes = diag
            .fixes
            .iter()
            .map(|f| self.convert_fix(f, &diag.file))
            .collect();

        let fingerprint = generate_fingerprint(&diag.rule_id, &diag.file, diag.line);

        let properties =
            if diag.confidence != kaizen_core::rules::Confidence::High || diag.suggestion.is_some() {
                Some(SarifResultProperties {
                    confidence: Some(format!("{:?}", diag.confidence).to_lowercase()),
                    suggestion: diag.suggestion.clone(),
                })
            } else {
                None
            };

        SarifResult {
            rule_id: diag.rule_id.clone(),
            level,
            message: SarifMessage {
                text: diag.message.clone(),
            },
            locations,
            partial_fingerprints: SarifPartialFingerprints {
                primary_location_line_hash: fingerprint,
            },
            fixes,
            properties,
        }
    }

    fn convert_fix(&self, fix: &Fix, file: &str) -> SarifFix {
        let (deleted_region, inserted_content) = match &fix.kind {
            FixKind::ReplaceWith { new_text } => (
                SarifRegion {
                    start_line: fix.line,
                    start_column: fix.column,
                    end_line: Some(fix.end_line),
                    end_column: Some(fix.end_column),
                },
                Some(SarifArtifactContent {
                    text: new_text.clone(),
                }),
            ),
            FixKind::InsertBefore { text } => (
                SarifRegion {
                    start_line: fix.line,
                    start_column: fix.column,
                    end_line: None,
                    end_column: None,
                },
                Some(SarifArtifactContent { text: text.clone() }),
            ),
        };

        SarifFix {
            description: SarifMessage {
                text: fix.title.clone(),
            },
            artifact_changes: vec![SarifArtifactChange {
                artifact_location: SarifArtifactLocation {
                    uri: normalize_path(file),
                    uri_base_id: Some("%SRCROOT%".to_string()),
                },
                replacements: vec![SarifReplacement {
                    deleted_region,
                    inserted_content,
                }],
            }],
        }
    }

    fn build_artifacts(&self, diagnostics: &[Diagnostic]) -> Vec<SarifArtifact> {
        let files: HashSet<&str> = diagnostics.iter().map(|d| d.file.as_str()).collect();
        files
            .into_iter()
            .map(|file| SarifArtifact {
                location: SarifArtifactLocation {
                    uri: normalize_path(file),
                    uri_base_id: Some("%SRCROOT%".to_string()),
                },
            })
            .collect()
    }

    fn severity_to_sarif(
        &self,
        severity: &Severity,
        category: &RuleCategory,
    ) -> (String, Option<String>) {
        let level = match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
            Severity::Hint => "note",
        };

        let security_severity = if *category == RuleCategory::Security {
            Some(
                match severity {
                    Severity::Error => "8.0",
                    Severity::Warning => "6.0",
                    Severity::Info => "3.0",
                    Severity::Hint => "1.0",
                }
                .to_string(),
            )
        } else {
            None
        };

        (level.to_string(), security_severity)
    }

    fn get_diagnostic_level(&self, diag: &Diagnostic) -> (String, Option<String>) {
        if let Some(registry) = self.registry {
            if let Some(rule) = registry.get_rule(&diag.rule_id) {
                return self.severity_to_sarif(&diag.severity, &rule.metadata().category);
            }
        }

        let level = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
            Severity::Hint => "note",
        };
        (level.to_string(), None)
    }
}

impl Default for SarifFormatter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_path(path: &str) -> String {
    path.trim_start_matches("./").to_string()
}

fn generate_fingerprint(rule_id: &str, file: &str, line: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    rule_id.hash(&mut hasher);
    file.hash(&mut hasher);
    line.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kaizen_core::rules::Confidence;

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic::new(
            "S001",
            Severity::Warning,
            "Potential XSS vulnerability",
            "src/components/App.tsx",
            42,
            10,
        )
        .with_end(42, 25)
        .with_confidence(Confidence::High)
        .with_suggestion("Use proper escaping or sanitization")
    }

    #[test]
    fn format_produces_valid_sarif() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["$schema"], SARIF_SCHEMA);
        assert_eq!(parsed["version"], SARIF_VERSION);
        assert!(parsed["runs"].is_array());
        assert_eq!(parsed["runs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn format_includes_tool_info() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let driver = &parsed["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "Kaizen");
        assert!(driver["semanticVersion"].is_string());
        assert_eq!(driver["informationUri"], "https://github.com/mpiton/kaizen");
    }

    #[test]
    fn format_includes_results() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result["ruleId"], "S001");
        assert_eq!(result["level"], "warning");
        assert_eq!(result["message"]["text"], "Potential XSS vulnerability");
    }

    #[test]
    fn format_includes_location() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let location = &parsed["runs"][0]["results"][0]["locations"][0];
        let physical = &location["physicalLocation"];

        assert_eq!(
            physical["artifactLocation"]["uri"],
            "src/components/App.tsx"
        );
        assert_eq!(physical["region"]["startLine"], 42);
        assert_eq!(physical["region"]["startColumn"], 10);
        assert_eq!(physical["region"]["endColumn"], 25);
    }

    #[test]
    fn format_includes_partial_fingerprints() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let fingerprints = &parsed["runs"][0]["results"][0]["partialFingerprints"];
        assert!(fingerprints["primaryLocationLineHash"].is_string());
        assert!(
            !fingerprints["primaryLocationLineHash"]
                .as_str()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn format_includes_rules() {
        let formatter = SarifFormatter::new();
        let diagnostics = vec![sample_diagnostic()];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let rules = parsed["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["id"], "S001");
    }

    #[test]
    fn severity_mapping_correct() {
        let formatter = SarifFormatter::new();

        let error_diag = Diagnostic::new("T001", Severity::Error, "Error", "test.js", 1, 0);
        let warning_diag = Diagnostic::new("T002", Severity::Warning, "Warning", "test.js", 2, 0);
        let info_diag = Diagnostic::new("T003", Severity::Info, "Info", "test.js", 3, 0);
        let hint_diag = Diagnostic::new("T004", Severity::Hint, "Hint", "test.js", 4, 0);

        let output = formatter.format(&[error_diag, warning_diag, info_diag, hint_diag]);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();

        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(results[2]["level"], "note");
        assert_eq!(results[3]["level"], "note");
    }

    #[test]
    fn empty_diagnostics_produces_valid_output() {
        let formatter = SarifFormatter::new();
        let diagnostics: Vec<Diagnostic> = vec![];

        let output = formatter.format(&diagnostics);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], SARIF_VERSION);
        assert!(parsed["runs"][0]["results"].as_array().unwrap().is_empty());
        assert!(
            parsed["runs"][0]["tool"]["driver"]["rules"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn normalize_path_removes_leading_dot_slash() {
        assert_eq!(normalize_path("./src/test.js"), "src/test.js");
        assert_eq!(normalize_path("src/test.js"), "src/test.js");
        assert_eq!(normalize_path("./././nested.js"), "nested.js");
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let fp1 = generate_fingerprint("S001", "test.js", 42);
        let fp2 = generate_fingerprint("S001", "test.js", 42);
        assert_eq!(fp1, fp2);

        let fp3 = generate_fingerprint("S001", "test.js", 43);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn format_includes_artifacts() {
        let formatter = SarifFormatter::new();
        let diag1 = Diagnostic::new("T001", Severity::Warning, "Issue", "src/a.js", 1, 0);
        let diag2 = Diagnostic::new("T002", Severity::Warning, "Issue", "src/b.js", 1, 0);

        let output = formatter.format(&[diag1, diag2]);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let artifacts = parsed["runs"][0]["artifacts"].as_array().unwrap();
        assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn format_includes_fix_when_present() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::new("Q030", Severity::Warning, "Avoid var", "test.js", 1, 0)
            .with_end(1, 3)
            .with_fix(kaizen_core::diagnostic::Fix::replace(
                "Replace with let",
                "let",
                1,
                0,
                1,
                3,
            ));

        let output = formatter.format(&[diag]);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let fixes = &parsed["runs"][0]["results"][0]["fixes"];
        assert!(fixes.is_array());
        assert_eq!(fixes.as_array().unwrap().len(), 1);
        assert_eq!(fixes[0]["description"]["text"], "Replace with let");
    }

    #[test]
    fn properties_include_confidence_when_not_high() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::new("T001", Severity::Warning, "Issue", "test.js", 1, 0)
            .with_confidence(Confidence::Medium);

        let output = formatter.format(&[diag]);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let props = &parsed["runs"][0]["results"][0]["properties"];
        assert_eq!(props["confidence"], "medium");
    }

    #[test]
    fn properties_include_suggestion_when_present() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::new("T001", Severity::Warning, "Issue", "test.js", 1, 0)
            .with_suggestion("Try this instead");

        let output = formatter.format(&[diag]);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let props = &parsed["runs"][0]["results"][0]["properties"];
        assert_eq!(props["suggestion"], "Try this instead");
    }
}
