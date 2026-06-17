mod config;
mod linker;
mod logging;

#[cfg(test)]
mod test_utils;

use anyhow::{Context, Result};
use clap::Parser;
use linker::DotLinker;
use std::path::PathBuf;

/// Simple dotfile symlink manager
#[derive(Parser, Debug)]
#[command(name = "dot.rs")]
#[command(about = "Simple dotfile package linker in Rust", long_about = None)]
#[command(version)]
struct Cli {
    /// Module names to process
    #[arg(value_name = "MODULES")]
    modules: Vec<String>,

    /// Perform a dry run without creating symlinks
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Use relative symlinks instead of absolute
    #[arg(long, default_value_t = false)]
    relative: bool,

    /// Force overwrite existing files/symlinks
    #[arg(long, default_value_t = false)]
    force: bool,

    /// Path to dotfiles directory (defaults to parent of this script)
    #[arg(long, value_name = "PATH")]
    dotfiles_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    logging::init(cli.verbose)?;

    // Print dry-run warning
    if cli.dry_run {
        log::warn!("--- DRY RUN MODE ---");
        log::warn!("--- No actual linking will be performed ---");
    }

    // Determine dotfiles directory
    let dotfiles_dir = match cli.dotfiles_dir {
        Some(dir) => dir,
        None => {
            // Default: parent directory of the current executable or cwd
            std::env::current_dir().context("Could not determine current directory")?
        }
    };

    // Initialize the linker
    let linker = DotLinker::new(
        dotfiles_dir,
        cli.dry_run,
        cli.verbose,
        cli.relative,
        cli.force,
    )?;

    // Process each module
    for module_name in &cli.modules {
        log::info!("Processing module {}", module_name);

        if let Err(e) = linker.process_module(module_name) {
            log::error!("Failed to process module '{}': {:?}", module_name, e);
        }
    }

    log::info!("All modules processed successfully");
    Ok(())
}
