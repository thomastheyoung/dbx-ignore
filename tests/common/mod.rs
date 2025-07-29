use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test utilities and helpers
#[allow(dead_code)]
pub struct TestEnvironment {
    _temp_dir: TempDir, // Prefixed with _ since we only need it to keep the directory alive
    pub temp_path: PathBuf,
}

#[allow(dead_code)]
impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        Self {
            _temp_dir: temp_dir,
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

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        &self.temp_path
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
}
