# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains a modern, cross-platform CLI tool designed to remove specific extended attributes (xattr) from files, particularly targeting Dropbox and Apple File Provider ignore attributes:

- **`dbx-ignore`** - Cross-platform Rust CLI with full git compatibility and comprehensive platform support

## Key Files

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Core library logic and public API
- `src/traits.rs` - Platform abstraction layer
- `src/platforms/` - Platform-specific implementations (macOS, Linux, Windows, unsupported)
- `tests/` - Comprehensive test suite with platform-specific testing
- `Makefile` - Build automation for development and distribution
- `install.sh` - Cross-platform installation script
- `BUILD.md` - Build and installation documentation
- `README.md` - Comprehensive user documentation

## Git Integration Behavior

### Implementation (`dbx-ignore`)
- **Full Compatibility**: Delegates all ignore logic to git using `git ls-files --ignored --exclude-standard -o`
- **Supports**: All git ignore features including negated patterns, complex globs, directory-specific rules
- **Efficiency**: Direct git command execution, no pattern parsing overhead
- **Maintenance**: Zero burden - automatically compatible with git evolution
- **Cross-platform**: Works consistently across macOS, Linux, and Windows

## Tool Usage Modes

The tool operates in two modes:

1. **Git mode** (default): 
   - Uses `git ls-files --ignored --exclude-standard -o` (fully compatible)
   - Must be run from within a Git repository
   - Automatically finds project root using `git rev-parse --show-toplevel`

2. **Specific files mode**: Processes only the files/directories provided as arguments
   - Usage: `dbx-ignore file1 file2 directory/`

## Architecture Notes

- **Platform Abstraction**: Uses trait-based design for cross-platform compatibility
- **Modular Structure**: Separate modules for each platform (macOS, Linux, Windows, unsupported)
- **Error Handling**: Comprehensive error handling with anyhow for detailed context
- **Performance**: Parallel processing with rayon for handling multiple files
- **Testing**: Extensive test suite with platform-specific test modules
- **Git Integration**: Direct delegation to git commands for maximum compatibility

## Development Commands

**Development Workflow:**
- `make build` - Build development binary (./target/release/)
- `make test` - Run comprehensive test suite
- `make build-dist` - Build distribution binaries (./bin/)
- `make build-docker` - Build Linux/Windows binaries via Docker
- `make clean` - Clean build artifacts

**Binary Organization:**
- Development: `./target/release/dbx-ignore` (for testing/development)
- Distribution: `./bin/dbx-ignore-*` (platform-specific for end users)

## Error Handling

The tool includes comprehensive error handling:
- **Platform Detection**: Automatically detects and handles unsupported platforms
- **Git Repository Validation**: Checks for git context when needed
- **File Existence**: Validates files before processing
- **Permission Handling**: Graceful handling of permission errors
- **Cross-platform**: Consistent behavior across all supported platforms