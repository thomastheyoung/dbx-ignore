# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains a modern, cross-platform CLI tool designed to prevent Dropbox from syncing unwanted files by adding ignore markers:

- **`dbx-ignore`** - Cross-platform Rust CLI that adds ignore markers to build artifacts, dependencies, and temporary files

## Key Files

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Core library logic and public API
- `src/traits.rs` - Platform abstraction layer
- `src/platforms/` - Platform-specific implementations (macOS, Linux, Windows, unsupported)
- `tests/` - Comprehensive test suite with platform-specific testing
- `.temp/` - Temporary files for development/testing (git ignored)
- `Makefile` - Build automation for development and distribution
- `install.sh` - Cross-platform installation script
- `BUILD.md` - Build and installation documentation
- `README.md` - Comprehensive user documentation

## Git Integration Behavior

### Implementation (`dbx-ignore`)

- **Purpose**: Prevents Dropbox sync by adding ignore markers to files
- **Git Integration**: Uses `git ls-files --ignored --exclude-standard -o` to find files to mark
- **Supports**: All git ignore features including negated patterns, complex globs, directory-specific rules
- **Efficiency**: Direct git command execution, no pattern parsing overhead
- **Cross-platform**: Adds extended attributes (macOS/Linux) or ADS (Windows)

## Tool Usage Modes

The tool operates in two modes:

1. **Git mode** (default):

   - Finds files ignored by git and marks them to prevent Dropbox sync
   - Must be run from within a Git repository
   - Automatically finds project root using `git rev-parse --show-toplevel`

2. **Specific files mode**: Marks only the files/directories provided as arguments
   - Usage: `dbx-ignore target/ node_modules/ dist/`

## Architecture Notes

- **Platform Abstraction**: Uses trait-based design for cross-platform ignore marker support
- **Modular Structure**: Separate modules for each platform (macOS, Linux, Windows, unsupported)
- **Ignore Logic**: Adds markers to files that don't already have them
- **Error Handling**: Comprehensive error handling with anyhow for detailed context
- **Performance**: Parallel processing with rayon for handling multiple files
- **Testing**: Extensive test suite with platform-specific test modules
- **Git Integration**: Direct delegation to git commands for file discovery

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

## Test File Organization

**IMPORTANT**: To maintain a clean repository and avoid accidental commits:

### Test File Locations:

- **Formal tests**: Place `.rs` test files in `./tests/` directory (part of test suite)
- **Temporary test files**: Place temporary files for development/debugging in `./.temp/` directory
- **Test data**: Use `TestEnvironment` from `tests/common/mod.rs` which creates proper temp directories

### .gitignore Patterns:

```
.temp/          # All temporary development files
test*.txt       # Temporary test text files
test*.md        # Temporary test markdown files
*_test_*        # Any files with _test_ in the name
```

### Usage:

```bash
# For temporary testing during development
echo "test content" > .temp/my_test_file.txt
./target/debug/dbx-ignore .temp/my_test_file.txt

# For formal test cases, use TestEnvironment in tests/
```

## Error Handling

The tool includes comprehensive error handling:

- **Platform Detection**: Automatically detects and handles unsupported platforms
- **Git Repository Validation**: Checks for git context when needed
- **File Existence**: Validates files before processing
- **Permission Handling**: Graceful handling of permission errors
- **Cross-platform**: Consistent behavior across all supported platforms
