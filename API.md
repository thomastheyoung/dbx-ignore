# API Reference

Complete reference for `dbx-ignore` command-line interface and behavior.

## Command Synopsis

```
dbx-ignore [OPTIONS] [FILE]...
```

## Arguments

### `[FILE]...`

Files, directories, or patterns to process. Accepts multiple values.

**Special behaviors:**

- **No arguments in git repo**: Automatically processes all git-ignored files
- **`.` (dot)**: Expands to all non-hidden files in current directory
- **`.gitignore` file**: When specified, processes all files that would be ignored by it
- **Wildcards**: Supports glob patterns (`*`, `**`, `?`, `[...]`)

**Examples:**

```bash
dbx-ignore                          # Auto-process git-ignored files
dbx-ignore file.txt dir/            # Specific files/directories
dbx-ignore "*.log" "**/*.tmp"       # Glob patterns
dbx-ignore .                        # Current directory contents
dbx-ignore src/.gitignore           # Files ignored by specific .gitignore
```

## Options

### Action Flags (Mutually Exclusive)

#### `-r, --reset`

Remove ignore markers from files and directories.

```bash
dbx-ignore --reset file.txt         # Remove markers from file
dbx-ignore --reset "*.log"          # Remove from all .log files
dbx-ignore --reset --git            # Remove from all git-ignored files
```

#### `-w, --watch`

Start daemon to continuously monitor files.

**Behaviors:**

1. **With patterns**: `dbx-ignore --watch "*.log"` - Marks files and starts monitoring
2. **After marking patterns**: Monitors files matching previously marked patterns
3. **No patterns/files tracked**: Monitors .gitignore changes
4. **Specific files tracked**: Monitors only those files

```bash
dbx-ignore --watch "*.log"          # Mark and watch
dbx-ignore --watch                  # Watch based on current state
```

#### `-u, --unwatch`

Stop the running watch daemon.

```bash
dbx-ignore --unwatch
```

#### `-s, --status`

Show current directory status.

```bash
dbx-ignore --status                 # Basic status
dbx-ignore --status --verbose       # Detailed with file listings
```

### Modifier Flags

#### `-g, --git`

Process git-ignored files. This is the default when no files are specified.

```bash
dbx-ignore --git                    # Explicit git mode
dbx-ignore                          # Implicit git mode (same effect)
dbx-ignore --git file.txt           # Combines git-ignored + specific file
```

#### `-n, --dry-run`

Preview what would be done without making changes.

```bash
dbx-ignore --dry-run                # Preview git-ignored files
dbx-ignore --dry-run "*.log"        # Preview pattern matches
```

#### `-v, --verbose`

Show detailed output for each file operation.

```bash
dbx-ignore --verbose file.txt       # Shows each attribute added
dbx-ignore --verbose --reset        # Shows each attribute removed
```

#### `-q, --quiet`

Suppress all output. Exit code indicates success/failure.

```bash
dbx-ignore --quiet                  # Silent operation
dbx-ignore --quiet && echo "Success" || echo "Failed"
```

### Information Flags

#### `-h, --help`

Display help information.

#### `-V, --version`

Display version information.

## Behavior Details

### Default Behavior (No Arguments)

When run without arguments, `dbx-ignore` checks if you're in a git repository:

1. **In git repo with .gitignore**: Processes all git-ignored files
2. **Not in git repo or no .gitignore**: Shows help

This provides a smart, zero-configuration experience for most users.

### Pattern Matching

Patterns use standard glob syntax:

| Pattern         | Matches                             |
| --------------- | ----------------------------------- |
| `*.log`         | All .log files in current directory |
| `**/*.log`      | All .log files recursively          |
| `test?.txt`     | test1.txt, test2.txt, etc.          |
| `[0-9]*.txt`    | Files starting with a digit         |
| `{src,test}/**` | All files under src/ or test/       |

**Quoting patterns:**

- Required when: Pattern contains spaces, no matching files exist yet, using watch mode
- Optional when: Files exist and shell can expand

### File Processing

#### Marking Files

When marking files for ignore:

1. Checks if file exists
2. Detects platform
3. Adds appropriate markers:
   - macOS: `com.dropbox.ignored`, `com.apple.fileprovider.ignore#P`
   - Linux: `user.com.dropbox.ignored`
   - Windows: `com.dropbox.ignored` ADS
4. Updates `.dbx-ignore/tracked_files.json`

#### Directory Handling

- Marks directory itself, not contents
- To mark contents, use patterns: `dir/**`

### Watch Mode Details

Watch mode operates differently based on what's being tracked:

#### Pattern-Based Monitoring

```bash
dbx-ignore --watch "*.log" "build/**"
```

- Continuously scans for new files matching patterns
- Automatically marks matching files
- Handles file renames (marks/unmarks as appropriate)

#### GitIgnore Monitoring

```bash
dbx-ignore --watch  # When no patterns/files tracked
```

- Monitors all .gitignore files in repository
- Automatically updates markers when .gitignore changes
- Marks new patterns, unmarks removed patterns

#### File-Based Monitoring

```bash
dbx-ignore target/ node_modules/
dbx-ignore --watch
```

- Monitors only specifically marked files
- Re-applies markers if files are modified
- Does not monitor for new files

### State Management

#### `.dbx-ignore/` Directory

Created automatically when files are marked. Contains:

- `tracked_files.json` - List of marked files and patterns
- `daemon_status.json` - Watch daemon information (when running)

**Automatic .gitignore Integration:**

- `.dbx-ignore/` is automatically added to .gitignore
- Prevents accidental commits of metadata
- Includes explanatory comment

#### Tracked Files Format

```json
{
  "marked_files": ["/path/to/file1.txt", "/path/to/directory/"],
  "patterns": ["*.log", "build/**"],
  "last_updated": "2024-01-20T10:30:00Z"
}
```

## Exit Codes

| Code | Meaning                                                |
| ---- | ------------------------------------------------------ |
| 0    | Success                                                |
| 1    | General error (invalid arguments, missing files, etc.) |
| 101  | Platform not supported                                 |

## Platform-Specific Behavior

### macOS

- Uses extended attributes via `xattr` system calls
- Both Dropbox attributes set for compatibility
- Requires macOS 10.15+

### Linux

- Uses extended attributes via `setfattr`/`getfattr`
- Requires filesystem with xattr support
- Single attribute: `user.com.dropbox.ignored`

### Windows

- Uses NTFS Alternate Data Streams
- Requires NTFS filesystem
- May need admin privileges for system directories

### Unsupported Platforms

- Exits gracefully with informative message
- No operations performed
- Exit code 0 (not treated as error)

## Safety Features

### Dangerous Operation Protection

Cannot mark entire directory with `.` or `*` unless:

1. In a git repository
2. Has a .gitignore file

This prevents accidentally marking everything for ignore.

### Validation

- **File existence**: Validates files exist before processing
- **Permission handling**: Gracefully handles permission errors
- **Platform detection**: Verifies platform support before operations

## Integration Examples

### Build Scripts

```bash
#!/bin/bash
# In your build script
make build
dbx-ignore --quiet target/ || echo "Warning: Could not mark build artifacts"
```

### Git Hooks

```bash
# .git/hooks/post-checkout
#!/bin/sh
dbx-ignore --quiet || true
```

### CI/CD

```yaml
# GitHub Actions example
- name: Build
  run: make build

- name: Prevent Dropbox sync
  run: |
    if command -v dbx-ignore &> /dev/null; then
      dbx-ignore --quiet
    fi
```

### Shell Aliases

```bash
# ~/.bashrc or ~/.zshrc
alias dbi='dbx-ignore'
alias dbis='dbx-ignore --status'
alias dbiw='dbx-ignore --watch'
```

## Performance Considerations

- **Parallel processing**: Files processed concurrently using rayon
- **Progress indication**: Shows progress bar for large operations
- **Efficient detection**: Only attempts operations on files missing markers
- **Cached git status**: Git operations minimized for performance

## Troubleshooting

### Common Issues

**"Not in a git repository"**

- Run `git init` or specify files explicitly

**"Permission denied"**

- Use `sudo` for system directories
- Check file ownership

**"Platform not supported"**

- Only macOS, Linux, Windows supported
- Check `dbx-ignore --version` for platform info

### Debug Information

Use `--verbose` for detailed operation information:

```bash
dbx-ignore --verbose --dry-run file.txt
```

Shows:

- Platform detection
- Each attribute operation
- Success/failure for each file
- Specific error messages
