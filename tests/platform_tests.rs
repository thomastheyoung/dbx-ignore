mod common;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use dbx_ignore::platforms::unsupported::UnsupportedHandler;
use dbx_ignore::traits::PlatformHandler;
use std::path::Path;

#[cfg(target_os = "macos")]
use dbx_ignore::platforms::macos::MacOSHandler;

#[cfg(target_os = "linux")]
use dbx_ignore::platforms::linux::LinuxHandler;

#[cfg(target_os = "windows")]
use dbx_ignore::platforms::windows::WindowsHandler;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
#[test]
fn test_unsupported_platform_handler() {
    // Test that unsupported platform handler behaves correctly
    assert_eq!(UnsupportedHandler::platform_name(), "Unsupported Platform");
    assert!(!UnsupportedHandler::is_supported());
    assert_eq!(UnsupportedHandler::get_target_attributes().len(), 0);

    let temp_path = Path::new("/tmp");

    // Should always return false for has_attribute
    assert!(!UnsupportedHandler::has_attribute(temp_path, "any.attr").unwrap());

    // Should be no-op for remove_attribute
    // Note: remove_attribute no longer exists in the current implementation
    // The current implementation only adds ignore markers, not removes them
    // assert!(UnsupportedHandler::remove_attribute(temp_path, "any.attr").is_ok());
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_platform_handler() {
    assert_eq!(MacOSHandler::platform_name(), "macOS");
    assert!(MacOSHandler::is_supported());

    let target_attrs = MacOSHandler::get_target_attributes();
    assert!(target_attrs.contains(&"com.dropbox.ignored"));
    assert!(target_attrs.contains(&"com.apple.fileprovider.ignore#P"));
    assert_eq!(target_attrs.len(), 2);
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_platform_handler() {
    assert_eq!(LinuxHandler::platform_name(), "Linux");
    assert!(LinuxHandler::is_supported());

    let target_attrs = LinuxHandler::get_target_attributes();
    assert!(target_attrs.contains(&"user.com.dropbox.ignored"));
    assert!(target_attrs.contains(&"user.com.apple.fileprovider.ignore#P"));
    assert_eq!(target_attrs.len(), 2);
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_platform_handler() {
    assert_eq!(WindowsHandler::platform_name(), "Windows");
    assert!(WindowsHandler::is_supported());

    let target_attrs = WindowsHandler::get_target_attributes();
    assert!(target_attrs.contains(&"com.dropbox.ignored"));
    assert!(target_attrs.contains(&"com.apple.fileprovider.ignore#P"));
    assert_eq!(target_attrs.len(), 2);
}

#[test]
fn test_current_platform_detection() {
    use dbx_ignore::platforms::{get_platform_info, CurrentPlatform};

    let (platform_name, is_supported) = get_platform_info();

    // Test that platform name is set correctly
    assert!(!platform_name.is_empty());

    // Test that current platform reports consistent information
    assert_eq!(platform_name, CurrentPlatform::platform_name());
    assert_eq!(is_supported, CurrentPlatform::is_supported());

    // Test that we have some target attributes on supported platforms
    if is_supported {
        assert!(!CurrentPlatform::get_target_attributes().is_empty());
    } else {
        assert_eq!(CurrentPlatform::get_target_attributes().len(), 0);
    }
}
