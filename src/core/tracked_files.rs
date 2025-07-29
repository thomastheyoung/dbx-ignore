use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
use crate::core::json_utils;

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
        
        // Use robust JSON reading with fallback to default
        match json_utils::read_json::<TrackedFiles>(&state_file) {
            Ok(mut tracked) => {
                // Validate and clean data
                tracked.marked_files.retain(|p| p.as_os_str().len() > 0);
                tracked.patterns.retain(|p| !p.is_empty());
                Ok(tracked)
            }
            Err(_) => {
                // If corrupted, return default and the corrupted file will be overwritten
                Ok(Self::default())
            }
        }
    }
    
    /// Save tracked files to the state file
    pub fn save(&self, repo_path: &Path) -> Result<()> {
        let state_file = Self::state_file_path(repo_path);
        
        // Use atomic write
        json_utils::write_json_atomic(&state_file, self)
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