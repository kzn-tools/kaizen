//! License loading and validation for the CLI
//!
//! Reads API keys from multiple sources in priority order:
//! 1. KAIZEN_API_KEY environment variable
//! 2. ~/.kaizen/credentials file
//! 3. kaizen.toml [license] section
//!
//! Keys are validated against the Kaizen Cloud API.

use kaizen_core::config::LicenseConfig;
use kaizen_core::licensing::PremiumTier;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::debug;

const ENV_VAR_NAME: &str = "KAIZEN_API_KEY";
const CREDENTIALS_FILE: &str = ".kaizen/credentials";
const DEFAULT_API_URL: &str = "https://api.kaizen.tools";
const API_TIMEOUT_SECS: u64 = 5;

pub struct LicenseResult {
    pub tier: PremiumTier,
    pub source: LicenseSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LicenseSource {
    Environment,
    Credentials,
    Config,
    None,
}

impl LicenseSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseSource::Environment => "environment variable",
            LicenseSource::Credentials => "credentials file",
            LicenseSource::Config => "kaizen.toml",
            LicenseSource::None => "none",
        }
    }
}

#[derive(Serialize)]
struct ValidateRequest {
    key: String,
}

#[derive(Deserialize)]
struct ValidateResponse {
    valid: bool,
    tier: Option<String>,
}

pub fn load_license(config: &LicenseConfig) -> LicenseResult {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(API_TIMEOUT_SECS))
        .build()
        .ok();

    if let Some(key) = read_from_env() {
        if let Some(result) = validate_key(client.as_ref(), &key, LicenseSource::Environment) {
            return result;
        }
    }

    if let Some(key) = read_from_credentials() {
        if let Some(result) = validate_key(client.as_ref(), &key, LicenseSource::Credentials) {
            return result;
        }
    }

    if let Some(key) = config.api_key.as_ref() {
        if let Some(result) = validate_key(client.as_ref(), key, LicenseSource::Config) {
            return result;
        }
    }

    LicenseResult {
        tier: PremiumTier::Free,
        source: LicenseSource::None,
    }
}

fn read_from_env() -> Option<String> {
    std::env::var(ENV_VAR_NAME).ok().filter(|s| !s.is_empty())
}

fn read_from_credentials() -> Option<String> {
    let home = dirs::home_dir()?;
    let credentials_path = home.join(CREDENTIALS_FILE);
    read_key_from_file(&credentials_path)
}

fn read_key_from_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn validate_key(
    client: Option<&reqwest::blocking::Client>,
    key: &str,
    source: LicenseSource,
) -> Option<LicenseResult> {
    let client = client?;
    let api_url = std::env::var("KAIZEN_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
    let url = format!("{}/keys/validate", api_url);

    match client
        .post(&url)
        .json(&ValidateRequest {
            key: key.to_string(),
        })
        .send()
    {
        Ok(resp) => match resp.json::<ValidateResponse>() {
            Ok(data) if data.valid => {
                let tier = data
                    .tier
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(PremiumTier::Free);
                Some(LicenseResult { tier, source })
            }
            Ok(_) => {
                debug!("License from {} is invalid", source.as_str());
                None
            }
            Err(e) => {
                debug!("Failed to parse API response: {}", e);
                None
            }
        },
        Err(e) => {
            debug!("License validation failed from {}: {}", source.as_str(), e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn license_source_as_str() {
        assert_eq!(LicenseSource::Environment.as_str(), "environment variable");
        assert_eq!(LicenseSource::Credentials.as_str(), "credentials file");
        assert_eq!(LicenseSource::Config.as_str(), "kaizen.toml");
        assert_eq!(LicenseSource::None.as_str(), "none");
    }

    #[test]
    #[serial]
    fn load_license_returns_free_when_no_license() {
        let config = LicenseConfig { api_key: None };
        unsafe { std::env::remove_var(ENV_VAR_NAME) };

        let result = load_license(&config);

        assert_eq!(result.tier, PremiumTier::Free);
        assert_eq!(result.source, LicenseSource::None);
    }

    #[test]
    fn read_from_env_returns_none_for_empty() {
        unsafe { std::env::set_var(ENV_VAR_NAME, "") };
        assert!(read_from_env().is_none());
        unsafe { std::env::remove_var(ENV_VAR_NAME) };
    }

    #[test]
    fn read_from_env_returns_value() {
        unsafe { std::env::set_var(ENV_VAR_NAME, "test-key") };
        assert_eq!(read_from_env(), Some("test-key".to_string()));
        unsafe { std::env::remove_var(ENV_VAR_NAME) };
    }
}
