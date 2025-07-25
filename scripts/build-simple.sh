#!/bin/bash
#
# Simple cross-platform build script for dbx-ignore
# Builds for current platform and attempts cross-compilation for macOS targets
#
# This script is designed to work without complex dependencies,
# making it suitable for local development and CI environments

set -euo pipefail

# Configuration
readonly BINARY_NAME="dbx-ignore"
readonly BIN_DIR="bin"
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Color codes
if [ -t 1 ]; then
    readonly RED=$(tput setaf 1 2>/dev/null || echo "")
    readonly GREEN=$(tput setaf 2 2>/dev/null || echo "")
    readonly YELLOW=$(tput setaf 3 2>/dev/null || echo "")
    readonly BLUE=$(tput setaf 4 2>/dev/null || echo "")
    readonly RESET=$(tput sgr0 2>/dev/null || echo "")
else
    readonly RED=""
    readonly GREEN=""
    readonly YELLOW=""
    readonly BLUE=""
    readonly RESET=""
fi

# Change to project root
cd "$PROJECT_ROOT"

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect platform
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case "$os" in
        Darwin)
            echo "macos"
            ;;
        Linux)
            echo "linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64)
            echo "x64"
            ;;
        arm64|aarch64)
            echo "arm64"
            ;;
        *)
            echo "$arch"
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    if ! command_exists cargo; then
        log_error "Rust toolchain not found"
        log_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi
    
    # Check for rustup (needed for cross-compilation)
    if ! command_exists rustup; then
        log_warn "rustup not found - cross-compilation may not work"
    fi
}

# Build for current platform
build_native() {
    log_info "Building for current platform..."
    
    if ! cargo build --release 2>&1; then
        log_error "Failed to build for current platform"
        return 1
    fi
    
    # Determine output name
    local platform=$(detect_platform)
    local arch=$(detect_arch)
    local output_name=""
    
    case "$platform-$arch" in
        macos-x64)
            output_name="dbx-ignore-macos-intel"
            ;;
        macos-arm64)
            output_name="dbx-ignore-macos-arm64"
            ;;
        linux-x64)
            output_name="dbx-ignore-linux-x64"
            ;;
        windows-*)
            output_name="dbx-ignore-windows-x64.exe"
            ;;
        *)
            output_name="dbx-ignore-$platform-$arch"
            ;;
    esac
    
    # Copy binary
    local source="target/release/$BINARY_NAME"
    if [[ "$platform" == "windows" ]]; then
        source="${source}.exe"
    fi
    
    if [ -f "$source" ]; then
        cp "$source" "$BIN_DIR/$output_name"
        log_success "Built native binary: $output_name"
        return 0
    else
        log_error "Native binary not found at: $source"
        return 1
    fi
}

# Attempt macOS cross-compilation
build_macos_cross() {
    local platform=$(detect_platform)
    
    if [[ "$platform" != "macos" ]]; then
        log_info "Skipping macOS cross-compilation (not on macOS)"
        return 0
    fi
    
    if ! command_exists rustup; then
        log_warn "rustup not available, skipping cross-compilation"
        return 0
    fi
    
    local current_arch=$(detect_arch)
    local target_arch=""
    local target_triple=""
    local output_name=""
    
    # Determine target architecture (opposite of current)
    if [[ "$current_arch" == "x64" ]]; then
        target_arch="arm64"
        target_triple="aarch64-apple-darwin"
        output_name="dbx-ignore-macos-arm64"
    else
        target_arch="x64"
        target_triple="x86_64-apple-darwin"
        output_name="dbx-ignore-macos-intel"
    fi
    
    log_info "Attempting cross-compilation for macOS $target_arch..."
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target_triple"; then
        log_info "Installing target: $target_triple"
        if ! rustup target add "$target_triple" 2>/dev/null; then
            log_warn "Failed to install target $target_triple"
            return 0
        fi
    fi
    
    # Attempt build
    if cargo build --release --target "$target_triple" 2>/dev/null; then
        cp "target/$target_triple/release/$BINARY_NAME" "$BIN_DIR/$output_name"
        log_success "Cross-compiled: $output_name"
        return 0
    else
        log_warn "Cross-compilation failed for $target_triple"
        log_info "This is often due to missing Xcode components"
        return 0
    fi
}

# Create universal macOS binary
create_universal_binary() {
    local intel="$BIN_DIR/dbx-ignore-macos-intel"
    local arm64="$BIN_DIR/dbx-ignore-macos-arm64"
    local universal="$BIN_DIR/dbx-ignore-macos-universal"
    
    if [ ! -f "$intel" ] || [ ! -f "$arm64" ]; then
        log_info "Skipping universal binary (need both Intel and ARM64 builds)"
        return 0
    fi
    
    if ! command_exists lipo; then
        log_warn "lipo not found, skipping universal binary"
        return 0
    fi
    
    log_info "Creating universal macOS binary..."
    
    if lipo -create "$intel" "$arm64" -output "$universal" 2>/dev/null; then
        log_success "Created universal binary: dbx-ignore-macos-universal"
    else
        log_warn "Failed to create universal binary"
    fi
}

# Generate checksums
generate_checksums() {
    if [ -z "$(ls -A "$BIN_DIR" 2>/dev/null)" ]; then
        return 0
    fi
    
    log_info "Generating checksums..."
    
    cd "$BIN_DIR"
    
    for file in *; do
        if [ -f "$file" ] && [[ ! "$file" == *.sha256 ]]; then
            if command_exists sha256sum; then
                sha256sum "$file" > "${file}.sha256"
            elif command_exists shasum; then
                shasum -a 256 "$file" > "${file}.sha256"
            fi
        fi
    done
    
    cd - >/dev/null
}

# Main function
main() {
    log "Simple build script for $BINARY_NAME"
    log ""
    
    # Check prerequisites
    check_prerequisites
    
    # Clean and create bin directory
    log_info "Preparing build directory..."
    rm -rf "$BIN_DIR"
    mkdir -p "$BIN_DIR"
    
    # Build for current platform
    if ! build_native; then
        log_error "Native build failed"
        exit 1
    fi
    
    # Try cross-compilation for macOS
    build_macos_cross
    
    # Create universal binary if possible
    create_universal_binary
    
    # Generate checksums
    generate_checksums
    
    # Summary
    log ""
    log_success "Build complete!"
    log ""
    
    if [ -d "$BIN_DIR" ] && [ -n "$(ls -A "$BIN_DIR" 2>/dev/null)" ]; then
        log "Generated binaries:"
        ls -la "$BIN_DIR/"
        
        log ""
        log "For development/testing:"
        log "  ./target/release/$BINARY_NAME"
        log ""
        log "For distribution:"
        log "  ./bin/*"
        log ""
        log "For other platforms:"
        log "  Use GitHub Actions CI or Docker-based builds"
    else
        log_warn "No distribution binaries generated"
    fi
}

# Run main function
main "$@"