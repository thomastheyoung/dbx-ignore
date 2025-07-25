# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-12-19

### Added

- Shell script optimization for xattr removal to prevent redundant operations
- Comprehensive test suite for validation of optimization logic
- Shellcheck compliance for improved code quality
- Version tracking in shell script and Rust implementation

### Fixed

- Fixed shellcheck warnings in both main script and test files
- Improved variable quoting for better shell safety
- Removed unused functions that caused unreachable code warnings

### Migration Notes

- **Breaking Change**: This release includes optimizations that change the execution pattern of xattr removal commands
- Scripts that depend on exact xattr call counts may need adjustment
- The optimization ensures that directories are processed only once, potentially reducing the number of operations
- Users upgrading should verify that their workflow still functions as expected after this optimization
- If you experience any issues with the optimized behavior, you can fall back to explicit file arguments mode

### Technical Details

- Improved deduplication logic for directory processing
- Enhanced pattern matching efficiency
- Better error handling for edge cases
- All tests passing with CI validation

## [0.1.0] - Initial Release

- Basic xattr removal functionality
- Git integration for .gitignore pattern processing
- Cross-platform support for macOS extended attributes
