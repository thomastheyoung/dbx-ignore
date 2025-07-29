# Installation Guide

This guide provides detailed installation instructions for `dbx-ignore` across all supported platforms.

## Table of Contents

- [Quick Install](#quick-install)
- [Pre-built Binaries](#pre-built-binaries)
- [Package Managers](#package-managers)
- [Building from Source](#building-from-source)
- [Platform-Specific Notes](#platform-specific-notes)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

## Quick Install

### macOS and Linux

The easiest way to install is using our install script:

```bash
curl -sSf https://raw.githubusercontent.com/thomastheyoung/dbx-ignore/main/install.sh | sh
```

This script automatically:

- Detects your platform and architecture
- Downloads the appropriate binary
- Installs to `/usr/local/bin` (may require sudo)
- Makes the binary executable

### Windows

For Windows, download the binary from [GitHub Releases](https://github.com/thomastheyoung/dbx-ignore/releases/latest) or use Cargo:

```bash
cargo install dbx-ignore
```

## Pre-built Binaries

### Download Links

Download the appropriate binary for your platform:

| Platform | Architecture            | Download                                                                                                                       |
| -------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| macOS    | Universal (Intel + ARM) | [dbx-ignore-macos-universal](https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-universal) |
| macOS    | Intel only              | [dbx-ignore-macos-intel](https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-intel)         |
| macOS    | Apple Silicon only      | [dbx-ignore-macos-arm64](https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-arm64)         |
| Linux    | x86_64                  | [dbx-ignore-linux-x64](https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-linux-x64)             |
| Windows  | x86_64                  | [dbx-ignore-windows-x64.exe](https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-windows-x64.exe) |

### Manual Installation Steps

#### macOS

```bash
# Universal binary (recommended - works on both Intel and Apple Silicon)
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-universal -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/

# Verify installation
dbx-ignore --version
```

#### Linux

```bash
# Download the binary
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-linux-x64 -o dbx-ignore

# Make executable and install
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/

# Verify installation
dbx-ignore --version
```

#### Windows

##### PowerShell Method

```powershell
# Download the executable
Invoke-WebRequest -Uri https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-windows-x64.exe -OutFile dbx-ignore.exe

# Move to a directory in your PATH, for example:
Move-Item dbx-ignore.exe "C:\Program Files\dbx-ignore\dbx-ignore.exe"

# Add to PATH if needed (requires admin privileges)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\dbx-ignore", [EnvironmentVariableTarget]::Machine)
```

##### Manual Method

1. Download `dbx-ignore-windows-x64.exe` from [GitHub Releases](https://github.com/thomastheyoung/dbx-ignore/releases/latest)
2. Rename to `dbx-ignore.exe` if desired
3. Move to a directory in your PATH (e.g., `C:\Windows\System32` or create `C:\Program Files\dbx-ignore`)
4. Or keep it in a convenient location and use the full path when running

## Package Managers

### Homebrew (macOS/Linux)

```bash
# Tap the repository (coming soon)
brew tap thomastheyoung/tap

# Install
brew install dbx-ignore
```

### Cargo (All Platforms)

If you have Rust installed:

```bash
cargo install dbx-ignore
```

This will:

- Download and compile the latest version
- Install to `~/.cargo/bin/`
- Automatically add to PATH (if Cargo is properly configured)

### Arch Linux (AUR)

```bash
# Using yay
yay -S dbx-ignore

# Using paru
paru -S dbx-ignore

# Manual installation
git clone https://aur.archlinux.org/dbx-ignore.git
cd dbx-ignore
makepkg -si
```

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- Git
- Platform-specific build tools:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: gcc or clang, pkg-config
  - **Windows**: Visual Studio Build Tools or MinGW

### Build Steps

#### Clone the Repository

```bash
git clone https://github.com/thomastheyoung/dbx-ignore.git
cd dbx-ignore
```

#### Development Build

For testing and development:

```bash
# Using make
make build

# Or using cargo directly
cargo build --release

# Binary location: ./target/release/dbx-ignore
```

#### Distribution Build

To build platform-specific optimized binaries:

```bash
# Builds all variants for your platform
make build-dist

# Binaries will be in ./bin/
# - dbx-ignore-macos-universal (macOS only)
# - dbx-ignore-linux-x64 (Linux only)
# - dbx-ignore-windows-x64.exe (Windows only)
```

#### Cross-Platform Build (Docker)

To build for multiple platforms from any host:

```bash
# Builds Linux and Windows binaries using Docker
make build-docker

# Requires Docker to be installed and running
```

#### Install from Source

```bash
# Install to Cargo's bin directory
cargo install --path .

# Or manually copy to system location
sudo cp target/release/dbx-ignore /usr/local/bin/
```

## Platform-Specific Notes

### macOS

- Universal binary supports both Intel and Apple Silicon Macs
- Requires macOS 10.15 (Catalina) or later
- May need to allow in System Preferences > Security & Privacy on first run
- Extended attributes require filesystem support (APFS, HFS+)

### Linux

- Requires glibc 2.17 or later (most distributions since 2012)
- Extended attributes require filesystem support (ext3, ext4, xfs, btrfs)
- Some filesystems may need to be mounted with `user_xattr` option

### Windows

- Requires Windows 10 or later
- Works only on NTFS filesystems (not FAT32/exFAT)
- May require running as Administrator for some system directories
- Windows Defender may scan on first run

## Troubleshooting

### Installation Issues

#### "Permission denied" during installation

```bash
# Use sudo for system-wide installation
sudo curl -sSf https://raw.githubusercontent.com/thomastheyoung/dbx-ignore/main/install.sh | sudo sh

# Or install to user directory
curl -sSf https://raw.githubusercontent.com/thomastheyoung/dbx-ignore/main/install.sh | sh -s -- --prefix=$HOME/.local
```

#### "Command not found" after installation

Check if the installation directory is in your PATH:

```bash
# Check current PATH
echo $PATH

# Add to PATH (bash)
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Add to PATH (zsh)
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Add to PATH (fish)
set -U fish_user_paths /usr/local/bin $fish_user_paths
```

#### "Cannot execute binary file"

This usually means you downloaded the wrong architecture:

```bash
# Check your system architecture
uname -m

# Download the correct version:
# x86_64 → use -x64 variants
# arm64/aarch64 → use -arm64 variants
# i386/i686 → not supported
```

### Runtime Issues

#### "Platform not supported"

The tool only supports:

- macOS (10.15+)
- Linux with extended attributes support
- Windows 10+ with NTFS

#### Extended attributes not working

**Linux:**

```bash
# Check if filesystem supports extended attributes
mount | grep -E "ext[34]|xfs|btrfs"

# Test extended attributes
touch test.txt
setfattr -n user.test -v "test" test.txt
getfattr test.txt
```

**macOS:**

```bash
# Test extended attributes
touch test.txt
xattr -w com.test "test" test.txt
xattr -l test.txt
```

### Build Issues

#### Cargo/Rust not found

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build fails with linking errors

**Linux:**

```bash
# Install development packages
sudo apt-get install build-essential pkg-config  # Debian/Ubuntu
sudo yum install gcc pkg-config                   # RHEL/CentOS
sudo pacman -S base-devel pkg-config             # Arch
```

**macOS:**

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

## Uninstallation

### Installed via Script or Manual Download

```bash
# Remove the binary
sudo rm /usr/local/bin/dbx-ignore

# Remove any state files (optional)
rm -rf .dbx-ignore/
```

### Installed via Cargo

```bash
cargo uninstall dbx-ignore
```

### Installed via Homebrew

```bash
brew uninstall dbx-ignore
```

### Windows

1. Delete the `dbx-ignore.exe` file from wherever you placed it
2. Remove from PATH if you added it there
3. Delete any `.dbx-ignore` folders in your projects (optional)

## Verification

After installation, verify everything is working:

```bash
# Check version
dbx-ignore --version

# Check help
dbx-ignore --help

# Test basic functionality (in a git repo)
mkdir test-dbx-ignore
cd test-dbx-ignore
git init
echo "test.log" > .gitignore
touch test.log
dbx-ignore --dry-run
```

## Getting Help

If you encounter issues not covered here:

1. Check the [GitHub Issues](https://github.com/thomastheyoung/dbx-ignore/issues)
2. Run with verbose mode: `dbx-ignore --verbose`
3. Check platform compatibility: `dbx-ignore --version`
4. Open a new issue with:
   - Your platform and version
   - The exact error message
   - Steps to reproduce
