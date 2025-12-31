#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# INSTALL.SH - Ifá-Lang Unix/macOS Installer
# The Yoruba Programming Language
# ═══════════════════════════════════════════════════════════════════════════
# Run with: chmod +x install.sh && ./install.sh
# Or: sudo ./install.sh (for system-wide installation)
# ═══════════════════════════════════════════════════════════════════════════

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Ifá-Lang Installer v1.0.0${NC}"
echo -e "${BLUE}     The Yoruba Programming Language${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════${NC}"
echo ""

# Detect if running as root
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/ifa-lang"
    BIN_LINK="/usr/local/bin/ifa"
    SUDO=""
else
    INSTALL_DIR="$HOME/.ifa-lang"
    BIN_LINK="$HOME/.local/bin/ifa"
    SUDO=""
    mkdir -p "$HOME/.local/bin"
fi

SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check for Python
echo -e "[1/4] ${YELLOW}Checking Python installation...${NC}"
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    echo -e "${RED}[ERROR] Python not found!${NC}"
    echo "Please install Python 3.8+ from https://python.org"
    exit 1
fi

PYTHON_VERSION=$($PYTHON_CMD --version 2>&1 | awk '{print $2}')
echo -e "       Found Python $PYTHON_VERSION"

# Install to directory
echo ""
echo -e "[2/4] ${YELLOW}Installing to $INSTALL_DIR...${NC}"

# Remove old installation
if [ -d "$INSTALL_DIR" ]; then
    echo "       Removing old installation..."
    rm -rf "$INSTALL_DIR"
fi

# Create directories
mkdir -p "$INSTALL_DIR"/{bin,src,lib/std,lib/ext,examples,docs}

# Copy files
echo "       Copying core files..."
cp -r "$SOURCE_DIR/bin/"* "$INSTALL_DIR/bin/" 2>/dev/null || true
cp -r "$SOURCE_DIR/src/"* "$INSTALL_DIR/src/" 2>/dev/null || true
cp -r "$SOURCE_DIR/lib/std/"* "$INSTALL_DIR/lib/std/" 2>/dev/null || true
cp -r "$SOURCE_DIR/lib/ext/"* "$INSTALL_DIR/lib/ext/" 2>/dev/null || true
cp -r "$SOURCE_DIR/examples/"* "$INSTALL_DIR/examples/" 2>/dev/null || true
cp -r "$SOURCE_DIR/docs/"* "$INSTALL_DIR/docs/" 2>/dev/null || true

# Copy documentation
cp "$SOURCE_DIR/README.md" "$INSTALL_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/DOCS.md" "$INSTALL_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/TUTORIAL.md" "$INSTALL_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/LICENSE" "$INSTALL_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/requirements.txt" "$INSTALL_DIR/" 2>/dev/null || true

# Make scripts executable
chmod +x "$INSTALL_DIR/bin/ifa"

echo -e "       ${GREEN}Done!${NC}"

# Install Python dependencies
echo ""
echo -e "[3/4] ${YELLOW}Installing Python dependencies...${NC}"
$PYTHON_CMD -m pip install --quiet --upgrade pip 2>/dev/null || true
$PYTHON_CMD -m pip install --quiet -r "$INSTALL_DIR/requirements.txt" 2>/dev/null || true
echo -e "       ${GREEN}Done!${NC}"

# Create symlink
echo ""
echo -e "[4/4] ${YELLOW}Creating command symlink...${NC}"

# Remove old symlink if exists
rm -f "$BIN_LINK" 2>/dev/null || true

# Create new symlink
ln -sf "$INSTALL_DIR/bin/ifa" "$BIN_LINK"
echo -e "       Created: $BIN_LINK"

# Add to PATH if needed (for non-root install)
if [ "$EUID" -ne 0 ]; then
    # Check shell and add to appropriate rc file
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        if ! grep -q ".local/bin" "$SHELL_RC" 2>/dev/null; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
            echo -e "       Added ~/.local/bin to PATH in $SHELL_RC"
        fi
    fi
fi

echo -e "       ${GREEN}Done!${NC}"

# Verify installation
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     Installation Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "  Location:   $INSTALL_DIR"
echo "  Command:    ifa"
echo ""
echo "  IMPORTANT: Restart your terminal or run:"
echo "             source ~/.bashrc   (or ~/.zshrc)"
echo ""
echo "  Quick Start:"
echo "      ifa --help              Show all commands"
echo "      ifa run hello.ifa      Run an Ifá program"
echo "      ifa repl               Start interactive REPL"
echo "      ifa bytecode app.ifa   Compile to bytecode"
echo "      ifa build app.ifa      Compile to native binary"
echo ""
echo "  VS Code Extension:"
echo "      Search 'Ifá-Lang' in VS Code Extensions"
echo ""
echo -e "  ${YELLOW}Àṣẹ! (It is done!)${NC}"
echo ""
