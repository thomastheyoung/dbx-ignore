use anyhow::Result;
use std::path::Path;

/// Platform abstraction trait for handling extended attributes/metadata
pub trait PlatformHandler {
    /// Get the list of target attributes to remove for this platform
    fn get_target_attributes() -> &'static [&'static str];
    
    /// Check if a specific attribute exists on the given path
    fn has_attribute(path: &Path, attr: &str) -> Result<bool>;
    
    /// Add a specific attribute to the given path to mark it as ignored
    fn add_attribute(path: &Path, attr: &str) -> Result<()>;
    
    /// Get the platform name for display purposes
    fn platform_name() -> &'static str;
    
    /// Check if this platform is supported
    fn is_supported() -> bool {
        true
    }
}

/// Platform-agnostic attribute removal statistics
#[derive(Debug, Default)]
pub struct AttributeStats {
    pub files_processed: usize,
    pub attributes_removed: usize,
    pub errors_encountered: usize,
}