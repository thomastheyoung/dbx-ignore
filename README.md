# DBX-Ignore

> CLI tool to prevent Dropbox from syncing unwanted files by adding ignore markers

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

A modern CLI tool for preventing Dropbox from syncing unwanted files by adding ignore markers to build artifacts, temporary files, and development folders.

**Common Use Cases:**
- 🗏️ **Ignore build artifacts** like target/, dist/, build/ folders
- 📁 **Skip large dependencies** like node_modules/, vendor/ directories
- ♾️ **Exclude temporary files** that shouldn't be synced
- 🔄 **Bulk ignore** files matching your .gitignore patterns

**Supported Platforms:**
- ✅ **macOS** (Intel & Apple Silicon) - Full functionality
- ✅ **Linux** (x86_64) - Full functionality  
- ✅ **Windows** (x86_64) - Graceful handling

## 🚀 Features

- ✨ **Beautiful progress bars** with colored output
- 🔍 **Dry-run mode** to preview what will be fixed
- 📊 **Verbose/quiet modes** for different output levels
- 🎯 **Smart detection** - only processes files with ignore markers
- 🔧 **Full Git integration** - automatically finds ignored files using git
- 🌍 **Cross-platform** - works consistently across operating systems
- ⚡ **High performance** with parallel processing
- 🛡️ **Safe operation** with comprehensive error handling
- 🧪 **Thoroughly tested** with platform-specific test coverage
- 📦 **Easy installation** with one-line installer script

## 📦 Installation

### Quick Install (Recommended)

#### One-line installer (macOS/Linux)
```bash
curl -sSf https://raw.githubusercontent.com/user/dbx-ignore/main/install.sh | sh
```

This automatically:
- Detects your platform (macOS Intel/ARM, Linux)
- Downloads the appropriate binary
- Installs to `/usr/local/bin/dbx-ignore`
- Makes it executable

### Manual Installation

#### Option 1: Download Pre-built Binaries

**macOS (Universal - works on Intel & Apple Silicon)**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-macos-universal -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**macOS (Intel only)**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-macos-intel -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**macOS (Apple Silicon only)**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-macos-arm64 -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Linux (x86_64)**
```bash
curl -L https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-linux-x64 -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Windows (x86_64)**
```powershell
# PowerShell
Invoke-WebRequest -Uri https://github.com/user/dbx-ignore/releases/latest/download/dbx-ignore-windows-x64.exe -OutFile dbx-ignore.exe
# Move to a directory in your PATH
```

#### Option 2: Package Managers

**Homebrew (macOS/Linux)**
```bash
# Coming soon
brew install user/tap/dbx-ignore
```

**Cargo (All platforms)**
```bash
cargo install dbx-ignore
```

#### Option 3: Build from Source

**Prerequisites**
- [Rust](https://rustup.rs/) 1.70 or later

**Development build**
```bash
git clone https://github.com/user/dbx-ignore.git
cd dbx-ignore
make build
# Binary: ./target/release/dbx-ignore
```

**Distribution build**
```bash
git clone https://github.com/user/dbx-ignore.git
cd dbx-ignore
make build-dist
# Platform-specific binaries: ./bin/dbx-ignore-*
```

**Install from source**
```bash
cargo install --path .
```

### Verify Installation

```bash
dbx-ignore --version
dbx-ignore --help
```

### Installation Troubleshooting

**"Permission denied" during installation**
```bash
# The installer needs sudo access to write to /usr/local/bin
# This is normal and expected
sudo ./install.sh  # Alternative: run with sudo if prompted
```

**"Command not found" after installation**
```bash
# Check if /usr/local/bin is in your PATH
echo $PATH | grep -q /usr/local/bin && echo "✓ PATH is correct" || echo "✗ PATH issue"

# Add to PATH if needed (add to ~/.bashrc or ~/.zshrc)
export PATH="/usr/local/bin:$PATH"
```

**"Binary not found" or download fails**
- Check if the GitHub release exists at the expected URL
- Verify internet connectivity
- Try manual download from [GitHub Releases](https://github.com/user/dbx-ignore/releases)

**Windows installation**
- Use PowerShell method from manual installation section
- Or download directly from GitHub releases
- Install script currently supports macOS/Linux only

## 🎯 Usage

### Command Line Interface
```
Prevent Dropbox from syncing files by adding ignore markers

Usage: dbx-ignore [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Files or directories to mark as ignored by Dropbox

Options:
  -n, --dry-run  Preview what files would be marked to ignore
  -v, --verbose  Show detailed progress of adding ignore markers
  -q, --quiet    Run silently (for scripts)
  -g, --git      Mark git-ignored files to prevent Dropbox sync (default if no files specified)
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

**Ignore all git-ignored files (default mode):**
```bash
dbx-ignore
# or explicitly:
dbx-ignore --git
```

**Mark specific files/folders to ignore:**
```bash
dbx-ignore target/ node_modules/
dbx-ignore build-artifacts/ .env.local
```

**Preview what will be ignored:**
```bash
dbx-ignore --dry-run --verbose
```

**Detailed progress output:**
```bash
dbx-ignore --verbose target/ dist/
```

**Silent mode for automation:**
```bash
dbx-ignore --quiet --git
```

**Preview specific files:**
```bash
dbx-ignore --dry-run --verbose target/ node_modules/
```

### Sample Output

**Normal mode with progress:**
```
✓ Platform: macOS
✓ Mode: Adding ignore markers to git-ignored files
⠁ [00:00:01] [████████████████████████████████████████] 1472/1472 Complete!
──────────────────────────────────────────────────
✓ 1472 files processed, 245 files marked to ignore
```

**Verbose mode:**
```
✓ Platform: macOS
✓ Mode: Adding ignore markers to specified files
   ✓ target/release/app: 2 ignore markers added
   - node_modules/: already ignored
   ✘ .env.local: Permission denied (os error 13)
──────────────────────────────────────────────────
✓ 2 files processed, 1 file marked to ignore
```

**Dry run mode:**
```
🔍 Dry run mode - previewing ignore markers
✓ Platform: macOS
✓ Mode: Checking specified files
   ✓ target/release/app: would add ignore markers (not currently ignored)
──────────────────────────────────────────────────
🔍 1 file would be marked to ignore
```

## 🔧 How It Works

### Sync Prevention Mechanism

Dropbox and Apple File Provider respect hidden markers that tell them to skip files:

| Marker | Purpose | Platform |
|--------|---------|----------|
| Extended attributes | Mark files as "do not sync" | macOS, Linux |
| Alternate Data Streams | Windows equivalent | Windows |

**The Goal**: Add these markers to files you don't want cluttering your Dropbox:
- Build artifacts (target/, dist/, build/)
- Large dependencies (node_modules/, vendor/) 
- Temporary files (.tmp, .cache, logs)
- Development-only files that shouldn't be shared

### Platform Support

| Platform | Architecture | Functionality | Details |
|----------|-------------|---------------|---------|
| **macOS** | Intel (x86_64) | ✅ Full ignore support | Adds extended attributes |
| **macOS** | Apple Silicon (ARM64) | ✅ Full ignore support | Adds extended attributes |
| **Linux** | x86_64 | ✅ Full ignore support | Adds extended attributes |
| **Windows** | x86_64 | ✅ Alternate Data Streams | Adds ADS markers |
| **Others** | Various | ⚠️ Unsupported | Reports platform limitation |

### Git Integration

The tool automatically discovers and processes files that are ignored by git using:
```bash
git ls-files --ignored --exclude-standard -o
```

This includes files matching patterns in:
- `.gitignore`
- `.git/info/exclude`  
- Global git exclude file
- Files explicitly added to git but later ignored

#### Full Git Compatibility

**Implementation Approach:**
The tool delegates all ignore logic to git itself using `git ls-files --ignored --exclude-standard -o`, providing:

**Advantages:**
- ⚡ **High Performance**: Direct git command execution
- 🔧 **Maintenance-Free**: Automatically compatible with git evolution
- 🎯 **100% Accurate**: Git determines what files are ignored
- 📦 **Complete Coverage**: All ignore sources included
- ✅ **Full Feature Support**: Negated patterns, complex globs, directory-specific rules

**Why Use Git Integration:**
Files in `.gitignore` are usually things you don't want to sync to Dropbox either:
- ✅ Automatically finds build artifacts, dependencies, temp files
- ✅ Respects your existing ignore patterns and project structure
- ✅ Supports complex patterns (negation, globs, directory-specific rules)
- ✅ No need to manually specify every file type to ignore

## 🔍 Common Use Cases

### Development Workflow
```bash
# Ignore all build artifacts and dependencies
dbx-ignore --git --quiet

# Preview what would be ignored before running
dbx-ignore --dry-run --verbose
```

### Project Setup
```bash
# Set up a new project to ignore common files
dbx-ignore target/ node_modules/ dist/ .env.local

# Ignore everything in .gitignore
dbx-ignore
```

### Automation
```bash
# Add to your build scripts
dbx-ignore --quiet
exit_code=$?
if [ $exit_code -ne 0 ]; then
    echo "Failed to mark files as ignored: $exit_code"
    exit 1
fi
```

## 🛠️ Development

### Building
```bash
# Development build (for testing)
make build

# Distribution build (for release)
make build-dist

# All platforms (Docker + local)
make build-docker

# Run tests
make test

# Clean build artifacts
make clean
```

### Project Structure
```
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Core library logic
│   ├── traits.rs        # Platform abstraction
│   └── platforms/       # Platform-specific implementations
│       ├── macos.rs     # macOS xattr handling
│       ├── linux.rs     # Linux xattr handling
│       ├── windows.rs   # Windows no-op handler
│       └── unsupported.rs # Fallback handler
├── tests/               # Comprehensive test suite
├── scripts/             # Build scripts
├── Makefile            # Build automation
├── install.sh          # Installation script
├── BUILD.md            # Build documentation
└── README.md           # This file
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🐛 Troubleshooting

### Common Issues

**"Permission denied" errors:**
```bash
# Run with elevated permissions if needed
sudo dbx-ignore /protected/path/
```

**"Not in a git repository" error:**
```bash
# Initialize git repo or specify files directly
git init
# or
dbx-ignore target/ node_modules/ dist/
```

### Getting Help

- Use `--help` flag for command line options
- Use `--verbose` flag to see detailed processing information
- Use `--dry-run` flag to preview changes safely

## 🙏 Acknowledgments

- Built with [clap](https://crates.io/crates/clap) for CLI parsing
- Uses [indicatif](https://crates.io/crates/indicatif) for progress bars
- Powered by [git2](https://crates.io/crates/git2) for git integration
- Extended attributes handled via [xattr](https://crates.io/crates/xattr) crate