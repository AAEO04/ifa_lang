#!/bin/bash
set -e

REPO="AAEO04/ifa-lang"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)     OS_NAME="linux";;
    Darwin*)    OS_NAME="macos";;
    *)          echo "Unsupported OS: $OS"; exit 1;;
esac

case "$ARCH" in
    x86_64)     ARCH_NAME="x86_64";;
    arm64|aarch64) ARCH_NAME="arm64";;
    *)          echo "Unsupported architecture: $ARCH"; exit 1;;
esac

BINARY_NAME="ifa-${OS_NAME}-${ARCH_NAME}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"

echo "Installing Ifa to $INSTALL_DIR..."

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download binary
echo "Downloading from $DOWNLOAD_URL..."
curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/ifa"
chmod +x "$INSTALL_DIR/ifa"

echo ""
echo "✅ Ifa-Lang installed successfully!"
echo "Location: $INSTALL_DIR/ifa"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo "Run 'ifa --version' to verify."
