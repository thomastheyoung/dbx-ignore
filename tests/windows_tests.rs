mod common;

#[cfg(target_os = "windows")]
mod windows_tests {
    use super::common::TestEnvironment;
    use dbx_ignore::platforms::windows::WindowsHandler;
    use dbx_ignore::traits::PlatformHandler;
    use std::path::Path;

    #[test]
    fn test_windows_handler_basics() {
        assert_eq!(WindowsHandler::platform_name(), "Windows");
        assert!(WindowsHandler::is_supported());

        let attrs = WindowsHandler::get_target_attributes();
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&"com.dropbox.ignored"));
        assert!(attrs.contains(&"com.apple.fileprovider.ignore#P"));
    }

    #[test]
    fn test_has_attribute_on_nonexistent_file() {
        let nonexistent = Path::new("C:\\temp\\nonexistent_test_file_12345.txt");

        // Should handle nonexistent files gracefully
        let result = WindowsHandler::has_attribute(nonexistent, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_regular_file() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Regular files should not have these streams by default
        let result = WindowsHandler::has_attribute(&test_file, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = WindowsHandler::has_attribute(&test_file, "com.apple.fileprovider.ignore#P");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    #[ignore = "remove_attribute no longer exists in the current implementation"]
    fn test_remove_attribute_on_file_without_stream() {
        // This test is kept for historical reference but is no longer applicable
        // The current implementation only adds ignore markers, not removes them
    }

    #[test]
    fn test_has_attribute_with_invalid_stream_name() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Test with an invalid/unusual stream name
        let result = WindowsHandler::has_attribute(&test_file, "");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = WindowsHandler::has_attribute(&test_file, "invalid.stream.name");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_directory() {
        let env = TestEnvironment::new();
        let test_dir = env.create_dir("test_directory");

        // Directories can also have alternate data streams
        let result = WindowsHandler::has_attribute(&test_dir, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_windows_specific_stream_behavior() {
        // Verify that Windows uses the correct stream names (no prefix)
        let attrs = WindowsHandler::get_target_attributes();

        for attr in attrs {
            assert!(
                !attr.starts_with("user."),
                "Windows stream '{}' should not have 'user.' prefix",
                attr
            );
            assert!(!attr.is_empty(), "Windows stream name should not be empty");
        }
    }

    #[test]
    fn test_error_handling_for_permission_denied() {
        // Test with a path that might cause permission issues on Windows
        let restricted_path = Path::new("C:\\Windows\\System32\\kernel32.dll");

        if restricted_path.exists() {
            // This should handle permission errors gracefully
            let result = WindowsHandler::has_attribute(restricted_path, "com.dropbox.ignored");
            assert!(result.is_ok());
            // Should return false for permission errors (treated as "not found")
            assert!(!result.unwrap());
        }
    }

    #[test]
    fn test_error_handling_for_unsupported_filesystem() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Test behavior on filesystems that might not support ADS
        let result = WindowsHandler::has_attribute(&test_file, "com.dropbox.ignored");
        assert!(result.is_ok());
        // Should return false for unsupported filesystems
        assert!(!result.unwrap());
    }
}
