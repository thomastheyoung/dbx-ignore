#!/bin/bash
#
# Docker-based cross-compilation script for dbx-ignore
# Builds Linux and Windows binaries using Docker
#
# Requirements:
# - Docker installed and running
# - Dockerfile.cross in project root

set -euo pipefail

# Configuration
readonly BINARY_NAME="dbx-ignore"
readonly BIN_DIR="bin"
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DOCKER_IMAGE="dbx-ignore-cross"
readonly DOCKERFILE="Dockerfile.cross"

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

# Check Docker availability
check_docker() {
    if ! command_exists docker; then
        log_error "Docker not found. Please install Docker to use this build method."
        log_info "Install Docker from: https://docs.docker.com/get-docker/"
        return 1
    fi
    
    # Check if Docker daemon is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker daemon is not running. Please start Docker."
        return 1
    fi
    
    # Check if we have permission to use Docker
    if ! docker ps >/dev/null 2>&1; then
        log_error "Permission denied accessing Docker. You may need to:"
        log_info "  - Add your user to the docker group: sudo usermod -aG docker \$USER"
        log_info "  - Log out and back in for changes to take effect"
        return 1
    fi
    
    return 0
}

# Check if Dockerfile exists
check_dockerfile() {
    if [ ! -f "$DOCKERFILE" ]; then
        log_error "Dockerfile not found: $DOCKERFILE"
        log_info "Creating a basic Dockerfile.cross..."
        create_dockerfile
    fi
}

# Create a basic cross-compilation Dockerfile
create_dockerfile() {
    cat > "$DOCKERFILE" <<'EOF'
# Multi-stage Dockerfile for cross-compiling dbx-ignore
FROM rust:latest as builder

# Install cross-compilation dependencies
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    gcc-multilib \
    && rm -rf /var/lib/apt/lists/*

# Add Windows target
RUN rustup target add x86_64-pc-windows-gnu

# Set up working directory
WORKDIR /build

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src/

# Build for Linux (native)
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Build for Windows
RUN cargo build --release --target x86_64-pc-windows-gnu

# Copy binaries to output stage
FROM alpine:latest
RUN apk add --no-cache ca-certificates

# Copy built binaries
COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release/dbx-ignore /output/dbx-ignore-linux-x64
COPY --from=builder /build/target/x86_64-pc-windows-gnu/release/dbx-ignore.exe /output/dbx-ignore-windows-x64.exe

# Set up volume for output
VOLUME /host

# Copy binaries to host on run
CMD cp -r /output/* /host/
EOF
    log_success "Created $DOCKERFILE"
}

# Build Docker image
build_docker_image() {
    log_info "Building Docker image: $DOCKER_IMAGE"
    
    # Build with BuildKit for better performance
    if ! DOCKER_BUILDKIT=1 docker build \
        --progress=plain \
        -f "$DOCKERFILE" \
        -t "$DOCKER_IMAGE" \
        . 2>&1; then
        log_error "Failed to build Docker image"
        return 1
    fi
    
    log_success "Docker image built successfully"
    return 0
}

# Extract binaries from Docker container
extract_binaries() {
    log_info "Extracting binaries from Docker container..."
    
    # Ensure bin directory exists
    mkdir -p "$BIN_DIR"
    
    # Run container and copy binaries
    if ! docker run --rm \
        -v "$(pwd)/$BIN_DIR:/host" \
        "$DOCKER_IMAGE" 2>&1; then
        log_error "Failed to extract binaries from container"
        return 1
    fi
    
    # Check if binaries were extracted
    local linux_binary="$BIN_DIR/dbx-ignore-linux-x64"
    local windows_binary="$BIN_DIR/dbx-ignore-windows-x64.exe"
    
    if [ ! -f "$linux_binary" ] && [ ! -f "$windows_binary" ]; then
        log_error "No binaries were extracted"
        return 1
    fi
    
    # Make Linux binary executable
    if [ -f "$linux_binary" ]; then
        chmod +x "$linux_binary"
        log_success "Extracted: dbx-ignore-linux-x64"
    fi
    
    if [ -f "$windows_binary" ]; then
        log_success "Extracted: dbx-ignore-windows-x64.exe"
    fi
    
    return 0
}

# Generate checksums
generate_checksums() {
    log_info "Generating checksums..."
    
    cd "$BIN_DIR"
    
    # Generate checksums for Linux and Windows binaries only
    for file in dbx-ignore-linux-x64 dbx-ignore-windows-x64.exe; do
        if [ -f "$file" ]; then
            if command_exists sha256sum; then
                sha256sum "$file" > "${file}.sha256"
            elif command_exists shasum; then
                shasum -a 256 "$file" > "${file}.sha256"
            fi
        fi
    done
    
    cd - >/dev/null
    log_success "Generated checksums"
}

# Clean up old images
cleanup_old_images() {
    log_info "Cleaning up old Docker images..."
    
    # Remove dangling images
    docker image prune -f >/dev/null 2>&1 || true
    
    # Remove old versions of our image (keep only latest)
    docker images "$DOCKER_IMAGE" --format "{{.ID}} {{.Tag}}" | \
        grep -v latest | \
        awk '{print $1}' | \
        xargs -r docker rmi -f >/dev/null 2>&1 || true
}

# Main function
main() {
    log "Docker-based cross-compilation for $BINARY_NAME"
    log ""
    
    # Check Docker availability
    if ! check_docker; then
        exit 1
    fi
    
    # Check Dockerfile
    check_dockerfile
    
    # Build Docker image
    if ! build_docker_image; then
        exit 1
    fi
    
    # Extract binaries
    if ! extract_binaries; then
        exit 1
    fi
    
    # Generate checksums
    generate_checksums
    
    # Clean up old images
    cleanup_old_images
    
    # Summary
    log ""
    log_success "Docker build complete!"
    log ""
    log "Generated binaries:"
    ls -la "$BIN_DIR/" | grep -E "(linux|windows)" || log_warn "No binaries found"
    
    log ""
    log "Next steps:"
    log "  - Test the Linux binary: ./$BIN_DIR/dbx-ignore-linux-x64 --version"
    log "  - Upload binaries to GitHub releases"
    log "  - Include checksums for verification"
}

# Run main function
main "$@"