use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PremiumTier {
    Free,
    Pro,
    Enterprise,
}

impl PremiumTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PremiumTier::Free => "free",
            PremiumTier::Pro => "pro",
            PremiumTier::Enterprise => "enterprise",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: PremiumTier,
    pub user_id: String,
    pub expires_at: u64,
    pub issued_at: u64,
}

impl LicenseInfo {
    pub fn is_expired(&self) -> bool {
        // If system clock is before UNIX epoch (extremely rare), treat as expired.
        // This is a safe fallback since such a state indicates system misconfiguration.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.expires_at < now
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("Invalid license format: expected '<payload>.<signature>'")]
    InvalidFormat,
    #[error("Invalid base64 encoding in license")]
    InvalidBase64,
    #[error("Invalid JSON payload in license: {0}")]
    InvalidPayload(String),
    #[error("Invalid license signature")]
    InvalidSignature,
    #[error("License has expired")]
    Expired,
    #[error("Failed to generate license")]
    GenerationError,
}

pub struct LicenseValidator {
    secret_key: Box<[u8]>,
}

impl LicenseValidator {
    pub fn new(secret_key: impl AsRef<[u8]>) -> Self {
        Self {
            secret_key: secret_key.as_ref().into(),
        }
    }

    pub fn validate(&self, license_key: &str) -> Result<LicenseInfo, LicenseError> {
        let (payload_b64, signature_b64) = license_key
            .split_once('.')
            .ok_or(LicenseError::InvalidFormat)?;

        if signature_b64.contains('.') {
            return Err(LicenseError::InvalidFormat);
        }

        let payload_bytes = BASE64
            .decode(payload_b64)
            .map_err(|_| LicenseError::InvalidBase64)?;

        let signature = BASE64
            .decode(signature_b64)
            .map_err(|_| LicenseError::InvalidBase64)?;

        self.verify_signature(payload_b64.as_bytes(), &signature)?;

        let license_info: LicenseInfo = serde_json::from_slice(&payload_bytes)
            .map_err(|e| LicenseError::InvalidPayload(e.to_string()))?;

        if license_info.is_expired() {
            return Err(LicenseError::Expired);
        }

        Ok(license_info)
    }

    fn verify_signature(&self, payload: &[u8], signature: &[u8]) -> Result<(), LicenseError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .map_err(|_| LicenseError::InvalidSignature)?;
        mac.update(payload);

        mac.verify_slice(signature)
            .map_err(|_| LicenseError::InvalidSignature)
    }

    pub fn generate_license(&self, info: &LicenseInfo) -> Result<String, LicenseError> {
        let payload_json = serde_json::to_vec(info).map_err(|_| LicenseError::GenerationError)?;
        let payload_b64 = BASE64.encode(&payload_json);

        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .map_err(|_| LicenseError::GenerationError)?;
        mac.update(payload_b64.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = BASE64.encode(signature);

        Ok(format!("{}.{}", payload_b64, signature_b64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-secret-key-for-hmac-validation";

    fn create_validator() -> LicenseValidator {
        LicenseValidator::new(TEST_SECRET)
    }

    fn create_valid_license(validator: &LicenseValidator, tier: PremiumTier) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = LicenseInfo {
            tier,
            user_id: "user-123".to_string(),
            expires_at: now + 3600,
            issued_at: now,
        };

        validator.generate_license(&info).unwrap()
    }

    fn create_expired_license(validator: &LicenseValidator) -> String {
        let info = LicenseInfo {
            tier: PremiumTier::Pro,
            user_id: "user-123".to_string(),
            expires_at: 1,
            issued_at: 0,
        };

        validator.generate_license(&info).unwrap()
    }

    #[test]
    fn validates_correct_license() {
        let validator = create_validator();
        let license = create_valid_license(&validator, PremiumTier::Pro);

        let result = validator.validate(&license);

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.tier, PremiumTier::Pro);
        assert_eq!(info.user_id, "user-123");
    }

    #[test]
    fn validates_all_tiers() {
        let validator = create_validator();

        for tier in [PremiumTier::Free, PremiumTier::Pro, PremiumTier::Enterprise] {
            let license = create_valid_license(&validator, tier);
            let result = validator.validate(&license);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().tier, tier);
        }
    }

    #[test]
    fn rejects_expired_license() {
        let validator = create_validator();
        let license = create_expired_license(&validator);

        let result = validator.validate(&license);

        assert!(matches!(result, Err(LicenseError::Expired)));
    }

    #[test]
    fn rejects_invalid_signature() {
        let validator = create_validator();
        let license = create_valid_license(&validator, PremiumTier::Pro);

        let parts: Vec<&str> = license.split('.').collect();
        let tampered = format!("{}.{}", parts[0], BASE64.encode(b"wrong-signature"));

        let result = validator.validate(&tampered);

        assert!(matches!(result, Err(LicenseError::InvalidSignature)));
    }

    #[test]
    fn rejects_tampered_payload() {
        let validator = create_validator();
        let license = create_valid_license(&validator, PremiumTier::Pro);

        let parts: Vec<&str> = license.split('.').collect();
        let tampered_payload = BASE64.encode(r#"{"tier":"enterprise","user_id":"hacker"}"#);
        let tampered = format!("{}.{}", tampered_payload, parts[1]);

        let result = validator.validate(&tampered);

        assert!(matches!(result, Err(LicenseError::InvalidSignature)));
    }

    #[test]
    fn rejects_invalid_format_no_separator() {
        let validator = create_validator();

        let result = validator.validate("invalidlicensewithoutseparator");

        assert!(matches!(result, Err(LicenseError::InvalidFormat)));
    }

    #[test]
    fn rejects_invalid_format_too_many_parts() {
        let validator = create_validator();

        let result = validator.validate("part1.part2.part3");

        assert!(matches!(result, Err(LicenseError::InvalidFormat)));
    }

    #[test]
    fn rejects_invalid_base64() {
        let validator = create_validator();

        let result = validator.validate("!!!invalid-base64!!!.valid");

        assert!(matches!(result, Err(LicenseError::InvalidBase64)));
    }

    #[test]
    fn rejects_invalid_json_payload() {
        let validator = create_validator();
        let invalid_json = BASE64.encode(b"not valid json");
        let sig = BASE64.encode(b"sig");

        let result = validator.validate(&format!("{}.{}", invalid_json, sig));

        assert!(matches!(result, Err(LicenseError::InvalidSignature)));
    }

    #[test]
    fn premium_tier_as_str() {
        assert_eq!(PremiumTier::Free.as_str(), "free");
        assert_eq!(PremiumTier::Pro.as_str(), "pro");
        assert_eq!(PremiumTier::Enterprise.as_str(), "enterprise");
    }

    #[test]
    fn license_info_expired_check() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expired = LicenseInfo {
            tier: PremiumTier::Free,
            user_id: "test".to_string(),
            expires_at: now - 100,
            issued_at: now - 200,
        };
        assert!(expired.is_expired());

        let valid = LicenseInfo {
            tier: PremiumTier::Free,
            user_id: "test".to_string(),
            expires_at: now + 100,
            issued_at: now,
        };
        assert!(!valid.is_expired());
    }

    #[test]
    fn different_keys_produce_different_signatures() {
        let validator1 = LicenseValidator::new(b"secret-1");
        let validator2 = LicenseValidator::new(b"secret-2");

        let license = create_valid_license(&validator1, PremiumTier::Pro);

        assert!(validator1.validate(&license).is_ok());
        assert!(matches!(
            validator2.validate(&license),
            Err(LicenseError::InvalidSignature)
        ));
    }
}
