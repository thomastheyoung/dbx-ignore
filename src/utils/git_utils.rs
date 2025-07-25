use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs git ls-files command to find ignored files
/// 
/// This is the core implementation that executes git's built-in ignore handling.
/// It supports all git ignore features including negated patterns.
pub fn run_git_ls_files_ignored(workdir: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(workdir)
        .args(["ls-files", "--ignored", "--exclude-standard", "-o"])
        .output()
        .context("Failed to execute git ls-files command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git ls-files failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|s| s.to_string())
        .collect())
}

/// Get all git-ignored files in the current directory
pub fn get_git_ignored_files() -> Result<Vec<PathBuf>> {
    get_git_ignored_files_in_path(&std::env::current_dir()?)
}

/// Get all git-ignored files in a specific path
pub fn get_git_ignored_files_in_path(path: &Path) -> Result<Vec<PathBuf>> {
    // Check if we're in a git repository
    let repo = git2::Repository::discover(path)
        .context("Not in a git repository or git repository not found")?;

    let workdir = repo.workdir()
        .context("Repository has no working directory")?;

    let ignored_files = run_git_ls_files_ignored(workdir)?;
    
    let mut paths = Vec::new();
    for file in ignored_files {
        let path = workdir.join(file);
        if path.exists() {
            paths.push(path);
        }
    }

    Ok(paths)
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