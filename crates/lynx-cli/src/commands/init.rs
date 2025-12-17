//! Init command - initializes Lynx configuration in a project

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use lynx_core::config::CONFIG_FILENAME;
use std::fs;
use std::path::Path;

const DEFAULT_CONFIG: &str = r#"# Lynx configuration file
# See https://github.com/mpiton/lynx for documentation

# File patterns to include in analysis
# include = ["src/**/*.ts", "src/**/*.tsx"]

# File patterns to exclude from analysis
# exclude = ["**/*.test.ts", "**/*.spec.ts"]

# Rule configuration
[rules]
# Enable specific rules (all rules enabled by default)
# enabled = []

# Disable specific rules
# disabled = ["no-console"]

# Override rule severity
# [rules.severity]
# no-console = "hint"
"#;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,
}

impl InitArgs {
    pub fn run(&self) -> Result<()> {
        let config_path = Path::new(CONFIG_FILENAME);

        if config_path.exists() && !self.force {
            anyhow::bail!(
                "Config file '{}' already exists. Use --force to overwrite.",
                CONFIG_FILENAME
            );
        }

        fs::write(config_path, DEFAULT_CONFIG)?;
        println!(
            "{} Created {} configuration file",
            "✓".green().bold(),
            CONFIG_FILENAME.cyan()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn init_creates_config_file() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let args = InitArgs { force: false };
        let result = args.run();

        assert!(result.is_ok());
        assert!(dir.path().join(CONFIG_FILENAME).exists());
    }

    #[test]
    fn init_fails_if_config_exists_without_force() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILENAME), "existing").unwrap();

        let args = InitArgs { force: false };
        let result = args.run();

        assert!(result.is_err());
        let content = fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        assert_eq!(content, "existing");
    }

    #[test]
    fn init_with_force_overwrites_existing() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILENAME), "existing").unwrap();

        let args = InitArgs { force: true };
        let result = args.run();

        assert!(result.is_ok());
        let content = fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        assert!(content.contains("[rules]"));
    }

    #[test]
    fn default_config_is_valid_toml() {
        let config: Result<toml::Table, _> = DEFAULT_CONFIG.parse();
        assert!(config.is_ok());
    }
}
