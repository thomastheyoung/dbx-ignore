use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_prevent_marking_current_dir_without_gitignore() {
    let temp_dir = TempDir::new().unwrap();

    // Try to mark current directory without git repo
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg(".")
        .output()
        .expect("Failed to execute command");

    // Should fail with error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot mark entire directory without a .gitignore"));
    assert!(stderr.contains("This safeguard prevents accidentally marking all files"));
}

#[test]
fn test_prevent_marking_wildcard_without_gitignore() {
    let temp_dir = TempDir::new().unwrap();

    // Try to mark everything with wildcard
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("*")
        .output()
        .expect("Failed to execute command");

    // Should fail with error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot mark entire directory without a .gitignore"));
}

#[test]
fn test_allow_marking_current_dir_with_gitignore() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo and create .gitignore
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(["init"])
        .output()
        .expect("Failed to init git");

    std::fs::write(temp_dir.path().join(".gitignore"), "*.log").unwrap();

    // Now marking current directory should work
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg(".")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute command");

    // Should succeed
    assert!(output.status.success());
}

#[test]
fn test_allow_specific_files_without_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    // Marking specific files should always work
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("test.txt")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute command");

    // Should succeed
    assert!(output.status.success());
}

#[test]
fn test_validation_with_various_patterns() {
    let temp_dir = TempDir::new().unwrap();

    // Test various dangerous patterns
    let dangerous_patterns = vec![".", "*", "./", "./*"];

    for pattern in dangerous_patterns {
        let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
            .current_dir(temp_dir.path())
            .arg(pattern)
            .output()
            .expect("Failed to execute command");

        assert!(
            !output.status.success(),
            "Pattern '{}' should have failed",
            pattern
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Cannot mark entire directory"));
    }
}
