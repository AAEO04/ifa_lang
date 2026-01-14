#!/bin/bash
set -e

REPO="AAEO04/ifa-lang"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Security: Enable strict mode
set -o pipefail

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)     OS_NAME="linux";;
    Darwin*)    OS_NAME="macos";;
    *)          echo "❌ Unsupported OS: $OS"; exit 1;;
esac

case "$ARCH" in
    x86_64)     ARCH_NAME="x86_64";;
    arm64|aarch64) ARCH_NAME="arm64";;
    *)          echo "❌ Unsupported architecture: $ARCH"; exit 1;;
esac

BINARY_NAME="ifa-${OS_NAME}-${ARCH_NAME}"
BASE_URL="https://github.com/${REPO}/releases/latest/download"
DOWNLOAD_URL="${BASE_URL}/${BINARY_NAME}"
CHECKSUM_URL="${BASE_URL}/SHA256SUMS"

echo "Installing Ifa-Lang to $INSTALL_DIR..."
echo ""

# Create install directory
mkdir -p "$INSTALL_DIR"

# Create temporary directory for secure download
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Download binary
echo "⬇️  Downloading binary from $DOWNLOAD_URL..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/ifa"; then
    echo "❌ Failed to download binary. Please check your internet connection."
    exit 1
fi

# Download checksums
echo "⬇️  Downloading checksums..."
if ! curl -fsSL "$CHECKSUM_URL" -o "$TEMP_DIR/SHA256SUMS"; then
    echo "❌ Failed to download checksums. Cannot verify binary integrity."
    exit 1
fi

# Check for checksum tool
if command -v sha256sum >/dev/null 2>&1; then
    SHA_TOOL="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
    SHA_TOOL="shasum -a 256"
else
    echo "❌ No checksum tool found (tried sha256sum and shasum). Please install coreutils or similar."
    exit 1
fi

# Verify checksum
echo "🔐 Verifying binary integrity..."
EXPECTED_HASH=$(grep "${BINARY_NAME}$" "$TEMP_DIR/SHA256SUMS" | cut -d' ' -f1)

if [ -z "$EXPECTED_HASH" ]; then
    echo "❌ Could not find checksum for ${BINARY_NAME} in SHA256SUMS"
    exit 1
fi

ACTUAL_HASH=$($SHA_TOOL "$TEMP_DIR/ifa" | cut -d' ' -f1)

if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
    echo ""
    echo "❌ SECURITY ALERT: Checksum verification failed!"
    echo "   Expected: $EXPECTED_HASH"
    echo "   Got:      $ACTUAL_HASH"
    echo ""
    echo "   The downloaded binary may have been tampered with."
    echo "   Installation aborted for your security."
    exit 1
fi

echo "✅ Checksum verified successfully."

# Move verified binary to install directory
mv "$TEMP_DIR/ifa" "$INSTALL_DIR/ifa"
chmod +x "$INSTALL_DIR/ifa"

echo ""
echo "✅ Ifa-Lang installed successfully!"
echo "   Location: $INSTALL_DIR/ifa"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo "Run 'ifa --version' to verify."
