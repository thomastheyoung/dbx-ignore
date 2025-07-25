mod common;

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::common::TestEnvironment;
    use dbx_ignore::platforms::linux::LinuxHandler;
    use dbx_ignore::traits::PlatformHandler;
    use std::path::Path;

    #[test]
    fn test_linux_handler_basics() {
        assert_eq!(LinuxHandler::platform_name(), "Linux");
        assert!(LinuxHandler::is_supported());

        let attrs = LinuxHandler::get_target_attributes();
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&"user.com.dropbox.ignored"));
        assert!(attrs.contains(&"user.com.apple.fileprovider.ignore#P"));
    }

    #[test]
    fn test_has_attribute_on_nonexistent_file() {
        let nonexistent = Path::new("/tmp/nonexistent_test_file_12345");

        // Should handle nonexistent files gracefully
        let result = LinuxHandler::has_attribute(nonexistent, "user.com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_regular_file() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Regular files should not have these attributes by default
        let result = LinuxHandler::has_attribute(&test_file, "user.com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result =
            LinuxHandler::has_attribute(&test_file, "user.com.apple.fileprovider.ignore#P");
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
        let result = LinuxHandler::has_attribute(&test_file, "");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = LinuxHandler::has_attribute(&test_file, "user.invalid.attribute.name");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_attribute_on_directory() {
        let env = TestEnvironment::new();
        let test_dir = env.create_dir("test_directory");

        // Directories should also not have these attributes by default
        let result = LinuxHandler::has_attribute(&test_dir, "user.com.dropbox.ignored");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_linux_specific_user_prefix() {
        // Verify that Linux uses the correct user.* prefix for extended attributes
        let attrs = LinuxHandler::get_target_attributes();

        for attr in attrs {
            assert!(
                attr.starts_with("user."),
                "Linux extended attribute '{}' should start with 'user.' prefix",
                attr
            );
        }
    }

    #[test]
    fn test_error_handling_for_unsupported_filesystem() {
        let env = TestEnvironment::new();
        let test_file = env.create_file("test.txt", "test content");

        // Test with attribute that might not be supported on all filesystems
        let result = LinuxHandler::has_attribute(&test_file, "user.com.dropbox.ignored");
        assert!(result.is_ok());
        // Should return false for unsupported filesystems (treated as "not found")
        assert!(!result.unwrap());
    }
}
