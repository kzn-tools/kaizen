//! Kaizen CLI - Command-line interface for the Kaizen static analyzer
//!
//! Ultra-fast JavaScript/TypeScript static analyzer written in Rust.

mod commands;
mod output;

use clap::Parser;
use commands::Commands;

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
    #[command(subcommand)]
    pub command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
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
        assert!(help.contains("check"));
        assert!(help.contains("init"));
        assert!(help.contains("explain"));
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
