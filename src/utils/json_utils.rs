use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Atomically write JSON data to a file
///
/// This function ensures that the file is either fully written or not written at all,
/// preventing partial writes that could corrupt the JSON file.
pub fn write_json_atomic<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create parent directory")?;
    }

    // Create a temporary file in the same directory
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir).context("Failed to create temporary file")?;

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(data).context("Failed to serialize to JSON")?;

    // Validate the JSON by parsing it back
    let _: serde_json::Value = serde_json::from_str(&json).context("Generated invalid JSON")?;

    // Write to temporary file
    temp_file
        .write_all(json.as_bytes())
        .context("Failed to write to temporary file")?;

    // Ensure all data is flushed to disk
    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    // Sync to ensure durability
    temp_file
        .as_file()
        .sync_all()
        .context("Failed to sync temporary file")?;

    // Atomically rename temp file to target path
    temp_file
        .persist(path)
        .context("Failed to persist file atomically")?;

    Ok(())
}

/// Read and deserialize JSON data from a file with validation
pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Try to parse as generic JSON first for better error messages
    let json_value: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("Invalid JSON in file: {}", path.display()))?;

    // Then deserialize to the target type
    serde_json::from_value(json_value)
        .with_context(|| format!("JSON schema mismatch in file: {}", path.display()))
}

/// Read JSON with a fallback value if the file doesn't exist or is invalid
pub fn read_json_or_default<T>(path: &Path) -> T
where
    T: for<'de> Deserialize<'de> + Default,
{
    read_json(path).unwrap_or_default()
}

/// Validate that a JSON file can be parsed as a specific type
pub fn validate_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<()> {
    let _: T = read_json(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        values: Vec<i32>,
    }

    #[test]
    fn test_write_and_read_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let data = TestData {
            id: 42,
            name: "test".to_string(),
            values: vec![1, 2, 3],
        };

        // Write
        write_json_atomic(&file_path, &data).unwrap();

        // Read back
        let read_data: TestData = read_json(&file_path).unwrap();
        assert_eq!(data, read_data);
    }

    #[test]
    fn test_atomic_write_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Write initial data
        let initial_data = TestData {
            id: 1,
            name: "initial".to_string(),
            values: vec![1],
        };
        write_json_atomic(&file_path, &initial_data).unwrap();

        // Create a type that will fail to serialize
        #[derive(Serialize)]
        struct BadData {
            #[serde(serialize_with = "fail_serializer")]
            bad_field: String,
        }

        fn fail_serializer<S>(_: &str, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("intentional failure"))
        }

        let bad_data = BadData {
            bad_field: "test".to_string(),
        };

        // Attempt to write bad data - should fail
        let result = write_json_atomic(&file_path, &bad_data);
        assert!(result.is_err());

        // Original file should still be intact
        let read_data: TestData = read_json(&file_path).unwrap();
        assert_eq!(initial_data, read_data);
    }

    #[test]
    fn test_read_corrupted_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("corrupted.json");

        // Write corrupted JSON
        fs::write(&file_path, "{ invalid json }").unwrap();

        // Should fail to read
        let result: Result<TestData> = read_json(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_read_json_or_default() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        #[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
        struct ConfigData {
            enabled: bool,
            items: Vec<String>,
        }

        // Should return default when file doesn't exist
        let data: ConfigData = read_json_or_default(&file_path);
        assert_eq!(data, ConfigData::default());

        // Write corrupted data
        fs::write(&file_path, "invalid").unwrap();

        // Should still return default
        let data: ConfigData = read_json_or_default(&file_path);
        assert_eq!(data, ConfigData::default());
    }

    #[test]
    fn test_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested/dir/test.json");

        let data = TestData {
            id: 99,
            name: "nested".to_string(),
            values: vec![],
        };

        // Should create parent directories
        write_json_atomic(&file_path, &data).unwrap();
        assert!(file_path.exists());

        let read_data: TestData = read_json(&file_path).unwrap();
        assert_eq!(data, read_data);
    }

    #[test]
    fn test_concurrent_writes() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let file_path = Arc::new(temp_dir.path().join("concurrent.json"));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let path = Arc::clone(&file_path);
                thread::spawn(move || {
                    let data = TestData {
                        id: i,
                        name: format!("thread-{}", i),
                        values: vec![i as i32],
                    };
                    write_json_atomic(&path, &data)
                })
            })
            .collect();

        // All writes should succeed
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }

        // File should be valid JSON
        let result: Result<TestData> = read_json(&file_path);
        assert!(result.is_ok());
    }
}
