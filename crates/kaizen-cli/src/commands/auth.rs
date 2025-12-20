//! Auth command - manage authentication for Kaizen

use crate::license::{LicenseSource, load_license};
use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use kaizen_core::config::LicenseConfig;
use kaizen_core::licensing::PremiumTier;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const CREDENTIALS_DIR: &str = ".kaizen";
const CREDENTIALS_FILE: &str = "credentials";

#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    /// Save API key to authenticate with Kaizen
    Login {
        /// Your Kaizen API key
        #[arg(value_name = "API_KEY")]
        api_key: String,
    },

    /// Remove saved API key
    Logout,

    /// Display current authentication status
    Status,
}

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

impl AuthArgs {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            AuthSubcommand::Login { api_key } => Self::handle_login(api_key),
            AuthSubcommand::Logout => Self::handle_logout(),
            AuthSubcommand::Status => Self::handle_status(),
        }
    }

    fn handle_login(api_key: &str) -> Result<()> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        let credentials_path = get_credentials_path()?;
        let credentials_dir = credentials_path.parent().unwrap();

        if !credentials_dir.exists() {
            fs::create_dir_all(credentials_dir)?;
        }

        fs::write(&credentials_path, api_key)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&credentials_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&credentials_path, perms)?;
        }

        println!(
            "{} API key saved to {}",
            "✓".green().bold(),
            credentials_path.display()
        );
        Ok(())
    }

    fn handle_logout() -> Result<()> {
        let credentials_path = get_credentials_path()?;

        if !credentials_path.exists() {
            println!("{} No credentials found", "!".yellow().bold());
            return Ok(());
        }

        fs::remove_file(&credentials_path)?;
        println!(
            "{} Credentials removed from {}",
            "✓".green().bold(),
            credentials_path.display()
        );
        Ok(())
    }

    fn handle_status() -> Result<()> {
        let config = LicenseConfig::default();
        let result = load_license(&config);

        let tier_display = match result.tier {
            PremiumTier::Free => "Free".white(),
            PremiumTier::Pro => "Pro".cyan().bold(),
            PremiumTier::Enterprise => "Enterprise".magenta().bold(),
        };

        println!("Tier: {}", tier_display);

        match result.source {
            LicenseSource::None => {
                println!("Status: {}", "Not authenticated".yellow());
            }
            source => {
                println!(
                    "Status: {} (from {})",
                    "Authenticated".green(),
                    source.as_str()
                );
            }
        }

        Ok(())
    }
}

fn get_credentials_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(CREDENTIALS_DIR).join(CREDENTIALS_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    fn with_temp_home<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let dir = tempdir().unwrap();
        let original_home = env::var("HOME").ok();
        unsafe { env::set_var("HOME", dir.path()) };
        let result = f();
        if let Some(home) = original_home {
            unsafe { env::set_var("HOME", home) };
        } else {
            unsafe { env::remove_var("HOME") };
        }
        result
    }

    #[test]
    #[serial]
    fn login_creates_credentials_file() {
        with_temp_home(|| {
            let args = AuthArgs {
                command: AuthSubcommand::Login {
                    api_key: "test-key-123".to_string(),
                },
            };
            let result = args.run();

            assert!(result.is_ok());
            let credentials_path = get_credentials_path().unwrap();
            assert!(credentials_path.exists());
            let content = fs::read_to_string(&credentials_path).unwrap();
            assert_eq!(content, "test-key-123");
        });
    }

    #[test]
    #[serial]
    fn login_creates_kaizen_directory() {
        with_temp_home(|| {
            let args = AuthArgs {
                command: AuthSubcommand::Login {
                    api_key: "test-key".to_string(),
                },
            };
            let result = args.run();

            assert!(result.is_ok());
            let home = dirs::home_dir().unwrap();
            assert!(home.join(CREDENTIALS_DIR).is_dir());
        });
    }

    #[test]
    #[serial]
    fn login_rejects_empty_key() {
        let args = AuthArgs {
            command: AuthSubcommand::Login {
                api_key: "  ".to_string(),
            },
        };
        let result = args.run();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    #[serial]
    fn logout_removes_credentials_file() {
        with_temp_home(|| {
            let credentials_path = get_credentials_path().unwrap();
            fs::create_dir_all(credentials_path.parent().unwrap()).unwrap();
            fs::write(&credentials_path, "test-key").unwrap();

            let args = AuthArgs {
                command: AuthSubcommand::Logout,
            };
            let result = args.run();

            assert!(result.is_ok());
            assert!(!credentials_path.exists());
        });
    }

    #[test]
    #[serial]
    fn logout_succeeds_when_no_credentials() {
        with_temp_home(|| {
            let args = AuthArgs {
                command: AuthSubcommand::Logout,
            };
            let result = args.run();

            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn status_shows_not_authenticated_when_no_license() {
        with_temp_home(|| {
            unsafe {
                env::remove_var("KAIZEN_API_KEY");
                env::remove_var("KAIZEN_LICENSE_SECRET");
            }

            let args = AuthArgs {
                command: AuthSubcommand::Status,
            };
            let result = args.run();

            assert!(result.is_ok());
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn login_sets_correct_permissions() {
        with_temp_home(|| {
            let args = AuthArgs {
                command: AuthSubcommand::Login {
                    api_key: "test-key".to_string(),
                },
            };
            args.run().unwrap();

            let credentials_path = get_credentials_path().unwrap();
            let perms = fs::metadata(&credentials_path).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        });
    }
}
