//! File operations utilities

use std::env;
use std::path::Path;
use std::process::Command;

/// Open a file in the user's preferred editor
pub fn open_editor(path: &str) -> Result<(), String> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| crate::constants::DEFAULT_EDITOR.to_string());

    Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("Editor '{}' exited with error", editor))
            }
        })
}

/// Find a config file in the standard locations
pub fn find_config_file(arg: &str) -> Result<String, String> {
    let home = env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;

    let base_dir = format!("{}/{}", home, crate::constants::OWL_DIR);
    let search_paths = [
        format!("{}/{}{}", base_dir, arg, crate::constants::OWL_EXT),
        format!("{}/{}", base_dir, arg),
        format!("{}/{}/{}{}", base_dir, crate::constants::HOSTS_DIR, arg, crate::constants::OWL_EXT),
        format!("{}/{}/{}", base_dir, crate::constants::HOSTS_DIR, arg),
        format!("{}/{}/{}{}", base_dir, crate::constants::GROUPS_DIR, arg, crate::constants::OWL_EXT),
        format!("{}/{}/{}", base_dir, crate::constants::GROUPS_DIR, arg),
    ];

    for path in &search_paths {
        if Path::new(path).exists() {
            return Ok(path.clone());
        }
    }

    Err("config file not found".to_string())
}

/// Get the path for a dotfile
pub fn get_dotfile_path(filename: &str) -> Result<String, String> {
    let home = env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;

    Ok(format!(
        "{}/{}/{}/{}",
        home,
        crate::constants::OWL_DIR,
        crate::constants::DOTFILES_DIR,
        filename
    ))
}