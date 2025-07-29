mod common;

use common::TestEnvironment;
use std::process::Command;


#[test]
fn test_wildcard_single_pattern() {
    let env = TestEnvironment::new();
    let _test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _test3 = env.create_file("test3.md", "content");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run"])  // Remove --quiet to see more output
        .arg(env.path().join("*.txt").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    if !output.status.success() {
        eprintln!("Command failed!");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr (full): {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Pattern: {}", env.path().join("*.txt").display());
        
        // List files in the temp directory
        eprintln!("\nFiles in temp directory:");
        for entry in std::fs::read_dir(env.path()).unwrap() {
            let entry = entry.unwrap();
            eprintln!("  {}", entry.path().display());
        }
    }
    assert!(output.status.success());
}

#[test]
fn test_wildcard_multiple_patterns() {
    let env = TestEnvironment::new();
    let _test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _test3 = env.create_file("test3.md", "content");
    let _test4 = env.create_file("test4.md", "content");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--quiet"])
        .arg(env.path().join("*.txt").to_str().unwrap())
        .arg(env.path().join("*.md").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
}

#[test]
fn test_wildcard_all_files() {
    let env = TestEnvironment::new();
    let _test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _dir = env.create_dir("testdir");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--quiet"])
        .arg(env.path().join("*").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
}

#[test]
fn test_wildcard_no_matches_error() {
    let env = TestEnvironment::new();
    // Don't create any files

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run"])
        .arg(env.path().join("*.nonexistent").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("No files found matching pattern"));
}

#[test]
fn test_wildcard_with_subdirectory() {
    let env = TestEnvironment::new();
    let subdir = env.create_dir("subdir");

    // Create files in subdirectory
    std::fs::write(subdir.join("file1.rs"), "content").unwrap();
    std::fs::write(subdir.join("file2.rs"), "content").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--verbose"])
        .arg(env.path().join("subdir/*.rs").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("file1.rs"));
    assert!(stdout.contains("file2.rs"));
}

#[test]
fn test_mixed_wildcard_and_literal() {
    let env = TestEnvironment::new();
    let test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _test3 = env.create_file("test3.md", "content");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--verbose"])
        .arg(test1.to_str().unwrap()) // literal path
        .arg(env.path().join("*.md").to_str().unwrap()) // wildcard
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test1.txt"));
    assert!(stdout.contains("test3.md"));
    assert!(stdout.contains("2 files would be processed"));
}

#[test]
fn test_wildcard_question_mark() {
    let env = TestEnvironment::new();
    let _test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _test = env.create_file("test.txt", "content");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--verbose"])
        .arg(env.path().join("test?.txt").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test1.txt"));
    assert!(stdout.contains("test2.txt"));
    assert!(!stdout.contains("test.txt")); // shouldn't match
}

#[test]
fn test_wildcard_brackets() {
    let env = TestEnvironment::new();
    let _test1 = env.create_file("test1.txt", "content");
    let _test2 = env.create_file("test2.txt", "content");
    let _test3 = env.create_file("test3.txt", "content");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--verbose"])
        .arg(env.path().join("test[12].txt").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test1.txt"));
    assert!(stdout.contains("test2.txt"));
    assert!(!stdout.contains("test3.txt")); // shouldn't match
}

#[test]
fn test_wildcard_folders() {
    let env = TestEnvironment::new();
    let _build1 = env.create_dir("build1");
    let _build2 = env.create_dir("build2");
    let _build3 = env.create_dir("build3");
    let _src = env.create_dir("src");

    let output = Command::new(env!("CARGO_BIN_EXE_dbx-ignore"))
        .args(["--dry-run", "--verbose"])
        .arg(env.path().join("build*").to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("build1"));
    assert!(stdout.contains("build2"));
    assert!(stdout.contains("build3"));
    assert!(!stdout.contains("src")); // shouldn't match
    assert!(stdout.contains("3 files would be processed"));
}
