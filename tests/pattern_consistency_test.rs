use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use std::fs;

/// Test that our pattern matcher produces identical results to git's pattern matching
#[test]
fn test_pattern_matching_consistency() -> Result<()> {
    // Test in a git repo
    let git_temp = TempDir::new()?;
    let git_path = git_temp.path();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(git_path)
        .args(["init"])
        .output()?;
    
    // Create identical file structure in both
    create_test_structure(git_path)?;
    
    // Test in a non-git directory
    let non_git_temp = TempDir::new()?;
    let non_git_path = non_git_temp.path();
    create_test_structure(non_git_path)?;
    
    // Test patterns
    let patterns = vec![
        "*.log",
        "**/*.log",
        "build/",
        "**/node_modules/",
        "src/**/*.bak",
        "*.tmp",
        ".DS_Store",
    ];
    
    for pattern in &patterns {
        println!("Testing pattern: {}", pattern);
        
        // Get files in git repo
        let git_results = dbx_ignore::utils::pattern_matcher::find_files_matching_patterns(
            git_path,
            &[pattern.to_string()]
        )?;
        
        // Get files in non-git directory  
        let non_git_results = dbx_ignore::utils::pattern_matcher::find_files_matching_patterns(
            non_git_path,
            &[pattern.to_string()]
        )?;
        
        // Convert to relative paths for comparison
        let git_set: HashSet<PathBuf> = git_results.into_iter()
            .map(|p| p.strip_prefix(git_path).unwrap().to_path_buf())
            .collect();
        let non_git_set: HashSet<PathBuf> = non_git_results.into_iter()
            .map(|p| p.strip_prefix(non_git_path).unwrap().to_path_buf())
            .collect();
        
        // They should be identical
        if git_set != non_git_set {
            println!("Mismatch for pattern: {}", pattern);
            println!("In git repo: {:?}", git_set);
            println!("In non-git: {:?}", non_git_set);
            panic!("Pattern matching inconsistent between git and non-git directories");
        }
    }
    
    println!("All patterns consistent between git and non-git directories!");
    Ok(())
}

/// Test that .gitignore patterns work the same as CLI patterns
#[test]
fn test_gitignore_vs_cli_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Create test files
    create_test_structure(temp_path)?;
    
    // Create .gitignore
    let gitignore_content = "*.log\n*.tmp\nbuild/\n";
    fs::write(temp_path.join(".gitignore"), gitignore_content)?;
    
    // Get files ignored by .gitignore
    let gitignore_files = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    let gitignore_set: HashSet<PathBuf> = gitignore_files.into_iter()
        .filter_map(|p| p.strip_prefix(temp_path).ok().map(|p| p.to_path_buf()))
        .collect();
    
    // Get files matching the same patterns via CLI
    let patterns = ["*.log", "*.tmp", "build/"];
    let pattern_files = dbx_ignore::utils::pattern_matcher::find_files_matching_patterns(
        temp_path,
        &patterns.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    )?;
    let pattern_set: HashSet<PathBuf> = pattern_files.into_iter()
        .filter_map(|p| p.strip_prefix(temp_path).ok().map(|p| p.to_path_buf()))
        .collect();
    
    // Compare - should be very similar (gitignore might include directories)
    println!("Gitignore files: {:?}", gitignore_set);
    println!("Pattern files: {:?}", pattern_set);
    
    // Check that all pattern files are in gitignore files
    for file in &pattern_set {
        if !gitignore_set.contains(file) {
            // Check if it's a directory issue (gitignore includes "build/" but pattern might list files)
            let as_dir = file.as_os_str().to_string_lossy().to_string() + "/";
            if !gitignore_set.contains(&PathBuf::from(&as_dir)) {
                panic!("Pattern file {:?} not found in gitignore results", file);
            }
        }
    }
    
    Ok(())
}

fn create_test_structure(base: &Path) -> Result<()> {
    // Create directories
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("build/debug"))?;
    fs::create_dir_all(base.join("node_modules/pkg"))?;
    fs::create_dir_all(base.join("docs"))?;
    
    // Create files
    fs::write(base.join("test.log"), "")?;
    fs::write(base.join("app.rs"), "")?;
    fs::write(base.join("temp.tmp"), "")?;
    fs::write(base.join(".DS_Store"), "")?;
    fs::write(base.join("src/main.rs"), "")?;
    fs::write(base.join("src/debug.log"), "")?;
    fs::write(base.join("src/backup.bak"), "")?;
    fs::write(base.join("build/output.js"), "")?;
    fs::write(base.join("build/debug/app.exe"), "")?;
    fs::write(base.join("node_modules/pkg/index.js"), "")?;
    fs::write(base.join("node_modules/pkg/test.log"), "")?;
    fs::write(base.join("docs/manual.pdf"), "")?;
    
    Ok(())
}