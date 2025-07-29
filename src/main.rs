use anyhow::Result;
use clap::{Arg, Command};
use colored::Colorize;
use dbx_ignore::{Action, Config, run};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut app = Command::new("dbx-ignore")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Manage Dropbox ignore markers on files and directories")
        .arg(
            Arg::new("reset")
                .long("reset")
                .short('r')
                .help("Remove ignore markers from files and directories")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("watch")
                .long("watch")
                .short('w')
                .help("Start daemon to monitor files/patterns. Can accept patterns directly: --watch \"*.log\"")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("unwatch")
                .long("unwatch")
                .short('u')
                .help("Stop the daemon watcher")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("status")
                .long("status")
                .short('s')
                .help("Show the status of the current folder")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("daemon-mode")
                .long("daemon-mode")
                .help("Internal flag for daemon mode")
                .hide(true)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('n')
                .help("Preview what files would be processed")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Suppress output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("git")
                .long("git")
                .short('g')
                .help("Process git-ignored files (default if no files specified)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("files")
                .help("Files, directories, wildcards, or .gitignore files to process. Use '.' for current directory contents")
                .num_args(0..)
                .value_name("FILE"),
        );

    // If no arguments provided, show help
    if std::env::args().len() == 1 {
        app.print_help()?;
        println!();
        return Ok(());
    }

    let matches = app.get_matches();

    // Check if status mode is requested
    if matches.get_flag("status") {
        let verbose = matches.get_flag("verbose");
        return dbx_ignore::show_status(verbose);
    }

    // Determine action based on flags
    let action = if matches.get_flag("reset") {
        if matches.get_flag("watch") || matches.get_flag("unwatch") {
            eprintln!("{}", "Error: Cannot combine --reset with --watch or --unwatch".red());
            std::process::exit(1);
        }
        Action::Reset
    } else if matches.get_flag("watch") {
        if matches.get_flag("unwatch") {
            eprintln!("{}", "Error: Cannot use both --watch and --unwatch".red());
            std::process::exit(1);
        }
        Action::Watch
    } else if matches.get_flag("unwatch") {
        Action::Unwatch
    } else {
        Action::Ignore
    };

    let file_args: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .cloned()
        .collect();
    
    let files: Vec<PathBuf> = file_args.iter()
        .map(PathBuf::from)
        .collect();
    
    // Detect which arguments are patterns (contain wildcards)
    let patterns: Vec<String> = file_args.iter()
        .filter(|arg| dbx_ignore::is_glob_pattern(arg))
        .cloned()
        .collect();
    
    // Validate dangerous operations
    if action == Action::Ignore && !files.is_empty() {
        // Check if user is trying to ignore current directory or everything
        let dangerous_patterns = [
            PathBuf::from("."),
            PathBuf::from("*"),
            PathBuf::from("./"),
            PathBuf::from("./*"),
        ];
        
        if files.iter().any(|f| dangerous_patterns.contains(f) || f.to_str() == Some("*")) {
            // Check if we have a git repository with .gitignore
            let current_dir = std::env::current_dir().unwrap_or_default();
            let has_gitignore = current_dir.join(".gitignore").exists();
            let in_git_repo = git2::Repository::discover(&current_dir).is_ok();
            
            if !has_gitignore || !in_git_repo {
                eprintln!("{}", "Error: Cannot mark entire directory without a .gitignore file in a git repository.".red());
                eprintln!("{}", "This safeguard prevents accidentally marking all files for Dropbox ignore.".yellow());
                eprintln!();
                eprintln!("{}", "Options:".bold());
                eprintln!("  1. Create a .gitignore file and specify which files to ignore");
                eprintln!("  2. Specify individual files or directories: dbx-ignore node_modules/ target/");
                eprintln!("  3. Use --git mode to process git-ignored files: dbx-ignore --git");
                eprintln!();
                eprintln!("Run 'dbx-ignore --help' for more information.");
                std::process::exit(1);
            }
        }
    }

    let config = Config {
        action,
        dry_run: matches.get_flag("dry-run"),
        verbose: matches.get_flag("verbose"),
        quiet: matches.get_flag("quiet"),
        files,
        patterns,
        git_mode: matches.get_flag("git") || matches.get_many::<String>("files").is_none(),
        daemon_mode: matches.get_flag("daemon-mode"),
    };

    if config.verbose && config.quiet {
        eprintln!("{}", "Error: Cannot use both --verbose and --quiet".red());
        std::process::exit(1);
    }

    run(config)
}