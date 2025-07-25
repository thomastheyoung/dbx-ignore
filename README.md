# dbx-ignore

CLI tool to prevent Dropbox from syncing unwanted files by adding platform-specific ignore markers.

## Overview

`dbx-ignore` adds extended attributes (macOS/Linux) or alternate data streams (Windows) to files and directories, instructing Dropbox to skip syncing them. This is particularly useful for development artifacts, dependencies, and temporary files that should remain local.

### Why use this tool?

Dropbox syncs all files in your Dropbox folder by default. For developers, this creates several problems:

1. **Bandwidth waste**: Build artifacts and dependencies consume significant bandwidth
2. **Storage bloat**: Generated files unnecessarily consume Dropbox storage quota
3. **Sync conflicts**: Temporary files and build outputs can cause sync conflicts
4. **Performance impact**: Syncing thousands of small files degrades performance

### How it works

The tool uses platform-specific mechanisms to mark files:

- **macOS**: Sets `com.dropbox.ignored` and `com.apple.fileprovider.ignore#P` extended attributes
- **Linux**: Sets `user.com.dropbox.ignored` extended attribute
- **Windows**: Creates `com.dropbox.ignored` alternate data stream

These markers are respected by Dropbox, causing it to skip the marked files during sync operations.

## Features

- **Git integration**: Automatically processes files matching .gitignore patterns
- **Batch operations**: Mark or unmark multiple files/directories in parallel
- **Watch mode**: Daemon process monitors for changes and maintains ignore markers
- **Dry-run mode**: Preview operations before execution
- **Wildcard support**: Process files matching glob patterns
- **Status reporting**: Query current ignore state of directories

## Installation

### Quick install (recommended)

#### One-line installer (macOS/Linux)

```bash
curl -sSf https://raw.githubusercontent.com/thomastheyoung/dbx-ignore/main/install.sh | sh
```

This automatically:

- Detects your platform (macOS Intel/ARM, Linux)
- Downloads the appropriate binary
- Installs to standard location (e.g. `/usr/local/bin` on macOS/Linux)
- Makes it executable

### Manual installation

#### Option 1: Download pre-built binaries

**macOS (Universal - works on Intel & Apple Silicon)**

```bash
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-universal -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**macOS (Intel only)**

```bash
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-intel -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**macOS (Apple Silicon only)**

```bash
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-macos-arm64 -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Linux (x86_64)**

```bash
curl -L https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-linux-x64 -o dbx-ignore
chmod +x dbx-ignore
sudo mv dbx-ignore /usr/local/bin/
```

**Windows (x86_64)**

```powershell
# PowerShell
Invoke-WebRequest -Uri https://github.com/thomastheyoung/dbx-ignore/releases/latest/download/dbx-ignore-windows-x64.exe -OutFile dbx-ignore.exe
# Move to a directory in your PATH
```

#### Option 2: Package managers

**Homebrew (macOS/Linux)**

```bash
# Coming soon
brew install thomastheyoung/tap/dbx-ignore
```

**Cargo (All platforms)**

```bash
cargo install dbx-ignore
```

#### Option 3: Build from source

**Prerequisites**

- [Rust](https://rustup.rs/) 1.70 or later

**Development build**

```bash
git clone https://github.com/thomastheyoung/dbx-ignore.git
cd dbx-ignore
make build
# Binary: ./target/release/dbx-ignore
```

**Distribution build**

```bash
git clone https://github.com/thomastheyoung/dbx-ignore.git
cd dbx-ignore
make build-dist
# Platform-specific binaries: ./bin/dbx-ignore-*
```

**Install from source**

```bash
cargo install --path .
```

### Verify installation

```bash
dbx-ignore --version
dbx-ignore --help
```

### Installation troubleshooting

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
- Try manual download from [GitHub Releases](https://github.com/thomastheyoung/dbx-ignore/releases)

**Windows installation**

- Use PowerShell method from manual installation section
- Or download directly from GitHub releases
- Install script currently supports macOS/Linux only

## Usage

### Command line interface

```
dbx-ignore [OPTIONS] [FILE]...
```

#### Arguments

- `[FILE]...` - Files, directories, wildcards, or .gitignore files to process
  - Supports glob patterns: `*.log`, `**/*.tmp`, `test[0-9].txt`
  - Special case: `.` expands to all non-hidden files in current directory
  - When a .gitignore file is specified, processes all files it would ignore

#### Options

**Actions (mutually exclusive):**

- `-r, --reset` - Remove ignore markers from specified files
- `-w, --watch` - Start daemon to monitor files/patterns (can accept patterns directly)
- `-u, --unwatch` - Stop the watch daemon
- `-s, --status` - Show current directory status (git, files, daemon)

**Modifiers:**

- `-g, --git` - Process git-ignored files (default when no files specified)
- `-n, --dry-run` - Preview operations without making changes
- `-v, --verbose` - Show detailed output for each file
- `-q, --quiet` - Suppress all output (exit code indicates result)

**Information:**

- `-h, --help` - Print help information
- `-V, --version` - Print version

### Usage examples

#### Basic operations

```bash
# Process all git-ignored files (most common usage)
dbx-ignore --git
dbx-ignore              # --git is implied when no files specified

# Mark specific files
dbx-ignore file1.txt file2.log

# Mark directories (marks the directory itself, not contents)
dbx-ignore target/ node_modules/ dist/

# Remove markers
dbx-ignore --reset file1.txt target/
```

#### Wildcard patterns

```bash
# Basic wildcards
dbx-ignore "*.log"                  # All .log files in current directory
dbx-ignore "*.tmp" "*.cache"        # Multiple patterns

# Recursive wildcards
dbx-ignore "**/*.log"               # All .log files in all subdirectories
dbx-ignore "src/**/*.test.js"       # Test files in src tree

# Character patterns
dbx-ignore "test?.txt"              # test1.txt, test2.txt, etc.
dbx-ignore "log[0-9].txt"           # log0.txt through log9.txt
dbx-ignore "data[0-9][0-9].csv"     # data00.csv through data99.csv

# Current directory expansion
dbx-ignore .                         # All non-hidden files in current dir
```

#### Preview and dry run

```bash
# Preview what would be processed
dbx-ignore --dry-run --git
dbx-ignore --dry-run "*.log"

# Preview with detailed output
dbx-ignore --dry-run --verbose target/

# Silent preview (exit code indicates what would happen)
dbx-ignore --dry-run --quiet --git
```

#### Watch mode

```bash
# Watch mode has three behaviors:

# 1. Combined mark & watch (NEW) - mark files and start watching in one command
dbx-ignore --watch "*.log" "*.tmp"  # Marks matching files AND starts watching

# 2. Pattern-based watching - monitors for files matching previously marked patterns
dbx-ignore "*.log" "*.tmp"          # Mark files with patterns
dbx-ignore --watch                  # Watches for any files matching these patterns

# 3. Git-based watching - monitors .gitignore changes (when no patterns/files tracked)
dbx-ignore --watch                  # Automatically marks/unmarks based on .gitignore

# 4. File-based watching - monitors specific tracked files
dbx-ignore target/ node_modules/    # Mark specific directories
dbx-ignore --watch                  # Watches only those specific files

# Check if daemon is running
dbx-ignore --status

# Stop daemon
dbx-ignore --unwatch
```

#### Status checking

```bash
# Basic status
dbx-ignore --status

# Detailed status with file listings
dbx-ignore --status --verbose
```

#### Advanced patterns

```bash
# Process files from a specific .gitignore
dbx-ignore path/to/.gitignore

# Combine multiple sources
dbx-ignore --git "additional/*.log" extra-file.tmp

# Reset all previously marked files in a directory
dbx-ignore --reset --git
```

#### Script integration

```bash
# Silent operation for scripts
dbx-ignore --quiet --git
if [ $? -eq 0 ]; then
    echo "Successfully marked files"
fi

# Check before processing
dbx-ignore --dry-run --quiet --git
if [ $? -eq 0 ]; then
    dbx-ignore --quiet --git
fi
```

### Output examples

**Standard operation:**

```
Platform: macOS
Mode: Adding ignore markers to git-ignored files
[00:00:01] [████████████████████████████████████████] 18/18 Complete!
──────────────────────────────────────────────────
34 files processed, 18 files marked to ignore
```

**Verbose mode:**

```
Platform: macOS
Mode: Adding ignore markers to specified files
   target/release/app: 2 ignore markers added
   node_modules/: already ignored
   .env.local: Permission denied (os error 13)
──────────────────────────────────────────────────
2 files processed, 1 file marked to ignore
```

**Status output:**

```
Status Report for: /path/to/project
.gitignore: Detected
Files: 125 files total
   23 files have ignore markers
   102 files don't have ignore markers
Daemon: Not running
```

## Technical details

### Sync prevention mechanism

Dropbox and Apple File Provider respect hidden markers that tell them to skip files:

| Marker                 | Purpose                     | Platform     |
| ---------------------- | --------------------------- | ------------ |
| Extended attributes    | Mark files as "do not sync" | macOS, Linux |
| Alternate Data Streams | Windows equivalent          | Windows      |

**The Goal**: Add these markers to files you don't want cluttering your Dropbox:

- Build artifacts (target/, dist/, build/)
- Large dependencies (node_modules/, vendor/)
- Temporary files (.tmp, .cache, logs)
- Development-only files that shouldn't be shared

### Platform support

| Platform | Architecture | Implementation         | Attributes                                               |
| -------- | ------------ | ---------------------- | -------------------------------------------------------- |
| macOS    | Intel/ARM64  | Extended attributes    | `com.dropbox.ignored`, `com.apple.fileprovider.ignore#P` |
| Linux    | x86_64       | Extended attributes    | `user.com.dropbox.ignored`                               |
| Windows  | x86_64       | Alternate Data Streams | `com.dropbox.ignored` ADS                                |
| Others   | Various      | Unsupported            | Reports platform limitation                              |

### Git integration

The tool automatically discovers and processes files that are ignored by git using:

```bash
git ls-files --ignored --exclude-standard -o
```

This includes files matching patterns in:

- `.gitignore`
- `.git/info/exclude`
- Global git exclude file
- Files explicitly added to git but later ignored

#### Full Git compatibility

**Implementation Approach:**
The tool delegates all ignore logic to git itself using `git ls-files --ignored --exclude-standard -o`, providing:

The tool delegates all git-ignore detection to git itself, ensuring:

- **Performance**: Direct git command execution without pattern parsing overhead
- **Accuracy**: Git's native logic determines ignored files
- **Compatibility**: Automatically supports all git ignore features
- **Coverage**: Includes .gitignore, .git/info/exclude, and global excludes

## API reference

### Operating modes

#### 1. Default mode (add markers)

```bash
dbx-ignore [OPTIONS] [FILE]...
```

- Adds ignore markers to specified files/directories
- If no files specified and no `--git` flag, shows help
- With `--git` or when no files specified, processes all git-ignored files

#### 2. Reset mode

```bash
dbx-ignore --reset [OPTIONS] [FILE]...
```

- Removes ignore markers from specified files/directories
- Cannot be combined with `--watch` or `--unwatch`
- Supports same file specification as default mode

#### 3. Watch mode

```bash
dbx-ignore --watch
```

- Starts background daemon to monitor previously marked files
- Daemon PID and status stored in `.dbx-ignore/daemon_status.json`
- Cannot be combined with `--reset` or `--unwatch`
- Ignores file arguments if provided

#### 4. Unwatch mode

```bash
dbx-ignore --unwatch
```

- Stops the running watch daemon
- Cannot be combined with `--reset` or `--watch`
- Ignores file arguments if provided

#### 5. Status mode

```bash
dbx-ignore --status [--verbose]
```

- Shows current directory information
- With `--verbose`, lists all files with their ignore status
- Exits immediately, ignores other flags

### Flag combinations and behaviors

#### Valid combinations

- `--dry-run --verbose` - Preview with detailed output
- `--git --dry-run` - Preview git-ignored files processing
- `--reset --verbose` - Remove markers with detailed output
- `--quiet --git` - Silent git-ignored files processing

#### Invalid combinations

- `--verbose --quiet` - Error: conflicting output modes
- `--reset --watch` - Error: conflicting actions
- `--watch --unwatch` - Error: conflicting actions

#### Special behaviors

1. **No arguments**: Shows help (since interactive mode was removed)
2. **Git mode auto-activation**: When no files specified, `--git` is implied
3. **Directory expansion**: When specifying `.`, expands to current directory contents (excluding hidden files)
4. **Gitignore file processing**: When a .gitignore file is specified as argument, processes files it would ignore
5. **Platform detection**: Automatically detects platform and uses appropriate attributes

### File processing rules

1. **Wildcards**: Processed using shell glob patterns
2. **Directories**: When a directory is specified, only the directory itself gets the marker (not recursive)
3. **Hidden files**: Files starting with `.` are excluded from wildcard expansion
4. **Parallel processing**: Multiple files processed concurrently for performance

### Exit codes

- `0` - Success
- `1` - General error (invalid arguments, missing files, permission denied)
- `101` - Platform not supported (non-macOS/Linux/Windows)

### State persistence

#### Tracked files

- Location: `.dbx-ignore/tracked_files.json`
- Contains: List of files explicitly marked by user
- Updated: On every add/remove operation
- Used by: Watch mode to monitor only user-marked files

#### Daemon status

- Location: `.dbx-ignore/daemon_status.json`
- Contains: PID, start time, repository path
- Created: When daemon starts
- Removed: When daemon stops

### Platform-specific behaviors

#### macOS

- Attempts to set both `com.dropbox.ignored` and `com.apple.fileprovider.ignore#P`
- Either attribute is sufficient for Dropbox to ignore the file
- Both are set for maximum compatibility

#### Linux

- Sets `user.com.dropbox.ignored` extended attribute
- Requires filesystem support for extended attributes

#### Windows

- Creates alternate data stream `com.dropbox.ignored`
- Works on NTFS filesystems

#### Unsupported platforms

- Tool exits gracefully with message
- No operations performed
- Exit code 0 (not an error)

## Watch mode details

### How watch mode works

Watch mode operates in three distinct modes based on what's being tracked:

#### Mode 1: Pattern-based monitoring (patterns tracked)

When patterns are provided (e.g., `*.log`, `build/**`), watch mode monitors for files matching those patterns:

1. **Pattern matching**: Continuously scans for files matching tracked patterns
2. **Automatic marking**: New files matching patterns are immediately marked
3. **Dynamic updates**: Files renamed to match patterns get marked, renamed away get unmarked
4. **Flexible patterns**: Supports standard glob patterns including `*`, `**`, `?`, `[...]`

#### Mode 2: GitIgnore monitoring (no patterns/files tracked)

When nothing is tracked, watch mode monitors your `.gitignore` files:

1. **Automatic marking**: Monitors all `.gitignore` files in the repository
2. **Real-time updates**: When `.gitignore` changes, automatically marks/unmarks affected files
3. **Comprehensive coverage**: Processes all git-ignored files in the repository
4. **Dynamic behavior**: Adding patterns marks new files, removing patterns unmarks them

#### Mode 3: File-based monitoring (specific files tracked)

When specific files/directories are marked (without patterns), watch mode monitors only those:

1. **Tracking**: Only monitors files explicitly marked by the user
2. **Persistence**: Tracked files stored in `.dbx-ignore/tracked_files.json`
3. **Re-marking**: Automatically re-applies markers if files are modified
4. **Selective updates**: Updates markers based on git ignore status changes

### Watch mode workflows

#### Workflow 1: Pattern-based monitoring

```bash
# Option A: Combined mark & watch (recommended)
dbx-ignore --watch "*.log" "*.tmp" "build/**"
# Output: Marking files before starting watch mode...
# ✓ 5 files processed, 10 ignore markers added
# ✓ Started daemon watcher (PID: 12345)

# Option B: Separate mark then watch
dbx-ignore "*.log" "*.tmp" "build/**"      # First mark files
dbx-ignore --watch                          # Then start watching
# Output: Starting file watcher daemon...
# Mode: Monitoring for files matching patterns:
#   - *.log
#   - *.tmp
#   - build/**

# Both options result in automatic marking of new files:
echo "test" > debug.log     # Automatically marked
echo "data" > temp.tmp      # Automatically marked
mkdir build && echo "x" > build/output.js  # Automatically marked
```

#### Workflow 2: Automatic .gitignore monitoring

```bash
# In a git repository with .gitignore (no files/patterns tracked)
dbx-ignore --watch
# Output: Starting file watcher daemon...
# Mode: Monitoring .gitignore changes to automatically mark/unmark files

# Daemon watches .gitignore and marks/unmarks files automatically
```

#### Workflow 3: Specific file monitoring

```bash
# Step 1: Mark specific files/directories (no patterns)
dbx-ignore target/ node_modules/

# Step 2: Start the watch daemon
dbx-ignore --watch
# Output: Starting file watcher daemon...
# Mode: Monitoring 2 tracked files for changes

# Step 3: Daemon monitors only those specific files
# If target/ is modified, daemon will ensure it stays marked

# Step 4: Check status anytime
dbx-ignore --status
# Shows daemon is running and which files are tracked

# Step 5: Stop when done
dbx-ignore --unwatch
```

### Important notes

- Watch mode behavior depends on what's tracked:
  - Patterns tracked → monitors for files matching patterns
  - Nothing tracked → monitors .gitignore changes
  - Specific files tracked → monitors only those files
- Pattern mode is most powerful for dynamic environments
- Daemon survives terminal closure (background process)
- One daemon per repository (multiple watchers not supported)
- Patterns use standard glob syntax (`*`, `**`, `?`, `[...]`)

## Safety features and validation

### Protection against dangerous operations

The tool includes safeguards to prevent accidentally marking too many files:

1. **Whole directory protection**: Cannot use `dbx-ignore .` or `dbx-ignore *` without a `.gitignore` file in a git repository
2. **Git repository check**: Validates git repository presence for git-mode operations
3. **File existence validation**: Checks that specified files exist before processing
4. **Permission handling**: Gracefully handles permission denied errors

### Validation examples

```bash
# This will fail without proper git setup
dbx-ignore .
# Error: Cannot mark entire directory without a .gitignore file in a git repository.

# This works (with .gitignore present)
cd my-git-project
dbx-ignore .

# This always works (specific files)
dbx-ignore specific-file.txt specific-folder/
```

## Troubleshooting

### Common issues

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

## Getting help

```bash
dbx-ignore --help       # Show available options
dbx-ignore --version    # Show version information
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
