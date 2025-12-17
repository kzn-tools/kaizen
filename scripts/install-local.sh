#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="lynx-lsp"

echo "Building Lynx LSP server..."

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Build in release mode
cargo build --release -p lynx-lsp

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Copy binary
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo ""
echo "Installation complete!"
echo "Binary installed to: $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo ""
echo "To configure Zed, add to ~/.config/zed/settings.json:"
echo ""
cat << 'EOF'
{
  "lsp": {
    "lynx-lsp": {
      "binary": {
        "path": "~/.local/bin/lynx-lsp"
      }
    }
  },
  "languages": {
    "JavaScript": {
      "language_servers": ["lynx-lsp", "..."]
    },
    "TypeScript": {
      "language_servers": ["lynx-lsp", "..."]
    }
  }
}
EOF
