# @kaizen/cli

Ultra-fast JavaScript/TypeScript static analyzer written in Rust with security focus.

## Installation

```bash
npm install -g @kaizen/cli
```

Or use without installing:

```bash
npx @kaizen/cli analyze src/
```

## Usage

```bash
# Analyze a directory
kaizen analyze src/

# Analyze specific files
kaizen analyze src/index.ts src/utils.ts

# Show help
kaizen --help
```

## Supported Platforms

- Linux (x64, ARM64)
- macOS (x64, Apple Silicon)
- Windows (x64)

## How It Works

This package automatically downloads the correct pre-built binary for your platform. The binaries are distributed through platform-specific npm packages:

- `@kaizen/cli-linux-x64`
- `@kaizen/cli-linux-arm64`
- `@kaizen/cli-darwin-x64`
- `@kaizen/cli-darwin-arm64`
- `@kaizen/cli-win32-x64`

If the platform-specific package installation fails, the postinstall script will download the binary directly from GitHub releases.

## License

MIT
