use anyhow::Result;
use clap::{Arg, Command};
use colored::*;
use dbx_ignore::{Config, run};
use std::path::PathBuf;

fn main() -> Result<()> {
    let app = Command::new("dbx-ignore")
        .version("0.1.0")
        .author("Claude Code")
        .about("Prevent Dropbox from syncing files by adding ignore markers")
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('n')
                .help("Preview what files would be marked to ignore")
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
                .help("Mark git-ignored files to prevent Dropbox sync (default if no files specified)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("files")
                .help("Files or directories to mark as ignored by Dropbox")
                .num_args(0..)
                .value_name("FILE"),
        );

    let matches = app.get_matches();

    let config = Config {
        dry_run: matches.get_flag("dry-run"),
        verbose: matches.get_flag("verbose"),
        quiet: matches.get_flag("quiet"),
        files: matches
            .get_many::<String>("files")
            .unwrap_or_default()
            .map(PathBuf::from)
            .collect(),
        git_mode: matches.get_flag("git") || matches.get_many::<String>("files").is_none(),
    };

    if config.verbose && config.quiet {
        eprintln!("{}", "Error: Cannot use both --verbose and --quiet".red());
        std::process::exit(1);
    }

    run(config)
}