use crate::{platforms::CurrentPlatform, traits::PlatformHandler};
use anyhow::Result;
use std::path::Path;
use std::io;

/// Check if a path has any of the target ignore attributes
pub fn has_any_ignore_attribute(path: &Path) -> bool {
    CurrentPlatform::get_target_attributes()
        .iter()
        .any(|attr| CurrentPlatform::has_attribute(path, attr).unwrap_or(false))
}

/// Add all target attributes to a path, optionally returning the count
/// If skip_existing is true, will skip attributes that already exist
pub fn add_ignore_attributes(path: &Path, skip_existing: bool) -> Result<usize> {
    let mut count = 0;
    for attr in CurrentPlatform::get_target_attributes() {
        if skip_existing && CurrentPlatform::has_attribute(path, attr)? {
            continue;
        }
        CurrentPlatform::add_attribute(path, attr)?;
        count += 1;
    }
    Ok(count)
}

/// Remove all target attributes from a path, returning the count removed
pub fn remove_ignore_attributes(path: &Path) -> Result<usize> {
    let mut count = 0;
    for attr in CurrentPlatform::get_target_attributes() {
        if CurrentPlatform::has_attribute(path, attr)? {
            CurrentPlatform::remove_attribute(path, attr)?;
            count += 1;
        }
    }
    Ok(count)
}


/// Helper function for consistent IO error handling across platforms
/// 
/// This function handles common patterns like treating NotFound as Ok(false)
/// and converting other errors to anyhow errors with context.
pub fn handle_attribute_check_error(e: io::Error, attr: &str) -> Result<bool> {
    match e.kind() {
        io::ErrorKind::NotFound => Ok(false),
        _ => Err(anyhow::anyhow!("Failed to check attribute {}: {}", attr, e)),
    }
}

/// Helper function for handling attribute removal errors
pub fn handle_attribute_remove_error(e: io::Error, attr: &str, path: &Path) -> Result<()> {
    match e.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(anyhow::anyhow!("Failed to remove attribute {} from {}: {}", attr, path.display(), e)),
    }
}