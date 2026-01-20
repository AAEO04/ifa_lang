#!/bin/sh
# Ifá-Lang Installer
# Usage: curl -sSL https://get.ifalang.io | sh
#
# "Ọ̀gbè ló bí ayé" — Ogbè gave birth to the world.

set -e

REPO="AAEO04/ifa_lang"
INSTALL_DIR="${IFA_INSTALL_DIR:-$HOME/.ifa/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { printf "${GREEN}▸${NC} %s\n" "$1"; }
warn() { printf "${YELLOW}▸${NC} %s\n" "$1"; }
error() { printf "${RED}✗${NC} %s\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported OS: $(uname -s). Download manually from GitHub." ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "arm64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest release tag from GitHub API
get_latest_version() {
    curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Main installation
main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║         Ifá-Lang Installer            ║"
    echo "  ║    \"Code with good character\"         ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    OS=$(detect_os)
    ARCH=$(detect_arch)
    VERSION=$(get_latest_version)
    
    if [ -z "$VERSION" ]; then
        error "Could not determine latest version. Check your internet connection."
    fi

    info "Detected: $OS-$ARCH"
    info "Latest version: $VERSION"

    # Construct download URL
    if [ "$OS" = "windows" ]; then
        BINARY="ifa-${VERSION}-${OS}-${ARCH}.exe"
    else
        BINARY="ifa-${VERSION}-${OS}-${ARCH}"
    fi
    
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download
    info "Downloading $BINARY..."
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$URL" -o "${INSTALL_DIR}/ifa" || error "Download failed. Check if release exists."
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$URL" -O "${INSTALL_DIR}/ifa" || error "Download failed."
    else
        error "curl or wget required"
    fi

    # Make executable
    chmod +x "${INSTALL_DIR}/ifa"

    info "Installed to: ${INSTALL_DIR}/ifa"

    # Add to PATH guidance
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        warn "Add to your PATH by running:"
        echo ""
        
        SHELL_NAME=$(basename "$SHELL")
        case "$SHELL_NAME" in
            zsh)
                echo "  echo 'export PATH=\"\$HOME/.ifa/bin:\$PATH\"' >> ~/.zshrc"
                echo "  source ~/.zshrc"
                ;;
            bash)
                echo "  echo 'export PATH=\"\$HOME/.ifa/bin:\$PATH\"' >> ~/.bashrc"
                echo "  source ~/.bashrc"
                ;;
            fish)
                echo "  fish_add_path ~/.ifa/bin"
                ;;
            *)
                echo "  export PATH=\"\$HOME/.ifa/bin:\$PATH\""
                ;;
        esac
        echo ""
    fi

    # Verify installation
    if "${INSTALL_DIR}/ifa" --version >/dev/null 2>&1; then
        echo ""
        info "✓ Installation complete!"
        echo ""
        "${INSTALL_DIR}/ifa" --version
        echo ""
        echo "  Get started: ifa repl"
        echo "  Documentation: https://aaeo04.github.io/ifa_lang/"
        echo ""
        echo "  Àṣẹ! (It is done!)"
        echo ""
    else
        error "Installation verification failed"
    fi
}

main "$@"
