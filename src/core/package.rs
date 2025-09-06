//! Package management utilities

use std::collections::HashSet;
use std::sync::OnceLock;
use crate::core::pm::{PackageManager, ParuPacman, SearchResult};
use crate::core::config::Config;
use crate::core::state::PackageState;

/// Package action types for planning installations and removals
#[derive(Debug, Clone, PartialEq)]
pub enum PackageAction {
    Install { name: String },
    Remove { name: String },
}

// Cache of installed packages for the current process run
static INSTALLED_CACHE: OnceLock<HashSet<String>> = OnceLock::new();
static PACKAGE_COUNT_CACHE: OnceLock<usize> = OnceLock::new();

fn query_installed_packages() -> Result<HashSet<String>, String> {
    ParuPacman::new().list_installed()
}

/// Plan package actions by comparing desired config with installed packages
pub fn plan_package_actions(
    config: &Config,
    state: &PackageState
) -> Result<Vec<PackageAction>, String> {
    let installed = get_installed_packages()?;
    let desired: HashSet<String> = config.packages.keys().cloned().collect();

    let mut actions = Vec::new();

    for package in &desired {
        if !installed.contains(package) {
            actions.push(PackageAction::Install { name: package.clone() });
        }
    }

    for package in &installed {
        if !desired.contains(package) && state.is_managed(package) {
            actions.push(PackageAction::Remove { name: package.clone() });
        }
    }

    Ok(actions)
}

/// Get list of all installed packages
pub fn get_installed_packages() -> Result<HashSet<String>, String> {
    if let Some(cached) = INSTALLED_CACHE.get() {
        return Ok(cached.clone());
    }
    let installed = query_installed_packages()?;
    let _ = INSTALLED_CACHE.set(installed.clone());
    Ok(installed)
}

/// Remove unmanaged packages
pub fn remove_unmanaged_packages(packages: &[String], quiet: bool) -> Result<(), String> {
    if packages.is_empty() { return Ok(()); }
    println!("Package cleanup (removing conflicting packages):");
    for package in packages {
        println!(
            "  {} Removing: {}",
            crate::internal::color::red("remove"),
            crate::internal::color::yellow(package)
        );
    }
    ParuPacman::new().remove_packages(packages, quiet)
}

/// Get the count of packages that can be upgraded
pub fn get_package_count() -> Result<usize, String> {
    if let Some(cached) = PACKAGE_COUNT_CACHE.get() { return Ok(*cached); }
    let count = ParuPacman::new().upgrade_count()?;
    let _ = PACKAGE_COUNT_CACHE.set(count);
    Ok(count)
}

/// Check if a package is installed
pub fn is_package_installed(package_name: &str) -> Result<bool, String> {
    if let Some(cached) = INSTALLED_CACHE.get() { return Ok(cached.contains(package_name)); }
    let installed = query_installed_packages()?;
    let contains = installed.contains(package_name);
    let _ = INSTALLED_CACHE.set(installed);
    Ok(contains)
}

/// Determine if a package is available in official repositories
#[cfg(test)]
pub fn is_repo_package(package_name: &str) -> Result<bool, String> {
    let set = ParuPacman::new().batch_repo_available(&[package_name.to_string()])?;
    Ok(set.contains(package_name))
}

/// Categorize packages into repo and AUR lists
pub fn categorize_packages(packages: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    if packages.is_empty() { return Ok((Vec::new(), Vec::new())); }
    let available = ParuPacman::new().batch_repo_available(packages)?;
    let mut repo_packages = Vec::new();
    let mut aur_packages = Vec::new();
    for p in packages {
        if available.contains(p) { repo_packages.push(p.clone()); }
        else { aur_packages.push(p.clone()); }
    }
    Ok((repo_packages, aur_packages))
}

/// Search packages using the PackageManager
pub fn search_packages(terms: &[String]) -> Result<Vec<SearchResult>, String> {
    ParuPacman::new().search_packages(terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_package_installed() {
        let result = is_package_installed("bash");
        assert!(result.is_ok());
        assert!(result.unwrap());
        let result = is_package_installed("nonexistentpackage12345");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_repo_package() {
        let result = is_repo_package("bash");
        assert!(result.is_ok());
        assert!(result.unwrap());
        let result = is_repo_package("nonexistentpackage12345");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_categorize_packages() {
        let packages = vec!["bash".to_string(), "nonexistentpackage12345".to_string()];
        let result = categorize_packages(&packages);
        assert!(result.is_ok());
        let (repo_packages, aur_packages) = result.unwrap();
        assert!(repo_packages.contains(&"bash".to_string()));
        assert!(aur_packages.contains(&"nonexistentpackage12345".to_string()));
    }
}
