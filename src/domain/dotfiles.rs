//! Dotfile synchronization functionality
//!
//! This module handles the synchronization of dotfiles from the dotfiles directory
//! to their target locations in the user's home directory.

use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

/// Represents a dotfile mapping from source to destination
#[derive(Debug, Clone)]
pub struct DotfileMapping {
    pub source: String,
    pub destination: String,
}

/// Status of a dotfile operation
#[derive(Debug, Clone, PartialEq)]
pub enum DotfileStatus {
    Create,
    Update,
    Conflict,
    Skip,
    UpToDate,
}

/// Result of analyzing a dotfile
#[derive(Debug, Clone)]
pub struct DotfileAction {
    pub source: String,
    pub destination: String,
    pub status: DotfileStatus,
    pub reason: Option<String>,
}

/// Resolve source path relative to dotfiles directory if not absolute
pub fn resolve_source_path(source: &str) -> Result<PathBuf, String> {
    if source.starts_with('/') || source.starts_with("./") || source.starts_with("../") {
        // Absolute or explicit relative path
        Ok(PathBuf::from(source))
    } else {
        // Relative to dotfiles directory
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home)
            .join(crate::infrastructure::constants::OWL_DIR)
            .join(crate::infrastructure::constants::DOTFILES_DIR)
            .join(source))
    }
}

/// Resolve destination path with tilde expansion
pub fn resolve_destination_path(dest: &str) -> Result<PathBuf, String> {
    if dest.starts_with('~') {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(dest.replacen('~', &home, 1)))
    } else {
        Ok(PathBuf::from(dest))
    }
}

/// Calculate SHA256 hash of a file
fn hash_file(path: &Path) -> Result<String, String> {
    if !path.exists() || !path.is_file() {
        return Ok(String::new());
    }

    let content = fs::read(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash_bytes = hasher.finalize();

    Ok(format!("{:x}", hash_bytes))
}

fn files_differ_quick(src: &Path, dest: &Path) -> Result<bool, String> {
    let src_meta = fs::metadata(src)
        .map_err(|e| format!("Failed to stat {}: {}", src.display(), e))?;
    let dst_meta = fs::metadata(dest)
        .map_err(|e| format!("Failed to stat {}: {}", dest.display(), e))?;

    // If sizes differ, definitely different
    if src_meta.len() != dst_meta.len() {
        return Ok(true);
    }

    // If mtimes match and sizes match, assume equal (fast path)
    let src_time = src_meta.modified().ok();
    let dst_time = dst_meta.modified().ok();
    if let (Some(st), Some(dt)) = (src_time, dst_time) {
        if st == dt {
            return Ok(false);
        }
    }

    // Otherwise, confirm by hashing
    let source_hash = hash_file(src)?;
    let dest_hash = hash_file(dest)?;
    if source_hash.is_empty() || dest_hash.is_empty() {
        // fall back to treating as different if hash failed
        return Ok(true);
    }
    Ok(source_hash != dest_hash)
}

/// Calculate SHA256 hash of a directory recursively
fn hash_directory(path: &Path) -> Result<String, String> {
    if !path.exists() || !path.is_dir() {
        return Ok(String::new());
    }

    let mut entries = Vec::new();

    // Walk directory recursively
    fn walk_dir(dir: &Path, base: &Path, entries: &mut Vec<String>) -> Result<(), String> {
        let entries_iter = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        let mut dir_entries = Vec::new();
        for entry in entries_iter {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            let rel_path = path.strip_prefix(base)
                .map_err(|e| format!("Failed to get relative path: {}", e))?
                .to_string_lossy()
                .replace("\\", "/"); // Normalize path separators

            if path.is_file() {
                let hash = hash_file(&path)?;
                if !hash.is_empty() {
                    dir_entries.push(format!("{}:{}", rel_path, hash));
                }
            } else if path.is_dir() {
                walk_dir(&path, base, entries)?;
            }
        }

        // Sort entries for deterministic hash
        dir_entries.sort();
        entries.extend(dir_entries);
        Ok(())
    }

    walk_dir(path, path, &mut entries)?;
    entries.sort();

    let combined = entries.join("\n");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_bytes = hasher.finalize();

    Ok(format!("{:x}", hash_bytes))
}

/// Calculate hash of a path (file or directory)
pub fn hash_path(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }

    if path.is_dir() {
        hash_directory(path)
    } else {
        hash_file(path)
    }
}

/// Analyze what actions need to be taken for dotfiles
pub fn analyze_dotfiles(mappings: &[DotfileMapping]) -> Result<Vec<DotfileAction>, String> {
    let mut actions = Vec::new();

    for mapping in mappings {
        let source_path = resolve_source_path(&mapping.source)?;
        let dest_path = resolve_destination_path(&mapping.destination)?;

        let mut action = DotfileAction {
            source: mapping.source.clone(),
            destination: mapping.destination.clone(),
            status: DotfileStatus::Skip,
            reason: None,
        };

        // Check if source exists
        if !source_path.exists() {
            action.status = DotfileStatus::Conflict;
            action.reason = Some("source file not found".to_string());
            actions.push(action);
            continue;
        }

        // Check if destination exists
        if !dest_path.exists() {
            action.status = DotfileStatus::Create;
            actions.push(action);
            continue;
        }

        // Check for file vs directory type mismatch
        let source_is_dir = source_path.is_dir();
        let dest_is_dir = dest_path.is_dir();

        if source_is_dir && !dest_is_dir {
            action.status = DotfileStatus::Conflict;
            action.reason = Some("destination is file, source is directory".to_string());
            actions.push(action);
            continue;
        }

        if !source_is_dir && dest_is_dir {
            action.status = DotfileStatus::Conflict;
            action.reason = Some("destination is directory, source is file".to_string());
            actions.push(action);
            continue;
        }

        // For files, do a quick metadata compare then hash if needed
        if !source_is_dir {
            match files_differ_quick(&source_path, &dest_path) {
                Ok(true) => action.status = DotfileStatus::Update,
                Ok(false) => {
                    action.status = DotfileStatus::UpToDate;
                    action.reason = Some("content matches".to_string());
                }
                Err(e) => {
                    action.status = DotfileStatus::Conflict;
                    action.reason = Some(format!("failed to compare: {}", e));
                }
            }
        } else {
            // Directories: compute full content hash as before
            let source_hash = hash_path(&source_path)?;
            let dest_hash = hash_path(&dest_path)?;
            if source_hash.is_empty() || dest_hash.is_empty() {
                action.status = DotfileStatus::Conflict;
                action.reason = Some("failed to calculate hash".to_string());
            } else if source_hash != dest_hash {
                action.status = DotfileStatus::Update;
            } else {
                action.status = DotfileStatus::UpToDate;
                action.reason = Some("content matches".to_string());
            }
        }

        actions.push(action);
    }

    Ok(actions)
}

/// Remove a path safely (file or directory)
fn remove_path_safely(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory {}: {}", path.display(), e))
    } else {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))
    }
}

/// Copy a path (file or directory) recursively
fn copy_path(src: &Path, dest: &Path) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
    }

    // Remove existing destination
    remove_path_safely(dest)?;

    if src.is_dir() {
        copy_directory_recursive(src, dest)
    } else {
        fs::copy(src, dest)
            .map_err(|e| format!("Failed to copy file {} to {}: {}", src.display(), dest.display(), e))?;
        Ok(())
    }
}

/// Recursively copy directory contents
fn copy_directory_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    if !dest.exists() {
        fs::create_dir_all(dest)
            .map_err(|e| format!("Failed to create directory {}: {}", dest.display(), e))?;
    }

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(file_name);

        if src_path.is_dir() {
            copy_directory_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {} to {}: {}", src_path.display(), dest_path.display(), e))?;
        }
    }

    Ok(())
}

/// Apply dotfile actions (actually copy files)
pub fn apply_dotfiles(mappings: &[DotfileMapping], dry_run: bool) -> Result<Vec<DotfileAction>, String> {
    let actions = analyze_dotfiles(mappings)?;

    if dry_run {
        return Ok(actions);
    }

    let mut results = Vec::new();

    for action in actions {
        if matches!(action.status, DotfileStatus::Conflict | DotfileStatus::UpToDate | DotfileStatus::Skip) {
            results.push(action);
            continue;
        }

        // Create or update -> copy
        let source_path = resolve_source_path(&action.source)?;
        let dest_path = resolve_destination_path(&action.destination)?;

        match copy_path(&source_path, &dest_path) {
            Ok(_) => {
                results.push(action);
            }
            Err(e) => {
                let mut failed_action = action;
                failed_action.status = DotfileStatus::Conflict;
                failed_action.reason = Some(format!("Copy failed: {}", e));
                results.push(failed_action);
            }
        }
    }

    Ok(results)
}

/// Check if any dotfile mappings have actionable status
pub fn has_actionable_dotfiles(mappings: &[DotfileMapping]) -> Result<bool, String> {
    let actions = analyze_dotfiles(mappings)?;
    Ok(actions.iter().any(|a| matches!(a.status, DotfileStatus::Create | DotfileStatus::Update | DotfileStatus::Conflict)))
}

/// Get dotfile mappings from config
use crate::domain::config;
pub fn get_dotfile_mappings(config: &config::Config) -> Vec<DotfileMapping> {
    config.packages.values()
        .filter_map(|pkg| {
            if let Some(config_str) = &pkg.config {
                // Parse the stored "source -> dest" format
                if let Some((src, dst)) = config_str.split_once(" -> ") {
                    Some(DotfileMapping {
                        source: src.trim().to_string(),
                        destination: dst.trim().to_string(),
                    })
                } else {
                    // For configs without source, assume source is the same as dest but in dotfiles dir
                    // Extract the filename from the destination path
                    let dest_path = config_str.trim();
                    let filename = std::path::Path::new(dest_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(dest_path);
                    Some(DotfileMapping {
                        source: filename.to_string(),
                        destination: dest_path.to_string(),
                    })
                }
            } else {
                None
            }
        })
        .collect()
}

/// Print a concise summary of dotfile actions with consistent formatting
pub fn print_actions(actions: &[DotfileAction], dry_run: bool) {
    // Count up-to-date dotfiles
    let up_to_date_count = actions
        .iter()
        .filter(|action| matches!(action.status, DotfileStatus::UpToDate))
        .count();

    if up_to_date_count > 0 {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::infrastructure::color::green("➔"),
            up_to_date_count
        );
    }

    // Show individual actions only for changes
    for action in actions {
        match action.status {
            DotfileStatus::Create => {
                if dry_run {
                    println!(
                        "  {} Would create: {} -> {}",
                        crate::infrastructure::color::blue("ℹ"),
                        action.source,
                        action.destination
                    );
                } else {
                    println!(
                        "  {} Created: {} -> {}",
                        crate::infrastructure::color::green("➔"),
                        action.source,
                        action.destination
                    );
                }
            }
            DotfileStatus::Update => {
                if dry_run {
                    println!(
                        "  {} Would update: {} -> {}",
                        crate::infrastructure::color::blue("ℹ"),
                        action.source,
                        action.destination
                    );
                } else {
                    println!(
                        "  {} Updated: {} -> {}",
                        crate::infrastructure::color::green("➔"),
                        action.source,
                        action.destination
                    );
                }
            }
            DotfileStatus::Conflict => {
                let reason = action
                    .reason
                    .clone()
                    .unwrap_or_else(|| "Unknown conflict".to_string());
                println!(
                    "  {} Conflict: {} ({})",
                    crate::infrastructure::color::red("✗"),
                    action.destination,
                    reason
                );
            }
            DotfileStatus::UpToDate => {}
            DotfileStatus::Skip => {}
        }
    }

    if dry_run {
        println!(
            "  {} Dotfile analysis completed (dry-run mode)",
            crate::infrastructure::color::blue("ℹ")
        );
    }
}
