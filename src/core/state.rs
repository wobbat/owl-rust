//! Package state management for tracking untracked and hidden packages

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::internal::constants;

/// Default system packages that should not be tracked
fn default_untracked_packages() -> Vec<String> {
    vec![
        "linux".to_string(),
        "linux-firmware".to_string(),
        "intel-ucode".to_string(),
        "amd-ucode".to_string(),
        "base".to_string(),
        "base-devel".to_string(),
        "glibc".to_string(),
        "filesystem".to_string(),
        "bash".to_string(),
        "coreutils".to_string(),
        "findutils".to_string(),
        "grep".to_string(),
        "gawk".to_string(),
        "sed".to_string(),
        "less".to_string(),
        "util-linux".to_string(),
        "procps-ng".to_string(),
        "shadow".to_string(),
        "iproute2".to_string(),
        "iputils".to_string(),
        "pacman".to_string(),
        "pacman-contrib".to_string(),
        "gzip".to_string(),
        "xz".to_string(),
        "tar".to_string(),
        "openssl".to_string(),
        "ca-certificates".to_string(),
        "e2fsprogs".to_string(),
    ]
}

/// Package state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageState {
    pub untracked: Vec<String>,
    pub hidden: Vec<String>,
    pub managed: Vec<String>,
}

impl PackageState {
    /// Load package state from ~/.owl/.state directory
    pub fn load() -> Result<Self, String> {
        let state_dir = Self::get_state_dir()?;
        if !state_dir.exists() { fs::create_dir_all(&state_dir).map_err(|e| format!("Failed to create state directory: {}", e))?; }
        let untracked = Self::load_untracked_packages(&state_dir)?;
        let hidden = Self::load_hidden_packages(&state_dir)?;
        let managed = Self::load_managed_packages(&state_dir)?;
        Ok(PackageState { untracked, hidden, managed })
    }

    /// Save package state to disk
    pub fn save(&self) -> Result<(), String> {
        let state_dir = Self::get_state_dir()?;
        if !state_dir.exists() { fs::create_dir_all(&state_dir).map_err(|e| format!("Failed to create state directory: {}", e))?; }
        Self::save_untracked_packages(&state_dir, &self.untracked)?;
        Self::save_hidden_packages(&state_dir, &self.hidden)?;
        Self::save_managed_packages(&state_dir, &self.managed)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_untracked(&self, package: &str) -> bool { self.untracked.contains(&package.to_string()) }
    #[allow(dead_code)]
    pub fn is_hidden(&self, package: &str) -> bool { self.hidden.contains(&package.to_string()) }
    pub fn is_managed(&self, package: &str) -> bool { self.managed.contains(&package.to_string()) }

    #[allow(dead_code)]
    pub fn add_untracked(&mut self, package: String) { if !self.untracked.contains(&package) { self.untracked.push(package); self.untracked.sort(); } }
    #[allow(dead_code)]
    pub fn remove_untracked(&mut self, package: &str) { self.untracked.retain(|p| p != package); }
    #[allow(dead_code)]
    pub fn add_hidden(&mut self, package: String) { if !self.hidden.contains(&package) { self.hidden.push(package); self.hidden.sort(); } }
    #[allow(dead_code)]
    pub fn remove_hidden(&mut self, package: &str) { self.hidden.retain(|p| p != package); }
    pub fn add_managed(&mut self, package: String) { if !self.managed.contains(&package) { self.managed.push(package); self.managed.sort(); } }
    pub fn remove_managed(&mut self, package: &str) { self.managed.retain(|p| p != package); }

    fn get_state_dir() -> Result<PathBuf, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(constants::OWL_DIR).join(constants::STATE_DIR))
    }

    fn load_untracked_packages(state_dir: &std::path::Path) -> Result<Vec<String>, String> {
        let untracked_path = state_dir.join(constants::UNTRACKED_STATE);
        if !untracked_path.exists() {
            let default_packages = default_untracked_packages();
            Self::save_untracked_packages(state_dir, &default_packages)?;
            return Ok(default_packages);
        }
        let content = fs::read_to_string(&untracked_path).map_err(|e| format!("Failed to read untracked packages file: {}", e))?;
        let packages: Vec<String> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse untracked packages JSON: {}", e))?;
        Ok(packages)
    }

    fn save_untracked_packages(state_dir: &std::path::Path, packages: &[String]) -> Result<(), String> {
        let untracked_path = state_dir.join(constants::UNTRACKED_STATE);
        let json = serde_json::to_string_pretty(packages).map_err(|e| format!("Failed to serialize untracked packages: {}", e))?;
        fs::write(&untracked_path, json).map_err(|e| format!("Failed to write untracked packages file: {}", e))?;
        Ok(())
    }

    fn load_hidden_packages(state_dir: &std::path::Path) -> Result<Vec<String>, String> {
        let hidden_path = state_dir.join(constants::HIDDEN_STATE);
        if !hidden_path.exists() { return Ok(Vec::new()); }
        let content = fs::read_to_string(&hidden_path).map_err(|e| format!("Failed to read hidden packages file: {}", e))?;
        let packages: Vec<String> = content.lines().map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
        Ok(packages)
    }

    fn save_hidden_packages(state_dir: &std::path::Path, packages: &[String]) -> Result<(), String> {
        let hidden_path = state_dir.join(constants::HIDDEN_STATE);
        let content = packages.join("\n") + "\n";
        fs::write(&hidden_path, content).map_err(|e| format!("Failed to write hidden packages file: {}", e))?;
        Ok(())
    }

    fn load_managed_packages(state_dir: &std::path::Path) -> Result<Vec<String>, String> {
        let managed_path = state_dir.join(constants::MANAGED_STATE);
        if !managed_path.exists() { let empty_packages: Vec<String> = Vec::new(); Self::save_managed_packages(state_dir, &empty_packages)?; return Ok(empty_packages); }
        let content = fs::read_to_string(&managed_path).map_err(|e| format!("Failed to read managed packages file: {}", e))?;
        let packages: Vec<String> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse managed packages JSON: {}", e))?;
        Ok(packages)
    }

    fn save_managed_packages(state_dir: &std::path::Path, packages: &[String]) -> Result<(), String> {
        let managed_path = state_dir.join(constants::MANAGED_STATE);
        let json = serde_json::to_string_pretty(packages).map_err(|e| format!("Failed to serialize managed packages: {}", e))?;
        fs::write(&managed_path, json).map_err(|e| format!("Failed to write managed packages file: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_initial_state() {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        let state = PackageState::load().unwrap();
        assert!(!state.untracked.is_empty());
        assert!(state.is_untracked("linux"));
        assert!(state.is_untracked("base"));
    }

    #[test]
    fn test_add_remove_untracked() {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        let mut state = PackageState::load().unwrap();
        state.add_untracked("test-package".to_string());
        assert!(state.is_untracked("test-package"));
        state.remove_untracked("test-package");
        assert!(!state.is_untracked("test-package"));
    }
}
