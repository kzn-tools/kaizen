use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PremiumTier {
    #[default]
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

    pub fn level(&self) -> u8 {
        match self {
            PremiumTier::Free => 0,
            PremiumTier::Pro => 1,
            PremiumTier::Enterprise => 2,
        }
    }
}

impl std::str::FromStr for PremiumTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(PremiumTier::Free),
            "pro" => Ok(PremiumTier::Pro),
            "enterprise" | "ent" => Ok(PremiumTier::Enterprise),
            _ => Err(format!("invalid tier: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premium_tier_as_str() {
        assert_eq!(PremiumTier::Free.as_str(), "free");
        assert_eq!(PremiumTier::Pro.as_str(), "pro");
        assert_eq!(PremiumTier::Enterprise.as_str(), "enterprise");
    }

    #[test]
    fn premium_tier_level_ordering() {
        assert!(
            PremiumTier::Free.level() < PremiumTier::Pro.level(),
            "Free tier must have lower level than Pro"
        );
        assert!(
            PremiumTier::Pro.level() < PremiumTier::Enterprise.level(),
            "Pro tier must have lower level than Enterprise"
        );
    }

    #[test]
    fn premium_tier_from_str() {
        assert_eq!("free".parse::<PremiumTier>().unwrap(), PremiumTier::Free);
        assert_eq!("pro".parse::<PremiumTier>().unwrap(), PremiumTier::Pro);
        assert_eq!(
            "enterprise".parse::<PremiumTier>().unwrap(),
            PremiumTier::Enterprise
        );
        assert_eq!(
            "ent".parse::<PremiumTier>().unwrap(),
            PremiumTier::Enterprise
        );
        assert!("invalid".parse::<PremiumTier>().is_err());
    }

    #[test]
    fn premium_tier_default() {
        assert_eq!(PremiumTier::default(), PremiumTier::Free);
    }
}
