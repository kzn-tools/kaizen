use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "kaizen-lsp",
    author,
    version,
    about = "Language Server Protocol implementation for Kaizen",
    long_about = "Kaizen LSP provides language server protocol support for JavaScript and TypeScript.\n\n\
                  Configure logging with --log-level and --log-file options."
)]
pub struct Cli {
    #[arg(long, value_enum, default_value = "info", help = "Set the log level")]
    pub log_level: LogLevel,

    #[arg(long, help = "Write logs to the specified file")]
    pub log_file: Option<PathBuf>,

    #[arg(long, help = "Output logs in JSON format")]
    pub log_json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_default_arguments() {
        let cli = Cli::try_parse_from(["kaizen-lsp"]).unwrap();
        assert_eq!(cli.log_level, LogLevel::Info);
        assert!(cli.log_file.is_none());
        assert!(!cli.log_json);
    }

    #[test]
    fn cli_parses_log_level_debug() {
        let cli = Cli::try_parse_from(["lynx-lsp", "--log-level", "debug"]).unwrap();
        assert_eq!(cli.log_level, LogLevel::Debug);
    }

    #[test]
    fn cli_parses_log_level_warn() {
        let cli = Cli::try_parse_from(["lynx-lsp", "--log-level", "warn"]).unwrap();
        assert_eq!(cli.log_level, LogLevel::Warn);
    }

    #[test]
    fn cli_parses_log_level_error() {
        let cli = Cli::try_parse_from(["lynx-lsp", "--log-level", "error"]).unwrap();
        assert_eq!(cli.log_level, LogLevel::Error);
    }

    #[test]
    fn cli_parses_log_level_trace() {
        let cli = Cli::try_parse_from(["lynx-lsp", "--log-level", "trace"]).unwrap();
        assert_eq!(cli.log_level, LogLevel::Trace);
    }

    #[test]
    fn cli_parses_log_file() {
        let cli = Cli::try_parse_from(["kaizen-lsp", "--log-file", "/tmp/kaizen.log"]).unwrap();
        assert_eq!(cli.log_file, Some(PathBuf::from("/tmp/kaizen.log")));
    }

    #[test]
    fn cli_parses_log_json() {
        let cli = Cli::try_parse_from(["lynx-lsp", "--log-json"]).unwrap();
        assert!(cli.log_json);
    }

    #[test]
    fn cli_parses_all_options() {
        let cli = Cli::try_parse_from([
            "lynx-lsp",
            "--log-level",
            "debug",
            "--log-file",
            "/tmp/kaizen.log",
            "--log-json",
        ])
        .unwrap();
        assert_eq!(cli.log_level, LogLevel::Debug);
        assert_eq!(cli.log_file, Some(PathBuf::from("/tmp/kaizen.log")));
        assert!(cli.log_json);
    }

    #[test]
    fn cli_help_shows_options() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("--log-level"));
        assert!(help.contains("--log-file"));
        assert!(help.contains("--log-json"));
    }

    #[test]
    fn log_level_converts_to_tracing_level() {
        assert_eq!(LogLevel::Debug.as_tracing_level(), tracing::Level::DEBUG);
        assert_eq!(LogLevel::Info.as_tracing_level(), tracing::Level::INFO);
        assert_eq!(LogLevel::Warn.as_tracing_level(), tracing::Level::WARN);
        assert_eq!(LogLevel::Error.as_tracing_level(), tracing::Level::ERROR);
        assert_eq!(LogLevel::Trace.as_tracing_level(), tracing::Level::TRACE);
    }
}
