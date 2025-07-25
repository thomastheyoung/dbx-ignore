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

/// Add all target attributes to a path
pub fn add_all_ignore_attributes(path: &Path) -> Result<()> {
    for attr in CurrentPlatform::get_target_attributes() {
        CurrentPlatform::add_attribute(path, attr)?;
    }
    Ok(())
}

/// Remove all target attributes from a path
pub fn remove_all_ignore_attributes(path: &Path) -> Result<()> {
    for attr in CurrentPlatform::get_target_attributes() {
        if CurrentPlatform::has_attribute(path, attr)? {
            CurrentPlatform::remove_attribute(path, attr)?;
        }
    }
    Ok(())
}

/// Try to add all attributes, returning the number added
pub fn try_add_ignore_attributes(path: &Path) -> Result<usize> {
    let mut count = 0;
    for attr in CurrentPlatform::get_target_attributes() {
        if !CurrentPlatform::has_attribute(path, attr)? {
            CurrentPlatform::add_attribute(path, attr)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Try to remove all attributes, returning the number removed
pub fn try_remove_ignore_attributes(path: &Path) -> Result<usize> {
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