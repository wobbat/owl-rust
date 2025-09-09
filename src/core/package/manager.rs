use std::collections::HashSet;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Repo,
    Aur,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub ver: String,
    pub source: PackageSource,
    pub repo: String,
    pub description: String,
    pub installed: bool,
}

pub trait PackageManager {
    fn list_installed(&self) -> Result<HashSet<String>, String>;
    fn batch_repo_available(&self, packages: &[String]) -> Result<HashSet<String>, String>;
    fn upgrade_count(&self) -> Result<usize, String>;
    fn get_aur_updates(&self) -> Result<Vec<String>, String>;
    fn install_repo(&self, packages: &[String]) -> Result<(), String>;
    fn install_aur(&self, packages: &[String]) -> Result<(), String>;
    fn update_repo(&self) -> Result<(), String>;
    fn update_aur(&self, packages: &[String]) -> Result<(), String>;
    fn remove_packages(&self, packages: &[String], quiet: bool) -> Result<(), String>;
    fn search_packages(&self, terms: &[String]) -> Result<Vec<SearchResult>, String>;
    fn is_package_group(&self, package_name: &str) -> Result<bool, String>;
    fn get_group_packages(&self, group_name: &str) -> Result<Vec<String>, String>;
}

pub struct ParuPacman;
impl ParuPacman {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for ParuPacman {
    fn list_installed(&self) -> Result<HashSet<String>, String> {
        let output = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .arg("-Qq")
            .output()
            .map_err(|e| format!("Failed to get installed packages: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "Package manager failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashSet::new();
        for line in stdout.lines() {
            let name = line.trim();
            if !name.is_empty() {
                installed.insert(name.to_string());
            }
        }
        Ok(installed)
    }

    fn batch_repo_available(&self, packages: &[String]) -> Result<HashSet<String>, String> {
        if packages.is_empty() {
            return Ok(HashSet::new());
        }
        let mut cmd = Command::new("pacman");
        cmd.arg("-Si");
        cmd.args(packages);
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to check package info: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut repo_names = HashSet::new();
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Name") {
                if let Some(idx) = rest.find(':') {
                    let value = rest[idx + 1..].trim();
                    if !value.is_empty() {
                        repo_names.insert(value.to_string());
                    }
                }
            }
        }
        Ok(repo_names)
    }

    fn upgrade_count(&self) -> Result<usize, String> {
        let output = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .args(["-Qu", "-q"])
            .output()
            .map_err(|e| {
                format!(
                    "Failed to run {} -Qu: {}",
                    crate::internal::constants::PACKAGE_MANAGER,
                    e
                )
            })?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.lines().count())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(1) && stderr.trim().is_empty() {
                Ok(0)
            } else {
                Err(format!(
                    "{} -Qu failed: {}",
                    crate::internal::constants::PACKAGE_MANAGER,
                    stderr
                ))
            }
        }
    }

    fn get_aur_updates(&self) -> Result<Vec<String>, String> {
        let output = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .args(["-Qua", "-q"])
            .output()
            .map_err(|e| format!("Failed to check AUR updates: {}", e))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let l = line.trim();
                    if l.is_empty() {
                        return None;
                    }
                    Some(l.split_whitespace().next().unwrap_or(l).to_string())
                })
                .collect();
            Ok(packages)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(1) && stderr.trim().is_empty() {
                // Treat as no updates
                Ok(Vec::new())
            } else {
                Err(format!("AUR update check failed: {}", stderr))
            }
        }
    }

    fn install_repo(&self, packages: &[String]) -> Result<(), String> {
        if packages.is_empty() {
            return Ok(());
        }
        let status = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .arg("-S")
            .arg("--noconfirm")
            .args(packages)
            .status()
            .map_err(|e| format!("Failed to install repo packages: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to install repo packages: {}", status))
        }
    }

    fn install_aur(&self, packages: &[String]) -> Result<(), String> {
        if packages.is_empty() {
            return Ok(());
        }
        let status = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .arg("-S")
            .args(packages)
            .status()
            .map_err(|e| format!("Failed to install AUR packages: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to install AUR packages: {}", status))
        }
    }

    fn update_repo(&self) -> Result<(), String> {
        let status = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .args(["-Syu", "--noconfirm"])
            .status()
            .map_err(|e| format!("Failed to update repo packages: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to update repo packages: {}", status))
        }
    }

    fn update_aur(&self, packages: &[String]) -> Result<(), String> {
        if packages.is_empty() {
            return Ok(());
        }
        let status = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .arg("-Sua")
            .args(packages)
            .status()
            .map_err(|e| format!("Failed to update AUR packages: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to update AUR packages: {}", status))
        }
    }

    fn remove_packages(&self, packages: &[String], quiet: bool) -> Result<(), String> {
        if packages.is_empty() {
            return Ok(());
        }
        let mut cmd = Command::new(crate::internal::constants::PACKAGE_MANAGER);
        cmd.arg("-R");
        if quiet {
            cmd.arg("--noconfirm");
        }
        cmd.args(packages);
        let status = cmd
            .status()
            .map_err(|e| format!("Failed to remove packages: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to remove packages: {}", status))
        }
    }

    fn search_packages(&self, terms: &[String]) -> Result<Vec<SearchResult>, String> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let mut cmd = Command::new(crate::internal::constants::PACKAGE_MANAGER);
        cmd.arg("-Ss");
        cmd.args(terms);
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to search packages: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "Package search failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        let mut current_result: Option<SearchResult> = None;
        for line in stdout.lines() {
            if line.starts_with(crate::internal::constants::PACKAGE_MANAGER) {
                // New package entry
                if let Some(result) = current_result.take() {
                    results.push(result);
                }
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    let repo = parts[0].to_string();
                    let name_and_ver = parts[1].trim();
                    let name_parts: Vec<&str> = name_and_ver.split(' ').collect();
                    if name_parts.len() >= 2 {
                        let name = name_parts[0].to_string();
                        let ver = name_parts[1].to_string();
                        let source = if repo == "aur" { PackageSource::Aur } else { PackageSource::Repo };
                        current_result = Some(SearchResult {
                            name,
                            ver,
                            source,
                            repo,
                            description: String::new(),
                            installed: false,
                        });
                    }
                }
            } else if let Some(ref mut result) = current_result {
                // Description line
                let desc = line.trim();
                if !desc.is_empty() {
                    result.description = desc.to_string();
                }
                // Check if installed (look for [installed] marker)
                if line.contains("[installed]") {
                    result.installed = true;
                }
            }
        }
        if let Some(result) = current_result {
            results.push(result);
        }
        Ok(results)
    }

    fn is_package_group(&self, package_name: &str) -> Result<bool, String> {
        let output = Command::new("pacman")
            .args(["-Sg", package_name])
            .output()
            .map_err(|e| format!("Failed to check if package is a group: {}", e))?;
        Ok(output.status.success())
    }

    fn get_group_packages(&self, group_name: &str) -> Result<Vec<String>, String> {
        let output = Command::new("pacman")
            .args(["-Sgg", group_name])
            .output()
            .map_err(|e| format!("Failed to get group packages: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "Failed to get group packages: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut packages = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                packages.push(parts[1].to_string());
            }
        }
        Ok(packages)
    }
}