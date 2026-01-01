//! Kaizen CLI - Command-line interface for the Kaizen static analyzer
//!
//! Ultra-fast JavaScript/TypeScript static analyzer written in Rust.

mod commands;
mod i18n;
mod license;
mod output;

use clap::Parser;
use commands::Commands;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Parser, Debug)]
#[command(
    name = "kaizen",
    author,
    version,
    about = "Ultra-fast JavaScript/TypeScript static analyzer",
    long_about = "Kaizen is a blazingly fast static analyzer for JavaScript and TypeScript.\n\n\
                  It detects code quality issues, security vulnerabilities, and provides\n\
                  actionable suggestions to improve your codebase."
)]
pub struct Cli {
    #[arg(long, global = true, help = "Set output language (en, fr)")]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    i18n::init_locale(cli.lang.as_deref());

    match cli.command {
        Commands::Auth(args) => args.run(),
        Commands::Check(args) => args.run(),
        Commands::Init(args) => args.run(),
        Commands::Explain(args) => args.run(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_check_command() {
        let cli = Cli::try_parse_from(["kaizen", "check", "./src"]).unwrap();
        match cli.command {
            Commands::Check(args) => {
                assert_eq!(args.path.unwrap().to_str().unwrap(), "./src");
            }
            _ => panic!("Expected Check command"),
        }
    }

    #[test]
    fn cli_parses_check_staged() {
        let cli = Cli::try_parse_from(["kaizen", "check", "--staged"]).unwrap();
        match cli.command {
            Commands::Check(args) => {
                assert!(args.staged);
                assert!(args.path.is_none());
            }
            _ => panic!("Expected Check command"),
        }
    }

    #[test]
    fn cli_parses_init_with_hook() {
        let cli = Cli::try_parse_from(["kaizen", "init", "--hook", "pre-commit"]).unwrap();
        match cli.command {
            Commands::Init(args) => {
                assert!(args.hook.is_some());
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn cli_parses_check_with_format() {
        let cli = Cli::try_parse_from(["kaizen", "check", "./src", "--format", "json"]).unwrap();
        match cli.command {
            Commands::Check(args) => {
                assert_eq!(args.format, "json");
            }
            _ => panic!("Expected Check command"),
        }
    }

    #[test]
    fn cli_parses_init_command() {
        let cli = Cli::try_parse_from(["kaizen", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init(_)));
    }

    #[test]
    fn cli_parses_init_with_force() {
        let cli = Cli::try_parse_from(["kaizen", "init", "--force"]).unwrap();
        match cli.command {
            Commands::Init(args) => {
                assert!(args.force);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn cli_parses_explain_command() {
        let cli = Cli::try_parse_from(["kaizen", "explain", "no-console"]).unwrap();
        match cli.command {
            Commands::Explain(args) => {
                assert_eq!(args.rule_id, "no-console");
            }
            _ => panic!("Expected Explain command"),
        }
    }

    #[test]
    fn cli_version_is_set() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_version(), Some("0.1.0"));
    }

    #[test]
    fn cli_help_contains_commands() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("auth"));
        assert!(help.contains("check"));
        assert!(help.contains("init"));
        assert!(help.contains("explain"));
    }

    #[test]
    fn cli_parses_auth_login_with_key() {
        let cli = Cli::try_parse_from(["kaizen", "auth", "login", "--key", "test-key"]).unwrap();
        match cli.command {
            Commands::Auth(args) => match args.command {
                commands::auth::AuthSubcommand::Login { api_key } => {
                    assert_eq!(api_key, Some("test-key".to_string()));
                }
                _ => panic!("Expected Login subcommand"),
            },
            _ => panic!("Expected Auth command"),
        }
    }

    #[test]
    fn cli_parses_auth_login_without_key() {
        let cli = Cli::try_parse_from(["kaizen", "auth", "login"]).unwrap();
        match cli.command {
            Commands::Auth(args) => match args.command {
                commands::auth::AuthSubcommand::Login { api_key } => {
                    assert_eq!(api_key, None);
                }
                _ => panic!("Expected Login subcommand"),
            },
            _ => panic!("Expected Auth command"),
        }
    }

    #[test]
    fn cli_parses_auth_status() {
        let cli = Cli::try_parse_from(["kaizen", "auth", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Auth(_)));
    }

    #[test]
    fn cli_parses_auth_logout() {
        let cli = Cli::try_parse_from(["kaizen", "auth", "logout"]).unwrap();
        assert!(matches!(cli.command, Commands::Auth(_)));
    }

    #[test]
    fn check_help_shows_options() {
        let mut cmd = Cli::command();
        let check_cmd = cmd
            .get_subcommands_mut()
            .find(|c| c.get_name() == "check")
            .unwrap();
        let help = check_cmd.render_help().to_string();
        assert!(help.contains("PATH"));
        assert!(help.contains("--format"));
    }

    #[test]
    fn cli_parses_explain_with_rule_id() {
        let cli = Cli::try_parse_from(["kaizen", "explain", "Q030"]).unwrap();
        match cli.command {
            Commands::Explain(args) => {
                assert_eq!(args.rule_id, "Q030");
            }
            _ => panic!("Expected Explain command"),
        }
    }
}
