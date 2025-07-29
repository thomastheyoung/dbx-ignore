use dbx_ignore::tracked_files::TrackedFiles;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_tracked_files_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path();

    // Create and save tracked files
    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[
        PathBuf::from("file1.txt"),
        PathBuf::from("file2.log"),
        PathBuf::from("dir/file3.rs"),
    ]);

    tracked.save(test_path).unwrap();

    // Load and verify
    let loaded = TrackedFiles::load(test_path).unwrap();
    assert_eq!(loaded.marked_files.len(), 3);
    assert!(loaded.is_tracked(&PathBuf::from("file1.txt")));
    assert!(loaded.is_tracked(&PathBuf::from("file2.log")));
    assert!(loaded.is_tracked(&PathBuf::from("dir/file3.rs")));
}

#[test]
fn test_tracked_files_add_remove() {
    let mut tracked = TrackedFiles::default();

    // Add files
    let files = vec![
        PathBuf::from("test1.txt"),
        PathBuf::from("test2.txt"),
        PathBuf::from("test3.txt"),
    ];
    tracked.add_files(&files);

    assert_eq!(tracked.marked_files.len(), 3);
    assert!(tracked.is_tracked(&files[0]));

    // Remove a file
    tracked.remove_files(&[files[1].clone()]);
    assert_eq!(tracked.marked_files.len(), 2);
    assert!(!tracked.is_tracked(&files[1]));
    assert!(tracked.is_tracked(&files[0]));
    assert!(tracked.is_tracked(&files[2]));
}

#[test]
fn test_tracked_files_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path();

    // Create tracked files with patterns
    let mut tracked = TrackedFiles::default();
    tracked.add_patterns(&[
        "*.log".to_string(),
        "build/**".to_string(),
        "*.tmp".to_string(),
    ]);

    // Verify patterns are stored
    assert_eq!(tracked.patterns.len(), 3);
    assert!(tracked.patterns.contains(&"*.log".to_string()));

    // Save and reload
    tracked.save(test_path).unwrap();
    let loaded = TrackedFiles::load(test_path).unwrap();

    // Verify patterns persist
    assert_eq!(loaded.patterns.len(), 3);
    assert!(loaded.patterns.contains(&"*.log".to_string()));
    assert!(loaded.patterns.contains(&"build/**".to_string()));
    assert!(loaded.patterns.contains(&"*.tmp".to_string()));
}

#[test]
fn test_tracked_files_remove_patterns() {
    let mut tracked = TrackedFiles::default();

    // Add patterns
    tracked.add_patterns(&[
        "*.log".to_string(),
        "*.tmp".to_string(),
        "*.cache".to_string(),
    ]);
    assert_eq!(tracked.patterns.len(), 3);

    // Remove some patterns
    tracked.remove_patterns(&["*.tmp".to_string(), "*.cache".to_string()]);

    // Verify only *.log remains
    assert_eq!(tracked.patterns.len(), 1);
    assert!(tracked.patterns.contains(&"*.log".to_string()));
    assert!(!tracked.patterns.contains(&"*.tmp".to_string()));
}

#[test]
fn test_tracked_files_empty_load() {
    let temp_dir = TempDir::new().unwrap();

    // Load from non-existent file
    let tracked = TrackedFiles::load(temp_dir.path()).unwrap();
    assert_eq!(tracked.marked_files.len(), 0);
}

#[test]
fn test_tracked_files_state_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path();

    // Save creates .dbx-ignore directory
    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[PathBuf::from("test.txt")]);
    tracked.save(test_path).unwrap();

    // Verify file exists
    let state_file = test_path.join(".dbx-ignore").join("tracked_files.json");
    assert!(state_file.exists());

    // Verify JSON content
    let content = std::fs::read_to_string(&state_file).unwrap();
    assert!(content.contains("test.txt"));
    assert!(content.contains("marked_files"));
    assert!(content.contains("last_updated"));
}

#[test]
fn test_tracked_files_remove_state_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path();

    // Create state file
    let mut tracked = TrackedFiles::default();
    tracked.add_files(&[PathBuf::from("test.txt")]);
    tracked.save(test_path).unwrap();

    let state_file = test_path.join(".dbx-ignore").join("tracked_files.json");
    assert!(state_file.exists());

    // Remove state file
    TrackedFiles::remove_state_file(test_path).unwrap();
    assert!(!state_file.exists());
}

#[test]
fn test_tracked_files_duplicate_handling() {
    let mut tracked = TrackedFiles::default();

    // Add same file multiple times
    let file = PathBuf::from("duplicate.txt");
    tracked.add_files(&[file.clone(), file.clone()]);

    // Should only be tracked once
    assert_eq!(tracked.marked_files.len(), 1);
}
