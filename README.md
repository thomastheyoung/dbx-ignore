# dbx-ignore

Prevent Dropbox from syncing unwanted files in your development projects.

## What it does

`dbx-ignore` adds platform-specific markers to files that tell Dropbox to skip syncing them. Perfect for keeping build artifacts, dependencies, and temporary files local-only.

### Key Features

- 🚀 **Smart defaults** - Run without arguments in a git repo to automatically process all git-ignored files
- 📁 **Batch operations** - Mark multiple files/directories with wildcards (`*.log`, `node_modules/`)
- 👁️ **Watch mode** - Daemon that continuously monitors and maintains ignore markers
- 🔄 **Cross-platform** - Works on macOS, Linux, and Windows
- 🛡️ **Safe** - Built-in protections against marking too many files accidentally

## Quick Start

```bash
# In a git repository - automatically marks all git-ignored files
dbx-ignore

# Mark specific files or directories
dbx-ignore node_modules/ target/ *.log

# Start watching for new files matching patterns
dbx-ignore --watch "*.log" "*.tmp"

# Check what's marked
dbx-ignore --status
```

## Installation

### macOS/Linux

```bash
# Quick install with script
curl -sSf https://raw.githubusercontent.com/thomastheyoung/dbx-ignore/main/install.sh | sh

# Or with Cargo
cargo install dbx-ignore
```

### Windows

Download from [GitHub Releases](https://github.com/thomastheyoung/dbx-ignore/releases/latest) or:

```bash
cargo install dbx-ignore
```

For more installation options, see [INSTALL.md](INSTALL.md). For complete API documentation, see [API.md](API.md).

## Usage

### Basic Commands

```bash
# Process all git-ignored files (default in git repos)
dbx-ignore

# Mark specific files/directories
dbx-ignore file.txt dir/ *.log

# Remove markers
dbx-ignore --reset file.txt

# Start watch daemon
dbx-ignore --watch

# Check status
dbx-ignore --status
```

### Working with Patterns

```bash
# Wildcards
dbx-ignore "*.log"              # All .log files
dbx-ignore "**/*.tmp"           # All .tmp files recursively
dbx-ignore "test[0-9].txt"      # test0.txt through test9.txt

# Watch for future files
dbx-ignore --watch "*.log"      # Monitors and marks new .log files

# Process current directory
dbx-ignore .                    # All non-hidden files in current dir
```

### Options

- `-r, --reset` - Remove ignore markers
- `-w, --watch` - Start daemon to monitor files
- `-u, --unwatch` - Stop daemon
- `-s, --status` - Show status
- `-n, --dry-run` - Preview changes
- `-v, --verbose` - Detailed output
- `-q, --quiet` - Suppress output

## How It Works

Dropbox respects platform-specific markers that indicate files should not be synced:

- **macOS**: Extended attributes (`com.dropbox.ignored`)
- **Linux**: Extended attributes (`user.com.dropbox.ignored`) 
- **Windows**: Alternate Data Streams

The tool automatically:
1. Detects your platform
2. Finds files to mark (via git integration or your specifications)
3. Adds the appropriate markers
4. Optionally monitors for new files (watch mode)

## Common Use Cases

### Development Projects

```bash
# In your project root
cd my-project
dbx-ignore    # Marks all git-ignored files

# This typically includes:
# - node_modules/
# - target/, dist/, build/
# - *.log, *.tmp
# - .env files
```

### Watch Mode

```bash
# Method 1: Mark and watch in one command
dbx-ignore --watch "*.log" "build/**"

# Method 2: Mark patterns, then watch
dbx-ignore "*.log" "build/**"
dbx-ignore --watch

# Stop watching
dbx-ignore --unwatch
```

### Specific Files

```bash
# Large datasets
dbx-ignore data/*.csv

# Build outputs
dbx-ignore dist/ build/ target/

# Temporary files
dbx-ignore "*.tmp" "*.cache" "*.log"
```

## Advanced Features

### State Management

The tool maintains state in `.dbx-ignore/`:
- `tracked_files.json` - Files you've explicitly marked
- Automatically added to `.gitignore` to prevent committing

### Safety Features

- Prevents marking entire directory without proper git setup
- Validates file existence before processing
- Handles permission errors gracefully

### Exit Codes

- `0` - Success
- `1` - Error (invalid arguments, missing files, etc.)
- `101` - Platform not supported

## Contributing

Issues and pull requests welcome at [GitHub](https://github.com/thomastheyoung/dbx-ignore).

## License

MIT - see [LICENSE](LICENSE) file