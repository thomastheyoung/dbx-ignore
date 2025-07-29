use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::daemon;
use crate::utils::platform_utils;

pub struct StatusInfo {
    pub has_gitignore: bool,
    pub total_files: usize,
    pub ignored_files: Vec<PathBuf>,
    pub non_ignored_files: Vec<PathBuf>,
    pub daemon_status: Option<daemon::DaemonStatus>,
    pub current_path: PathBuf,
}

impl StatusInfo {
    pub fn gather() -> Result<Self> {
        let current_path = std::env::current_dir().context("Failed to get current directory")?;

        // Check for .gitignore
        let has_gitignore = current_path.join(".gitignore").exists();

        // Get daemon status
        let daemon_status = daemon::DaemonStatus::read(&current_path)?;

        // Get all files in the current directory (non-recursive)
        let mut all_files = Vec::new();
        let mut file_status = HashMap::new();

        for entry in std::fs::read_dir(&current_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files (starting with .)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            // Check if file has ignore markers
            let is_ignored = platform_utils::has_any_ignore_attribute(&path);

            all_files.push(path.clone());
            file_status.insert(path, is_ignored);
        }

        // Sort files for consistent output
        all_files.sort();

        // Separate ignored and non-ignored files
        let ignored_files: Vec<PathBuf> = all_files
            .iter()
            .filter(|f| *file_status.get(*f).unwrap_or(&false))
            .cloned()
            .collect();

        let non_ignored_files: Vec<PathBuf> = all_files
            .iter()
            .filter(|f| !*file_status.get(*f).unwrap_or(&false))
            .cloned()
            .collect();

        Ok(StatusInfo {
            has_gitignore,
            total_files: all_files.len(),
            ignored_files,
            non_ignored_files,
            daemon_status,
            current_path,
        })
    }

    pub fn display(&self, verbose: bool) -> Result<()> {
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════".blue()
        );
        println!(
            "{} {}",
            "Status Report for:".blue().bold(),
            self.current_path.display()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════".blue()
        );
        println!();

        // Gitignore status
        println!(
            "{} {}",
            ".gitignore:".yellow().bold(),
            if self.has_gitignore {
                "✓ Detected".green()
            } else {
                "✗ Not found".red()
            }
        );

        // File counts
        println!(
            "{} {} files total",
            "Files:".yellow().bold(),
            self.total_files
        );
        println!(
            "   {} {} files have ignore markers",
            "✓".green(),
            self.ignored_files.len().to_string().green()
        );
        println!(
            "   {} {} files don't have ignore markers",
            "✗".red(),
            self.non_ignored_files.len().to_string().red()
        );

        // Daemon status
        println!(
            "{} {}",
            "Daemon:".yellow().bold(),
            if let Some(ref status) = self.daemon_status {
                format!("✓ Running (PID: {})", status.pid).green()
            } else {
                "✗ Not running".red()
            }
        );

        // Verbose file listing
        if verbose && self.total_files > 0 {
            println!();
            println!("{}", "File Details:".yellow().bold());
            println!("{}", "─────────────".yellow());

            // Show ignored files first
            if !self.ignored_files.is_empty() {
                println!("{}", "Ignored files:".green());
                for file in &self.ignored_files {
                    if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
                        println!("  {} {}", "✓".green(), name.green());
                    }
                }
            }

            // Show non-ignored files
            if !self.non_ignored_files.is_empty() {
                if !self.ignored_files.is_empty() {
                    println!();
                }
                println!("{}", "Not ignored files:".red());
                for file in &self.non_ignored_files {
                    if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
                        println!("  {} {}", "✗".red(), name.red());
                    }
                }
            }
        }

        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════".blue()
        );

        Ok(())
    }
}

/// Main entry point for the status command
pub fn show_status(verbose: bool) -> Result<()> {
    let status = StatusInfo::gather()?;
    status.display(verbose)
}
