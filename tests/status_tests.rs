mod common;

use common::TestEnvironment;
use dbx_ignore::status::StatusInfo;
use std::fs;

#[test]
fn test_status_basic_info() {
    let env = TestEnvironment::new();
    
    // Create some test files
    env.create_file("test1.txt", "content1");
    env.create_file("test2.log", "content2");
    env.create_dir("subdir");
    
    // Change to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    // Gather status
    let status = StatusInfo::gather().unwrap();
    
    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Verify basic info
    assert_eq!(status.total_files, 3);
    assert!(!status.has_gitignore);
    assert!(status.daemon_status.is_none());
    assert_eq!(status.ignored_files.len(), 0);
    assert_eq!(status.non_ignored_files.len(), 3);
}

#[test]
fn test_status_with_gitignore() {
    let env = TestEnvironment::new();
    
    // Create .gitignore
    env.create_gitignore(&["*.log"]);
    env.create_file("test.txt", "content");
    env.create_file("test.log", "log content");
    
    // Change to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    // Gather status
    let status = StatusInfo::gather().unwrap();
    
    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Verify gitignore detected
    assert!(status.has_gitignore);
    assert_eq!(status.total_files, 2);
}

#[test]
fn test_status_with_ignored_files() {
    let env = TestEnvironment::new();
    
    // Create test files
    let file1 = env.create_file("ignored.txt", "content");
    env.create_file("normal.txt", "content");
    
    // Mark one file as ignored (platform-specific)
    #[cfg(target_os = "macos")]
    {
        use xattr;
        // Try both possible attributes for macOS
        let _ = xattr::set(&file1, "com.dropbox.ignored", b"1");
        let _ = xattr::set(&file1, "com.apple.fileprovider.ignore#P", b"1");
    }
    #[cfg(target_os = "linux")]
    {
        use xattr;
        xattr::set(&file1, "user.com.dropbox.ignored", b"1").ok();
    }
    
    // Change to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    // Gather status
    let status = StatusInfo::gather().unwrap();
    
    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Verify counts
    assert_eq!(status.total_files, 2);
    #[cfg(target_os = "macos")]
    {
        assert_eq!(status.ignored_files.len(), 1);
        assert_eq!(status.non_ignored_files.len(), 1);
    }
}

#[test] 
fn test_status_empty_directory() {
    let env = TestEnvironment::new();
    
    // Change to empty test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    // Gather status
    let status = StatusInfo::gather().unwrap();
    
    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Verify empty
    assert_eq!(status.total_files, 0);
    assert_eq!(status.ignored_files.len(), 0);
    assert_eq!(status.non_ignored_files.len(), 0);
}

#[test]
fn test_status_hidden_files_excluded() {
    let env = TestEnvironment::new();
    
    // Create visible and hidden files
    env.create_file("visible.txt", "content");
    fs::write(env.path().join(".hidden"), "hidden content").unwrap();
    fs::write(env.path().join(".gitignore"), "*.log").unwrap();
    
    // Change to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(env.path()).unwrap();
    
    // Gather status
    let status = StatusInfo::gather().unwrap();
    
    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Verify hidden files are excluded from count
    assert_eq!(status.total_files, 1);
    assert!(status.has_gitignore); // .gitignore is detected even though hidden
}