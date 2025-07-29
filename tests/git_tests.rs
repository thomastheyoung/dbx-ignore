mod common;

use common::TestEnvironment;
use dbx_ignore::{run, Config, Action};
use std::fs;
use serial_test::serial;

#[test]
#[serial]
fn test_git_mode_in_valid_repository() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Create some test files
    env.create_file("tracked.txt", "tracked content");
    env.create_file("ignored.txt", "ignored content");

    // Create .gitignore
    env.create_gitignore(&["ignored.txt", "*.tmp"]);

    // Test git mode
    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_git_mode_outside_repository() {
    let env = TestEnvironment::new();

    // Don't initialize git repository
    env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    // Should fail when not in a git repository
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_git_mode_with_empty_gitignore() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Create empty .gitignore
    env.create_gitignore(&[]);

    // Create test file
    env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    // Should succeed even with empty .gitignore
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_git_mode_with_no_gitignore() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Don't create .gitignore file
    env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    // Should succeed even without .gitignore
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_git_mode_with_complex_gitignore() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Create complex directory structure
    env.create_file("normal.txt", "normal");
    env.create_file("ignored.txt", "ignored");
    env.create_file("temp.tmp", "temporary");

    let subdir = env.create_dir("subdir");
    fs::write(subdir.join("file.txt"), "subdir content").unwrap();
    fs::write(subdir.join("ignored.log"), "log content").unwrap();

    // Create .gitignore with various patterns
    env.create_gitignore(&[
        "*.tmp",
        "ignored.txt",
        "*.log",
        "# This is a comment",
        "", // Empty line
        "build/",
    ]);

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_git_mode_with_negated_patterns() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Create test files
    env.create_file("ignored.txt", "ignored");
    env.create_file("not_ignored.txt", "not ignored");

    // Create .gitignore with negated patterns (should be skipped by our implementation)
    env.create_gitignore(&[
        "*.txt",
        "!not_ignored.txt", // Negated pattern - should be skipped
    ]);

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the test
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let result = run(config);

    // Restore original directory
    

    // Should succeed - negated patterns are skipped
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_mixed_mode_vs_git_mode() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    let test_file = env.create_file("test.txt", "content");
    env.create_gitignore(&["test.txt"]);

    // Test specific file mode
    let file_config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Test git mode
    let git_config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to temp directory for the tests
    
    std::env::set_current_dir(&env.temp_path).unwrap();

    let file_result = run(file_config);
    let git_result = run(git_config);

    // Restore original directory
    

    // Both should succeed
    assert!(file_result.is_ok());
    assert!(git_result.is_ok());
}

#[test]
#[serial]
fn test_git_repository_discovery() {
    let env = TestEnvironment::new();

    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");

    // Create subdirectory
    let subdir = env.create_dir("subdir");

    // Create .gitignore in root
    env.create_gitignore(&["*.tmp"]);

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![],
        patterns: vec![],
        git_mode: true,
        daemon_mode: false,
    };

    // Change to subdirectory and test git discovery
    
    std::env::set_current_dir(&subdir).unwrap();

    let result = run(config);

    // Restore original directory
    

    // Should succeed - git repository should be discovered from parent
    assert!(result.is_ok());
}
