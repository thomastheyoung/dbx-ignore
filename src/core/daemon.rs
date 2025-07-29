use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use crate::utils::json_utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub pid: u32,
    pub repo_path: PathBuf,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl DaemonStatus {
    pub fn status_file_path(repo_path: &Path) -> PathBuf {
        repo_path.join(".dbx-ignore").join("daemon.json")
    }

    pub fn read(repo_path: &Path) -> Result<Option<Self>> {
        let status_file = Self::status_file_path(repo_path);
        if !status_file.exists() {
            return Ok(None);
        }

        // Use robust JSON reading
        let status = match json_utils::read_json::<DaemonStatus>(&status_file) {
            Ok(s) => s,
            Err(_) => {
                // If file is corrupted, remove it and return None
                let _ = fs::remove_file(&status_file);
                return Ok(None);
            }
        };

        // Validate required fields
        if status.pid == 0 {
            let _ = fs::remove_file(&status_file);
            return Ok(None);
        }

        // Check if process is still running
        if is_process_running(status.pid) {
            Ok(Some(status))
        } else {
            // Clean up stale status file
            let _ = fs::remove_file(&status_file);
            Ok(None)
        }
    }

    pub fn write(&self, repo_path: &Path) -> Result<()> {
        let status_file = Self::status_file_path(repo_path);
        
        // Validate before writing
        if self.pid == 0 {
            return Err(anyhow::anyhow!("Invalid PID: 0"));
        }

        // Use atomic write
        json_utils::write_json_atomic(&status_file, self)
            .context("Failed to write daemon status file")?;

        Ok(())
    }

    pub fn remove(repo_path: &Path) -> Result<()> {
        let status_file = Self::status_file_path(repo_path);
        if status_file.exists() {
            fs::remove_file(&status_file)
                .context("Failed to remove daemon status file")?;
        }
        Ok(())
    }
}

/// Check if a process with the given PID is running
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    // Send signal 0 to check if process exists
    match Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    use std::os::windows::process::CommandExt;
    
    // Use tasklist to check if process exists
    match Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

/// Spawn a daemon process in the background
pub fn spawn_daemon(repo_path: &Path) -> Result<u32> {
    let exe_path = std::env::current_exe()
        .context("Failed to get current executable path")?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        
        let child = Command::new(&exe_path)
            .arg("--watch")
            .arg("--daemon-mode")  // Special flag to indicate we're running as daemon
            .current_dir(repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)  // Create new process group
            .spawn()
            .context("Failed to spawn daemon process")?;

        Ok(child.id())
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        
        let child = Command::new(&exe_path)
            .arg("--watch")
            .arg("--daemon-mode")
            .current_dir(repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x00000008 | 0x00000200) // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
            .spawn()
            .context("Failed to spawn daemon process")?;

        Ok(child.id())
    }
}

/// Stop a running daemon
pub fn stop_daemon(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()
            .context("Failed to send termination signal to daemon")?;
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .status()
            .context("Failed to terminate daemon process")?;
    }

    Ok(())
}