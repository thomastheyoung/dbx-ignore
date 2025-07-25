use crate::traits::PlatformHandler;
use crate::utils::platform_utils;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::OnceLock;

pub struct MacOSHandler;

/// Cached result of File Provider detection
static IS_FILE_PROVIDER: OnceLock<bool> = OnceLock::new();

/// Detect if Dropbox is using the File Provider API
fn is_using_file_provider() -> bool {
    *IS_FILE_PROVIDER.get_or_init(|| {
        // Check if the user's home directory contains the CloudStorage path
        if let Ok(home) = std::env::var("HOME") {
            let cloud_storage_path = format!("{}/Library/CloudStorage", home);
            if std::path::Path::new(&cloud_storage_path).exists() {
                // Check for Dropbox folder in CloudStorage
                let dropbox_patterns = ["Dropbox", "Dropbox (Personal)", "Dropbox (Business)"];

                for pattern in &dropbox_patterns {
                    let dropbox_path = format!("{}/{}", cloud_storage_path, pattern);
                    if std::path::Path::new(&dropbox_path).exists() {
                        return true;
                    }
                }
            }
        }
        false
    })
}

impl PlatformHandler for MacOSHandler {
    fn get_target_attributes() -> &'static [&'static str] {
        // Return both for compatibility, but we'll only use the appropriate one
        &["com.dropbox.ignored", "com.apple.fileprovider.ignore#P"]
    }

    fn has_attribute(path: &Path, attr: &str) -> Result<bool> {
        // Only check for the appropriate attribute based on File Provider detection
        let should_check = if is_using_file_provider() {
            attr == "com.apple.fileprovider.ignore#P"
        } else {
            attr == "com.dropbox.ignored"
        };

        if !should_check {
            // If it's not the appropriate attribute for this system, consider it as not present
            return Ok(false);
        }

        match xattr::get(path, attr) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => {
                // Some filesystems don't support xattrs, treat as "not found"
                // Also handle permission errors gracefully
                match e.kind() {
                    std::io::ErrorKind::Other
                    | std::io::ErrorKind::PermissionDenied
                    | std::io::ErrorKind::NotFound => Ok(false),
                    _ => platform_utils::handle_attribute_check_error(e, attr),
                }
            }
        }
    }

    fn add_attribute(path: &Path, attr: &str) -> Result<()> {
        // Only add the appropriate attribute based on File Provider detection
        let should_add = if is_using_file_provider() {
            attr == "com.apple.fileprovider.ignore#P"
        } else {
            attr == "com.dropbox.ignored"
        };

        if should_add {
            xattr::set(path, attr, b"1")
                .with_context(|| format!("Failed to add xattr {} to {}", attr, path.display()))
        } else {
            // Silently skip the inappropriate attribute
            Ok(())
        }
    }

    fn remove_attribute(path: &Path, attr: &str) -> Result<()> {
        // Only remove the appropriate attribute based on File Provider detection
        let should_remove = if is_using_file_provider() {
            attr == "com.apple.fileprovider.ignore#P"
        } else {
            attr == "com.dropbox.ignored"
        };

        if should_remove {
            match xattr::remove(path, attr) {
                Ok(()) => Ok(()),
                Err(e) => platform_utils::handle_attribute_remove_error(e, attr, path)
            }
        } else {
            // Silently skip the inappropriate attribute
            Ok(())
        }
    }

    fn platform_name() -> &'static str {
        "macOS"
    }
}
