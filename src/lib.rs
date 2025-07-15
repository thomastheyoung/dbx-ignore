use anyhow::{Context, Result};
use colored::*;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub mod traits;
pub mod platforms;

use crate::platforms::CurrentPlatform;
use crate::traits::PlatformHandler;

#[derive(Debug)]
pub struct Config {
    pub dry_run: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub files: Vec<PathBuf>,
    pub git_mode: bool,
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

    let files_to_process = if config.git_mode && config.files.is_empty() {
        get_git_ignored_files()?
    } else {
        get_files_from_paths(&config.files)?
    };

    if !config.quiet {
        if config.dry_run {
            println!("{}", "🔍 Dry run mode - no changes will be made".yellow());
        }
        
        println!("{} Platform: {}", "✓".green(), CurrentPlatform::platform_name());
        
        if config.git_mode && config.files.is_empty() {
            println!("{}", "✓ Mode: Adding ignore markers to git-ignored files".green());
        } else {
            println!("{}", "✓ Mode: Adding ignore markers to specified files".green());
        }
    }

    let total_files = files_to_process.len();
    let processed_count = Arc::new(AtomicUsize::new(0));
    let added_count = Arc::new(AtomicUsize::new(0));

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
                Ok(attrs_added) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    added_count.fetch_add(attrs_added, Ordering::Relaxed);
                    
                    if config.verbose {
                        let item_type = if path.is_dir() { "directory" } else { "file" };
                        if attrs_added > 0 {
                            println!("   {} {} {}: {} ignore markers added", 
                                "✓".green(), item_type, path.display(), attrs_added);
                        } else {
                            println!("   {} {} {}: already ignored", 
                                "-".yellow(), item_type, path.display());
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
    let final_added = added_count.load(Ordering::Relaxed);

    if !config.quiet {
        println!("{}", "─".repeat(50));
        if config.dry_run {
            println!("{} {} files would be processed, {} ignore markers would be added", 
                "🔍".yellow(), final_processed, final_added);
        } else {
            println!("{} {} files processed, {} ignore markers added", 
                "✓".green(), final_processed, final_added);
        }
    }

    Ok(())
}

/// Discovers git-ignored files and directories by reading .gitignore patterns.
/// 
/// This function reads .gitignore patterns and resolves them to actual paths,
/// applying xattr operations directly to matched items without walking directories.
/// 
/// **Efficiency Rationale:**
/// - When a directory is ignored, we apply xattr to the directory itself
/// - No recursive walking of ignored directories
/// - Results in N xattr operations for N ignored patterns
/// 
/// **Implementation Notes:**
/// - Reads .gitignore patterns directly
/// - Uses glob pattern matching for resolution
/// - Handles both files and directories
/// - Simple patterns only (no negation support currently)
fn get_git_ignored_files() -> Result<Vec<PathBuf>> {
    let repo = git2::Repository::discover(".")
        .context("Not in a git repository or git repository not found")?;

    let workdir = repo.workdir()
        .context("Repository has no working directory")?;

    let gitignore_path = workdir.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    let mut processed_paths = std::collections::HashSet::new();
    
    // Read .gitignore patterns
    let contents = fs::read_to_string(&gitignore_path)
        .context("Failed to read .gitignore")?;
    
    for line in contents.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Skip negated patterns for now (like the shell script)
        if line.starts_with('!') {
            continue;
        }
        
        // Resolve the pattern
        let pattern = if line.starts_with('/') {
            // Absolute path from repo root
            workdir.join(&line[1..]).to_string_lossy().to_string()
        } else {
            // Relative pattern
            workdir.join(line).to_string_lossy().to_string()
        };
        
        // Use glob to find matches
        match glob(&pattern) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    // Avoid duplicates
                    if processed_paths.insert(entry.clone()) {
                        paths.push(entry);
                    }
                }
            }
            Err(_) => {
                // If glob fails, try as a direct path
                let path = PathBuf::from(&pattern);
                if path.exists() && processed_paths.insert(path.clone()) {
                    paths.push(path);
                }
            }
        }
    }

    Ok(paths)
}

fn get_files_from_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut items = Vec::new();
    
    for path in paths {
        if path.exists() {
            // Add the path directly without walking directories
            items.push(path.clone());
        } else {
            return Err(anyhow::anyhow!("Path not found: {}", path.display()));
        }
    }
    
    Ok(items)
}

fn process_path(path: &Path, config: &Config) -> Result<usize> {
    let mut added_count = 0;

    for attr_name in CurrentPlatform::get_target_attributes() {
        if !CurrentPlatform::has_attribute(path, attr_name)? {
            if !config.dry_run {
                CurrentPlatform::add_attribute(path, attr_name)?;
            }
            added_count += 1;
        }
    }

    Ok(added_count)
}