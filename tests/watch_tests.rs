use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;
use tempfile::TempDir;

#[test]
fn test_watch_daemon_startup() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init"])
        .output()
        .expect("Failed to init git");
    
    // Start watch daemon
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--watch")
        .output()
        .expect("Failed to execute command");
    
    // Should succeed
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Started daemon watcher"));
    assert!(stdout.contains("PID:"));
    
    // Clean up - stop the daemon
    let _ = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--unwatch")
        .output();
}

#[test]
fn test_watch_requires_tracked_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize git repo and create files
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init"])
        .output()
        .expect("Failed to init git");
        
    std::fs::write(temp_dir.path().join(".gitignore"), "*.log").unwrap();
    std::fs::write(temp_dir.path().join("test.log"), "content").unwrap();
    
    // Start daemon in foreground mode to see output
    let mut child = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .args(&["--watch", "--daemon-mode"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start daemon");
    
    // Give it time to start
    thread::sleep(Duration::from_millis(500));
    
    // Kill the daemon
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No files are being tracked"));
}

#[test]
fn test_unwatch_daemon() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init"])
        .output()
        .expect("Failed to init git");
    
    // Start daemon
    Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--watch")
        .output()
        .expect("Failed to start daemon");
    
    // Stop daemon
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--unwatch")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Stopped daemon watcher"));
}

#[test]
fn test_watch_prevents_duplicate_daemons() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize git repo
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init"])
        .output()
        .expect("Failed to init git");
    
    // Start first daemon
    Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--watch")
        .output()
        .expect("Failed to start daemon");
    
    // Try to start second daemon
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--watch")
        .output()
        .expect("Failed to execute command");
    
    // Should succeed but warn about existing daemon
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daemon is already watching"));
    
    // Clean up
    let _ = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--unwatch")
        .output();
}

#[test]
fn test_unwatch_without_daemon() {
    let temp_dir = TempDir::new().unwrap();
    
    // Try to stop non-existent daemon
    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .current_dir(temp_dir.path())
        .arg("--unwatch")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No active daemon found"));
}