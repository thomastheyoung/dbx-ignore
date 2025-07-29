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

#[derive(Debug, Clone)]
pub struct Config {
    pub action: Action,
    pub dry_run: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub files: Vec<PathBuf>,
    pub patterns: Vec<String>,  // Original patterns provided by user
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

    // Cache current directory for the entire run
    let current_dir = std::env::current_dir()?;

    // Handle watch/unwatch modes
    match config.action {
        Action::Watch => {
            let repo_path = current_dir.clone();
            
            // Check if daemon is already running
            if let Some(status) = core::daemon::DaemonStatus::read(&repo_path)? {
                println!("{} A daemon is already watching this repository (PID: {})", 
                    "⚠".yellow(), status.pid);
                return Ok(());
            }
            
            // If files/patterns provided with --watch, process them first
            if !config.files.is_empty() && !config.daemon_mode {
                if !config.quiet {
                    println!("{} Marking files before starting watch mode...", "🔍".yellow());
                }
                
                // Create a temporary config for marking files
                let mut mark_config = config.clone();
                mark_config.action = Action::Ignore;
                
                // Process the files/patterns
                process_files_and_patterns(&mark_config, &current_dir)?;
                
                if !config.quiet {
                    println!();
                }
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
            let repo_path = current_dir.clone();
            
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

    process_files_and_patterns(&config, &current_dir)
}

fn process_files_and_patterns(config: &Config, current_dir: &Path) -> Result<()> {
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
    let mut tracked = core::tracked_files::TrackedFiles::load(current_dir)?;
    let files_to_add = Arc::new(std::sync::Mutex::new(Vec::new()));
    let files_to_remove = Arc::new(std::sync::Mutex::new(Vec::new()));

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

            match process_path(path, config) {
                Ok(operations_performed) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    operation_count.fetch_add(operations_performed, Ordering::Relaxed);
                    
                    // Collect files to update after parallel processing
                    if operations_performed > 0 && !config.dry_run {
                        match config.action {
                            Action::Ignore => files_to_add.lock().unwrap().push(path.clone()),
                            Action::Reset => files_to_remove.lock().unwrap().push(path.clone()),
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

    // Apply collected changes and save tracked files state
    if !config.dry_run && (config.action == Action::Ignore || config.action == Action::Reset) {
        // Apply file changes collected during parallel processing
        let files_to_add = files_to_add.lock().unwrap();
        if !files_to_add.is_empty() {
            tracked.add_files(&files_to_add);
        }
        
        let files_to_remove = files_to_remove.lock().unwrap();
        if !files_to_remove.is_empty() {
            tracked.remove_files(&files_to_remove);
        }
        
        // Store patterns if we're ignoring files
        if config.action == Action::Ignore && !config.patterns.is_empty() {
            tracked.add_patterns(&config.patterns);
        } else if config.action == Action::Reset && !config.patterns.is_empty() {
            tracked.remove_patterns(&config.patterns);
        }
        
        tracked.save(current_dir)?;
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


/// Check if a path string contains glob pattern characters
pub fn is_glob_pattern(path_str: &str) -> bool {
    path_str.contains('*') || path_str.contains('?') || path_str.contains('[')
}

/// Classification of path types for special handling
enum PathType {
    CurrentDirectory,
    GitIgnoreFile,
    Regular,
}

/// Classify a path for special handling
fn classify_path(path: &Path) -> PathType {
    // Check if it's the current directory
    if path.to_str() == Some(".") || path.file_name().and_then(|n| n.to_str()) == Some(".") {
        PathType::CurrentDirectory
    }
    // Check if it's a .gitignore file
    else if path.file_name().and_then(|n| n.to_str()) == Some(".gitignore") {
        PathType::GitIgnoreFile
    }
    else {
        PathType::Regular
    }
}

/// Check if a path is a hidden file (starts with .)
fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Process a glob pattern and add matching files to items
/// Returns true if any matches were found
fn process_glob_pattern(pattern: &str, items: &mut Vec<PathBuf>) -> Result<bool> {
    let initial_count = items.len();
    
    match glob::glob(pattern) {
        Ok(mut glob_paths) => {
            for entry in &mut glob_paths {
                match entry {
                    Ok(p) => {
                        if p.exists() {
                            items.push(p);
                        }
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Glob error: {}", e));
                    }
                }
            }
            Ok(items.len() > initial_count)
        }
        Err(e) => {
            Err(anyhow::anyhow!("Invalid pattern '{}': {}", pattern, e))
        }
    }
}

fn get_files_from_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut items = Vec::new();
    let mut regular_paths = Vec::new();
    let mut empty_patterns = Vec::new();
    
    // Process each path, categorizing as pattern or regular path
    for path in paths {
        let path_str = path.to_string_lossy();
        
        if is_glob_pattern(&path_str) {
            // Handle glob patterns
            match process_glob_pattern(&path_str, &mut items) {
                Ok(found_matches) => {
                    if !found_matches {
                        empty_patterns.push(path_str.to_string());
                    }
                }
                Err(e) => return Err(e),
            }
        } else {
            regular_paths.push(path.clone());
        }
    }
    
    // Report error if any patterns matched nothing
    if !empty_patterns.is_empty() {
        return Err(anyhow::anyhow!("No files found matching patterns: {}", empty_patterns.join(", ")));
    }
    
    // Process regular paths
    for path in regular_paths {
        if !path.exists() {
            return Err(anyhow::anyhow!("Path not found: {}", path.display()));
        }

        match classify_path(&path) {
            PathType::CurrentDirectory => {
                // Expand current directory contents, skipping hidden files
                for entry in std::fs::read_dir(path)? {
                    let entry_path = entry?.path();
                    if !is_hidden_file(&entry_path) {
                        items.push(entry_path);
                    }
                }
            }
            PathType::GitIgnoreFile => {
                // Process .gitignore file and add the ignored files
                let gitignore_files = utils::git_utils::get_git_ignored_files_from_gitignore(&path)?;
                items.extend(gitignore_files);
            }
            PathType::Regular => {
                // Add the path directly
                items.push(path);
            }
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
                utils::platform_utils::add_ignore_attributes(path, true)
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
                utils::platform_utils::remove_ignore_attributes(path)
            }
        }
        Action::Watch | Action::Unwatch => {
            // Watch/Unwatch modes are handled separately in the run function
            unreachable!("Watch/Unwatch modes should be handled before reaching process_path");
        }
    }
}

