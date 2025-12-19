# Kaizen

[![CI](https://github.com/mpiton/kaizen/actions/workflows/ci.yml/badge.svg)](https://github.com/mpiton/kaizen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ultra-fast JavaScript/TypeScript static analyzer written in Rust.

## What is Kaizen?

Kaizen is a modern static analysis tool designed for JavaScript and TypeScript codebases, providing:

- Fast parsing using SWC
- Semantic analysis and taint tracking
- IDE integration via Language Server Protocol (LSP)
- Command-line interface for CI/CD integration

## Installation

### Prerequisites

- Rust 1.85.0+ (Edition 2024)
- [pre-commit](https://pre-commit.com/) for git hooks (development only)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/mpiton/kaizen.git
cd kaizen

# Build the project
cargo build --release

# The binary will be available at target/release/kaizen-cli
```

## Usage

### Analyze Files

Run static analysis on your JavaScript/TypeScript files:

```bash
# Analyze a directory
kaizen check ./src

# Analyze with JSON output
kaizen check ./src --format json

# Analyze specific files
kaizen check ./src/index.ts
```

### Initialize Configuration

Create a Kaizen configuration file in your project:

```bash
# Create default configuration
kaizen init

# Overwrite existing configuration
kaizen init --force
```

### Get Rule Information

Display detailed explanation for a specific rule:

```bash
kaizen explain <rule-id>

# Example
kaizen explain no-console
```

## IDE Integration

### Installing the LSP Server

```bash
# Build and install kaizen-lsp to ~/.local/bin
chmod +x scripts/install-local.sh
./scripts/install-local.sh
```

Make sure `~/.local/bin` is in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Zed

Install the Kaizen extension as a dev extension:

1. Open Zed
2. Open the Extensions panel (View → Extensions)
3. Click "Install Dev Extension"
4. Select the `editors/zed` directory from this repository

Or build and install manually:

```bash
cd editors/zed
cargo build --release --target wasm32-wasip1
```

The extension will automatically find `kaizen-lsp` from your PATH.

### VS Code

Install the Kaizen extension:

```bash
cd editors/vscode
npm install
npm run compile
```

Then install in VS Code:
1. Open VS Code
2. Run "Extensions: Install from VSIX..." from Command Palette
3. Or copy the folder to `~/.vscode/extensions/kaizen-lsp`

Configure the LSP path if needed in VS Code settings:

```json
{
  "kaizen.serverPath": "/home/youruser/.local/bin/kaizen-lsp"
}
```

### Verify Installation

To verify the LSP is working:
1. Open a JavaScript or TypeScript file
2. Introduce a syntax error (e.g., `const x = `)
3. The error should be underlined in red

## Development

### Quick Start

```bash
# Clone the repository
git clone https://github.com/mpiton/kaizen.git
cd kaizen

# Setup development environment (installs pre-commit hooks)
chmod +x scripts/setup-hooks.sh
./scripts/setup-hooks.sh

# Build the project
cargo build

# Run tests
cargo test
```

### Project Structure

```
kaizen/
├── crates/
│   ├── kaizen-core/   # Core analysis engine
│   ├── kaizen-lsp/    # Language Server Protocol implementation
│   └── kaizen-cli/    # Command-line interface
├── editors/
│   ├── vscode/      # VS Code extension
│   └── zed/         # Zed extension
└── scripts/         # Development utilities
```

### Cargo Aliases

The project provides convenient cargo aliases:

| Alias      | Command                                           |
|------------|---------------------------------------------------|
| `cargo b`  | `cargo build`                                     |
| `cargo c`  | `cargo check`                                     |
| `cargo t`  | `cargo test`                                      |
| `cargo cl` | `cargo clippy --workspace --all-targets -D warnings` |
| `cargo r`  | `cargo run`                                       |
| `cargo d`  | `cargo doc --no-deps --open`                      |

### Pre-commit Hooks

Pre-commit hooks run automatically before each commit:

- **Format check**: Ensures code is formatted with `rustfmt`
- **Clippy**: Catches common mistakes and enforces best practices
- **File hygiene**: Trailing whitespace, EOF newlines, YAML/TOML validation

To run hooks manually:

```bash
pre-commit run --all-files
```

## Contributing

Contributions are welcome! Here's how to get started:

### Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/kaizen.git`
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Set up the development environment (see [Development](#development))

### Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write tests for new functionality

### Submitting Changes

1. Ensure all tests pass: `cargo test`
2. Ensure code is formatted: `cargo fmt --check`
3. Ensure no clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`
4. Push to your fork and submit a Pull Request

### Reporting Issues

Found a bug or have a feature request? [Open an issue](https://github.com/mpiton/kaizen/issues/new) with:

- Clear description of the problem or feature
- Steps to reproduce (for bugs)
- Expected vs actual behavior

## License

MIT License - see [LICENSE](LICENSE) for details.
