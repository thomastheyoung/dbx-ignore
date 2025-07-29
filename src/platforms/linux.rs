use crate::traits::PlatformHandler;
use crate::utils::platform_utils;
use anyhow::{Context, Result};
use std::path::Path;

pub struct LinuxHandler;

impl PlatformHandler for LinuxHandler {
    fn get_target_attributes() -> &'static [&'static str] {
        &["com.dropbox.ignored"]
    }

    fn has_attribute(path: &Path, attr: &str) -> Result<bool> {
        match xattr::get(path, attr) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => {
                // Some filesystems don't support xattrs, treat as "not found"
                // Also handle permission errors gracefully
                match e.kind() {
                    std::io::ErrorKind::Other
                    | std::io::ErrorKind::PermissionDenied
                    | std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::InvalidInput => Ok(false), // ENODATA on Linux
                    _ => platform_utils::handle_attribute_check_error(e, attr),
                }
            }
        }
    }

    fn add_attribute(path: &Path, attr: &str) -> Result<()> {
        // Add the attribute with a simple marker value
        xattr::set(path, attr, b"1")
            .with_context(|| format!("Failed to add xattr {} to {}", attr, path.display()))
    }

    fn remove_attribute(path: &Path, attr: &str) -> Result<()> {
        match xattr::remove(path, attr) {
            Ok(()) => Ok(()),
            Err(e) => {
                // If the attribute doesn't exist, that's fine
                match e.kind() {
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput => Ok(()), // ENODATA on Linux
                    _ => platform_utils::handle_attribute_remove_error(e, attr, path),
                }
            }
        }
    }

    fn platform_name() -> &'static str {
        "Linux"
    }
}
