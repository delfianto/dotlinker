use anyhow::{Context, Result};
use chrono::Local;
use pathdiff::diff_paths;
use std::fs;
use std::path::{Path, PathBuf};

/// Create a symbolic link from source to destination
pub fn create_symlink(
    source: &Path,
    destination: &Path,
    use_relative: bool,
    dry_run: bool,
    force: bool,
    module_name: &str,
) -> Result<()> {
    // Check if source exists
    if !source.exists() {
        anyhow::bail!(
            "[MOD:{}] Source path does not exist: {:?}",
            module_name.to_uppercase(),
            source
        );
    }

    // Compute the link target (relative or absolute)
    let link_target = if use_relative {
        compute_relative_path(source, destination)?
    } else {
        source.to_path_buf()
    };

    // Check destination status and handle conflicts
    let dest_status = check_destination(destination)?;

    match dest_status {
        DestinationStatus::NotExists => {
            // Ensure parent directory exists
            if let Some(parent) = destination.parent() {
                if !dry_run {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent directory: {:?}", parent)
                    })?;
                }
                log::debug!(
                    "[MOD:{}] Ensured parent directory exists: {:?}",
                    module_name.to_uppercase(),
                    parent
                );
            }
        }
        DestinationStatus::ExistingSymlink(target) => {
            if target == link_target {
                log::info!(
                    "[MOD:{}] Symlink already correct: {:?} -> {:?}",
                    module_name.to_uppercase(),
                    destination,
                    link_target
                );
                return Ok(());
            } else if force {
                backup_and_remove(destination, dry_run, module_name)?;
            } else {
                log::warn!(
                    "[MOD:{}] Symlink exists with different target: {:?} -> {:?} (expected: {:?}). Use --force to overwrite.",
                    module_name.to_uppercase(),
                    destination,
                    target,
                    link_target
                );
                return Ok(());
            }
        }
        DestinationStatus::ExistingFile | DestinationStatus::ExistingDirectory => {
            if force {
                backup_and_remove(destination, dry_run, module_name)?;
            } else {
                log::warn!(
                    "[MOD:{}] Destination already exists: {:?}. Use --force to overwrite.",
                    module_name.to_uppercase(),
                    destination
                );
                return Ok(());
            }
        }
        DestinationStatus::BrokenSymlink => {
            log::warn!(
                "[MOD:{}] Removing broken symlink: {:?}",
                module_name.to_uppercase(),
                destination
            );
            if !dry_run {
                fs::remove_file(destination).with_context(|| {
                    format!("Failed to remove broken symlink: {:?}", destination)
                })?;
            }
        }
    }

    // Create the symlink
    log::info!(
        "[MOD:{}] Creating symlink: {:?} -> {:?}",
        module_name.to_uppercase(),
        destination,
        link_target
    );

    if !dry_run {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&link_target, destination).with_context(|| {
            format!(
                "Failed to create symlink: {:?} -> {:?}",
                destination, link_target
            )
        })?;

        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(&link_target, destination).with_context(
                    || {
                        format!(
                            "Failed to create directory symlink: {:?} -> {:?}",
                            destination, link_target
                        )
                    },
                )?;
            } else {
                std::os::windows::fs::symlink_file(&link_target, destination).with_context(
                    || {
                        format!(
                            "Failed to create file symlink: {:?} -> {:?}",
                            destination, link_target
                        )
                    },
                )?;
            }
        }
    }

    Ok(())
}

/// Compute relative path from destination's parent to source
fn compute_relative_path(source: &Path, destination: &Path) -> Result<PathBuf> {
    let dest_parent = destination
        .parent()
        .context("Destination has no parent directory")?;

    let relative = diff_paths(source, dest_parent).context("Failed to compute relative path")?;
    Ok(relative)
}

/// Check the status of the destination path
fn check_destination(path: &Path) -> Result<DestinationStatus> {
    if !path.exists() {
        // Check if it's a broken symlink
        if path.symlink_metadata().is_ok() {
            return Ok(DestinationStatus::BrokenSymlink);
        }
        return Ok(DestinationStatus::NotExists);
    }

    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

    if metadata.file_type().is_symlink() {
        let target =
            fs::read_link(path).with_context(|| format!("Failed to read symlink: {:?}", path))?;
        Ok(DestinationStatus::ExistingSymlink(target))
    } else if metadata.is_dir() {
        Ok(DestinationStatus::ExistingDirectory)
    } else {
        Ok(DestinationStatus::ExistingFile)
    }
}

/// Backup and remove an existing file or directory
fn backup_and_remove(path: &Path, dry_run: bool, module_name: &str) -> Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_dir = path
        .parent()
        .context("Path has no parent")?
        .join("_dot_backup");

    let filename = path.file_name().context("Path has no filename")?;
    let backup_path = backup_dir.join(format!("{}_{}", filename.to_string_lossy(), timestamp));

    log::info!(
        "[MOD:{}] Backing up: {:?} -> {:?}",
        module_name.to_uppercase(),
        path,
        backup_path
    );

    if !dry_run {
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;

        fs::rename(path, &backup_path)
            .with_context(|| format!("Failed to backup file: {:?}", path))?;
    }

    Ok(())
}

/// Status of a destination path
#[derive(Debug)]
enum DestinationStatus {
    NotExists,
    ExistingSymlink(PathBuf),
    ExistingFile,
    ExistingDirectory,
    BrokenSymlink,
}
