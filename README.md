# Lynx

Ultra-fast JavaScript/TypeScript static analyzer written in Rust.

## Overview

Lynx is a modern static analysis tool designed for JavaScript and TypeScript codebases, providing:

- Fast parsing using SWC
- Semantic analysis and taint tracking
- IDE integration via Language Server Protocol (LSP)
- Command-line interface for CI/CD integration

## Project Structure

```
lynx/
├── crates/
│   ├── lynx-core/   # Core analysis engine
│   ├── lynx-lsp/    # Language Server Protocol implementation
│   └── lynx-cli/    # Command-line interface
└── scripts/         # Development utilities
```

## Development Setup

### Prerequisites

- Rust 1.85.0+ (Edition 2024)
- [pre-commit](https://pre-commit.com/) for git hooks

### Quick Start

```bash
# Clone the repository
git clone https://github.com/mpiton/lynx.git
cd lynx

# Setup development environment (installs pre-commit hooks)
chmod +x scripts/setup-hooks.sh
./scripts/setup-hooks.sh

# Build the project
cargo build

# Run tests
cargo test
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

## License

MIT License - see [LICENSE](LICENSE) for details.
