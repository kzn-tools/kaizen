# Contributing to Kaizen

Thank you for your interest in contributing to Kaizen! This document provides guidelines and information for contributors.

## Prerequisites

- Rust 1.85.0+ (Edition 2024)
- [pre-commit](https://pre-commit.com/) for git hooks

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/kaizen.git
   cd kaizen
   ```
3. Set up the development environment:
   ```bash
   chmod +x scripts/setup-hooks.sh
   ./scripts/setup-hooks.sh
   ```
4. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature
   ```

## Development Workflow

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Quality

Before committing, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings
```

## Code Style

- Follow Rust conventions and idioms
- Use clear, descriptive variable and function names
- Write tests for new functionality
- Keep functions focused and small

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(cli): add JSON output format
fix(core): resolve parsing error for arrow functions
docs(readme): update installation instructions
```

## Pull Request Process

1. Ensure all tests pass
2. Ensure code is formatted (`cargo fmt --check`)
3. Ensure no clippy warnings
4. Update documentation if needed
5. Submit a pull request with a clear description

### PR Title

Use the same conventional commit format for PR titles.

### PR Description

Include:
- Summary of changes
- Related issue number (e.g., "Closes #123")
- Any breaking changes
- Testing performed

## Reporting Issues

When opening an issue, include:

- Clear description of the problem or feature request
- Steps to reproduce (for bugs)
- Expected vs actual behavior
- Rust version and OS information
- Relevant code snippets or error messages

## Pre-commit Hooks

Pre-commit hooks run automatically before each commit:

- **Format check**: Ensures code is formatted with `rustfmt`
- **Clippy**: Catches common mistakes and enforces best practices
- **File hygiene**: Trailing whitespace, EOF newlines, YAML/TOML validation

To run hooks manually:

```bash
pre-commit run --all-files
```

## Questions?

Feel free to open an issue for any questions about contributing.
