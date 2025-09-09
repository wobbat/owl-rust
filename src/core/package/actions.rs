use std::collections::HashSet;
use crate::core::config::Config;
use crate::core::state::PackageState;

/// Package action types for planning installations and removals
#[derive(Debug, Clone, PartialEq)]
pub enum PackageAction {
    Install { name: String },
    Remove { name: String },
}

/// Plan package actions by comparing desired config with installed packages
pub fn plan_package_actions(
    config: &Config,
    state: &PackageState
) -> Result<Vec<PackageAction>, String> {
    let installed = super::get_installed_packages()?;
    let desired: HashSet<String> = config.packages.keys().cloned().collect();

    let mut actions = Vec::new();

    for package in &desired {
        if !super::is_package_or_group_installed(package)? {
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
    super::ParuPacman::new().remove_packages(packages, quiet)
}

/// Categorize packages into repo and AUR lists
pub fn categorize_packages(packages: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    if packages.is_empty() { return Ok((Vec::new(), Vec::new())); }
    let available = super::ParuPacman::new().batch_repo_available(packages)?;
    let mut repo_packages = Vec::new();
    let mut aur_packages = Vec::new();
    for p in packages {
        if available.contains(p) { repo_packages.push(p.clone()); }
        else { aur_packages.push(p.clone()); }
    }
    Ok((repo_packages, aur_packages))
}

/// Search packages using the PackageManager
pub fn search_packages(terms: &[String]) -> Result<Vec<super::SearchResult>, String> {
    super::ParuPacman::new().search_packages(terms)
}