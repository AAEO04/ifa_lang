#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# UNINSTALL.SH - Ifá-Lang Unix/macOS Uninstaller
# ═══════════════════════════════════════════════════════════════════════════

set -e

echo ""
echo "Ifá-Lang Uninstaller"
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

# Detect installation location
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/ifa-lang"
    BIN_LINK="/usr/local/bin/ifa"
else
    INSTALL_DIR="$HOME/.ifa-lang"
    BIN_LINK="$HOME/.local/bin/ifa"
fi

if [ ! -d "$INSTALL_DIR" ]; then
    echo "[INFO] Ifá-Lang is not installed at $INSTALL_DIR"
    exit 0
fi

echo "This will remove Ifá-Lang from your system."
echo "Location: $INSTALL_DIR"
echo ""
read -p "Are you sure? (y/n): " CONFIRM

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "Removing files..."
rm -rf "$INSTALL_DIR"

echo "Removing symlink..."
rm -f "$BIN_LINK"

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "     Ifá-Lang has been uninstalled."
echo "═══════════════════════════════════════════════════════════════════════"
echo ""
