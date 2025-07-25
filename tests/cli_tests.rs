mod common;

use common::TestEnvironment;
use std::process::Command;

fn get_binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    path.push("target");
    path.push("release");
    path.push("dbx-ignore");
    path
}

#[test]
fn test_cli_help() {
    let output = Command::new(&get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Manage Dropbox ignore markers on files and directories"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--quiet"));
    assert!(stdout.contains("--git"));
    assert!(stdout.contains("--reset"));
    assert!(stdout.contains("--watch"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(&get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("dbx-ignore"));
}

#[test]
fn test_cli_dry_run_with_nonexistent_file() {
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", "/tmp/definitely_nonexistent_file_12345"])
        .output()
        .expect("Failed to execute binary");

    // Should fail with nonexistent file
    assert!(!output.status.success());
}

#[test]
fn test_cli_dry_run_git_mode() {
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", "--git"])
        .current_dir(".")
        .output()
        .expect("Failed to execute binary");

    // Should succeed in git mode (we're in a git repo)
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Dry run mode"));
    assert!(stdout.contains("Platform:"));
}

#[test]
fn test_cli_quiet_mode() {
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", "--quiet", "--git"])
        .current_dir(".")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Quiet mode should produce minimal output
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn test_cli_verbose_mode() {
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", "--verbose", "--git"])
        .current_dir(".")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Platform:"));
    assert!(stdout.contains("Mode:"));
}

#[test]
fn test_cli_with_existing_file() {
    let env = TestEnvironment::new();
    let test_file = env.create_file("test.txt", "test content");

    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", test_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("files would be processed"));
}

#[test]
fn test_cli_with_directory() {
    let env = TestEnvironment::new();
    let test_dir = env.create_dir("test_directory");

    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", test_dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("files would be processed"));
}

#[test]
fn test_cli_with_multiple_files() {
    let env = TestEnvironment::new();
    let test_file1 = env.create_file("test1.txt", "content1");
    let test_file2 = env.create_file("test2.txt", "content2");

    let output = Command::new(&get_binary_path())
        .args(&[
            "--dry-run",
            test_file1.to_str().unwrap(),
            test_file2.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("2 files would be processed"));
}

#[test]
fn test_cli_conflicting_flags() {
    // Test that CLI handles conflicting verbose and quiet flags appropriately
    // The behavior depends on implementation - some CLIs error, others use precedence
    let output = Command::new(&get_binary_path())
        .args(&["--verbose", "--quiet", "--dry-run", "--git"])
        .current_dir(".")
        .output()
        .expect("Failed to execute binary");

    // The binary should either:
    // 1. Error out (exit code != 0)
    // 2. Use one flag over the other (exit code == 0)
    // Both are acceptable behaviors

    // We don't assert on status.success() because both behaviors are valid
    // Just ensure the binary doesn't crash
    let _stdout = String::from_utf8(output.stdout).unwrap();
    let _stderr = String::from_utf8(output.stderr).unwrap();
}

#[test]
fn test_cli_default_git_mode() {
    // When no files are specified, should default to git mode
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run"])
        .current_dir(".")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("git-ignored files") || stdout.contains("Platform:"));
}

#[test]
fn test_cli_outside_git_repo() {
    let env = TestEnvironment::new();

    // Run in temp directory without git repo
    let output = Command::new(&get_binary_path())
        .args(&["--dry-run", "--git"])
        .current_dir(env.path())
        .output()
        .expect("Failed to execute binary");

    // Should fail when trying to use git mode outside a git repository
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("git") || stderr.contains("repository"));
}

#[test]
fn test_cli_binary_exists() {
    // Build first to ensure binary exists
    let build_result = std::process::Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to run cargo build");

    assert!(build_result.status.success(), "Cargo build failed");

    // Verify that the binary exists and is executable
    let binary_path = get_binary_path();
    assert!(
        binary_path.exists(),
        "Binary should exist at {}",
        binary_path.display()
    );

    // Try to run the binary with --help to ensure it's executable
    let output = Command::new(&get_binary_path()).arg("--help").output();

    assert!(output.is_ok(), "Binary should be executable");
}
