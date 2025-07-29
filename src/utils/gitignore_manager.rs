use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const DBX_IGNORE_COMMENT: &str = "# dbx-ignore metadata folder - not needed in version control";
const DBX_IGNORE_PATTERN: &str = ".dbx-ignore/";

/// Ensures .dbx-ignore/ is in .gitignore when in a git repository
pub fn ensure_dbx_ignore_in_gitignore(repo_path: &Path) -> Result<()> {
    // Check if we're in a git repository
    if git2::Repository::discover(repo_path).is_err() {
        // Not in a git repo, nothing to do
        return Ok(());
    }

    let gitignore_path = repo_path.join(".gitignore");

    // Read existing .gitignore content or create empty string
    let content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path).context("Failed to read .gitignore")?
    } else {
        String::new()
    };

    // Check if .dbx-ignore/ is already in .gitignore
    if content
        .lines()
        .any(|line| line.trim() == DBX_IGNORE_PATTERN || line.trim() == ".dbx-ignore")
    {
        // Already present, nothing to do
        return Ok(());
    }

    // Prepare new content
    let mut new_content = String::new();

    // Add comment and pattern at the beginning
    new_content.push_str(DBX_IGNORE_COMMENT);
    new_content.push('\n');
    new_content.push_str(DBX_IGNORE_PATTERN);
    new_content.push('\n');

    // Add a blank line if there's existing content
    if !content.is_empty() {
        new_content.push('\n');
        new_content.push_str(&content);

        // Ensure file ends with newline
        if !content.ends_with('\n') {
            new_content.push('\n');
        }
    }

    // Write the updated content
    fs::write(&gitignore_path, new_content).context("Failed to update .gitignore")?;

    Ok(())
}
