mod common;

#[cfg(target_os = "macos")]
mod macos_tests {
    use super::common::TestEnvironment;
    use dbx_ignore::platforms::macos::MacOSHandler;
    use dbx_ignore::traits::PlatformHandler;
    use std::path::Path;

    #[test]
    fn test_macos_handler_basics() {
        assert_eq!(MacOSHandler::platform_name(), "macOS");
        assert!(MacOSHandler::is_supported());

        let attrs = MacOSHandler::get_target_attributes();
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&"com.dropbox.ignored"));
        assert!(attrs.contains(&"com.apple.fileprovider.ignore#P"));
    }

    #[test]
    fn test_has_attribute_on_nonexistent_file() {
        let nonexistent = Path::new("/tmp/nonexistent_test_file_12345");

        // Should handle nonexistent files gracefully
        let result = MacOSHandler::has_attribute(nonexistent, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_regular_file() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Regular files should not have these attributes by default
        let result = MacOSHandler::has_attribute(&test_file, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = MacOSHandler::has_attribute(&test_file, "com.apple.fileprovider.ignore#P");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    #[ignore = "remove_attribute no longer exists in the current implementation"]
    fn test_remove_attribute_on_file_without_attribute() {
        // This test is kept for historical reference but is no longer applicable
        // The current implementation only adds ignore markers, not removes them
    }

    #[test]
    fn test_has_attribute_with_invalid_attribute_name() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Test with an invalid/unusual attribute name
        let result = MacOSHandler::has_attribute(&test_file, "");
        // Empty attribute name might cause an error, which is acceptable
        match result {
            Ok(exists) => assert!(!exists),
            Err(_) => {} // Error is acceptable for invalid attribute names
        }

        let result = MacOSHandler::has_attribute(&test_file, "invalid.attribute.name");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_directory() {
        let env = TestEnvironment::new();
        let test_dir = env.create_dir("test_directory");

        // Directories should also not have these attributes by default
        let result = MacOSHandler::has_attribute(&test_dir, "com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_error_handling_for_permission_denied() {
        // Test with a path that might cause permission issues
        let restricted_path = Path::new("/System/Library/Kernels/kernel");

        if restricted_path.exists() {
            // This should handle permission errors gracefully
            let result = MacOSHandler::has_attribute(restricted_path, "com.dropbox.ignored");
            assert!(result.is_ok());
            // Should return false for permission errors (treated as "not found")
            assert!(!result.unwrap());
        }
    }
}
