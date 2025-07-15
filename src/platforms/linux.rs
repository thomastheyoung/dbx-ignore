use crate::traits::PlatformHandler;
use anyhow::{Context, Result};
use std::path::Path;

pub struct LinuxHandler;

impl PlatformHandler for LinuxHandler {
    fn get_target_attributes() -> &'static [&'static str] {
        &[
            "user.com.dropbox.ignored",
            "user.com.apple.fileprovider.ignore#P",
        ]
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
                    _ => Err(anyhow::anyhow!("Failed to check xattr {}: {}", attr, e)),
                }
            }
        }
    }

    fn add_attribute(path: &Path, attr: &str) -> Result<()> {
        // Add the attribute with a simple marker value
        xattr::set(path, attr, b"1")
            .with_context(|| format!("Failed to add xattr {} to {}", attr, path.display()))
    }

    fn platform_name() -> &'static str {
        "Linux"
    }
}