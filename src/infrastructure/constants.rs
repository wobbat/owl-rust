//! Application-wide constants

// Command names
pub const CMD_APPLY: &str = "apply";
pub const CMD_EDIT: &str = "edit";
pub const CMD_DE: &str = "de";
pub const CMD_CE: &str = "ce";
pub const CMD_DOTS: &str = "dots";
pub const CMD_ADD: &str = "add";
pub const CMD_ADOPT: &str = "adopt";
pub const CMD_CONFIGCHECK: &str = "configcheck";
pub const CMD_CONFIGHOST: &str = "confighost";

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

// State management paths
pub const STATE_DIR: &str = ".state";
pub const UNTRACKED_STATE: &str = "untracked.json";
pub const HIDDEN_STATE: &str = "hidden.txt";
pub const MANAGED_STATE: &str = "managed.json";

// Package manager
pub const PACKAGE_MANAGER: &str = "paru";

// Host name will be read from system
pub fn get_host_name() -> Result<String, std::io::Error> {
    std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string())
}

// Timing constants
pub const SPINNER_DELAY_MS: u64 = 120;
