use anyhow::{Context, Result};
use colored::Colorize;
use git2::Repository;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time;

use crate::utils::{git_utils, platform_utils};
use crate::core::tracked_files;

#[derive(Debug, Clone)]
enum WatchMode {
    TrackedFiles,
    GitIgnore,
    Patterns(Vec<String>),
}

pub struct WatchConfig {
    pub repo_path: PathBuf,
    pub debounce_duration: Duration,
}

impl WatchConfig {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            debounce_duration: Duration::from_millis(500),
        }
    }
}

pub async fn watch_repository(
    config: WatchConfig,
) -> Result<()> {
    let repo = Repository::open(&config.repo_path)
        .context("Failed to open git repository")?;
    
    let repo_root = repo.workdir()
        .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
        .to_path_buf();

    // Determine watch mode based on tracked files and patterns
    let tracked = tracked_files::TrackedFiles::load(&repo_root)?;
    let watch_mode = if !tracked.patterns.is_empty() {
        WatchMode::Patterns(tracked.patterns.clone())
    } else if tracked.marked_files.is_empty() {
        WatchMode::GitIgnore
    } else {
        WatchMode::TrackedFiles
    };

    println!("{}", "Starting file watcher daemon...".green().bold());
    println!("Watching repository at: {}", repo_root.display());
    match &watch_mode {
        WatchMode::TrackedFiles => {
            println!("Mode: Monitoring {} tracked files for changes", tracked.marked_files.len());
        }
        WatchMode::GitIgnore => {
            println!("Mode: Monitoring .gitignore changes to automatically mark/unmark files");
        }
        WatchMode::Patterns(patterns) => {
            println!("Mode: Monitoring for files matching patterns:");
            for pattern in patterns {
                println!("  - {}", pattern);
            }
        }
    }
    println!("Press Ctrl+C to stop\n");

    // Initial scan
    perform_scan(&repo_root, &watch_mode)?;

    // Set up channels for file system events
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Track pending events for debouncing
    let pending_events = Arc::new(Mutex::new(HashSet::new()));
    
    // Create file watcher
    let mut watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    // Watch the repository root
    watcher.watch(&repo_root, RecursiveMode::Recursive)?;

    // Also watch .gitignore files specifically
    let gitignore_paths = find_gitignore_files(&repo_root)?;
    for gitignore_path in &gitignore_paths {
        watcher.watch(gitignore_path, RecursiveMode::NonRecursive)?;
    }

    // Set up Ctrl+C handler
    let shutdown = Arc::new(Mutex::new(false));
    let shutdown_clone = shutdown.clone();
    
    ctrlc::set_handler(move || {
        let shutdown_clone = shutdown_clone.clone();
        tokio::spawn(async move {
            *shutdown_clone.lock().await = true;
        });
    })?;

    // Event processing loop
    let mut debounce_timer = time::interval(config.debounce_duration);
    
    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                if should_trigger_rescan(&event, &watch_mode) {
                    let mut events = pending_events.lock().await;
                    events.insert(event.paths.first().cloned().unwrap_or_default());
                }
            }
            _ = debounce_timer.tick() => {
                let mut events = pending_events.lock().await;
                if !events.is_empty() {
                    println!("\n{}", "Detected changes, re-scanning...".yellow());
                    if let Err(e) = perform_scan(&repo_root, &watch_mode) {
                        eprintln!("{} {}", "Error during scan:".red(), e);
                    }
                    events.clear();
                }
            }
        }

        // Check for shutdown
        if *shutdown.lock().await {
            println!("\n{}", "Shutting down watcher...".yellow());
            break;
        }
    }

    Ok(())
}

fn should_trigger_rescan(event: &Event, watch_mode: &WatchMode) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            match watch_mode {
                WatchMode::Patterns(_) => {
                    // For pattern mode, trigger on any file creation/removal
                    matches!(event.kind, EventKind::Create(_) | EventKind::Remove(_))
                }
                _ => {
                    // For other modes, check if it's a .gitignore file or within .git
                    event.paths.iter().any(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name == ".gitignore")
                            .unwrap_or(false)
                        || path.components().any(|c| c.as_os_str() == ".git")
                    })
                }
            }
        }
        _ => false,
    }
}

fn perform_scan(repo_root: &Path, watch_mode: &WatchMode) -> Result<()> {
    match watch_mode {
        WatchMode::TrackedFiles => perform_tracked_files_scan(repo_root),
        WatchMode::GitIgnore => perform_gitignore_scan(repo_root),
        WatchMode::Patterns(patterns) => perform_pattern_scan(repo_root, patterns),
    }
}

fn perform_tracked_files_scan(repo_root: &Path) -> Result<()> {
    // Load tracked files
    let mut tracked = tracked_files::TrackedFiles::load(repo_root)?;
    
    if tracked.marked_files.is_empty() {
        println!("{}", "No files are being tracked. Use 'dbx-ignore <files>' to mark files first.".yellow());
        return Ok(());
    }
    
    // Get current git-ignored files
    let git_ignored = git_utils::get_git_ignored_files_in_path(repo_root)?;
    let git_ignored_set: HashSet<_> = git_ignored.into_iter().collect();
    
    let mut updated = 0;
    let mut removed = 0;
    let mut errors = 0;
    
    // Check each tracked file
    for tracked_file in tracked.marked_files.clone() {
        if !tracked_file.exists() {
            // File no longer exists, remove from tracking
            tracked.remove_files(&[tracked_file.clone()]);
            removed += 1;
            continue;
        }
        
        let should_be_ignored = git_ignored_set.contains(&tracked_file);
        let has_marker = platform_utils::has_any_ignore_attribute(&tracked_file);
        
        if should_be_ignored && !has_marker {
            // File should be ignored but isn't - add marker
            match platform_utils::add_all_ignore_attributes(&tracked_file) {
                Ok(_) => {
                    updated += 1;
                    println!("  {} Added ignore marker to: {}", "✓".green(), tracked_file.display());
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("  {} Failed to add marker to {}: {}", "✗".red(), tracked_file.display(), e);
                }
            }
        } else if !should_be_ignored && has_marker {
            // File should not be ignored but has marker - remove it
            match platform_utils::remove_all_ignore_attributes(&tracked_file) {
                Ok(_) => {
                    updated += 1;
                    println!("  {} Removed ignore marker from: {}", "✓".green(), tracked_file.display());
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("  {} Failed to remove marker from {}: {}", "✗".red(), tracked_file.display(), e);
                }
            }
        }
    }
    
    // Save updated tracked files
    tracked.save(repo_root)?;
    
    if updated > 0 || removed > 0 || errors > 0 {
        println!(
            "{} {} files updated, {} removed from tracking, {} errors",
            "Summary:".green().bold(),
            updated,
            removed,
            errors
        );
    } else {
        println!("{}", "All tracked files are up to date.".green());
    }

    Ok(())
}

fn perform_gitignore_scan(repo_root: &Path) -> Result<()> {
    // Get all git-ignored files
    let git_ignored = git_utils::get_git_ignored_files_in_path(repo_root)?;
    
    let mut added = 0;
    let mut removed = 0;
    let mut errors = 0;
    
    // Process all git-ignored files
    for file_path in &git_ignored {
        if !platform_utils::has_any_ignore_attribute(file_path) {
            // File should be ignored but isn't - add marker
            match platform_utils::add_all_ignore_attributes(file_path) {
                Ok(_) => {
                    added += 1;
                    if added <= 10 {  // Limit output to first 10 files
                        println!("  {} Added ignore marker to: {}", "✓".green(), file_path.display());
                    }
                }
                Err(e) => {
                    errors += 1;
                    if errors <= 5 {  // Limit error output
                        eprintln!("  {} Failed to add marker to {}: {}", "✗".red(), file_path.display(), e);
                    }
                }
            }
        }
    }
    
    // Check for files that have markers but are no longer git-ignored
    let git_ignored_set: HashSet<_> = git_ignored.into_iter().collect();
    
    // Get all files with markers in the repository
    let marked_files = find_marked_files(repo_root)?;
    
    for marked_file in marked_files {
        if !git_ignored_set.contains(&marked_file) && platform_utils::has_any_ignore_attribute(&marked_file) {
            // File has marker but is no longer git-ignored - remove it
            match platform_utils::remove_all_ignore_attributes(&marked_file) {
                Ok(_) => {
                    removed += 1;
                    if removed <= 10 {  // Limit output
                        println!("  {} Removed ignore marker from: {}", "✓".green(), marked_file.display());
                    }
                }
                Err(e) => {
                    errors += 1;
                    if errors <= 5 {  // Limit error output
                        eprintln!("  {} Failed to remove marker from {}: {}", "✗".red(), marked_file.display(), e);
                    }
                }
            }
        }
    }
    
    if added > 10 {
        println!("  ... and {} more files", added - 10);
    }
    if removed > 10 {
        println!("  ... and {} more files", removed - 10);
    }
    if errors > 5 {
        eprintln!("  ... and {} more errors", errors - 5);
    }
    
    if added > 0 || removed > 0 || errors > 0 {
        println!(
            "{} {} markers added, {} removed, {} errors",
            "Summary:".green().bold(),
            added,
            removed,
            errors
        );
    } else {
        println!("{}", "All git-ignored files are properly marked.".green());
    }

    Ok(())
}

fn find_marked_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut marked_files = Vec::new();
    let mut stack = vec![repo_root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        // Check files in current directory
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() && platform_utils::has_any_ignore_attribute(&path) {
                        marked_files.push(path);
                    } else if file_type.is_dir() {
                        // Skip .git directory
                        if path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name != ".git")
                            .unwrap_or(false)
                        {
                            // Check if directory itself has marker
                            if platform_utils::has_any_ignore_attribute(&path) {
                                marked_files.push(path.clone());
                            }
                            stack.push(path);
                        }
                    }
                }
            }
        }
    }

    Ok(marked_files)
}

fn perform_pattern_scan(repo_root: &Path, patterns: &[String]) -> Result<()> {
    use glob::glob;
    
    let mut added = 0;
    let mut removed = 0;
    let mut errors = 0;
    
    // Build a set of all files that should be marked based on patterns
    let mut files_to_mark = HashSet::new();
    
    for pattern in patterns {
        // Convert relative pattern to absolute path pattern
        let abs_pattern = if pattern.starts_with('/') {
            pattern.to_string()
        } else {
            repo_root.join(pattern).to_string_lossy().to_string()
        };
        
        // Find all files matching the pattern
        match glob(&abs_pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.exists() {
                        files_to_mark.insert(entry);
                    }
                }
            }
            Err(e) => {
                eprintln!("  {} Invalid pattern '{}': {}", "✗".red(), pattern, e);
                errors += 1;
            }
        }
    }
    
    // Mark files that match patterns but aren't marked
    for file_path in &files_to_mark {
        if !platform_utils::has_any_ignore_attribute(file_path) {
            match platform_utils::add_all_ignore_attributes(file_path) {
                Ok(_) => {
                    added += 1;
                    if added <= 10 {
                        println!("  {} Added ignore marker to: {}", "✓".green(), file_path.display());
                    }
                }
                Err(e) => {
                    errors += 1;
                    if errors <= 5 {
                        eprintln!("  {} Failed to add marker to {}: {}", "✗".red(), file_path.display(), e);
                    }
                }
            }
        }
    }
    
    // Find all marked files and remove markers from those that don't match patterns
    let marked_files = find_marked_files(repo_root)?;
    
    for marked_file in marked_files {
        if !files_to_mark.contains(&marked_file) && platform_utils::has_any_ignore_attribute(&marked_file) {
            // Check if file matches any pattern
            let matches_pattern = patterns.iter().any(|pattern| {
                let abs_pattern = if pattern.starts_with('/') {
                    pattern.to_string()
                } else {
                    repo_root.join(pattern).to_string_lossy().to_string()
                };
                
                glob::Pattern::new(&abs_pattern)
                    .ok()
                    .map(|p| p.matches_path(&marked_file))
                    .unwrap_or(false)
            });
            
            if !matches_pattern {
                match platform_utils::remove_all_ignore_attributes(&marked_file) {
                    Ok(_) => {
                        removed += 1;
                        if removed <= 10 {
                            println!("  {} Removed ignore marker from: {}", "✓".green(), marked_file.display());
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        if errors <= 5 {
                            eprintln!("  {} Failed to remove marker from {}: {}", "✗".red(), marked_file.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    if added > 10 {
        println!("  ... and {} more files", added - 10);
    }
    if removed > 10 {
        println!("  ... and {} more files", removed - 10);
    }
    if errors > 5 {
        eprintln!("  ... and {} more errors", errors - 5);
    }
    
    if added > 0 || removed > 0 || errors > 0 {
        println!(
            "{} {} markers added, {} removed, {} errors",
            "Summary:".green().bold(),
            added,
            removed,
            errors
        );
    } else {
        println!("{}", "All files matching patterns are properly marked.".green());
    }

    Ok(())
}

fn find_gitignore_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut gitignore_files = Vec::new();
    let mut stack = vec![repo_root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let gitignore = dir.join(".gitignore");
        if gitignore.exists() {
            gitignore_files.push(gitignore);
        }

        // Recurse into subdirectories
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let path = entry.path();
                        // Skip .git directory
                        if path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name != ".git")
                            .unwrap_or(false)
                        {
                            stack.push(path);
                        }
                    }
                }
            }
        }
    }

    Ok(gitignore_files)
}