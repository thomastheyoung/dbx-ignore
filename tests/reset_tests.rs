mod common;

use common::TestEnvironment;
use dbx_ignore::{run, Config, Action};

#[test]
fn test_reset_removes_markers() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");
    
    // First, add ignore markers
    let ignore_config = Config {
        action: Action::Ignore,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        git_mode: false,
        daemon_mode: false,
    };
    
    let result = run(ignore_config);
    assert!(result.is_ok());
    
    // Then reset them
    let reset_config = Config {
        action: Action::Reset,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        git_mode: false,
        daemon_mode: false,
    };
    
    let result = run(reset_config);
    assert!(result.is_ok());
    
    // Verify markers are removed (platform-specific check)
    #[cfg(target_os = "macos")]
    {
        use xattr;
        let attrs = xattr::list(&test_file).unwrap();
        let attr_vec: Vec<_> = attrs.collect();
        assert!(!attr_vec.iter().any(|a| a.to_str().unwrap().contains("dropbox")));
    }
}

#[test]
fn test_reset_on_unmarked_file() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("unmarked.txt", "content");
    
    // Try to reset a file that has no markers
    let config = Config {
        action: Action::Reset,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![test_file],
        git_mode: false,
        daemon_mode: false,
    };
    
    let result = run(config);
    assert!(result.is_ok()); // Should succeed with no-op
}

#[test]
fn test_reset_multiple_files() {
    let env = TestEnvironment::new();
    let file1 = env.create_file("file1.txt", "content1");
    let file2 = env.create_file("file2.txt", "content2");
    let file3 = env.create_file("file3.txt", "content3");
    
    // Mark all files first
    let ignore_config = Config {
        action: Action::Ignore,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![file1.clone(), file2.clone(), file3.clone()],
        git_mode: false,
        daemon_mode: false,
    };
    
    run(ignore_config).unwrap();
    
    // Reset all files
    let reset_config = Config {
        action: Action::Reset,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![file1, file2, file3],
        git_mode: false,
        daemon_mode: false,
    };
    
    let result = run(reset_config);
    assert!(result.is_ok());
}

#[test]
fn test_reset_git_mode() {
    let env = TestEnvironment::new();
    
    // Initialize git repository
    let _repo = env.init_git_repo().expect("Failed to init git repo");
    
    // Create files and .gitignore
    let ignored_file = env.create_file("ignored.log", "log content");
    env.create_file("normal.txt", "normal content");
    env.create_gitignore(&["*.log"]);
    
    // First mark git-ignored files
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    let ignore_config = Config {
        action: Action::Ignore,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![],
        git_mode: true,
        daemon_mode: false,
    };
    
    run(ignore_config).unwrap();
    
    // Then reset git-ignored files
    let reset_config = Config {
        action: Action::Reset,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![],
        git_mode: true,
        daemon_mode: false,
    };
    
    let result = run(reset_config);
    
    std::env::set_current_dir(original_dir).unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_reset_dry_run() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");
    
    // Add marker first
    let ignore_config = Config {
        action: Action::Ignore,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        git_mode: false,
        daemon_mode: false,
    };
    
    run(ignore_config).unwrap();
    
    // Reset with dry run
    let reset_config = Config {
        action: Action::Reset,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        git_mode: false,
        daemon_mode: false,
    };
    
    let result = run(reset_config);
    assert!(result.is_ok());
    
    // Verify marker is still there (dry run shouldn't remove it)
    #[cfg(target_os = "macos")]
    {
        use xattr;
        let attrs = xattr::list(&test_file).unwrap();
        let attr_vec: Vec<_> = attrs.collect();
        assert!(attr_vec.iter().any(|a| a.to_str().unwrap().contains("dropbox") || 
                                      a.to_str().unwrap().contains("fileprovider")));
    }
}