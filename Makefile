.PHONY: build clean install uninstall help build-all cross-install

BINARY_NAME := dbx-ignore
TARGET_DIR := target/release
BIN_DIR := bin
INSTALL_DIR := /usr/local/bin

# Cross-compilation targets
MACOS_INTEL_TARGET := x86_64-apple-darwin
MACOS_ARM64_TARGET := aarch64-apple-darwin
LINUX_TARGET := x86_64-unknown-linux-gnu
WINDOWS_TARGET := x86_64-pc-windows-gnu

# Default target
all: build

# Build the release binary for current platform (development use)
build:
	@echo "Building $(BINARY_NAME) for development..."
	cargo build --release
	@echo "Development binary: ./$(TARGET_DIR)/$(BINARY_NAME)"

# Build distribution binaries for end users
build-dist:
	@echo "🔨 Building distribution binaries..."
	@./scripts/build-simple.sh

# Build for all platforms (alias for build-dist)
build-all: build-dist

# Build for all platforms (advanced - requires proper OpenSSL setup)
build-all-advanced:
	@echo "🔨 Building $(BINARY_NAME) for all platforms..."
	@./scripts/build-all.sh

# Install cross-compilation targets
cross-install:
	@echo "📦 Installing cross-compilation targets..."
	rustup target add $(MACOS_INTEL_TARGET)
	rustup target add $(MACOS_ARM64_TARGET)
	rustup target add $(LINUX_TARGET)
	rustup target add $(WINDOWS_TARGET)
	@echo "✅ Cross-compilation targets installed"

# Clean build artifacts
clean:
	cargo clean
	@rm -rf $(BIN_DIR)
	@echo "Cleaned build artifacts and $(BIN_DIR) directory"

# Install binary to system PATH (from development build)
install: build
	@echo "Installing $(BINARY_NAME) to $(INSTALL_DIR)..."
	@cp $(TARGET_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/
	@echo "$(BINARY_NAME) installed to $(INSTALL_DIR)"

# Install distribution binary to system PATH  
install-dist: build-dist
	@echo "Installing distribution $(BINARY_NAME) to $(INSTALL_DIR)..."
	@cp $(BIN_DIR)/dbx-ignore-macos-universal $(INSTALL_DIR)/$(BINARY_NAME) 2>/dev/null || \
	 cp $(BIN_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(BINARY_NAME) installed to $(INSTALL_DIR)"

# Uninstall binary from system PATH
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(BINARY_NAME) uninstalled from $(INSTALL_DIR)"

# Build Linux/Windows binaries using Docker
build-docker:
	@echo "🐳 Building Linux/Windows binaries using Docker..."
	@./scripts/build-docker.sh

# Run tests
test:
	cargo test

help:
	@echo "Available targets:"
	@echo ""
	@echo "Development:"
	@echo "  build           - Build development binary (./target/release/)"
	@echo "  test            - Run test suite"
	@echo "  install         - Install development binary to /usr/local/bin"
	@echo ""
	@echo "Distribution:"
	@echo "  build-dist      - Build distribution binaries (./bin/)"
	@echo "  build-all       - Alias for build-dist"
	@echo "  install-dist    - Install distribution binary to /usr/local/bin"
	@echo ""
	@echo "Cross-platform:"
	@echo "  build-all-advanced - Build for all platforms (requires OpenSSL setup)"
	@echo "  build-docker    - Build Linux/Windows binaries using Docker"
	@echo "  cross-install   - Install cross-compilation targets"
	@echo ""
	@echo "Maintenance:"
	@echo "  clean           - Clean build artifacts"
	@echo "  uninstall       - Remove binary from /usr/local/bin"
	@echo "  help            - Show this help message"