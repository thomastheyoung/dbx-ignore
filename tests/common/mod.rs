use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test utilities and helpers
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub temp_path: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        Self {
            temp_dir,
            temp_path,
        }
    }

    /// Create a test file in the temp directory
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.temp_path.join(name);
        fs::write(&file_path, content).expect("Failed to create test file");
        file_path
    }

    /// Create a test directory in the temp directory
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let dir_path = self.temp_path.join(name);
        fs::create_dir_all(&dir_path).expect("Failed to create test directory");
        dir_path
    }

    /// Create a .gitignore file with specified patterns
    pub fn create_gitignore(&self, patterns: &[&str]) -> PathBuf {
        let gitignore_content = patterns.join("\n");
        self.create_file(".gitignore", &gitignore_content)
    }

    /// Initialize a git repository in the temp directory
    pub fn init_git_repo(&self) -> Result<git2::Repository, git2::Error> {
        git2::Repository::init(&self.temp_path)
    }

    /// Get the temp directory path
    pub fn path(&self) -> &Path {
        &self.temp_path
    }
}

/// Mock platform handler for testing
pub struct MockPlatformHandler {
    pub target_attributes: Vec<String>,
    pub existing_attributes: std::collections::HashMap<PathBuf, Vec<String>>,
    pub should_fail: bool,
}

impl MockPlatformHandler {
    pub fn new() -> Self {
        Self {
            target_attributes: vec!["test.attr1".to_string(), "test.attr2".to_string()],
            existing_attributes: std::collections::HashMap::new(),
            should_fail: false,
        }
    }

    pub fn with_attributes(mut self, attrs: Vec<String>) -> Self {
        self.target_attributes = attrs;
        self
    }

    pub fn with_existing_attributes(mut self, path: PathBuf, attrs: Vec<String>) -> Self {
        self.existing_attributes.insert(path, attrs);
        self
    }

    pub fn set_should_fail(mut self, fail: bool) -> Self {
        self.should_fail = fail;
        self
    }
}
