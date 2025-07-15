#!/bin/bash
# DBX-Ignore Installer Script
# Automatically downloads and installs the appropriate binary for your platform

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="user/dbx-ignore"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="dbx-ignore"

# Print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Detect platform and architecture
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case "$os" in
        Darwin)
            case "$arch" in
                x86_64)
                    echo "dbx-ignore-macos-intel"
                    ;;
                arm64)
                    echo "dbx-ignore-macos-arm64"
                    ;;
                *)
                    # Default to universal binary for macOS
                    echo "dbx-ignore-macos-universal"
                    ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64)
                    echo "dbx-ignore-linux-x64"
                    ;;
                *)
                    print_status "$RED" "❌ Unsupported Linux architecture: $arch"
                    print_status "$YELLOW" "   Supported: x86_64"
                    exit 1
                    ;;
            esac
            ;;
        *)
            print_status "$RED" "❌ Unsupported operating system: $os"
            print_status "$YELLOW" "   Supported: macOS (Darwin), Linux"
            print_status "$YELLOW" "   For Windows, please download manually from GitHub releases"
            exit 1
            ;;
    esac
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file with progress
download_file() {
    local url=$1
    local output=$2
    
    if command_exists curl; then
        curl -L --progress-bar "$url" -o "$output"
    elif command_exists wget; then
        wget --progress=bar:force "$url" -O "$output"
    else
        print_status "$RED" "❌ Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Main installation function
main() {
    print_status "$BLUE" "🚀 DBX-Ignore Installer"
    echo ""
    
    # Detect platform
    print_status "$YELLOW" "🔍 Detecting platform..."
    local binary_name
    binary_name=$(detect_platform)
    print_status "$GREEN" "   Platform detected: $binary_name"
    echo ""
    
    # Check for existing installation
    if command_exists "$BINARY_NAME"; then
        print_status "$YELLOW" "⚠️  dbx-ignore is already installed:"
        echo "   $($BINARY_NAME --version 2>/dev/null || echo 'Unknown version')"
        echo ""
        read -p "Do you want to continue and overwrite? (y/N): " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "$BLUE" "Installation cancelled."
            exit 0
        fi
        echo ""
    fi
    
    # Check permissions
    if [ ! -w "$INSTALL_DIR" ]; then
        print_status "$YELLOW" "⚠️  Installation requires sudo privileges for $INSTALL_DIR"
        echo ""
    fi
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    # Download binary
    print_status "$YELLOW" "📥 Downloading $binary_name..."
    local download_url="https://github.com/$GITHUB_REPO/releases/latest/download/$binary_name"
    local temp_binary="$temp_dir/$BINARY_NAME"
    
    if ! download_file "$download_url" "$temp_binary"; then
        print_status "$RED" "❌ Failed to download binary"
        print_status "$YELLOW" "   URL: $download_url"
        print_status "$YELLOW" "   Please check if the release exists or download manually"
        exit 1
    fi
    
    # Verify download
    if [ ! -f "$temp_binary" ] || [ ! -s "$temp_binary" ]; then
        print_status "$RED" "❌ Downloaded file is empty or missing"
        exit 1
    fi
    
    print_status "$GREEN" "   Downloaded successfully!"
    echo ""
    
    # Make executable
    chmod +x "$temp_binary"
    
    # Install binary
    print_status "$YELLOW" "📦 Installing to $INSTALL_DIR..."
    
    if [ -w "$INSTALL_DIR" ]; then
        mv "$temp_binary" "$INSTALL_DIR/$BINARY_NAME"
    else
        if ! sudo mv "$temp_binary" "$INSTALL_DIR/$BINARY_NAME"; then
            print_status "$RED" "❌ Failed to install binary to $INSTALL_DIR"
            print_status "$YELLOW" "   You may need to run with sudo or install manually"
            exit 1
        fi
    fi
    
    print_status "$GREEN" "   Installed successfully!"
    echo ""
    
    # Verify installation
    print_status "$YELLOW" "🧪 Verifying installation..."
    if command_exists "$BINARY_NAME"; then
        local version
        version=$($BINARY_NAME --version 2>/dev/null || echo "Could not get version")
        print_status "$GREEN" "   ✅ $version"
        print_status "$GREEN" "   ✅ Location: $(which $BINARY_NAME)"
    else
        print_status "$RED" "❌ Installation verification failed"
        print_status "$YELLOW" "   The binary was installed but is not in PATH"
        print_status "$YELLOW" "   You may need to restart your terminal or add $INSTALL_DIR to PATH"
        exit 1
    fi
    
    echo ""
    print_status "$GREEN" "🎉 Installation complete!"
    echo ""
    print_status "$BLUE" "📖 Quick start:"
    echo "   dbx-ignore --help"
    echo "   dbx-ignore --dry-run --git"
    echo ""
    print_status "$BLUE" "📚 Documentation:"
    echo "   https://github.com/$GITHUB_REPO"
}

# Show help
show_help() {
    echo "DBX-Ignore Installer"
    echo ""
    echo "USAGE:"
    echo "    curl -sSf https://raw.githubusercontent.com/$GITHUB_REPO/main/install.sh | sh"
    echo "    ./install.sh"
    echo ""
    echo "OPTIONS:"
    echo "    -h, --help    Show this help message"
    echo ""
    echo "SUPPORTED PLATFORMS:"
    echo "    macOS (Intel & Apple Silicon)"
    echo "    Linux (x86_64)"
    echo ""
    echo "For Windows, please download manually from:"
    echo "    https://github.com/$GITHUB_REPO/releases"
}

# Parse arguments
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_status "$RED" "❌ Unknown option: $1"
        echo ""
        show_help
        exit 1
        ;;
esac