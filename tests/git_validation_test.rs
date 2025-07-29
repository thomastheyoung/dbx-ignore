use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use std::fs;

/// Comprehensive test to validate our gitignore implementation matches git exactly
#[test]
fn test_gitignore_implementation_matches_git() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize a git repository
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Create a complex file structure
    create_complex_test_structure(temp_path)?;
    
    // Create multiple .gitignore files with various patterns
    create_gitignore_files(temp_path)?;
    
    // Get ignored files using git ls-files
    let git_ignored = get_git_ignored_files_using_git(temp_path)?;
    
    // Get ignored files using our implementation
    let our_ignored = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    
    // Convert to sets of relative paths for comparison
    let git_set: HashSet<PathBuf> = git_ignored.into_iter()
        .filter_map(|p| p.strip_prefix(temp_path).ok().map(|p| p.to_path_buf()))
        .collect();
    
    let our_set: HashSet<PathBuf> = our_ignored.into_iter()
        .filter_map(|p| p.strip_prefix(temp_path).ok().map(|p| p.to_path_buf()))
        .collect();
    
    // Allow for directory consolidation differences
    let mut our_expanded = HashSet::new();
    for path in &our_set {
        if path.is_dir() || path.to_string_lossy().ends_with("/") {
            // If we have a directory, git might list individual files
            // This is OK as long as all files in git_set under this directory are accounted for
            our_expanded.insert(path.clone());
            
            // Add a marker that this directory covers its contents
            for git_path in &git_set {
                if git_path.starts_with(path) {
                    our_expanded.insert(git_path.clone());
                }
            }
        } else {
            our_expanded.insert(path.clone());
        }
    }
    
    // Check for mismatches
    let only_in_git: Vec<_> = git_set.difference(&our_expanded).collect();
    let only_in_ours: Vec<_> = our_expanded.difference(&git_set)
        .filter(|p| !p.is_dir() && !p.to_string_lossy().ends_with("/"))
        .collect();
    
    if !only_in_git.is_empty() || !only_in_ours.is_empty() {
        println!("Git ignored files: {:?}", git_set);
        println!("Our ignored files: {:?}", our_set);
        
        if !only_in_git.is_empty() {
            println!("Only in git: {:?}", only_in_git);
        }
        if !only_in_ours.is_empty() {
            println!("Only in ours: {:?}", only_in_ours);
        }
        
        panic!("Gitignore implementation mismatch!");
    }
    
    println!("✓ Gitignore implementation matches git exactly!");
    Ok(())
}

/// Test edge cases and complex patterns
#[test]
fn test_gitignore_edge_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Test case 1: Negation patterns
    fs::write(temp_path.join(".gitignore"), "*.log\n!important.log\n")?;
    fs::write(temp_path.join("test.log"), "")?;
    fs::write(temp_path.join("important.log"), "")?;
    
    let git_ignored = get_git_ignored_files_using_git(temp_path)?;
    let our_ignored = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    
    println!("Test case 1 - Negation patterns:");
    println!("Git ignored: {:?}", git_ignored);
    println!("Our ignored: {:?}", our_ignored);
    
    assert_eq!(git_ignored.len(), 1);
    assert_eq!(our_ignored.len(), 1);
    
    // Normalize paths to handle /var vs /private/var on macOS
    let git_normalized: Vec<_> = git_ignored.iter()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
        .collect();
    let our_normalized: Vec<_> = our_ignored.iter()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
        .collect();
    
    assert!(git_normalized[0].ends_with("test.log"));
    assert!(our_normalized[0].ends_with("test.log"));
    
    // Test case 2: Directory patterns with trailing slash
    fs::create_dir_all(temp_path.join("build/debug"))?;
    fs::write(temp_path.join("build/debug/app"), "")?;
    fs::write(temp_path.join("build.txt"), "")?;
    fs::write(temp_path.join(".gitignore"), "build/\n")?;
    
    let git_ignored = get_git_ignored_files_using_git(temp_path)?;
    let our_ignored = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    
    // Git will list the directory, we should too
    let git_has_build_dir = git_ignored.iter().any(|p| p.ends_with("build") || p.ends_with("build/"));
    let our_has_build_dir = our_ignored.iter().any(|p| p.ends_with("build") || p.ends_with("build/"));
    assert_eq!(git_has_build_dir, our_has_build_dir);
    
    // build.txt should NOT be ignored
    assert!(!git_ignored.iter().any(|p| p.ends_with("build.txt")));
    assert!(!our_ignored.iter().any(|p| p.ends_with("build.txt")));
    
    println!("✓ Edge cases handled correctly!");
    Ok(())
}

/// Test .git/info/exclude file
#[test]
fn test_git_info_exclude() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Create .git/info/exclude
    let git_dir = temp_path.join(".git");
    fs::create_dir_all(git_dir.join("info"))?;
    fs::write(git_dir.join("info/exclude"), "secret.txt\n")?;
    
    // Create files
    fs::write(temp_path.join("secret.txt"), "")?;
    fs::write(temp_path.join("public.txt"), "")?;
    
    let git_ignored = get_git_ignored_files_using_git(temp_path)?;
    let our_ignored = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    
    assert_eq!(git_ignored.len(), 1);
    assert_eq!(our_ignored.len(), 1);
    assert!(git_ignored[0].ends_with("secret.txt"));
    assert!(our_ignored[0].ends_with("secret.txt"));
    
    println!("✓ .git/info/exclude handled correctly!");
    Ok(())
}

/// Get ignored files using actual git command for validation
fn get_git_ignored_files_using_git(workdir: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .current_dir(workdir)
        .args(["ls-files", "--ignored", "--exclude-standard", "-o"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("git ls-files failed"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut paths = Vec::new();
    
    for line in stdout.lines() {
        let line = line.trim();
        if !line.is_empty() {
            paths.push(workdir.join(line));
        }
    }
    
    Ok(paths)
}

fn create_complex_test_structure(base: &Path) -> Result<()> {
    // Create directories
    fs::create_dir_all(base.join("src/components/ui"))?;
    fs::create_dir_all(base.join("src/utils"))?;
    fs::create_dir_all(base.join("build/debug"))?;
    fs::create_dir_all(base.join("build/release"))?;
    fs::create_dir_all(base.join("node_modules/package1/dist"))?;
    fs::create_dir_all(base.join("node_modules/package2/lib"))?;
    fs::create_dir_all(base.join("docs/api"))?;
    fs::create_dir_all(base.join("test/fixtures"))?;
    fs::create_dir_all(base.join(".hidden/cache"))?;
    
    // Create files
    let files = vec![
        // Source files
        "src/main.rs",
        "src/lib.rs",
        "src/components/App.tsx",
        "src/components/ui/Button.tsx",
        "src/utils/helper.js",
        "src/test.log",
        
        // Build artifacts
        "build/debug/app",
        "build/debug/app.dSYM",
        "build/release/app",
        "build.log",
        
        // Dependencies
        "node_modules/package1/index.js",
        "node_modules/package1/dist/bundle.js",
        "node_modules/package2/lib/index.js",
        
        // Documentation
        "docs/README.md",
        "docs/api/spec.yaml",
        "docs/notes.tmp",
        
        // Test files
        "test/test.js",
        "test/fixtures/data.json",
        "test.log",
        
        // Hidden files
        ".DS_Store",
        ".hidden/secret.key",
        ".hidden/cache/data.cache",
        
        // Temporary files
        "temp.tmp",
        "backup.bak",
        "~tempfile",
        
        // Log files
        "app.log",
        "debug.log",
        "error.log",
        
        // Config files
        "config.json",
        "settings.ini",
        ".env.local",
    ];
    
    for file in files {
        let path = base.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, "test content")?;
    }
    
    Ok(())
}

fn create_gitignore_files(base: &Path) -> Result<()> {
    // Root .gitignore
    let root_gitignore = r#"
# Dependencies
node_modules/
vendor/

# Build artifacts
build/
dist/
target/

# Logs
*.log

# Temporary files
*.tmp
*.bak
~*

# OS files
.DS_Store
Thumbs.db

# Hidden directories
.hidden/

# But keep important files
!important.log
"#;
    fs::write(base.join(".gitignore"), root_gitignore)?;
    
    // src/.gitignore (more specific)
    let src_gitignore = r#"
# Test files in src
test.*
*.test.js

# But not test directories
!test/
"#;
    fs::write(base.join("src/.gitignore"), src_gitignore)?;
    
    // docs/.gitignore
    let docs_gitignore = r#"
# Temporary documentation
*.tmp
*.draft.md

# API keys in docs
*.key
"#;
    fs::write(base.join("docs/.gitignore"), docs_gitignore)?;
    
    Ok(())
}