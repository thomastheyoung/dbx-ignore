use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::utils::pattern_matcher;


/// Get all git-ignored files in the current directory
pub fn get_git_ignored_files() -> Result<Vec<PathBuf>> {
    get_git_ignored_files_in_path(&std::env::current_dir()?)
}

/// Get all git-ignored files in a specific path using our own implementation
pub fn get_git_ignored_files_in_path(path: &Path) -> Result<Vec<PathBuf>> {
    // Check if we're in a git repository
    let _repo = git2::Repository::discover(path)
        .context("Not in a git repository or git repository not found")?;
    
    // Build two walkers - one that respects gitignore, one that doesn't
    use ignore::WalkBuilder;
    
    // Walker that sees everything (to get all files)
    let mut all_files_builder = WalkBuilder::new(path);
    all_files_builder
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false);
    
    // Walker that respects gitignore (to get non-ignored files)
    let mut filtered_builder = WalkBuilder::new(path);
    filtered_builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);
    
    // Collect all files
    let mut all_files = HashSet::new();
    for entry in all_files_builder.build().flatten() {
        let path = entry.path();
        // Skip .git directory and only collect files (not directories)
        if !path.components().any(|c| c.as_os_str() == ".git") && path.is_file() {
            all_files.insert(path.to_path_buf());
        }
    }
    
    // Collect non-ignored files
    let mut non_ignored_files = HashSet::new();
    for entry in filtered_builder.build().flatten() {
        let path = entry.path();
        // Skip .git directory and only collect files (not directories)
        if !path.components().any(|c| c.as_os_str() == ".git") && path.is_file() {
            non_ignored_files.insert(path.to_path_buf());
        }
    }
    
    // The ignored files are the difference
    let mut ignored_files: Vec<PathBuf> = all_files.difference(&non_ignored_files)
        .cloned()
        .collect();
    
    // Sort for consistent output
    ignored_files.sort();
    
    Ok(ignored_files)
}

/// Get git-ignored files from a specific .gitignore file's directory
pub fn get_git_ignored_files_from_gitignore(gitignore_path: &Path) -> Result<Vec<PathBuf>> {
    // Get the directory containing the .gitignore file
    let gitignore_dir = gitignore_path.parent()
        .context("Unable to get parent directory of .gitignore file")?;
    
    // Get all ignored files
    let all_ignored = get_git_ignored_files_in_path(gitignore_dir)?;
    
    // Filter to only include files within the .gitignore's directory
    Ok(all_ignored.into_iter()
        .filter(|path| path.starts_with(gitignore_dir))
        .collect())
}

/// Find files matching patterns using gitignore-style pattern matching
/// This ensures consistent behavior whether in a git repository or not
pub fn find_files_matching_patterns(base_path: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    pattern_matcher::find_files_matching_patterns(base_path, patterns)
}