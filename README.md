# Kaizen

[![CI](https://github.com/kzn-tools/kaizen/actions/workflows/ci.yml/badge.svg)](https://github.com/kzn-tools/kaizen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ultra-fast JavaScript/TypeScript static analyzer written in Rust with security-focused rules and taint analysis.

## Features

- **Fast**: Built with Rust and SWC for blazing-fast analysis
- **Security-focused**: Detects SQL injection, XSS, command injection, and more via taint analysis
- **Quality rules**: Unused code detection, complexity checks, modern JS patterns
- **IDE support**: Language Server Protocol (LSP) for real-time diagnostics
- **CI/CD ready**: GitHub Actions integration with SARIF output for Code Scanning
- **Auto-fix**: Many rules support automatic fixes

## Quick Start

```bash
# 1. Install
cargo install kaizen-cli

# 2. Initialize configuration (optional)
kaizen init

# 3. Analyze your code
kaizen check ./src
```

## Installation

### From crates.io (Recommended)

```bash
cargo install kaizen-cli
```

### From Source

```bash
git clone https://github.com/kzn-tools/kaizen.git
cd kaizen
cargo install --path crates/kaizen-cli
```

### Verify Installation

```bash
kaizen --version
kaizen --help
```

## Usage

### Analyze Files

```bash
# Analyze a directory
kaizen check ./src

# Analyze specific files
kaizen check ./src/index.ts ./src/utils.ts

# Analyze with different output formats
kaizen check ./src --format pretty    # Human-readable (default)
kaizen check ./src --format json      # JSON for tooling
kaizen check ./src --format sarif     # SARIF for GitHub Code Scanning

# Filter by severity
kaizen check ./src --severity error   # Only errors
kaizen check ./src --severity warning # Errors and warnings

# Fail on warnings (useful for CI)
kaizen check ./src --fail-on-warnings
```

### Initialize Configuration

```bash
# Create kaizen.toml in current directory
kaizen init

# Overwrite existing configuration
kaizen init --force
```

### Get Rule Information

```bash
# Show rule explanation
kaizen explain no-console
kaizen explain Q032  # By rule ID

# List all available rules
kaizen explain --list
```

## Configuration

Create a `kaizen.toml` file in your project root:

```toml
# Include/exclude patterns
include = ["src/**/*.ts", "src/**/*.js"]
exclude = ["node_modules", "dist", "**/*.test.ts"]

# Rule configuration
[rules]
# Disable specific rules
disabled = ["no-console"]

# Or disable by rule ID
# disabled = ["Q032"]

# Enable/disable rule categories
quality = true
security = true

# Custom severity overrides
[rules.severity]
"no-console" = "error"     # Upgrade to error
"no-unused-vars" = "hint"  # Downgrade to hint
```

### Example Configurations

**Minimal (security only):**

```toml
[rules]
quality = false
security = true
```

**Strict mode:**

```toml
[rules]
disabled = []

[rules.severity]
"no-console" = "error"
"no-unused-vars" = "error"
```

**Library/package development:**

```toml
exclude = ["examples", "tests", "benchmarks"]

[rules]
disabled = ["no-console"]  # Allow console in examples
```

## CI/CD Integration

### GitHub Actions

Use the official Kaizen GitHub Action for seamless integration:

```yaml
name: Security Analysis

on: [push, pull_request]

jobs:
  kaizen:
    runs-on: ubuntu-latest
    permissions:
      security-events: write  # Required for SARIF upload
    steps:
      - uses: actions/checkout@v4

      - name: Run Kaizen
        uses: kzn-tools/kaizen@main
        with:
          path: './src'
          severity: 'warning'
          sarif-upload: 'true'
```

**Action Options:**

| Input | Description | Default |
|-------|-------------|---------|
| `path` | Path to analyze | `.` |
| `severity` | Minimum severity (error, warning, info, hint) | `hint` |
| `min-confidence` | Minimum confidence (high, medium, low) | `medium` |
| `fail-on-warnings` | Exit with code 1 if warnings found | `false` |
| `sarif-upload` | Upload SARIF to GitHub Code Scanning | `true` |
| `sarif-category` | Category for SARIF results | `kaizen` |

### GitLab CI

```yaml
kaizen:
  image: rust:latest
  script:
    - cargo install kaizen-cli
    - kaizen check ./src --fail-on-warnings
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

### Generic CI

```bash
#!/bin/bash
set -e

# Install
cargo install kaizen-cli

# Run analysis
kaizen check ./src --fail-on-warnings

# Or generate SARIF for upload
kaizen check ./src --format sarif > results.sarif
```

### Pre-commit Hook

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: kaizen
        name: Kaizen
        entry: kaizen check
        language: system
        types: [javascript, ts]
        pass_filenames: true
```

## Rules

Kaizen includes 21 built-in rules in two categories:

### Quality Rules (14 rules)

| Rule | Description | Auto-fix |
|------|-------------|----------|
| `no-unused-vars` | Detect unused variables | - |
| `no-unused-imports` | Detect unused imports | ✓ |
| `no-unreachable` | Detect unreachable code | - |
| `max-complexity` | Enforce cyclomatic complexity limit | - |
| `max-depth` | Enforce nesting depth limit | - |
| `prefer-using` | Encourage `using` for disposables | ✓ |
| `no-floating-promises` | Require Promise handling | ✓ |
| `prefer-optional-chaining` | Suggest `?.` over `&&` | - |
| `prefer-nullish-coalescing` | Suggest `??` over `\|\|` | - |
| `no-var` | Disallow `var` declarations | ✓ |
| `prefer-const` | Encourage `const` over `let` | ✓ |
| `no-console` | Warn on console.* calls | - |
| `eqeqeq` | Require strict equality | ✓ |
| `no-eval` | Disallow eval() | - |

### Security Rules (7 rules)

| Rule | Description | Analysis |
|------|-------------|----------|
| `no-sql-injection` | Detect SQL injection | Taint |
| `no-xss` | Detect XSS vulnerabilities | Taint |
| `no-command-injection` | Detect command injection | Taint |
| `no-eval-injection` | Detect code injection | Taint |
| `no-hardcoded-secrets` | Detect hardcoded secrets | Pattern |
| `no-weak-hashing` | Detect weak hash algorithms | Pattern |
| `no-insecure-random` | Detect Math.random() misuse | Pattern |

See [docs/rules/](docs/rules/) for detailed rule documentation.

## IDE Integration

### VS Code

1. Build and install the extension:
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   ```

2. Install in VS Code:
   - Copy folder to `~/.vscode/extensions/kaizen-lsp`
   - Or use "Extensions: Install from VSIX..."

3. Configure (optional):
   ```json
   {
     "kaizen.serverPath": "~/.local/bin/kaizen-lsp"
   }
   ```

### Zed

1. Install the extension:
   - Open Zed → Extensions
   - Click "Install Dev Extension"
   - Select `editors/zed` directory

2. Or build manually:
   ```bash
   cd editors/zed
   cargo build --release --target wasm32-wasip1
   ```

### LSP Server Setup

For other editors, install the LSP server:

```bash
# Install to ~/.local/bin
./scripts/install-local.sh

# Ensure PATH includes ~/.local/bin
export PATH="$HOME/.local/bin:$PATH"
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Quick setup:**

```bash
git clone https://github.com/kzn-tools/kaizen.git
cd kaizen
./scripts/setup-hooks.sh  # Install pre-commit hooks
cargo build
cargo test
```

**Before submitting:**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Project Structure

```
kaizen/
├── crates/
│   ├── kaizen-core/   # Analysis engine, rules, taint tracking
│   ├── kaizen-cli/    # Command-line interface
│   └── kaizen-lsp/    # Language Server Protocol
├── editors/
│   ├── vscode/        # VS Code extension
│   └── zed/           # Zed extension
├── docs/
│   └── rules/         # Rule documentation
└── scripts/           # Development utilities
```

## License

MIT License - see [LICENSE](LICENSE) for details.
