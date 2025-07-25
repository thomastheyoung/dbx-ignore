use crate::traits::PlatformHandler;
use anyhow::{Context, Result};
use std::path::Path;

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

pub struct WindowsHandler;

impl PlatformHandler for WindowsHandler {
    fn get_target_attributes() -> &'static [&'static str] {
        &["com.dropbox.ignored"]
    }

    fn has_attribute(path: &Path, attr: &str) -> Result<bool> {
        #[cfg(target_os = "windows")]
        {
            let stream_path = format!("{}:{}", path.display(), attr);
            match std::fs::metadata(&stream_path) {
                Ok(_) => Ok(true),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => Ok(false),
                    std::io::ErrorKind::PermissionDenied => Ok(false),
                    _ => Err(anyhow::anyhow!("Failed to check stream {}: {}", attr, e)),
                },
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (path, attr);
            Ok(false)
        }
    }

    fn add_attribute(path: &Path, attr: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            let stream_path = format!("{}:{}", path.display(), attr);
            std::fs::write(&stream_path, b"1")
                .with_context(|| format!("Failed to add stream {} to {}", attr, path.display()))
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (path, attr);
            Err(anyhow::anyhow!(
                "Windows ADS not supported on this platform"
            ))
        }
    }

    fn remove_attribute(path: &Path, attr: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            let stream_path = format!("{}:{}", path.display(), attr);
            match std::fs::remove_file(&stream_path) {
                Ok(()) => Ok(()),
                Err(e) => {
                    // If the stream doesn't exist, that's fine
                    match e.kind() {
                        std::io::ErrorKind::NotFound => Ok(()),
                        _ => Err(anyhow::anyhow!("Failed to remove stream {} from {}: {}", attr, path.display(), e)),
                    }
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (path, attr);
            Err(anyhow::anyhow!(
                "Windows ADS not supported on this platform"
            ))
        }
    }

    fn platform_name() -> &'static str {
        "Windows"
    }
}
