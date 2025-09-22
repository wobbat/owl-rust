//! File operations utilities

use std::env;
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Result};

/// Scan a directory for .owl files and add them to the files vector
pub fn scan_directory_for_owl_files(directory_path: &str, files: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(directory_path) {
        for entry in entries.flatten() {
            if let Some(path) = entry.path().to_str()
                && path.ends_with(crate::internal::constants::OWL_EXT) {
                    files.push(path.to_string());
                }
        }
    }
}

/// Open a file in the user's preferred editor
pub fn open_editor(path: &str) -> Result<()> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| crate::internal::constants::DEFAULT_EDITOR.to_string());

    Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| anyhow!("Failed to open editor '{}': {}", editor, e))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow!("Editor '{}' exited with error", editor))
            }
        })
}

/// Find a config file in the standard locations
pub fn find_config_file(arg: &str) -> Result<String> {
    let home = env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;

    let base_dir = format!("{}/{}", home, crate::internal::constants::OWL_DIR);
    let search_paths = [
        format!(
            "{}/{}{}",
            base_dir,
            arg,
            crate::internal::constants::OWL_EXT
        ),
        format!("{}/{}", base_dir, arg),
        format!(
            "{}/{}/{}{}",
            base_dir,
            crate::internal::constants::HOSTS_DIR,
            arg,
            crate::internal::constants::OWL_EXT
        ),
        format!(
            "{}/{}/{}",
            base_dir,
            crate::internal::constants::HOSTS_DIR,
            arg
        ),
        format!(
            "{}/{}/{}{}",
            base_dir,
            crate::internal::constants::GROUPS_DIR,
            arg,
            crate::internal::constants::OWL_EXT
        ),
        format!(
            "{}/{}/{}",
            base_dir,
            crate::internal::constants::GROUPS_DIR,
            arg
        ),
    ];

    for path in &search_paths {
        if Path::new(path).exists() {
            return Ok(path.clone());
        }
    }

    Err(anyhow!("config file not found"))
}

/// Get the path for a dotfile
pub fn get_dotfile_path(filename: &str) -> Result<String> {
    let home = env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;

    Ok(format!(
        "{}/{}/{}/{}",
        home,
        crate::internal::constants::OWL_DIR,
        crate::internal::constants::DOTFILES_DIR,
        filename
    ))
}
