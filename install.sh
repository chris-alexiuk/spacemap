#!/bin/bash
# Installation script for storage-check

set -e

echo "Building storage-check in release mode..."
cargo build --release

echo ""
echo "Build complete!"
echo ""
echo "To install system-wide, run one of the following:"
echo ""
echo "  # Install to /usr/local/bin (requires sudo)"
echo "  sudo cp target/release/storage-check /usr/local/bin/"
echo ""
echo "  # Install to ~/.local/bin (no sudo required)"
echo "  mkdir -p ~/.local/bin"
echo "  cp target/release/storage-check ~/.local/bin/"
echo "  # Make sure ~/.local/bin is in your PATH"
echo ""
echo "Or run directly from:"
echo "  ./target/release/storage-check"
echo ""
