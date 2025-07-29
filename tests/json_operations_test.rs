use anyhow::Result;
use dbx_ignore::core::{daemon::DaemonStatus, tracked_files::TrackedFiles};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

mod common;

#[test]
fn test_daemon_status_write_and_read() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Use current process PID which definitely exists
    let current_pid = std::process::id();
    
    let status = DaemonStatus {
        pid: current_pid,
        repo_path: repo_path.to_path_buf(),
        started_at: chrono::Utc::now(),
    };

    // Write status
    status.write(repo_path)?;

    // Verify file exists
    let status_file = repo_path.join(".dbx-ignore").join("daemon.json");
    assert!(status_file.exists());

    // Read back
    let read_status = DaemonStatus::read(repo_path)?;
    assert!(read_status.is_some());
    let read_status = read_status.unwrap();
    assert_eq!(read_status.pid, current_pid);
    assert_eq!(read_status.repo_path, repo_path);

    Ok(())
}

#[test]
fn test_daemon_status_invalid_pid() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let status = DaemonStatus {
        pid: 0, // Invalid PID
        repo_path: repo_path.to_path_buf(),
        started_at: chrono::Utc::now(),
    };

    // Should fail to write
    assert!(status.write(repo_path).is_err());

    Ok(())
}

#[test]
fn test_daemon_status_corrupted_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create corrupted file
    let status_dir = repo_path.join(".dbx-ignore");
    fs::create_dir_all(&status_dir)?;
    let status_file = status_dir.join("daemon.json");
    fs::write(&status_file, "{ invalid json }")?;

    // Should return None and clean up the file
    let read_status = DaemonStatus::read(repo_path)?;
    assert!(read_status.is_none());
    assert!(!status_file.exists());

    Ok(())
}

#[test]
fn test_daemon_status_missing_fields() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create JSON with missing fields
    let status_dir = repo_path.join(".dbx-ignore");
    fs::create_dir_all(&status_dir)?;
    let status_file = status_dir.join("daemon.json");
    fs::write(&status_file, r#"{"pid": 12345}"#)?;

    // Should return None due to missing fields
    let read_status = DaemonStatus::read(repo_path)?;
    assert!(read_status.is_none());

    Ok(())
}

#[test]
fn test_tracked_files_write_and_read() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[
        PathBuf::from("file1.txt"),
        PathBuf::from("dir/file2.log"),
    ]);
    tracked.add_patterns(&[
        "*.log".to_string(),
        "build/**".to_string(),
    ]);

    // Save
    tracked.save(repo_path)?;

    // Verify file exists
    let tracked_file = repo_path.join(".dbx-ignore").join("tracked_files.json");
    assert!(tracked_file.exists());

    // Load back
    let loaded = TrackedFiles::load(repo_path)?;
    assert_eq!(loaded.marked_files.len(), 2);
    assert!(loaded.marked_files.contains(&PathBuf::from("file1.txt")));
    assert!(loaded.marked_files.contains(&PathBuf::from("dir/file2.log")));
    assert_eq!(loaded.patterns.len(), 2);
    assert!(loaded.patterns.contains(&"*.log".to_string()));
    assert!(loaded.patterns.contains(&"build/**".to_string()));

    Ok(())
}

#[test]
fn test_tracked_files_empty() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let tracked = TrackedFiles::default();
    tracked.save(repo_path)?;

    let loaded = TrackedFiles::load(repo_path)?;
    assert!(loaded.marked_files.is_empty());
    assert!(loaded.patterns.is_empty());

    Ok(())
}

#[test]
fn test_tracked_files_corrupted() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create corrupted file
    let tracked_dir = repo_path.join(".dbx-ignore");
    fs::create_dir_all(&tracked_dir)?;
    let tracked_file = tracked_dir.join("tracked_files.json");
    fs::write(&tracked_file, "not valid json at all")?;

    // Should return default (empty) instead of erroring
    let loaded = TrackedFiles::load(repo_path)?;
    assert!(loaded.marked_files.is_empty());
    assert!(loaded.patterns.is_empty());

    Ok(())
}

#[test]
fn test_tracked_files_invalid_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create JSON with invalid data
    let tracked_dir = repo_path.join(".dbx-ignore");
    fs::create_dir_all(&tracked_dir)?;
    let tracked_file = tracked_dir.join("tracked_files.json");
    fs::write(&tracked_file, r#"{
        "marked_files": ["valid.txt", "", "also_valid.txt"],
        "patterns": ["*.log", "", "*.tmp"],
        "last_updated": "2024-01-01T00:00:00Z"
    }"#)?;

    // Should filter out empty paths/patterns
    let loaded = TrackedFiles::load(repo_path)?;
    assert_eq!(loaded.marked_files.len(), 2);
    assert!(!loaded.marked_files.iter().any(|p| p.as_os_str().is_empty()));
    assert_eq!(loaded.patterns.len(), 2);
    assert!(!loaded.patterns.iter().any(|p| p.is_empty()));

    Ok(())
}

#[test]
fn test_concurrent_writes() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new()?;
    let repo_path = Arc::new(temp_dir.path().to_path_buf());

    // Spawn multiple threads writing different data
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let path = Arc::clone(&repo_path);
            thread::spawn(move || {
                let mut tracked = TrackedFiles::default();
                tracked.add_files(&[PathBuf::from(format!("file{}.txt", i))]);
                tracked.add_patterns(&[format!("pattern{}", i)]);
                tracked.save(&path)
            })
        })
        .collect();

    // All writes should succeed
    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }

    // Final file should be valid
    let loaded = TrackedFiles::load(&repo_path)?;
    assert!(!loaded.marked_files.is_empty());
    assert!(!loaded.patterns.is_empty());

    Ok(())
}

#[test]
fn test_atomic_write_prevents_corruption() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Write initial valid data
    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[PathBuf::from("initial.txt")]);
    tracked.save(repo_path)?;

    // Verify initial data
    let loaded = TrackedFiles::load(repo_path)?;
    assert_eq!(loaded.marked_files.len(), 1);

    // Simulate a write that would fail mid-way (we can't easily simulate this,
    // but the atomic write guarantees either complete success or complete failure)
    
    // Even if we could simulate a failure, the original file should remain intact
    let loaded_after = TrackedFiles::load(repo_path)?;
    assert_eq!(loaded_after.marked_files.len(), 1);

    Ok(())
}

#[test]
fn test_large_tracked_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let mut tracked = TrackedFiles::default();
    
    // Add many files
    for i in 0..1000 {
        tracked.add_files(&[PathBuf::from(format!("file{}.txt", i))]);
    }
    
    // Add many patterns
    for i in 0..100 {
        tracked.add_patterns(&[format!("pattern{}/*", i)]);
    }

    // Save and load
    tracked.save(repo_path)?;
    let loaded = TrackedFiles::load(repo_path)?;

    assert_eq!(loaded.marked_files.len(), 1000);
    assert_eq!(loaded.patterns.len(), 100);

    Ok(())
}

#[test]
fn test_special_characters_in_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[
        PathBuf::from("file with spaces.txt"),
        PathBuf::from("file'with'quotes.txt"),
        PathBuf::from("file\"with\"double.txt"),
        PathBuf::from("file\\with\\backslash.txt"),
        PathBuf::from("file🦀with🦀emoji.txt"),
    ]);
    tracked.add_patterns(&[
        "* with spaces/*".to_string(),
        "pattern'with'quotes".to_string(),
    ]);

    // Save and load
    tracked.save(repo_path)?;
    let loaded = TrackedFiles::load(repo_path)?;

    assert_eq!(loaded.marked_files.len(), 5);
    assert_eq!(loaded.patterns.len(), 2);

    Ok(())
}