# Cross-Platform Build Guide

This document explains how to build `dbx-ignore` binaries for different platforms.

## 📁 Binary Organization

This project separates development and distribution binaries:

- **Development**: `./target/release/dbx-ignore` 
  - Built with `make build` or `cargo build --release`
  - Used for testing, development, and local CLI tests
  - Single binary for current platform only

- **Distribution**: `./bin/dbx-ignore-*`
  - Built with `make build-dist`
  - Platform-specific binaries for end users
  - Ready for installation and distribution

## 🚀 Quick Start

### Development Build
```bash
make build
# Development binary: ./target/release/dbx-ignore
```

### Distribution Binaries
```bash
make build-dist
# Distribution binaries: ./bin/dbx-ignore-*
```

### Cross-Platform Distribution
```bash
make build-dist
# Platform-specific binaries: ./bin/dbx-ignore-*
```

### All Platforms via Docker
```bash
make build-docker
# Requires Docker installed
```

### All Platforms via GitHub Actions
Push a tag to trigger automated builds:
```bash
git tag v1.0.0
git push origin v1.0.0
```

## 📋 Build Methods

### Method 1: Local Cross-Compilation (Recommended)

**Advantages:**
- Fast builds
- Works well for macOS Intel ↔ Apple Silicon
- No external dependencies

**Setup:**
```bash
make cross-install  # Install cross-compilation targets
make build-dist     # Build distribution binaries for compatible platforms
```

**Generated Files:**
```
bin/
├── dbx-ignore-macos-intel       # macOS x86_64
├── dbx-ignore-macos-arm64       # macOS Apple Silicon
└── dbx-ignore-macos-universal   # Universal binary
```

### Method 2: Docker Cross-Compilation

**Advantages:**
- Builds Linux and Windows binaries on macOS
- Consistent build environment
- No host system contamination

**Requirements:**
- Docker installed and running

**Usage:**
```bash
make build-docker
```

**Generated Files:**
```
bin/
├── dbx-ignore-linux-x64         # Linux x86_64
└── dbx-ignore-windows-x64.exe   # Windows x86_64
```

### Method 3: GitHub Actions (Production)

**Advantages:**
- Builds on native platforms
- Automatic releases
- Matrix builds for all targets
- Creates universal macOS binaries

**Triggers:**
- Push tags matching `v*` (e.g., `v1.0.0`)
- Manual workflow dispatch

**Generated Artifacts:**
- `dbx-ignore-macos-universal` (Universal macOS)
- `dbx-ignore` (Linux x86_64)
- `dbx-ignore.exe` (Windows x86_64)

## 🎯 Platform Support Matrix

| Platform | Architecture | Local Build | Docker Build | GitHub Actions |
|----------|-------------|-------------|--------------|----------------|
| macOS | Intel (x86_64) | ✅ | ❌ | ✅ |
| macOS | Apple Silicon (ARM64) | ✅ | ❌ | ✅ |
| macOS | Universal | ✅ | ❌ | ✅ |
| Linux | x86_64 | ⚠️* | ✅ | ✅ |
| Windows | x86_64 | ⚠️* | ✅ | ✅ |

*\* Requires complex OpenSSL cross-compilation setup*

## 🔧 Troubleshooting

### OpenSSL Cross-Compilation Issues

**Problem:** `Could not find directory of OpenSSL installation`

**Solutions:**
1. Use Docker method: `make build-docker`
2. Use GitHub Actions for full cross-platform builds
3. Install platform-specific OpenSSL development packages

### Missing Cross-Compilation Targets

**Problem:** `error: toolchain 'stable-x86_64-apple-darwin' is not installed`

**Solution:**
```bash
make cross-install
```

### Docker Not Available

**Problem:** `Docker not found`

**Solution:**
Install Docker Desktop or use GitHub Actions for Linux/Windows builds.

## 📦 Distribution

### For End Users
- **macOS**: Use `dbx-ignore-macos-universal` (works on both Intel and Apple Silicon)
- **Linux**: Use `dbx-ignore-linux-x64`
- **Windows**: Use `dbx-ignore-windows-x64.exe`

### For Developers
- Use `make build` for development builds (creates `./target/release/dbx-ignore`)
- Use `make test` to run the test suite (tests use development binary)
- Use `make build-dist` for creating distribution binaries (creates `./bin/dbx-ignore-*`)

## 🎛️ Available Make Targets

```bash
make help  # Show all available targets
```

| Target | Description |
|--------|-------------|
| `build` | Build development binary (./target/release/) |
| `build-dist` | Build distribution binaries (./bin/) |
| `build-all` | Alias for build-dist |
| `build-docker` | Build Linux/Windows via Docker |
| `cross-install` | Install cross-compilation targets |
| `test` | Run test suite (uses development binary) |
| `clean` | Clean build artifacts |
| `install` | Install development binary to system PATH |
| `install-dist` | Install distribution binary to system PATH |
| `uninstall` | Remove from system PATH |

## 🚀 Automated Releases

The project uses GitHub Actions to automatically:

1. **Build** binaries for all platforms
2. **Test** on each platform
3. **Create** universal macOS binaries
4. **Upload** release assets
5. **Generate** release notes

To create a release:
```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers the full build pipeline and creates a GitHub release with downloadable binaries.

## 📦 Installation for End Users

Once binaries are built and released, users can install them using several methods:

### Quick Install Script (Recommended)
```bash
curl -sSf https://raw.githubusercontent.com/user/dbx-ignore/main/install.sh | sh
```

The install script automatically:
- Detects platform (macOS Intel/ARM, Linux x86_64)
- Downloads the correct binary from GitHub releases
- Installs to `/usr/local/bin/dbx-ignore`
- Verifies the installation

### Manual Download
Users can download platform-specific binaries from GitHub releases:

**macOS Universal (Intel + Apple Silicon)**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-macos-universal -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Linux x86_64**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-linux-x64 -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Windows x86_64**
```powershell
Invoke-WebRequest -Uri https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-windows-x64.exe -OutFile dbx-ignore.exe
```

### Package Managers
Future package manager support:
- Homebrew: `brew install user/tap/dbx-ignore`
- Chocolatey: `choco install dbx-ignore`
- Cargo: `cargo install dbx-ignore`