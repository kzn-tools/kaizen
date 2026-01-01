//! Auth command - manage authentication for Kaizen

use crate::license::{LicenseSource, load_license};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::Colorize;
use kaizen_core::config::LicenseConfig;
use kaizen_core::licensing::PremiumTier;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::debug;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const CREDENTIALS_DIR: &str = ".kaizen";
const CREDENTIALS_FILE: &str = "credentials";
const DEFAULT_API_URL: &str = "https://api.kaizen.tools";
const DEVICE_FLOW_TIMEOUT_SECS: u64 = 900;
const API_TIMEOUT_SECS: u64 = 10;

#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    #[command(
        about = "Authenticate with Kaizen (opens browser for device flow, or use --key for direct API key)"
    )]
    Login {
        #[arg(
            long = "key",
            value_name = "API_KEY",
            help = "Your Kaizen API key (optional - if not provided, uses browser-based device flow)"
        )]
        api_key: Option<String>,
    },

    #[command(about = "Remove saved API key")]
    Logout,

    #[command(about = "Display current authentication status")]
    Status,
}

#[derive(Debug, Serialize)]
struct DeviceFlowRequest {
    scope: String,
    client_type: String,
}

#[derive(Debug, Deserialize)]
struct DeviceFlowResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

impl AuthArgs {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            AuthSubcommand::Login { api_key } => match api_key {
                Some(key) => Self::handle_login_with_key(key),
                None => Self::handle_device_flow(),
            },
            AuthSubcommand::Logout => Self::handle_logout(),
            AuthSubcommand::Status => Self::handle_status(),
        }
    }

    fn handle_login_with_key(api_key: &str) -> Result<()> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            anyhow::bail!("{}", t!("auth.key_empty"));
        }

        save_credentials(api_key)?;
        Ok(())
    }

    fn handle_device_flow() -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(API_TIMEOUT_SECS))
            .build()
            .context(t!("error.failed_create_client").to_string())?;

        let api_url =
            std::env::var("KAIZEN_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());

        println!("{}", t!("auth.starting").cyan());

        let device_response = initiate_device_flow(&client, &api_url)?;

        println!();
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
        );
        println!();
        println!(
            "  {}  {}",
            "1.".bold(),
            t!(
                "auth.step1",
                url = device_response.verification_uri.cyan().underline()
            )
        );
        println!();
        println!(
            "  {}  {}",
            "2.".bold(),
            t!(
                "auth.step2",
                code = device_response.user_code.yellow().bold()
            )
        );
        println!();
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
        );
        println!();

        if webbrowser::open(&device_response.verification_uri).is_ok() {
            println!("{} {}", "→".blue(), t!("auth.browser_opened"));
        } else {
            println!("{} {}", "!".yellow(), t!("auth.browser_failed"));
        }

        print!("{}", t!("auth.waiting").dimmed());
        io::stdout().flush().ok();

        let token = poll_for_token(
            &client,
            &api_url,
            &device_response.device_code,
            device_response.interval,
            device_response.expires_in,
        )?;

        println!();
        println!();

        save_credentials(&token.access_token)?;

        Ok(())
    }

    fn handle_logout() -> Result<()> {
        let credentials_path = get_credentials_path()?;

        if !credentials_path.exists() {
            println!("{} {}", "!".yellow().bold(), t!("auth.no_credentials"));
            return Ok(());
        }

        fs::remove_file(&credentials_path).with_context(|| {
            t!(
                "error.failed_remove_credentials",
                path = credentials_path.display()
            )
            .to_string()
        })?;
        println!(
            "{} {}",
            "✓".green().bold(),
            t!(
                "auth.credentials_removed",
                path = credentials_path.display()
            )
        );
        Ok(())
    }

    fn handle_status() -> Result<()> {
        let config = LicenseConfig::default();
        let result = load_license(&config);

        let tier_display = match result.tier {
            PremiumTier::Free => t!("tier.free").white(),
            PremiumTier::Pro => t!("tier.pro").cyan().bold(),
            PremiumTier::Enterprise => t!("tier.enterprise").magenta().bold(),
        };

        println!("{}", t!("auth.tier", tier = tier_display));

        match result.source {
            LicenseSource::None => {
                println!("Status: {}", t!("auth.status.not_authenticated").yellow());
            }
            source => {
                println!(
                    "{}",
                    t!(
                        "auth.status_from",
                        status = t!("auth.status.authenticated").green(),
                        source = source.as_str()
                    )
                );
            }
        }

        Ok(())
    }
}

fn get_credentials_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("{}", t!("error.home_not_found")))?;
    Ok(home.join(CREDENTIALS_DIR).join(CREDENTIALS_FILE))
}

fn save_credentials(token: &str) -> Result<()> {
    let credentials_path = get_credentials_path()?;
    let credentials_dir = credentials_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("{}", t!("error.invalid_credentials_path")))?;

    if !credentials_dir.exists() {
        fs::create_dir_all(credentials_dir)?;
    }

    fs::write(&credentials_path, token).with_context(|| {
        t!(
            "error.failed_write_credentials",
            path = credentials_path.display()
        )
        .to_string()
    })?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&credentials_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&credentials_path, perms)?;
    }

    println!(
        "{} {}",
        "✓".green().bold(),
        t!("auth.success", path = credentials_path.display())
    );
    Ok(())
}

fn initiate_device_flow(
    client: &reqwest::blocking::Client,
    api_url: &str,
) -> Result<DeviceFlowResponse> {
    let url = format!("{}/auth/device", api_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .json(&DeviceFlowRequest {
            scope: "read:user".to_string(),
            client_type: "cli".to_string(),
        })
        .send()
        .context(t!("error.failed_initiate_flow").to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_default();
        debug!("Device flow initiation failed: {} - {}", status, error_text);
        anyhow::bail!("{}", t!("auth.failed_status", status = status));
    }

    response
        .json::<DeviceFlowResponse>()
        .context(t!("error.failed_parse_response").to_string())
}

fn poll_for_token(
    client: &reqwest::blocking::Client,
    api_url: &str,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<TokenResponse> {
    let url = format!(
        "{}/auth/device/token?device_code={}",
        api_url.trim_end_matches('/'),
        device_code
    );

    let poll_interval = Duration::from_secs(interval.max(5));
    let timeout = Duration::from_secs(expires_in.min(DEVICE_FLOW_TIMEOUT_SECS));
    let start = Instant::now();
    let mut current_interval = poll_interval;

    loop {
        if start.elapsed() > timeout {
            println!();
            anyhow::bail!("{}", t!("auth.timeout"));
        }

        std::thread::sleep(current_interval);

        print!(".");
        io::stdout().flush().ok();

        match client.get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<TokenResponse>() {
                        Ok(token) => return Ok(token),
                        Err(e) => {
                            debug!("Failed to parse token response: {}", e);
                            continue;
                        }
                    }
                }

                let status = response.status();
                let body = response.text().unwrap_or_default();

                if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&body) {
                    match error_resp.error.as_deref() {
                        Some("authorization_pending") => continue,
                        Some("slow_down") => {
                            current_interval = Duration::from_secs(
                                current_interval.as_secs().saturating_add(5).min(30),
                            );
                            continue;
                        }
                        Some("access_denied") => {
                            println!();
                            anyhow::bail!("{}", t!("auth.denied"));
                        }
                        Some("expired_token") => {
                            println!();
                            anyhow::bail!("{}", t!("auth.expired"));
                        }
                        _ => {
                            debug!("Unknown error response: {} - {}", status, body);
                            continue;
                        }
                    }
                }

                debug!("Unexpected response: {} - {}", status, body);
            }
            Err(e) => {
                debug!("Request failed: {}", e);
            }
        }
    }
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
                    api_key: Some("test-key-123".to_string()),
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
                    api_key: Some("test-key".to_string()),
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
                api_key: Some("  ".to_string()),
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
                    api_key: Some("test-key".to_string()),
                },
            };
            args.run().unwrap();

            let credentials_path = get_credentials_path().unwrap();
            let perms = fs::metadata(&credentials_path).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        });
    }
}
