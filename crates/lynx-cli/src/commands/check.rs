//! Check command - analyzes JavaScript/TypeScript files for issues

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use lynx_core::analysis::AnalysisEngine;
use lynx_core::diagnostic::Diagnostic;
use lynx_core::parser::ParsedFile;
use lynx_core::rules::Severity;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Path to file or directory to analyze
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// Output format for diagnostics
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Fail on warnings (exit code 1)
    #[arg(long)]
    pub fail_on_warnings: bool,

    /// Filter diagnostics by minimum severity level (error, warning, info, hint)
    #[arg(long, value_name = "LEVEL")]
    pub severity: Option<String>,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    rule_id: String,
    severity: String,
    message: String,
    file: String,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    suggestion: Option<String>,
}

impl From<&Diagnostic> for JsonDiagnostic {
    fn from(d: &Diagnostic) -> Self {
        Self {
            rule_id: d.rule_id.clone(),
            severity: format!("{:?}", d.severity).to_lowercase(),
            message: d.message.clone(),
            file: d.file.clone(),
            line: d.line,
            column: d.column,
            end_line: d.end_line,
            end_column: d.end_column,
            suggestion: d.suggestion.clone(),
        }
    }
}

impl CheckArgs {
    pub fn run(&self) -> Result<()> {
        let files = discover_files(&self.path)?;

        if files.is_empty() {
            println!("No JavaScript/TypeScript files found.");
            return Ok(());
        }

        let engine = AnalysisEngine::new();
        let min_severity = self.parse_severity()?;

        let results: Vec<(PathBuf, Vec<Diagnostic>)> = files
            .par_iter()
            .filter_map(|file| {
                let content = fs::read_to_string(file).ok()?;
                let parsed = ParsedFile::from_source(&file.to_string_lossy(), &content);
                let diagnostics = engine.analyze(&parsed);
                Some((file.clone(), diagnostics))
            })
            .collect();

        let all_diagnostics: Vec<Diagnostic> = results
            .into_iter()
            .flat_map(|(_, diags)| diags)
            .filter(|d| severity_level(&d.severity) >= severity_level(&min_severity))
            .collect();

        let error_count = all_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .count();
        let warning_count = all_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();

        match self.format.as_str() {
            "json" => self.output_json(&all_diagnostics)?,
            _ => self.output_text(&all_diagnostics),
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

    fn output_json(&self, diagnostics: &[Diagnostic]) -> Result<()> {
        let json_diags: Vec<JsonDiagnostic> = diagnostics.iter().map(Into::into).collect();
        let json = serde_json::to_string_pretty(&json_diags)?;
        println!("{}", json);
        Ok(())
    }
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
            path: PathBuf::from("."),
            format: "text".to_string(),
            fail_on_warnings: false,
            severity: Some("error".to_string()),
        };

        assert!(matches!(args.parse_severity().unwrap(), Severity::Error));
    }

    #[test]
    fn check_args_parse_severity_invalid() {
        let args = CheckArgs {
            path: PathBuf::from("."),
            format: "text".to_string(),
            fail_on_warnings: false,
            severity: Some("invalid".to_string()),
        };

        assert!(args.parse_severity().is_err());
    }

    #[test]
    fn check_runs_analysis_on_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.js");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "var x = 1;").unwrap();

        let args = CheckArgs {
            path: file_path,
            format: "json".to_string(),
            fail_on_warnings: false,
            severity: None,
        };

        // This will exit with code 0 since we're not checking exit in tests
        // but it exercises the code path
        let _ = args.run();
    }
}
