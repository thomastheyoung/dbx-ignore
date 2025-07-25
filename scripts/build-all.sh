#!/bin/bash
#
# Cross-platform build script for dbx-ignore
# Creates binaries for macOS, Linux, and Windows in ./bin/
#
# Requirements:
# - Rust toolchain with cross-compilation targets
# - macOS: Xcode command line tools
# - Linux/Windows: cross-compilation toolchain or Docker

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

# Check prerequisites
check_prerequisites() {
    local missing=()
    
    if ! command_exists cargo; then
        missing+=("cargo (Rust toolchain)")
    fi
    
    if ! command_exists rustup; then
        missing+=("rustup")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi
}

# Install cross-compilation target
install_target() {
    local target="$1"
    local name="$2"
    
    if rustup target list --installed | grep -q "^$target$"; then
        log_info "$name target already installed"
    else
        log_info "Installing $name target: $target"
        if ! rustup target add "$target"; then
            log_warn "Failed to install $target"
            return 1
        fi
    fi
    return 0
}

# Build for target
build_target() {
    local target="$1"
    local output_name="$2"
    local desc="$3"
    
    log_info "Building for $desc..."
    
    if ! cargo build --release --target "$target" 2>&1; then
        log_error "Failed to build for $target"
        log_warn "This might be due to missing cross-compilation tools"
        return 1
    fi
    
    # Determine source path (Windows binaries have .exe extension)
    local source_path="target/$target/release/$BINARY_NAME"
    if [[ "$target" == *"windows"* ]]; then
        source_path="${source_path}.exe"
        output_name="${output_name}.exe"
    fi
    
    # Copy to bin directory
    if [ -f "$source_path" ]; then
        cp "$source_path" "$BIN_DIR/$output_name"
        log_success "Built: $output_name"
        return 0
    else
        log_error "Binary not found at: $source_path"
        return 1
    fi
}

# Generate checksums
generate_checksums() {
    log_info "Generating checksums..."
    
    cd "$BIN_DIR"
    
    # Remove old checksum files
    rm -f *.sha256
    
    # Generate individual checksum files
    for file in *; do
        if [ -f "$file" ] && [[ ! "$file" == *.sha256 ]]; then
            if command_exists sha256sum; then
                sha256sum "$file" > "${file}.sha256"
            elif command_exists shasum; then
                shasum -a 256 "$file" > "${file}.sha256"
            else
                log_warn "No SHA256 tool available, skipping checksums"
                break
            fi
        fi
    done
    
    # Create combined checksum file
    if command_exists sha256sum || command_exists shasum; then
        rm -f SHA256SUMS
        for file in *.sha256; do
            if [ -f "$file" ]; then
                cat "$file" >> SHA256SUMS
            fi
        done
        if [ -f SHA256SUMS ]; then
            log_success "Generated checksums"
        fi
    fi
    
    cd - >/dev/null
}

# Main build function
main() {
    log "Cross-platform build script for $BINARY_NAME"
    log ""
    
    # Check prerequisites
    check_prerequisites
    
    # Clean and create bin directory
    log_info "Preparing build directory..."
    rm -rf "$BIN_DIR"
    mkdir -p "$BIN_DIR"
    
    # Track successful builds
    local builds_completed=0
    local builds_failed=0
    
    # Define build targets
    declare -A targets=(
        ["x86_64-apple-darwin"]="dbx-ignore-macos-intel|macOS Intel"
        ["aarch64-apple-darwin"]="dbx-ignore-macos-arm64|macOS Apple Silicon"
        ["x86_64-unknown-linux-gnu"]="dbx-ignore-linux-x64|Linux x64"
        ["x86_64-pc-windows-gnu"]="dbx-ignore-windows-x64|Windows x64"
    )
    
    # Install targets and build
    log ""
    log_info "Installing compilation targets..."
    
    for target in "${!targets[@]}"; do
        IFS='|' read -r output_name desc <<< "${targets[$target]}"
        
        if install_target "$target" "$desc"; then
            if build_target "$target" "$output_name" "$desc"; then
                ((builds_completed++))
            else
                ((builds_failed++))
            fi
        else
            ((builds_failed++))
        fi
        
        log ""
    done
    
    # Create universal macOS binary if both architectures built successfully
    if [ -f "$BIN_DIR/dbx-ignore-macos-intel" ] && [ -f "$BIN_DIR/dbx-ignore-macos-arm64" ]; then
        log_info "Creating universal macOS binary..."
        if command_exists lipo; then
            if lipo -create \
                "$BIN_DIR/dbx-ignore-macos-intel" \
                "$BIN_DIR/dbx-ignore-macos-arm64" \
                -output "$BIN_DIR/dbx-ignore-macos-universal"; then
                log_success "Created universal macOS binary"
                ((builds_completed++))
            else
                log_error "Failed to create universal binary"
                ((builds_failed++))
            fi
        else
            log_warn "lipo not found, skipping universal binary creation"
        fi
    fi
    
    # Generate checksums
    if [ $builds_completed -gt 0 ]; then
        generate_checksums
    fi
    
    # Summary
    log ""
    log "Build summary:"
    log_success "Completed: $builds_completed builds"
    if [ $builds_failed -gt 0 ]; then
        log_warn "Failed: $builds_failed builds"
    fi
    
    log ""
    log "Generated binaries:"
    ls -la "$BIN_DIR/" 2>/dev/null || log_warn "No binaries generated"
    
    # Exit with error if no builds succeeded
    if [ $builds_completed -eq 0 ]; then
        log_error "No builds completed successfully"
        exit 1
    fi
    
    log ""
    log_success "Build complete!"
    
    # Show distribution recommendations
    if [ -f "$BIN_DIR/dbx-ignore-macos-universal" ]; then
        log ""
        log "Recommended binaries for distribution:"
        log "  - $BIN_DIR/dbx-ignore-macos-universal (macOS - works on both Intel and Apple Silicon)"
        [ -f "$BIN_DIR/dbx-ignore-linux-x64" ] && log "  - $BIN_DIR/dbx-ignore-linux-x64 (Linux)"
        [ -f "$BIN_DIR/dbx-ignore-windows-x64.exe" ] && log "  - $BIN_DIR/dbx-ignore-windows-x64.exe (Windows)"
    fi
}

# Run main function
main "$@"