//! Application-wide constants

 use anyhow::{anyhow, Result};

// Command names
pub const CMD_APPLY: &str = "apply";
pub const CMD_EDIT: &str = "edit";
pub const CMD_DOTS: &str = "dots";
pub const CMD_ADD: &str = "add";
pub const CMD_ADOPT: &str = "adopt";
pub const CMD_FIND: &str = "find";
pub const CMD_CONFIGCHECK: &str = "configcheck";
pub const CMD_CONFIGHOST: &str = "confighost";
pub const CMD_CLEAN: &str = "clean";

// Edit types
pub const EDIT_TYPE_DOTS: &str = "dots";
pub const EDIT_TYPE_CONFIG: &str = "config";

// Default editor
pub const DEFAULT_EDITOR: &str = "vim";

// Directory paths
pub const OWL_DIR: &str = ".owl";
pub const DOTFILES_DIR: &str = "dotfiles";
pub const HOSTS_DIR: &str = "hosts";
pub const GROUPS_DIR: &str = "groups";
pub const OWL_EXT: &str = ".owl";

// Config filenames
// MAIN_CONFIG_BASENAME removed; use MAIN_CONFIG_FILE directly
pub const MAIN_CONFIG_FILE: &str = "main.owl";

// Environment filenames under ~/.owl
pub const ENV_BASH_FILE: &str = "env.sh";
pub const ENV_FISH_FILE: &str = "env.fish";

// State management paths
pub const STATE_DIR: &str = ".state";

// Package manager
pub const PACKAGE_MANAGER: &str = "paru";

// Host name will be read from system
pub fn get_host_name() -> Result<String> {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow!("Failed to read hostname: {}", e))
}

// Timing constants
pub const SPINNER_DELAY_MS: u64 = 120;
