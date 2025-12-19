#!/bin/bash
set -e

echo "Setting up Kaizen development environment..."

# Check for pre-commit
if ! command -v pre-commit &> /dev/null; then
    echo "pre-commit not found. Installing..."
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v pipx &> /dev/null; then
        pipx install pre-commit
    else
        echo "Error: pip, pip3, or pipx not found. Please install pre-commit manually."
        echo "See: https://pre-commit.com/#installation"
        exit 1
    fi
fi

# Ensure Rust toolchain components are installed
echo "Verifying Rust toolchain components..."
rustup component add rustfmt clippy rust-analyzer rust-src

# Install pre-commit hooks
echo "Installing pre-commit hooks..."
pre-commit install

echo ""
echo "Setup complete! Available cargo aliases:"
echo "  cargo b   - build"
echo "  cargo c   - check"
echo "  cargo t   - test"
echo "  cargo cl  - clippy with warnings as errors"
echo "  cargo r   - run"
echo "  cargo d   - generate and open docs"
