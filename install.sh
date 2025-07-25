#!/bin/bash
#
# dbx-ignore installer script
# https://github.com/user/dbx-ignore
#
# This script downloads and installs dbx-ignore to /usr/local/bin

set -euo pipefail

# Configuration
readonly GITHUB_REPO="thomastheyoung/dbx-ignore"
readonly BINARY_NAME="dbx-ignore"
readonly DEFAULT_INSTALL_DIR="/usr/local/bin"
readonly VERSION="${VERSION:-latest}"

# Color codes (using tput for better compatibility)
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    readonly RED=$(tput setaf 1)
    readonly GREEN=$(tput setaf 2)
    readonly YELLOW=$(tput setaf 3)
    readonly BLUE=$(tput setaf 4)
    readonly RESET=$(tput sgr0)
else
    readonly RED=""
    readonly GREEN=""
    readonly YELLOW=""
    readonly BLUE=""
    readonly RESET=""
fi

# Global variables
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
PLATFORM=""
ARCH=""
OS=""

# Logging functions
log() {
    echo "$@"
}

log_error() {
    echo "${RED}error:${RESET} $*" >&2
}

log_warn() {
    echo "${YELLOW}warning:${RESET} $*" >&2
}

log_info() {
    echo "${BLUE}info:${RESET} $*"
}

log_success() {
    echo "${GREEN}success:${RESET} $*"
}

# Error handling
die() {
    log_error "$@"
    exit 1
}

# Cleanup on exit
cleanup() {
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

# Check if running with sufficient permissions
check_permissions() {
    local dir="${1:-$INSTALL_DIR}"
    
    # Try to create the directory if it doesn't exist
    if [ ! -d "$dir" ]; then
        if ! mkdir -p "$dir" 2>/dev/null; then
            if ! sudo -n true 2>/dev/null; then
                log_warn "Installation directory $dir does not exist and requires sudo to create"
                return 1
            fi
            sudo mkdir -p "$dir" || return 1
        fi
    fi
    
    # Check write permission
    if [ -w "$dir" ]; then
        return 0
    else
        return 1
    fi
}

# Detect operating system and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        darwin)
            OS="macos"
            case "$ARCH" in
                x86_64)
                    PLATFORM="macos-intel"
                    ;;
                arm64)
                    PLATFORM="macos-arm64"
                    ;;
                *)
                    # Try to use universal binary as fallback
                    PLATFORM="macos-universal"
                    log_warn "Unknown macOS architecture: $ARCH, trying universal binary"
                    ;;
            esac
            ;;
        linux)
            case "$ARCH" in
                x86_64|amd64)
                    PLATFORM="linux-x64"
                    ;;
                aarch64|arm64)
                    die "ARM64 Linux is not yet supported. Please build from source."
                    ;;
                *)
                    die "Unsupported Linux architecture: $ARCH"
                    ;;
            esac
            ;;
        mingw*|msys*|cygwin*|windows*)
            die "Windows is not supported by this installer. Please download the Windows binary manually from: https://github.com/$GITHUB_REPO/releases"
            ;;
        *)
            die "Unsupported operating system: $OS"
            ;;
    esac
    
    log_info "Detected platform: $OS ($ARCH) -> $PLATFORM"
}

# Check for required commands
check_dependencies() {
    local missing=()
    
    # Check for download tools
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        missing+=("curl or wget")
    fi
    
    # Check for other required tools
    for cmd in chmod mktemp; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done
    
    if [ ${#missing[@]} -gt 0 ]; then
        die "Missing required commands: ${missing[*]}"
    fi
}

# Get proxy settings
get_proxy_args() {
    local proxy_args=""
    local proxy="${HTTPS_PROXY:-${https_proxy:-${HTTP_PROXY:-${http_proxy:-}}}}"
    
    if [ -n "$proxy" ]; then
        log_info "Using proxy: $proxy"
        if command -v curl >/dev/null 2>&1; then
            proxy_args="--proxy $proxy"
        elif command -v wget >/dev/null 2>&1; then
            proxy_args="--https-proxy=$proxy"
        fi
    fi
    
    echo "$proxy_args"
}

# Check network connectivity
check_network() {
    local test_url="https://api.github.com"
    local timeout=5
    local proxy_args
    proxy_args=$(get_proxy_args)
    
    log_info "Checking network connectivity..."
    
    if command -v curl >/dev/null 2>&1; then
        if ! curl -s --connect-timeout "$timeout" --head $proxy_args "$test_url" >/dev/null 2>&1; then
            die "Cannot reach GitHub. Please check your internet connection and proxy settings."
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q --timeout="$timeout" --spider $proxy_args "$test_url" 2>/dev/null; then
            die "Cannot reach GitHub. Please check your internet connection and proxy settings."
        fi
    fi
}

# Download file with progress and retry
download_file() {
    local url="$1"
    local output="$2"
    local max_retries=3
    local retry_delay=2
    local proxy_args
    proxy_args=$(get_proxy_args)
    
    for i in $(seq 1 $max_retries); do
        log_info "Downloading from: $url (attempt $i/$max_retries)"
        
        if command -v curl >/dev/null 2>&1; then
            if curl -fL --progress-bar --connect-timeout 30 --retry 3 $proxy_args "$url" -o "$output"; then
                return 0
            fi
        elif command -v wget >/dev/null 2>&1; then
            if wget --progress=bar:force --timeout=30 --tries=3 $proxy_args "$url" -O "$output"; then
                return 0
            fi
        fi
        
        if [ $i -lt $max_retries ]; then
            log_warn "Download failed, retrying in ${retry_delay}s..."
            sleep "$retry_delay"
        fi
    done
    
    return 1
}

# Get download URL
get_download_url() {
    local platform="$1"
    local version="$2"
    
    if [ "$version" = "latest" ]; then
        echo "https://github.com/$GITHUB_REPO/releases/latest/download/$BINARY_NAME-$platform"
    else
        echo "https://github.com/$GITHUB_REPO/releases/download/$version/$BINARY_NAME-$platform"
    fi
}

# Download and verify checksum if available
download_checksum() {
    local url="$1"
    local output="$2"
    local proxy_args
    proxy_args=$(get_proxy_args)
    
    # Try to download checksum file (SHA256SUMS)
    local checksum_url="${url}.sha256"
    local checksum_file="${output}.sha256"
    
    log_info "Attempting to download checksum..."
    
    if command -v curl >/dev/null 2>&1; then
        if curl -fL -s --connect-timeout 10 $proxy_args "$checksum_url" -o "$checksum_file" 2>/dev/null; then
            return 0
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q --timeout=10 $proxy_args "$checksum_url" -O "$checksum_file" 2>/dev/null; then
            return 0
        fi
    fi
    
    # Checksum not available
    rm -f "$checksum_file"
    return 1
}

# Verify checksum
verify_checksum() {
    local binary="$1"
    local checksum_file="${binary}.sha256"
    
    if [ ! -f "$checksum_file" ]; then
        log_warn "No checksum file available, skipping verification"
        return 0
    fi
    
    # Check if we have sha256sum or shasum
    local sha_cmd=""
    if command -v sha256sum >/dev/null 2>&1; then
        sha_cmd="sha256sum"
    elif command -v shasum >/dev/null 2>&1; then
        sha_cmd="shasum -a 256"
    else
        log_warn "No SHA256 tool available, skipping checksum verification"
        return 0
    fi
    
    # Verify checksum
    local expected_checksum
    expected_checksum=$(cat "$checksum_file" | awk '{print $1}')
    local actual_checksum
    actual_checksum=$($sha_cmd "$binary" | awk '{print $1}')
    
    if [ "$expected_checksum" = "$actual_checksum" ]; then
        log_success "Checksum verified: $expected_checksum"
        return 0
    else
        log_error "Checksum verification failed!"
        log_error "Expected: $expected_checksum"
        log_error "Actual:   $actual_checksum"
        return 1
    fi
}

# Verify downloaded binary
verify_binary() {
    local binary="$1"
    
    # Check if file exists and is not empty
    if [ ! -f "$binary" ] || [ ! -s "$binary" ]; then
        return 1
    fi
    
    # Check if it's a valid executable (basic check)
    if command -v file >/dev/null 2>&1; then
        if ! file "$binary" 2>/dev/null | grep -q "executable\|binary"; then
            # Not a conclusive test, just a warning
            log_warn "File may not be a valid executable"
        fi
    fi
    
    # Verify checksum if available
    if ! verify_checksum "$binary"; then
        return 1
    fi
    
    return 0
}

# Install binary
install_binary() {
    local source="$1"
    local dest="$2"
    local use_sudo=false
    
    # Check if we need sudo
    if ! check_permissions "$(dirname "$dest")"; then
        use_sudo=true
        log_warn "sudo required for installation to $dest"
    fi
    
    # Make executable
    chmod +x "$source" || die "Failed to make binary executable"
    
    # Create backup if file exists
    if [ -f "$dest" ]; then
        local backup="${dest}.backup.$(date +%Y%m%d%H%M%S)"
        log_info "Backing up existing binary to: $backup"
        if [ "$use_sudo" = true ]; then
            sudo cp "$dest" "$backup" || log_warn "Failed to create backup"
        else
            cp "$dest" "$backup" || log_warn "Failed to create backup"
        fi
    fi
    
    # Install the binary
    log_info "Installing to: $dest"
    if [ "$use_sudo" = true ]; then
        if ! sudo mv "$source" "$dest"; then
            die "Failed to install binary. Please check permissions."
        fi
    else
        if ! mv "$source" "$dest"; then
            die "Failed to install binary. Please check permissions."
        fi
    fi
}

# Verify installation
verify_installation() {
    local binary_path="$1"
    local binary_name="$(basename "$binary_path")"
    
    # Check if binary exists
    if [ ! -f "$binary_path" ]; then
        return 1
    fi
    
    # Check if in PATH
    if ! command -v "$binary_name" >/dev/null 2>&1; then
        log_warn "$binary_name installed but not in PATH"
        log_warn "Add $(dirname "$binary_path") to your PATH or use the full path: $binary_path"
        return 0  # Installation successful, just not in PATH
    fi
    
    # Try to get version
    local version
    if version=$("$binary_name" --version 2>&1); then
        log_success "Installed: $version"
    else
        log_success "Installed successfully (version check failed)"
    fi
    
    return 0
}

# Show help
show_help() {
    cat <<EOF
dbx-ignore installer

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -d, --install-dir DIR   Set installation directory (default: $DEFAULT_INSTALL_DIR)
    -v, --version VERSION   Install specific version (default: latest)
    -y, --yes               Skip confirmation prompts

ENVIRONMENT VARIABLES:
    INSTALL_DIR    Installation directory (overrides --install-dir)
    VERSION        Version to install (overrides --version)
    HTTPS_PROXY    HTTPS proxy to use for downloads (e.g., http://proxy:8080)
    HTTP_PROXY     HTTP proxy to use for downloads

EXAMPLES:
    # Install latest version to default location
    $0

    # Install to custom directory
    $0 --install-dir ~/bin

    # Install specific version
    $0 --version v0.1.0

    # Non-interactive installation
    $0 --yes

SUPPORTED PLATFORMS:
    - macOS (Intel & Apple Silicon)
    - Linux (x86_64)

For Windows, please download manually from:
    https://github.com/$GITHUB_REPO/releases
EOF
}

# Parse command line arguments
parse_args() {
    local skip_confirmation=false
    
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                show_help
                exit 0
                ;;
            -d|--install-dir)
                shift
                if [ $# -eq 0 ]; then
                    die "Missing argument for --install-dir"
                fi
                INSTALL_DIR="$1"
                ;;
            -v|--version)
                shift
                if [ $# -eq 0 ]; then
                    die "Missing argument for --version"
                fi
                VERSION="$1"
                ;;
            -y|--yes)
                skip_confirmation=true
                ;;
            *)
                die "Unknown option: $1"
                ;;
        esac
        shift
    done
    
    echo "$skip_confirmation"
}

# Main installation function
main() {
    local skip_confirmation
    skip_confirmation=$(parse_args "$@")
    
    log "dbx-ignore installer"
    log ""
    
    # Preliminary checks
    check_dependencies
    detect_platform
    check_network
    
    # Expand paths
    INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"
    local install_path="$INSTALL_DIR/$BINARY_NAME"
    
    # Check for existing installation
    if [ -f "$install_path" ] && [ "$skip_confirmation" != "true" ]; then
        log_warn "$BINARY_NAME is already installed at: $install_path"
        printf "Do you want to overwrite it? [y/N] "
        read -r response
        case "$response" in
            [yY][eE][sS]|[yY])
                ;;
            *)
                log "Installation cancelled."
                exit 0
                ;;
        esac
    fi
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    log_info "Using temporary directory: $TEMP_DIR"
    
    # Download binary
    local download_url
    download_url=$(get_download_url "$PLATFORM" "$VERSION")
    local temp_binary="$TEMP_DIR/$BINARY_NAME"
    
    if ! download_file "$download_url" "$temp_binary"; then
        die "Failed to download binary from: $download_url"
    fi
    
    # Try to download checksum
    download_checksum "$download_url" "$temp_binary"
    
    # Verify download
    if ! verify_binary "$temp_binary"; then
        die "Downloaded file appears to be invalid or corrupted"
    fi
    
    log_success "Download complete"
    
    # Install binary
    install_binary "$temp_binary" "$install_path"
    
    # Verify installation
    if ! verify_installation "$install_path"; then
        die "Installation verification failed"
    fi
    
    # Show quick start
    log ""
    log_success "Installation complete!"
    log ""
    log "Quick start:"
    log "  $BINARY_NAME --help"
    log "  $BINARY_NAME --status"
    log "  $BINARY_NAME --dry-run --git"
    log ""
    log "Documentation: https://github.com/$GITHUB_REPO"
}

# Run main function if not sourced
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi