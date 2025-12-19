//! Check command - analyzes JavaScript/TypeScript files for issues

use crate::output::json::JsonFormatter;
use crate::output::pretty::PrettyFormatter;
use crate::output::sarif::SarifFormatter;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use kaizen_core::analysis::AnalysisEngine;
use kaizen_core::config::load_config_or_default_with_warnings;
use kaizen_core::diagnostic::Diagnostic;
use kaizen_core::parser::ParsedFile;
use kaizen_core::rules::{Confidence, Severity};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Path to file or directory to analyze
    #[arg(value_name = "PATH", required_unless_present = "staged")]
    pub path: Option<PathBuf>,

    /// Analyze only git staged files
    #[arg(long)]
    pub staged: bool,

    /// Output format for diagnostics (pretty, text, json, ndjson, sarif)
    #[arg(short, long, default_value = "pretty")]
    pub format: String,

    /// Fail on warnings (exit code 1)
    #[arg(long)]
    pub fail_on_warnings: bool,

    /// Filter diagnostics by minimum severity level (error, warning, info, hint)
    #[arg(long, value_name = "LEVEL")]
    pub severity: Option<String>,

    /// Filter diagnostics by minimum confidence level (high, medium, low)
    #[arg(long, value_name = "LEVEL", default_value = "medium")]
    pub min_confidence: String,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

impl CheckArgs {
    pub fn run(&self) -> Result<()> {
        self.configure_colors();

        let config_path = self.path.clone().unwrap_or_else(|| PathBuf::from("."));
        let config_result = load_config_or_default_with_warnings(&config_path);
        for warning in &config_result.warnings {
            eprintln!("{} {}", "warning:".yellow().bold(), warning);
        }
        let config = config_result.config;

        let files = if self.staged {
            get_staged_files()?
        } else {
            discover_files(&config_path)?
        };

        if files.is_empty() {
            if self.staged {
                println!("No staged JavaScript/TypeScript files found.");
            } else {
                println!("No JavaScript/TypeScript files found.");
            }
            return Ok(());
        }

        let engine = AnalysisEngine::with_config(&config);
        let min_severity = self.parse_severity()?;
        let min_confidence = self.parse_confidence()?;

        let results: Vec<(PathBuf, String, Vec<Diagnostic>)> = files
            .par_iter()
            .filter_map(|file| {
                let content = fs::read_to_string(file).ok()?;
                let parsed = ParsedFile::from_source(&file.to_string_lossy(), &content);
                let diagnostics = engine.analyze(&parsed);
                Some((file.clone(), content, diagnostics))
            })
            .collect();

        let sources: HashMap<String, String> = results
            .iter()
            .map(|(path, content, _)| (path.to_string_lossy().to_string(), content.clone()))
            .collect();

        let all_diagnostics: Vec<Diagnostic> = results
            .into_iter()
            .flat_map(|(_, _, diags)| diags)
            .filter(|d| severity_level(&d.severity) >= severity_level(&min_severity))
            .filter(|d| d.confidence.level() >= min_confidence.level())
            .collect();

        let error_count = all_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .count();
        let warning_count = all_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();

        let total_files = files.len();
        let analyzed_path = if self.staged {
            "(staged files)".to_string()
        } else {
            config_path.to_string_lossy().to_string()
        };

        match self.format.as_str() {
            "json" => self.output_json(&all_diagnostics, &engine, total_files, &analyzed_path),
            "ndjson" => {
                self.output_ndjson(&all_diagnostics, &engine, total_files, &analyzed_path)?
            }
            "sarif" => self.output_sarif(&all_diagnostics, &engine),
            "text" => self.output_text(&all_diagnostics),
            _ => self.output_pretty(&all_diagnostics, &sources),
        }

        let has_errors = error_count > 0;
        let has_warnings = warning_count > 0 && self.fail_on_warnings;

        if has_errors || has_warnings {
            process::exit(1);
        }

        Ok(())
    }

    fn parse_severity(&self) -> Result<Severity> {
        match self.severity.as_deref() {
            Some("error") => Ok(Severity::Error),
            Some("warning") => Ok(Severity::Warning),
            Some("info") => Ok(Severity::Info),
            Some("hint") => Ok(Severity::Hint),
            Some(other) => anyhow::bail!(
                "Invalid severity '{}'. Valid values: error, warning, info, hint",
                other
            ),
            None => Ok(Severity::Hint),
        }
    }

    fn parse_confidence(&self) -> Result<Confidence> {
        match self.min_confidence.as_str() {
            "high" => Ok(Confidence::High),
            "medium" => Ok(Confidence::Medium),
            "low" => Ok(Confidence::Low),
            other => anyhow::bail!(
                "Invalid confidence '{}'. Valid values: high, medium, low",
                other
            ),
        }
    }

    fn configure_colors(&self) {
        let no_color_env = std::env::var("NO_COLOR").is_ok();
        if self.no_color || no_color_env {
            colored::control::set_override(false);
        }
    }

    fn output_text(&self, diagnostics: &[Diagnostic]) {
        for diag in diagnostics {
            let severity_str = match diag.severity {
                Severity::Error => "error".red().bold(),
                Severity::Warning => "warning".yellow().bold(),
                Severity::Info => "info".blue().bold(),
                Severity::Hint => "hint".cyan().bold(),
            };

            println!(
                "{}:{}:{}: {} [{}]: {}",
                diag.file,
                diag.line,
                diag.column,
                severity_str,
                diag.rule_id.dimmed(),
                diag.message
            );

            if let Some(suggestion) = &diag.suggestion {
                println!("  {} {}", "suggestion:".green(), suggestion);
            }
        }

        let error_count = diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();

        if !diagnostics.is_empty() {
            println!();
            println!(
                "Found {} error(s) and {} warning(s)",
                error_count, warning_count
            );
        }
    }

    fn output_json(
        &self,
        diagnostics: &[Diagnostic],
        engine: &AnalysisEngine,
        total_files: usize,
        analyzed_path: &str,
    ) {
        let formatter = JsonFormatter::with_registry(engine.registry());
        println!(
            "{}",
            formatter.format(diagnostics, total_files, analyzed_path)
        );
    }

    fn output_ndjson(
        &self,
        diagnostics: &[Diagnostic],
        engine: &AnalysisEngine,
        total_files: usize,
        analyzed_path: &str,
    ) -> Result<()> {
        let formatter = JsonFormatter::with_registry(engine.registry());
        let mut stdout = io::stdout().lock();
        formatter.format_ndjson(diagnostics, total_files, analyzed_path, &mut stdout)?;
        Ok(())
    }

    fn output_pretty(&self, diagnostics: &[Diagnostic], sources: &HashMap<String, String>) {
        let formatter = PrettyFormatter::with_sources(sources.clone());
        print!("{}", formatter.format(diagnostics));
    }

    fn output_sarif(&self, diagnostics: &[Diagnostic], engine: &AnalysisEngine) {
        let formatter = SarifFormatter::with_registry(engine.registry());
        println!("{}", formatter.format(diagnostics));
    }
}

fn get_staged_files() -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git: {}. Is this a git repository?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<PathBuf> = stdout
        .lines()
        .map(PathBuf::from)
        .filter(|p| is_supported_file(p))
        .filter(|p| p.exists())
        .collect();

    Ok(files)
}

fn discover_files(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if path.is_file() {
        if is_supported_file(path) {
            return Ok(vec![path.to_path_buf()]);
        } else {
            return Ok(vec![]);
        }
    }

    let files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    entry
        .file_name()
        .to_str()
        .map(|name| name.starts_with('.') || name == "node_modules")
        .unwrap_or(false)
}

fn severity_level(severity: &Severity) -> u8 {
    match severity {
        Severity::Error => 4,
        Severity::Warning => 3,
        Severity::Info => 2,
        Severity::Hint => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn discover_files_finds_single_js_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.js");
        File::create(&file_path).unwrap();

        let files = discover_files(&file_path).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn discover_files_finds_files_in_directory() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("a.js")).unwrap();
        File::create(dir.path().join("b.ts")).unwrap();
        File::create(dir.path().join("c.tsx")).unwrap();

        let files = discover_files(dir.path()).unwrap();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn discover_files_ignores_unsupported_extensions() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("test.js")).unwrap();
        File::create(dir.path().join("readme.md")).unwrap();
        File::create(dir.path().join("config.json")).unwrap();

        let files = discover_files(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn discover_files_skips_hidden_directories() {
        let dir = tempdir().unwrap();
        let hidden_dir = dir.path().join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        File::create(hidden_dir.join("hidden.js")).unwrap();
        File::create(dir.path().join("visible.js")).unwrap();

        let files = discover_files(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("visible.js"));
    }

    #[test]
    fn discover_files_skips_node_modules() {
        let dir = tempdir().unwrap();
        let nm_dir = dir.path().join("node_modules");
        fs::create_dir(&nm_dir).unwrap();
        File::create(nm_dir.join("dep.js")).unwrap();
        File::create(dir.path().join("src.js")).unwrap();

        let files = discover_files(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("src.js"));
    }

    #[test]
    fn discover_files_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("src");
        fs::create_dir(&subdir).unwrap();
        File::create(dir.path().join("root.js")).unwrap();
        File::create(subdir.join("nested.ts")).unwrap();

        let files = discover_files(dir.path()).unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn is_supported_file_accepts_all_extensions() {
        assert!(is_supported_file(Path::new("test.js")));
        assert!(is_supported_file(Path::new("test.jsx")));
        assert!(is_supported_file(Path::new("test.ts")));
        assert!(is_supported_file(Path::new("test.tsx")));
        assert!(is_supported_file(Path::new("test.mjs")));
        assert!(is_supported_file(Path::new("test.cjs")));
        assert!(is_supported_file(Path::new("test.mts")));
        assert!(is_supported_file(Path::new("test.cts")));
    }

    #[test]
    fn is_supported_file_rejects_other_extensions() {
        assert!(!is_supported_file(Path::new("test.md")));
        assert!(!is_supported_file(Path::new("test.json")));
        assert!(!is_supported_file(Path::new("test.rs")));
    }

    #[test]
    fn severity_level_ordering() {
        assert!(severity_level(&Severity::Error) > severity_level(&Severity::Warning));
        assert!(severity_level(&Severity::Warning) > severity_level(&Severity::Info));
        assert!(severity_level(&Severity::Info) > severity_level(&Severity::Hint));
    }

    #[test]
    fn check_args_parse_severity_valid() {
        let args = CheckArgs {
            path: Some(PathBuf::from(".")),
            staged: false,
            format: "pretty".to_string(),
            fail_on_warnings: false,
            severity: Some("error".to_string()),
            min_confidence: "medium".to_string(),
            no_color: false,
        };

        assert!(matches!(args.parse_severity().unwrap(), Severity::Error));
    }

    #[test]
    fn check_args_parse_severity_invalid() {
        let args = CheckArgs {
            path: Some(PathBuf::from(".")),
            staged: false,
            format: "pretty".to_string(),
            fail_on_warnings: false,
            severity: Some("invalid".to_string()),
            min_confidence: "medium".to_string(),
            no_color: false,
        };

        assert!(args.parse_severity().is_err());
    }

    #[test]
    fn check_args_parse_confidence_valid() {
        let args = CheckArgs {
            path: Some(PathBuf::from(".")),
            staged: false,
            format: "pretty".to_string(),
            fail_on_warnings: false,
            severity: None,
            min_confidence: "high".to_string(),
            no_color: false,
        };

        assert!(matches!(args.parse_confidence().unwrap(), Confidence::High));
    }

    #[test]
    fn check_args_parse_confidence_invalid() {
        let args = CheckArgs {
            path: Some(PathBuf::from(".")),
            staged: false,
            format: "pretty".to_string(),
            fail_on_warnings: false,
            severity: None,
            min_confidence: "invalid".to_string(),
            no_color: false,
        };

        assert!(args.parse_confidence().is_err());
    }

    #[test]
    fn check_runs_analysis_on_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.js");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "var x = 1;").unwrap();

        let args = CheckArgs {
            path: Some(file_path),
            staged: false,
            format: "json".to_string(),
            fail_on_warnings: false,
            severity: None,
            min_confidence: "medium".to_string(),
            no_color: false,
        };

        // This will exit with code 0 since we're not checking exit in tests
        // but it exercises the code path
        let _ = args.run();
    }
}
