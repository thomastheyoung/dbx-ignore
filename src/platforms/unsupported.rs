use crate::traits::PlatformHandler;
use anyhow::Result;
use std::path::Path;

pub struct UnsupportedHandler;

impl PlatformHandler for UnsupportedHandler {
    fn get_target_attributes() -> &'static [&'static str] {
        &[]
    }

    fn has_attribute(_path: &Path, _attr: &str) -> Result<bool> {
        Ok(false)
    }

    fn add_attribute(_path: &Path, _attr: &str) -> Result<()> {
        // No-op on unsupported platforms - cannot add ignore markers
        Err(anyhow::anyhow!(
            "Adding ignore markers not supported on this platform"
        ))
    }

    fn remove_attribute(_path: &Path, _attr: &str) -> Result<()> {
        // No-op on unsupported platforms - cannot remove ignore markers
        Err(anyhow::anyhow!(
            "Removing ignore markers not supported on this platform"
        ))
    }

    fn platform_name() -> &'static str {
        "Unsupported Platform"
    }

    fn is_supported() -> bool {
        false
    }
}
