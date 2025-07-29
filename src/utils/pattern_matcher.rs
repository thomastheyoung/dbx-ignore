use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

/// A pattern matcher that provides gitignore-style pattern matching
/// Works consistently whether inside or outside a git repository
pub struct PatternMatcher {
    gitignore: Gitignore,
    base_path: PathBuf,
}

impl PatternMatcher {
    /// Create a new pattern matcher with the given patterns
    pub fn new(base_path: &Path, patterns: &[String]) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(base_path);

        // Add each pattern to the builder
        for pattern in patterns {
            builder
                .add_line(None, pattern)
                .with_context(|| format!("Invalid pattern: {}", pattern))?;
        }

        let gitignore = builder.build()?;

        Ok(Self {
            gitignore,
            base_path: base_path.to_path_buf(),
        })
    }

    /// Check if a path matches any of the patterns
    pub fn is_ignored(&self, path: &Path) -> bool {
        // The ignore crate expects relative paths from the base
        let relative_path = if path.is_absolute() {
            match path.strip_prefix(&self.base_path) {
                Ok(rel) => rel,
                Err(_) => return false, // Path outside base directory
            }
        } else {
            path
        };

        self.gitignore
            .matched(relative_path, path.is_dir())
            .is_ignore()
    }

    /// Find all files matching the patterns in a directory
    pub fn find_matching_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        use ignore::WalkBuilder;

        let mut matching_files = Vec::new();

        // Create a walker that respects our patterns
        let walker = WalkBuilder::new(root)
            .standard_filters(false) // Don't use default filters
            .hidden(false) // Include hidden files
            .parents(false) // Don't look for .gitignore in parent dirs
            .git_ignore(false) // Don't use .gitignore files
            .git_global(false) // Don't use global gitignore
            .git_exclude(false) // Don't use .git/info/exclude
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            // Check if this path matches our patterns
            // Only include files, not directories (to match git ls-files behavior)
            if path.is_file() && self.is_ignored(path) {
                matching_files.push(path.to_path_buf());
            }
        }

        Ok(matching_files)
    }
}

/// Find files matching gitignore-style patterns
/// This provides consistent behavior whether in a git repo or not
pub fn find_files_matching_patterns(base_path: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let matcher = PatternMatcher::new(base_path, patterns)?;
    matcher.find_matching_files(base_path)
}

/// Check if a file matches any of the given patterns
pub fn matches_patterns(base_path: &Path, file_path: &Path, patterns: &[String]) -> Result<bool> {
    let matcher = PatternMatcher::new(base_path, patterns)?;
    Ok(matcher.is_ignored(file_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_matching() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create test files
        fs::create_dir_all(base.join("src")).unwrap();
        fs::write(base.join("test.log"), "").unwrap();
        fs::write(base.join("src/main.rs"), "").unwrap();
        fs::write(base.join("src/test.log"), "").unwrap();

        // Test single wildcard
        let patterns = vec!["*.log".to_string()];
        let matcher = PatternMatcher::new(base, &patterns).unwrap();

        assert!(matcher.is_ignored(&base.join("test.log")));
        assert!(!matcher.is_ignored(&base.join("src/main.rs")));
        // Note: In gitignore, *.log matches in any directory
        assert!(matcher.is_ignored(&base.join("src/test.log")));

        // Test recursive wildcard
        let patterns = vec!["**/*.log".to_string()];
        let matcher = PatternMatcher::new(base, &patterns).unwrap();

        assert!(matcher.is_ignored(&base.join("test.log")));
        assert!(matcher.is_ignored(&base.join("src/test.log")));
        assert!(!matcher.is_ignored(&base.join("src/main.rs")));
    }
}
