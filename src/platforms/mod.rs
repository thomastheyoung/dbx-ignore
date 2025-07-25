use crate::traits::PlatformHandler;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSHandler as CurrentPlatform;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxHandler as CurrentPlatform;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsHandler as CurrentPlatform;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub use unsupported::UnsupportedHandler as CurrentPlatform;

/// Get platform-specific information
pub fn get_platform_info() -> (&'static str, bool) {
    (CurrentPlatform::platform_name(), CurrentPlatform::is_supported())
}