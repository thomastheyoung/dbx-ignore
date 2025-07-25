use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

/// Stores information about files that have been marked with ignore attributes
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TrackedFiles {
    /// Set of file paths that have been marked by the user
    pub marked_files: HashSet<PathBuf>,
    /// Patterns used to mark files (e.g., "*.log", "build/", "**/*.tmp")
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Timestamp of last update
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl TrackedFiles {
    /// Load tracked files from the state file
    pub fn load(repo_path: &Path) -> Result<Self> {
        let state_file = Self::state_file_path(repo_path);
        
        if !state_file.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&state_file)
            .context("Failed to read tracked files state")?;
            
        serde_json::from_str(&content)
            .context("Failed to parse tracked files state")
    }
    
    /// Save tracked files to the state file
    pub fn save(&self, repo_path: &Path) -> Result<()> {
        let state_file = Self::state_file_path(repo_path);
        
        // Ensure the .dbx-ignore directory exists
        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create .dbx-ignore directory")?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize tracked files")?;
            
        fs::write(&state_file, content)
            .context("Failed to write tracked files state")?;
            
        Ok(())
    }
    
    /// Add files to the tracked set
    pub fn add_files(&mut self, files: &[PathBuf]) {
        for file in files {
            self.marked_files.insert(file.clone());
        }
        self.last_updated = chrono::Utc::now();
    }
    
    /// Add patterns to track
    pub fn add_patterns(&mut self, patterns: &[String]) {
        for pattern in patterns {
            if !self.patterns.contains(pattern) {
                self.patterns.push(pattern.clone());
            }
        }
        self.last_updated = chrono::Utc::now();
    }
    
    /// Remove files from the tracked set
    pub fn remove_files(&mut self, files: &[PathBuf]) {
        for file in files {
            self.marked_files.remove(file);
        }
        self.last_updated = chrono::Utc::now();
    }
    
    /// Remove patterns from tracking
    pub fn remove_patterns(&mut self, patterns: &[String]) {
        self.patterns.retain(|p| !patterns.contains(p));
        self.last_updated = chrono::Utc::now();
    }
    
    /// Check if a file is being tracked
    pub fn is_tracked(&self, file: &Path) -> bool {
        self.marked_files.contains(file)
    }
    
    /// Get the state file path
    fn state_file_path(repo_path: &Path) -> PathBuf {
        repo_path.join(".dbx-ignore").join("tracked_files.json")
    }
    
    /// Remove the state file
    pub fn remove_state_file(repo_path: &Path) -> Result<()> {
        let state_file = Self::state_file_path(repo_path);
        if state_file.exists() {
            fs::remove_file(&state_file)
                .context("Failed to remove tracked files state")?;
        }
        Ok(())
    }
}