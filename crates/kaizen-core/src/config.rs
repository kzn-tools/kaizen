//! Configuration loading and parsing for Kaizen
//!
//! Provides functionality to load and parse `kaizen.toml` configuration files.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::rules::{Confidence, Severity};

pub const CONFIG_FILENAME: &str = "kaizen.toml";

const KNOWN_TOP_LEVEL_KEYS: &[&str] = &["include", "exclude", "rules", "license"];
const KNOWN_RULES_KEYS: &[&str] = &[
    "enabled",
    "disabled",
    "severity",
    "quality",
    "security",
    "min_confidence",
];

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Invalid TOML in '{path}': {message}")]
    ParseError { path: PathBuf, message: String },
}

#[derive(Debug, Clone, Default)]
pub struct ConfigResult {
    pub config: Config,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub rules: RulesConfig,
    pub license: LicenseConfig,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct LicenseConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct RulesConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    #[serde(default)]
    pub severity: HashMap<String, SeverityValue>,
    pub quality: Option<bool>,
    pub security: Option<bool>,
    pub min_confidence: Option<ConfidenceValue>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SeverityValue {
    Error,
    Warning,
    Info,
    Hint,
}

impl From<SeverityValue> for Severity {
    fn from(value: SeverityValue) -> Self {
        match value {
            SeverityValue::Error => Severity::Error,
            SeverityValue::Warning => Severity::Warning,
            SeverityValue::Info => Severity::Info,
            SeverityValue::Hint => Severity::Hint,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceValue {
    High,
    Medium,
    Low,
}

impl From<ConfidenceValue> for Confidence {
    fn from(value: ConfidenceValue) -> Self {
        match value {
            ConfidenceValue::High => Confidence::High,
            ConfidenceValue::Medium => Confidence::Medium,
            ConfidenceValue::Low => Confidence::Low,
        }
    }
}

pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let config_path = current.join(CONFIG_FILENAME);
        if config_path.exists() {
            return Some(config_path);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str(&content).map_err(|e| ConfigError::ParseError {
        path: path.to_path_buf(),
        message: e.message().to_string(),
    })
}

pub fn load_config_with_warnings(path: &Path) -> Result<ConfigResult, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let config: Config = toml::from_str(&content).map_err(|e| ConfigError::ParseError {
        path: path.to_path_buf(),
        message: e.message().to_string(),
    })?;

    let warnings = detect_unknown_keys(&content);

    Ok(ConfigResult { config, warnings })
}

fn detect_unknown_keys(content: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return warnings,
    };

    let known_top: HashSet<&str> = KNOWN_TOP_LEVEL_KEYS.iter().copied().collect();
    for key in table.keys() {
        if !known_top.contains(key.as_str()) {
            warnings.push(format!("Unknown config option: '{}'", key));
        }
    }

    if let Some(toml::Value::Table(rules)) = table.get("rules") {
        let known_rules: HashSet<&str> = KNOWN_RULES_KEYS.iter().copied().collect();
        for key in rules.keys() {
            if !known_rules.contains(key.as_str()) {
                warnings.push(format!("Unknown config option in [rules]: '{}'", key));
            }
        }
    }

    warnings
}

pub fn load_config_or_default(start_dir: &Path) -> Config {
    find_config_file(start_dir)
        .and_then(|path| load_config(&path).ok())
        .unwrap_or_default()
}

pub fn load_config_or_default_with_warnings(start_dir: &Path) -> ConfigResult {
    match find_config_file(start_dir) {
        Some(path) => load_config_with_warnings(&path).unwrap_or_default(),
        None => ConfigResult::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn load_config_from_file() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
include = ["src/**/*.ts"]
exclude = ["**/*.test.ts"]

[rules]
enabled = ["no-var"]
disabled = ["no-console"]

[rules.severity]
no-eval = "error"
"#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(config.include, vec!["src/**/*.ts"]);
        assert_eq!(config.exclude, vec!["**/*.test.ts"]);
        assert_eq!(config.rules.enabled, vec!["no-var"]);
        assert_eq!(config.rules.disabled, vec!["no-console"]);
        assert_eq!(
            config.rules.severity.get("no-eval"),
            Some(&SeverityValue::Error)
        );
    }

    #[test]
    fn default_config_when_missing() {
        let dir = create_temp_dir();
        let config = load_config_or_default(dir.path());

        assert_eq!(config, Config::default());
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
        assert!(config.rules.enabled.is_empty());
        assert!(config.rules.disabled.is_empty());
    }

    #[test]
    fn error_on_invalid_toml() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "this is not valid { toml }").unwrap();

        let result = load_config(&config_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ConfigError::ParseError { path, message } => {
                assert_eq!(path, config_path);
                assert!(!message.is_empty());
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn find_config_file_in_current_directory() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "").unwrap();

        let found = find_config_file(dir.path());

        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn find_config_file_in_parent_directory() {
        let parent = create_temp_dir();
        let child = parent.path().join("subdir");
        fs::create_dir(&child).unwrap();
        let config_path = parent.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "").unwrap();

        let found = find_config_file(&child);

        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn find_config_file_returns_none_when_not_found() {
        let dir = create_temp_dir();

        let found = find_config_file(dir.path());

        assert!(found.is_none());
    }

    #[test]
    fn partial_config_uses_defaults() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "[rules]\nenabled = [\"no-var\"]").unwrap();

        let config = load_config(&config_path).unwrap();

        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
        assert_eq!(config.rules.enabled, vec!["no-var"]);
        assert!(config.rules.disabled.is_empty());
    }

    #[test]
    fn empty_config_file_uses_defaults() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "").unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn severity_values_parse_correctly() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
[rules.severity]
rule1 = "error"
rule2 = "warning"
rule3 = "info"
rule4 = "hint"
"#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(
            config.rules.severity.get("rule1"),
            Some(&SeverityValue::Error)
        );
        assert_eq!(
            config.rules.severity.get("rule2"),
            Some(&SeverityValue::Warning)
        );
        assert_eq!(
            config.rules.severity.get("rule3"),
            Some(&SeverityValue::Info)
        );
        assert_eq!(
            config.rules.severity.get("rule4"),
            Some(&SeverityValue::Hint)
        );
    }

    #[test]
    fn severity_value_converts_to_severity() {
        assert_eq!(Severity::from(SeverityValue::Error), Severity::Error);
        assert_eq!(Severity::from(SeverityValue::Warning), Severity::Warning);
        assert_eq!(Severity::from(SeverityValue::Info), Severity::Info);
        assert_eq!(Severity::from(SeverityValue::Hint), Severity::Hint);
    }

    #[test]
    fn confidence_value_converts_to_confidence() {
        assert_eq!(Confidence::from(ConfidenceValue::High), Confidence::High);
        assert_eq!(
            Confidence::from(ConfidenceValue::Medium),
            Confidence::Medium
        );
        assert_eq!(Confidence::from(ConfidenceValue::Low), Confidence::Low);
    }

    #[test]
    fn min_confidence_parses_correctly() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
[rules]
min_confidence = "medium"
"#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(config.rules.min_confidence, Some(ConfidenceValue::Medium));
    }

    #[test]
    fn config_error_display_is_helpful() {
        let err = ConfigError::ParseError {
            path: PathBuf::from("/path/to/kaizen.toml"),
            message: "expected `=`".to_string(),
        };

        let msg = format!("{}", err);

        assert!(msg.contains("/path/to/kaizen.toml"));
        assert!(msg.contains("expected `=`"));
    }

    #[test]
    fn load_config_or_default_loads_existing_config() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "include = [\"src/**\"]").unwrap();

        let config = load_config_or_default(dir.path());

        assert_eq!(config.include, vec!["src/**"]);
    }

    #[test]
    fn warns_on_unknown_top_level_option() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
include = ["src/**"]
unknown_option = true
"#,
        )
        .unwrap();

        let result = load_config_with_warnings(&config_path).unwrap();

        assert_eq!(result.config.include, vec!["src/**"]);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown_option"));
    }

    #[test]
    fn warns_on_unknown_rules_option() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
[rules]
enabled = ["no-var"]
unknown_rule_option = true
"#,
        )
        .unwrap();

        let result = load_config_with_warnings(&config_path).unwrap();

        assert_eq!(result.config.rules.enabled, vec!["no-var"]);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown_rule_option"));
        assert!(result.warnings[0].contains("[rules]"));
    }

    #[test]
    fn no_warnings_for_valid_config() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
include = ["src/**"]
exclude = ["node_modules/**"]

[rules]
enabled = ["no-var"]
disabled = ["no-console"]

[rules.severity]
no-eval = "error"
"#,
        )
        .unwrap();

        let result = load_config_with_warnings(&config_path).unwrap();

        assert!(result.warnings.is_empty());
    }

    #[test]
    fn load_config_or_default_with_warnings_returns_warnings() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "typo = true").unwrap();

        let result = load_config_or_default_with_warnings(dir.path());

        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("typo"));
    }

    #[test]
    fn load_config_or_default_with_warnings_returns_empty_when_no_config() {
        let dir = create_temp_dir();

        let result = load_config_or_default_with_warnings(dir.path());

        assert_eq!(result.config, Config::default());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn license_config_parses_correctly() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(
            &config_path,
            r#"
[license]
api_key = "test-license-key-123"
"#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(
            config.license.api_key,
            Some("test-license-key-123".to_string())
        );
    }

    #[test]
    fn license_config_defaults_to_none() {
        let dir = create_temp_dir();
        let config_path = dir.path().join(CONFIG_FILENAME);
        fs::write(&config_path, "").unwrap();

        let config = load_config(&config_path).unwrap();

        assert_eq!(config.license.api_key, None);
    }
}
