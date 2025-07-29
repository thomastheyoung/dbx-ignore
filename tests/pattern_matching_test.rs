use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use std::fs;

/// Compare our git implementation with actual git
#[test] 
#[ignore = "This test was for individual pattern matching which we no longer support"]
fn test_pattern_matching_parity_with_git() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize a git repository
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Create test file structure
    create_test_files(temp_path)?;
    
    // Create a .gitignore file with various patterns
    let gitignore_content = r#"
# Test patterns
*.log
*.tmp
build/
target/
**/node_modules/
!important.log
.DS_Store
**/*.cache
src/**/*.bak
"#;
    fs::write(temp_path.join(".gitignore"), gitignore_content)?;
    
    // Test various patterns
    let test_patterns = vec![
        "*.log",
        "*.tmp", 
        "**/*.cache",
        "**/node_modules/**",
        "build/**",
        "src/**/*.bak",
    ];
    
    for pattern in &test_patterns {
        println!("Testing pattern: {}", pattern);
        
        // Get files using git ls-files
        let git_files = get_git_ignored_files_with_pattern(temp_path, pattern)?;
        
        // Get files using our pattern matcher
        let our_files = dbx_ignore::utils::git_utils::find_files_matching_patterns(
            temp_path, 
            &[pattern.to_string()]
        )?;
        
        // Convert to sets for comparison
        let git_set: HashSet<PathBuf> = git_files.into_iter()
            .map(|p| p.strip_prefix(temp_path).unwrap().to_path_buf())
            .collect();
        let our_set: HashSet<PathBuf> = our_files.into_iter()
            .map(|p| p.strip_prefix(temp_path).unwrap().to_path_buf())
            .collect();
        
        // Compare results
        if git_set != our_set {
            println!("Mismatch for pattern: {}", pattern);
            println!("Git files: {:?}", git_set);
            println!("Our files: {:?}", our_set);
            
            let only_in_git: Vec<_> = git_set.difference(&our_set).collect();
            let only_in_ours: Vec<_> = our_set.difference(&git_set).collect();
            
            if !only_in_git.is_empty() {
                println!("Only in git: {:?}", only_in_git);
            }
            if !only_in_ours.is_empty() {
                println!("Only in ours: {:?}", only_in_ours);
            }
            
            panic!("Pattern matching mismatch for: {}", pattern);
        }
    }
    
    println!("All patterns matched successfully!");
    Ok(())
}

/// Create a comprehensive test file structure
fn create_test_files(base: &Path) -> Result<()> {
    // Create directories
    fs::create_dir_all(base.join("src/components"))?;
    fs::create_dir_all(base.join("src/utils"))?;
    fs::create_dir_all(base.join("build/debug"))?;
    fs::create_dir_all(base.join("target/release"))?;
    fs::create_dir_all(base.join("node_modules/package1"))?;
    fs::create_dir_all(base.join("lib/node_modules/package2"))?;
    fs::create_dir_all(base.join("docs"))?;
    fs::create_dir_all(base.join(".hidden"))?;
    
    // Create files
    let files = vec![
        "test.log",
        "debug.log",
        "important.log",  // Should be excluded by !important.log
        "README.md",
        "config.json",
        "temp.tmp",
        ".DS_Store",
        "src/main.rs",
        "src/test.log",
        "src/backup.bak",
        "src/components/App.jsx",
        "src/components/test.cache",
        "src/utils/helper.js",
        "src/utils/old.bak",
        "build/output.js",
        "build/debug/app.exe",
        "build/test.log",
        "target/debug.log",
        "target/release/app",
        "node_modules/package1/index.js",
        "node_modules/package1/test.log",
        "lib/node_modules/package2/index.js",
        "docs/manual.pdf",
        "docs/cache.cache",
        ".hidden/secret.txt",
        ".hidden/data.cache",
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

/// Get ignored files using git ls-files with a specific pattern
fn get_git_ignored_files_with_pattern(workdir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    // First, get all files in the repository
    let all_output = Command::new("git")
        .current_dir(workdir)
        .args(["ls-files", "-o", "--exclude-standard"])
        .output()?;
    
    if !all_output.status.success() {
        return Err(anyhow::anyhow!("git ls-files failed"));
    }
    
    let all_files = String::from_utf8_lossy(&all_output.stdout);
    let all_files_vec: Vec<String> = all_files
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Now check which files match the pattern using git check-ignore
    let mut matching_files = Vec::new();
    
    for file in all_files_vec {
        let check_output = Command::new("git")
            .current_dir(workdir)
            .args(["check-ignore", "-v", "-n", "--", &file])
            .output()?;
        
        if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);
            // Check if this file was matched by our specific pattern
            if output_str.contains(pattern) {
                matching_files.push(workdir.join(&file));
            }
        }
    }
    
    Ok(matching_files)
}

#[test]
fn test_gitignore_file_reading() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_path)
        .args(["init"])
        .output()?;
    
    // Create .gitignore
    fs::write(temp_path.join(".gitignore"), "*.log\ntarget/\n")?;
    
    // Create some files
    fs::write(temp_path.join("test.log"), "")?;
    fs::write(temp_path.join("app.rs"), "")?;
    fs::create_dir_all(temp_path.join("target/debug"))?;
    fs::write(temp_path.join("target/debug/app"), "")?;
    
    // Get git ignored files
    let git_ignored = dbx_ignore::utils::git_utils::get_git_ignored_files_in_path(temp_path)?;
    
    // Should include test.log and target/ directory
    let relative_paths: HashSet<PathBuf> = git_ignored.iter()
        .map(|p| p.strip_prefix(temp_path).unwrap().to_path_buf())
        .collect();
    
    assert!(relative_paths.contains(&PathBuf::from("test.log")));
    // Our implementation now returns files inside target/, not the directory itself
    assert!(relative_paths.iter().any(|p| p.starts_with("target/")));
    assert!(!relative_paths.contains(&PathBuf::from("app.rs")));
    
    Ok(())
}