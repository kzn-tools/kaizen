use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    prelude::*,
};

use crate::cli::Cli;

pub fn init_logging(cli: &Cli) -> Option<WorkerGuard> {
    let level = cli.log_level.as_tracing_level();
    let filter = EnvFilter::from_default_env().add_directive(level.into());

    if let Some(ref log_file) = cli.log_file {
        init_file_logging(log_file, filter, cli.log_json)
    } else {
        init_stderr_logging(filter, cli.log_json);
        None
    }
}

fn init_stderr_logging(filter: EnvFilter, json: bool) {
    let subscriber = tracing_subscriber::registry().with(filter);

    if json {
        subscriber
            .with(
                fmt::layer()
                    .json()
                    .with_writer(std::io::stderr)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        subscriber
            .with(
                fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    }
}

fn init_file_logging(path: &Path, filter: EnvFilter, json: bool) -> Option<WorkerGuard> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("kaizen-lsp.log");

    let file_appender = tracing_appender::rolling::never(parent, filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry().with(filter);

    if json {
        subscriber
            .with(
                fmt::layer()
                    .json()
                    .with_writer(non_blocking)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        subscriber
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    }

    Some(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::LogLevel;
    use tracing::Level;

    fn parse_cli(args: &[&str]) -> Cli {
        use clap::Parser;
        Cli::try_parse_from(args).unwrap()
    }

    #[test]
    fn log_level_info_filters_debug_messages() {
        let info_level = LogLevel::Info.as_tracing_level();
        let debug_level = LogLevel::Debug.as_tracing_level();
        assert_eq!(info_level, Level::INFO);
        assert!(info_level < debug_level);
    }

    #[test]
    fn log_level_debug_includes_debug_messages() {
        let level = LogLevel::Debug.as_tracing_level();
        assert_eq!(level, Level::DEBUG);
    }

    #[test]
    fn log_level_warn_filters_info_messages() {
        let warn_level = LogLevel::Warn.as_tracing_level();
        let info_level = LogLevel::Info.as_tracing_level();
        assert_eq!(warn_level, Level::WARN);
        assert!(warn_level < info_level);
    }

    #[test]
    fn log_level_error_filters_warn_messages() {
        let error_level = LogLevel::Error.as_tracing_level();
        let warn_level = LogLevel::Warn.as_tracing_level();
        assert_eq!(error_level, Level::ERROR);
        assert!(error_level < warn_level);
    }

    #[test]
    fn default_log_level_is_info() {
        let cli = parse_cli(&["kaizen-lsp"]);
        assert_eq!(cli.log_level, LogLevel::Info);
    }

    #[test]
    fn log_file_none_by_default() {
        let cli = parse_cli(&["kaizen-lsp"]);
        assert!(cli.log_file.is_none());
    }

    #[test]
    fn log_json_false_by_default() {
        let cli = parse_cli(&["kaizen-lsp"]);
        assert!(!cli.log_json);
    }
}
