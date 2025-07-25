use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub mod traits;
pub mod platforms;
pub mod core;
pub mod utils;

use crate::platforms::CurrentPlatform;
use crate::traits::PlatformHandler;

// Re-export the show_status function and modules
pub use crate::core::status::show_status;
pub use crate::core::status;
pub use crate::core::tracked_files;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Ignore,
    Reset,
    Watch,
    Unwatch,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Ignore => write!(f, "ignore"),
            Action::Reset => write!(f, "reset"),
            Action::Watch => write!(f, "watch"),
            Action::Unwatch => write!(f, "unwatch"),
        }
    }
}

impl std::str::FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ignore" => Ok(Action::Ignore),
            "reset" => Ok(Action::Reset),
            "watch" => Ok(Action::Watch),
            "unwatch" => Ok(Action::Unwatch),
            _ => Err(anyhow::anyhow!("Invalid action: {}. Valid actions are: ignore, reset, watch, unwatch", s)),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub action: Action,
    pub dry_run: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub files: Vec<PathBuf>,
    pub git_mode: bool,
    pub daemon_mode: bool,
}

pub fn run(config: Config) -> Result<()> {
    // Check platform support
    if !CurrentPlatform::is_supported() {
        if !config.quiet {
            println!("{} Platform '{}' is not supported for extended attribute operations",
                "⚠".yellow(), CurrentPlatform::platform_name());
            println!("Supported platforms: macOS, Linux, Windows");
        }
        return Ok(());
    }

    // Handle watch/unwatch modes
    match config.action {
        Action::Watch => {
            let repo_path = std::env::current_dir()?;
            
            // Check if daemon is already running
            if let Some(status) = core::daemon::DaemonStatus::read(&repo_path)? {
                println!("{} A daemon is already watching this repository (PID: {})", 
                    "⚠".yellow(), status.pid);
                return Ok(());
            }
            
            // Check if we're being run as a daemon
            if config.daemon_mode {
                // Running as daemon - start the watcher
                let runtime = tokio::runtime::Runtime::new()?;
                let watch_config = core::watch::WatchConfig::new(repo_path.clone());
                
                // Save daemon status
                let status = core::daemon::DaemonStatus {
                    pid: std::process::id(),
                    repo_path: repo_path.clone(),
                    started_at: chrono::Utc::now(),
                };
                status.write(&repo_path)?;
                
                // Run the watcher
                let result = runtime.block_on(core::watch::watch_repository(watch_config));
                
                // Clean up status file on exit
                let _ = core::daemon::DaemonStatus::remove(&repo_path);
                
                return result;
            }
            // Spawn daemon in background
            let pid = core::daemon::spawn_daemon(&repo_path)?;
            println!("{} Started daemon watcher (PID: {})", "✓".green(), pid);
            println!("Run 'dbx-ignore --unwatch' to stop the daemon");
            return Ok(());
        }
        Action::Unwatch => {
            let repo_path = std::env::current_dir()?;
            
            if let Some(status) = core::daemon::DaemonStatus::read(&repo_path)? {
                core::daemon::stop_daemon(status.pid)?;
                core::daemon::DaemonStatus::remove(&repo_path)?;
                println!("{} Stopped daemon watcher (PID: {})", "✓".green(), status.pid);
            } else {
                println!("{} No active daemon found for this repository", "⚠".yellow());
            }
            return Ok(());
        }
        _ => {} // Continue with normal processing
    }

    let files_to_process = if config.git_mode && config.files.is_empty() {
        utils::git_utils::get_git_ignored_files()?
    } else {
        get_files_from_paths(&config.files)?
    };

    if !config.quiet {
        if config.dry_run {
            println!("{}", "🔍 Dry run mode - no changes will be made".yellow());
        }
        
        println!("{} Platform: {}", "✓".green(), CurrentPlatform::platform_name());
        
        let action_description = match config.action {
            Action::Ignore => "Adding ignore markers to",
            Action::Reset => "Removing ignore markers from",
            Action::Watch => "Setting up monitoring for",
            Action::Unwatch => "Stopping monitoring for",
        };
        
        if config.git_mode && config.files.is_empty() {
            println!("{} Mode: {} git-ignored files", "✓".green(), action_description.green());
        } else {
            println!("{} Mode: {} specified files", "✓".green(), action_description.green());
        }
    }

    let total_files = files_to_process.len();
    let processed_count = Arc::new(AtomicUsize::new(0));
    let operation_count = Arc::new(AtomicUsize::new(0));
    
    // Track files that are being marked/unmarked
    let current_dir = std::env::current_dir()?;
    let tracked = core::tracked_files::TrackedFiles::load(&current_dir)?;
    let tracked_mutex = Arc::new(std::sync::Mutex::new(tracked));

    let progress = if !config.quiet && !config.verbose {
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(Arc::new(pb))
    } else {
        None
    };

    // Process files in parallel
    files_to_process
        .par_iter()
        .for_each(|path| {
            if let Some(ref pb) = progress {
                pb.set_message(format!("Processing: {}", path.display()));
            }

            match process_path(path, &config) {
                Ok(operations_performed) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    operation_count.fetch_add(operations_performed, Ordering::Relaxed);
                    
                    // Update tracked files based on action
                    if operations_performed > 0 && !config.dry_run {
                        let mut tracked = tracked_mutex.lock().unwrap();
                        match config.action {
                            Action::Ignore => tracked.add_files(&[path.clone()]),
                            Action::Reset => tracked.remove_files(&[path.clone()]),
                            _ => {}
                        }
                    }
                    
                    if config.verbose {
                        let item_type = if path.is_dir() { "directory" } else { "file" };
                        if operations_performed > 0 {
                            let operation_msg = match config.action {
                                Action::Ignore => "ignore markers added",
                                Action::Reset => "ignore markers removed",
                                Action::Watch => "monitoring set up",
                                Action::Unwatch => "monitoring stopped",
                            };
                            println!("   {} {} {}: {} {}", 
                                "✓".green(), item_type, path.display(), operations_performed, operation_msg);
                        } else {
                            let status_msg = match config.action {
                                Action::Ignore => "already ignored",
                                Action::Reset => "no markers to remove",
                                Action::Watch => "already monitored",
                                Action::Unwatch => "not monitored",
                            };
                            println!("   {} {} {}: {}", 
                                "-".yellow(), item_type, path.display(), status_msg);
                        }
                    }
                }
                Err(e) => {
                    if config.verbose {
                        println!("   {} {}: {}", 
                            "✘".red(), path.display(), e);
                    } else if !config.quiet {
                        eprintln!("   {} Warning: {}: {}", 
                            "⚠".yellow(), path.display(), e);
                    }
                }
            }

            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        });

    if let Some(ref pb) = progress {
        pb.finish_with_message("Complete!");
    }

    let final_processed = processed_count.load(Ordering::Relaxed);
    let final_operations = operation_count.load(Ordering::Relaxed);

    // Save tracked files state
    if !config.dry_run && (config.action == Action::Ignore || config.action == Action::Reset) {
        let tracked = tracked_mutex.lock().unwrap();
        tracked.save(&current_dir)?;
    }

    if !config.quiet {
        println!("{}", "─".repeat(50));
        let operation_description = match config.action {
            Action::Ignore => "ignore markers added",
            Action::Reset => "ignore markers removed", 
            Action::Watch => "items set up for monitoring",
            Action::Unwatch => "monitoring stopped",
        };
        
        if config.dry_run {
            println!("{} {} files would be processed, {} {}", 
                "🔍".yellow(), final_processed, final_operations, operation_description);
        } else {
            println!("{} {} files processed, {} {}", 
                "✓".green(), final_processed, final_operations, operation_description);
        }
    }

    Ok(())
}


fn get_files_from_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut items = Vec::new();
    
    for path in paths {
        if path.exists() {
            // Special case: if path is '.' (current directory), expand to contents
            if path.to_str() == Some(".") || path.file_name().and_then(|n| n.to_str()) == Some(".") {
                // Get all files and directories in current directory
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    // Skip hidden files starting with '.' (like .git, .gitignore, etc.)
                    if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            items.push(entry_path);
                        }
                    }
                }
            }
            // Check if this is a .gitignore file
            else if path.file_name().and_then(|n| n.to_str()) == Some(".gitignore") {
                // Process .gitignore file and add the ignored files to the list
                let gitignore_files = utils::git_utils::get_git_ignored_files_from_gitignore(path)?;
                items.extend(gitignore_files);
            } else {
                // Add the path directly without walking directories
                items.push(path.clone());
            }
        } else {
            return Err(anyhow::anyhow!("Path not found: {}", path.display()));
        }
    }
    
    Ok(items)
}


fn process_path(path: &Path, config: &Config) -> Result<usize> {
    match config.action {
        Action::Ignore => {
            if config.dry_run {
                // Count how many attributes would be added
                let mut count = 0;
                for attr in CurrentPlatform::get_target_attributes() {
                    if !CurrentPlatform::has_attribute(path, attr)? {
                        count += 1;
                    }
                }
                Ok(count)
            } else {
                utils::platform_utils::try_add_ignore_attributes(path)
            }
        }
        Action::Reset => {
            if config.dry_run {
                // Count how many attributes would be removed
                let mut count = 0;
                for attr in CurrentPlatform::get_target_attributes() {
                    if CurrentPlatform::has_attribute(path, attr)? {
                        count += 1;
                    }
                }
                Ok(count)
            } else {
                utils::platform_utils::try_remove_ignore_attributes(path)
            }
        }
        Action::Watch | Action::Unwatch => {
            // Watch/Unwatch modes are handled separately in the run function
            unreachable!("Watch/Unwatch modes should be handled before reaching process_path");
        }
    }
}

/// Process multiple files and return statistics
pub fn process_files(files: Vec<PathBuf>) -> Result<ProcessStats> {
    use rayon::prelude::*;
    
    let total = files.len();
    let already_marked = Arc::new(AtomicUsize::new(0));
    let newly_marked = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));
    
    files.par_iter().for_each(|path| {
        use crate::utils::platform_utils;
        
        // Check if already marked
        if platform_utils::has_any_ignore_attribute(path) {
            already_marked.fetch_add(1, Ordering::Relaxed);
        } else {
            // Try to add marker
            if platform_utils::add_all_ignore_attributes(path).is_err() {
                errors.fetch_add(1, Ordering::Relaxed);
            } else {
                newly_marked.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    
    Ok(ProcessStats {
        total,
        already_marked: already_marked.load(Ordering::Relaxed),
        newly_marked: newly_marked.load(Ordering::Relaxed),
        errors: errors.load(Ordering::Relaxed),
    })
}

#[derive(Debug)]
pub struct ProcessStats {
    pub total: usize,
    pub already_marked: usize,
    pub newly_marked: usize,
    pub errors: usize,
}