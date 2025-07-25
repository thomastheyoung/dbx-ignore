use anyhow::Result;
use std::path::Path;

/// Platform abstraction trait for handling extended attributes/metadata
pub trait PlatformHandler: Send + Sync {
    /// Get the list of target attributes to remove for this platform
    fn get_target_attributes() -> &'static [&'static str];
    
    /// Check if a specific attribute exists on the given path
    fn has_attribute(path: &Path, attr: &str) -> Result<bool>;
    
    /// Add a specific attribute to the given path to mark it as ignored
    fn add_attribute(path: &Path, attr: &str) -> Result<()>;
    
    /// Remove a specific attribute from the given path to unmark it as ignored
    fn remove_attribute(path: &Path, attr: &str) -> Result<()>;
    
    /// Get the platform name for display purposes
    fn platform_name() -> &'static str;
    
    /// Check if this platform is supported
    fn is_supported() -> bool {
        true
    }
}