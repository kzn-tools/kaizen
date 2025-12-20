//! License loading and validation for the CLI
//!
//! Reads API keys from multiple sources in priority order:
//! 1. KAIZEN_API_KEY environment variable
//! 2. ~/.kaizen/credentials file
//! 3. kaizen.toml [license] section

use kaizen_core::config::LicenseConfig;
use kaizen_core::licensing::{LicenseError, LicenseInfo, LicenseValidator, PremiumTier};
use std::fs;
use std::path::PathBuf;

const ENV_VAR_NAME: &str = "KAIZEN_API_KEY";
const CREDENTIALS_FILE: &str = ".kaizen/credentials";

pub struct LicenseResult {
    pub tier: PremiumTier,
    #[allow(dead_code)]
    pub info: Option<LicenseInfo>,
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

pub fn load_license(config: &LicenseConfig) -> LicenseResult {
    if let Some(key) = read_from_env() {
        if let Some(result) = validate_key(&key, LicenseSource::Environment) {
            return result;
        }
    }

    if let Some(key) = read_from_credentials() {
        if let Some(result) = validate_key(&key, LicenseSource::Credentials) {
            return result;
        }
    }

    if let Some(key) = config.api_key.as_ref() {
        if let Some(result) = validate_key(key, LicenseSource::Config) {
            return result;
        }
    }

    LicenseResult {
        tier: PremiumTier::Free,
        info: None,
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

fn read_key_from_file(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn validate_key(key: &str, source: LicenseSource) -> Option<LicenseResult> {
    let secret = get_validation_secret()?;
    let validator = LicenseValidator::new(&secret);

    match validator.validate(key) {
        Ok(info) => Some(LicenseResult {
            tier: info.tier,
            info: Some(info),
            source,
        }),
        Err(LicenseError::Expired) => None,
        Err(_) => None,
    }
}

fn get_validation_secret() -> Option<Vec<u8>> {
    if let Ok(secret) = std::env::var("KAIZEN_LICENSE_SECRET") {
        if !secret.is_empty() {
            return Some(secret.into_bytes());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_secret() -> Vec<u8> {
        b"test-secret-for-license-validation".to_vec()
    }

    fn create_test_license(tier: PremiumTier, secret: &[u8]) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = LicenseInfo {
            tier,
            user_id: "test-user".to_string(),
            expires_at: now + 3600,
            issued_at: now,
        };

        let validator = LicenseValidator::new(secret);
        validator.generate_license(&info).unwrap()
    }

    #[test]
    fn license_source_as_str() {
        assert_eq!(LicenseSource::Environment.as_str(), "environment variable");
        assert_eq!(LicenseSource::Credentials.as_str(), "credentials file");
        assert_eq!(LicenseSource::Config.as_str(), "kaizen.toml");
        assert_eq!(LicenseSource::None.as_str(), "none");
    }

    #[test]
    fn load_license_returns_free_when_no_license() {
        let config = LicenseConfig { api_key: None };
        unsafe { std::env::remove_var(ENV_VAR_NAME) };

        let result = load_license(&config);

        assert_eq!(result.tier, PremiumTier::Free);
        assert!(result.info.is_none());
        assert_eq!(result.source, LicenseSource::None);
    }

    #[test]
    fn load_license_from_config() {
        let secret = create_test_secret();
        let license = create_test_license(PremiumTier::Pro, &secret);
        unsafe {
            std::env::set_var("KAIZEN_LICENSE_SECRET", String::from_utf8(secret).unwrap());
            std::env::remove_var(ENV_VAR_NAME);
        }

        let config = LicenseConfig {
            api_key: Some(license),
        };

        let result = load_license(&config);

        assert_eq!(result.tier, PremiumTier::Pro);
        assert!(result.info.is_some());
        assert_eq!(result.source, LicenseSource::Config);

        unsafe { std::env::remove_var("KAIZEN_LICENSE_SECRET") };
    }

    #[test]
    fn env_takes_priority_over_config() {
        let secret = create_test_secret();
        let env_license = create_test_license(PremiumTier::Enterprise, &secret);
        let config_license = create_test_license(PremiumTier::Pro, &secret);
        unsafe {
            std::env::set_var("KAIZEN_LICENSE_SECRET", String::from_utf8(secret).unwrap());
            std::env::set_var(ENV_VAR_NAME, &env_license);
        }

        let config = LicenseConfig {
            api_key: Some(config_license),
        };

        let result = load_license(&config);

        assert_eq!(result.tier, PremiumTier::Enterprise);
        assert_eq!(result.source, LicenseSource::Environment);

        unsafe {
            std::env::remove_var(ENV_VAR_NAME);
            std::env::remove_var("KAIZEN_LICENSE_SECRET");
        }
    }
}
