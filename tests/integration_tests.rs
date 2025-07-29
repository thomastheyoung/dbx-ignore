mod common;

use common::TestEnvironment;
use dbx_ignore::{Action, Config, run};
use serial_test::serial;
use std::path::PathBuf;

#[test]
fn test_config_creation() {
    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: false,
        files: vec![PathBuf::from("test.txt")],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    assert!(config.dry_run);
    assert!(!config.verbose);
    assert!(!config.quiet);
    assert!(!config.git_mode);
    assert_eq!(config.files.len(), 1);
    assert_eq!(config.files[0], PathBuf::from("test.txt"));
}

#[test]
fn test_run_with_empty_file_list() {
    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true, // Suppress output for tests
        files: vec![],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed even with empty file list when not in git mode
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
fn test_run_with_nonexistent_file() {
    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![PathBuf::from("/tmp/definitely_nonexistent_file_12345")],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should fail with nonexistent file
    let result = run(config);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_run_with_existing_files() {
    let env = TestEnvironment::new();
    let test_file1 = env.create_file("test1.txt", "content1");
    let test_file2 = env.create_file("test2.txt", "content2");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file1, test_file2],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed with existing files
    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_run_with_directory() {
    let env = TestEnvironment::new();
    let test_dir = env.create_dir("test_directory");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_dir],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed with directories
    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_run_with_mixed_files_and_directories() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");
    let test_dir = env.create_dir("test_directory");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file, test_dir],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed with mixed files and directories
    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_dry_run_vs_actual_run() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");

    // Test dry run
    let dry_config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file.clone()],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(dry_config);
    assert!(result.is_ok());

    // Test actual run (should also succeed even without attributes)
    let actual_config = Config {
        action: Action::Ignore,
        dry_run: false,
        verbose: false,
        quiet: true,
        files: vec![test_file],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    let result = run(actual_config);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_verbose_mode() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: true,
        quiet: false, // verbose overrides quiet behavior
        files: vec![test_file],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed in verbose mode
    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_quiet_mode() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    // Should succeed in quiet mode
    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[cfg(target_os = "macos")]
#[test]
#[serial]
fn test_run_on_supported_platform() {
    use dbx_ignore::platforms::CurrentPlatform;
    use dbx_ignore::traits::PlatformHandler;

    assert!(CurrentPlatform::is_supported());
    assert_eq!(CurrentPlatform::platform_name(), "macOS");

    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "content");

    let config = Config {
        action: Action::Ignore,
        dry_run: true,
        verbose: false,
        quiet: true,
        files: vec![test_file],
        patterns: vec![],
        git_mode: false,
        daemon_mode: false,
    };

    std::env::set_current_dir(&env.temp_path).unwrap();
    let result = run(config);
    assert!(result.is_ok());
}

#[test]
fn test_platform_detection_consistency() {
    use dbx_ignore::platforms::{CurrentPlatform, get_platform_info};
    use dbx_ignore::traits::PlatformHandler;

    let (platform_name, is_supported) = get_platform_info();

    assert_eq!(platform_name, CurrentPlatform::platform_name());
    assert_eq!(is_supported, CurrentPlatform::is_supported());

    if is_supported {
        assert!(!CurrentPlatform::get_target_attributes().is_empty());
    } else {
        assert_eq!(CurrentPlatform::get_target_attributes().len(), 0);
    }
}
