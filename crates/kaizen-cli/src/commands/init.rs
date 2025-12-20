//! Init command - initializes Kaizen configuration in a project

use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::Colorize;
use kaizen_core::config::CONFIG_FILENAME;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const DEFAULT_CONFIG: &str = r#"# Kaizen configuration file
# See https://github.com/kzn-tools/kaizen for documentation

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

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# Kaizen pre-commit hook - analyzes staged JavaScript/TypeScript files

kaizen check --staged

exit $?
"#;

#[derive(ValueEnum, Clone, Debug)]
pub enum HookType {
    PreCommit,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,

    /// Install a git hook (e.g., pre-commit)
    #[arg(long, value_name = "HOOK")]
    pub hook: Option<HookType>,
}

impl InitArgs {
    pub fn run(&self) -> Result<()> {
        if let Some(hook_type) = &self.hook {
            return self.install_hook(hook_type);
        }

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

    fn install_hook(&self, hook_type: &HookType) -> Result<()> {
        let git_dir = find_git_dir()?;
        let hooks_dir = git_dir.join("hooks");

        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir)?;
        }

        let (hook_name, hook_content) = match hook_type {
            HookType::PreCommit => ("pre-commit", PRE_COMMIT_HOOK),
        };

        let hook_path = hooks_dir.join(hook_name);

        if hook_path.exists() && !self.force {
            anyhow::bail!(
                "Hook '{}' already exists. Use --force to overwrite.",
                hook_name
            );
        }

        fs::write(&hook_path, hook_content)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        println!(
            "{} Installed {} hook at {}",
            "✓".green().bold(),
            hook_name.cyan(),
            hook_path.display()
        );
        Ok(())
    }
}

fn find_git_dir() -> Result<std::path::PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let git_dir = current.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
        if !current.pop() {
            anyhow::bail!("Not a git repository (or any parent up to mount point)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn init_creates_config_file() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let args = InitArgs {
            force: false,
            hook: None,
        };
        let result = args.run();

        assert!(result.is_ok());
        assert!(dir.path().join(CONFIG_FILENAME).exists());
    }

    #[test]
    #[serial]
    fn init_fails_if_config_exists_without_force() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILENAME), "existing").unwrap();

        let args = InitArgs {
            force: false,
            hook: None,
        };
        let result = args.run();

        assert!(result.is_err());
        let content = fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        assert_eq!(content, "existing");
    }

    #[test]
    #[serial]
    fn init_with_force_overwrites_existing() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILENAME), "existing").unwrap();

        let args = InitArgs {
            force: true,
            hook: None,
        };
        let result = args.run();

        assert!(result.is_ok());
        let content = fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        assert!(content.contains("[rules]"));
    }

    #[test]
    #[serial]
    fn init_installs_pre_commit_hook() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        fs::create_dir(dir.path().join(".git")).unwrap();

        let args = InitArgs {
            force: false,
            hook: Some(HookType::PreCommit),
        };
        let result = args.run();

        assert!(result.is_ok());
        let hook_path = dir.path().join(".git/hooks/pre-commit");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("kaizen check --staged"));
    }

    #[test]
    #[serial]
    fn init_hook_fails_if_not_git_repo() {
        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let args = InitArgs {
            force: false,
            hook: Some(HookType::PreCommit),
        };
        let result = args.run();

        assert!(result.is_err());
    }

    #[test]
    fn default_config_is_valid_toml() {
        let config: Result<toml::Table, _> = DEFAULT_CONFIG.parse();
        assert!(config.is_ok());
    }
}
