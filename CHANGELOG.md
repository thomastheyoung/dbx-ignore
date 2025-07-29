# Changelog

All notable changes to dbx-ignore will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Created comprehensive API documentation (API.md)
- Created detailed installation guide (INSTALL.md)

### Changed

- Streamlined README.md with cleaner structure and focused content

## [0.4.0] - 2025-07-29

### Added

- Auto-process git-ignored files when run without arguments in a git repository
- Automatically add `.dbx-ignore/` to `.gitignore` to prevent committing metadata
- Always mark `.dbx-ignore` folder itself as Dropbox-ignored

### Changed

- Improved default behavior for better user experience
- Updated tests to handle new automatic behaviors

### Fixed

- Fixed test failures related to new automatic behaviors
- Updated test expectations for multiple file processing scenarios

## [0.3.0] - 2025-07-28

### Added

- Major architectural improvements and code quality enhancements
- Atomic JSON operations for better reliability
- Comprehensive test suite improvements

### Changed

- Refactored core functionality for better maintainability
- Enhanced error handling throughout the codebase

### Fixed

- Fixed directory handling in gitignore processing
- Improved wildcard support implementation

## [0.2.1] - 2025-07-28

### Fixed

- Removed unused dependencies
- Code cleanup and optimization

## [0.2.0] - 2025-07-25

### Added

- Enhanced watch mode with pattern-based monitoring
- Combined mark & watch operation with single command
- Ability to specify patterns directly with --watch flag
- Comprehensive documentation for pattern quoting

### Changed

- Watch mode now supports three distinct behaviors based on tracked state
- Improved user experience with clearer mode indicators

## [0.1.0] - 2025-07-14

### Added

- Initial release with core functionality
- Cross-platform support (macOS, Linux, Windows)
- Git integration for processing ignored files
- Basic ignore marker management (add/remove)
- Wildcard and pattern matching support
- Dry-run mode for safe operations
- Verbose and quiet output modes
- Status command to check directory state
