//! Package management utilities

use std::process::Command;
use std::collections::HashSet;
use crate::config::Config;
use crate::state::PackageState;

/// Package source types
#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Repo,
    Aur,
    Any,
}

/// Search result from package search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub ver: String,
    pub source: PackageSource,
    pub repo: String,
    pub description: String,
    pub installed: bool,
}

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
    let installed = get_installed_packages()?;
    let desired: HashSet<String> = config.packages.keys().cloned().collect();

    let mut actions = Vec::new();

    // Find packages to install (desired but not installed)
    for package in &desired {
        if !installed.contains(package) {
            actions.push(PackageAction::Install {
                name: package.clone()
            });
        }
    }

    // Find packages to remove (installed but not desired, and not untracked/hidden)
    // Find packages to remove (installed but not desired, and previously managed)
    for package in &installed {
        if !desired.contains(package) && state.is_managed(package) {
            actions.push(PackageAction::Remove {
                name: package.clone()
            });
        }
    }

    Ok(actions)
}

/// Get list of all installed packages
pub fn get_installed_packages() -> Result<HashSet<String>, String> {
    let output = Command::new(crate::constants::PACKAGE_MANAGER)
        .arg("-Q")
        .output()
        .map_err(|e| format!("Failed to get installed packages: {}", e))?;

    if !output.status.success() {
        return Err(format!("Package manager failed: {}",
            String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut installed = HashSet::new();

    for line in stdout.lines() {
        if let Some(package_name) = line.split_whitespace().next() {
            installed.insert(package_name.to_string());
        }
    }

    Ok(installed)
}

/// Remove unmanaged packages
pub fn remove_unmanaged_packages(packages: &[String], quiet: bool) -> Result<(), String> {
    if packages.is_empty() {
        return Ok(());
    }

    println!("Package cleanup (removing conflicting packages):");
    for package in packages {
        println!("  {} Removing: {}",
            crate::colo::red("remove"),
            crate::colo::yellow(package)
        );
    }

    let mut cmd = Command::new(crate::constants::PACKAGE_MANAGER);
    cmd.arg("-Rns"); // Remove with dependencies, no save

    if quiet {
        cmd.arg("--noconfirm");
    }

    cmd.args(packages);

    let status = cmd.status()
        .map_err(|e| format!("Failed to remove packages: {}", e))?;

    if !status.success() {
        return Err("Package removal failed".to_string());
    }

    println!("  {} Removed {} package(s)",
        crate::colo::green("✓"),
        packages.len()
    );

    Ok(())
}

/// Install packages using the package manager
pub fn install_packages(items: &[String]) -> Result<(), String> {
    if items.is_empty() {
        return Err("No packages specified for installation".to_string());
    }

    validate_package_names(items)?;

    println!("{}", crate::colo::blue("Installing packages..."));
    run_package_command(&["-S"], items, "install packages")
}

/// Validate package names for basic correctness
fn validate_package_names(items: &[String]) -> Result<(), String> {
    for item in items {
        if item.trim().is_empty() {
            return Err("Package names cannot be empty or whitespace only".to_string());
        }
        if item.contains(' ') {
            return Err(format!("Invalid package name '{}': names cannot contain spaces", item));
        }
    }
    Ok(())
}

/// Get the count of packages that can be upgraded
pub fn get_package_count() -> Result<usize, String> {
    let output = Command::new(crate::constants::PACKAGE_MANAGER)
        .arg("-Qu")
        .output()
        .map_err(|e| format!("Failed to run {} -Qu: {}", crate::constants::PACKAGE_MANAGER, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line_count = stdout.lines().count();
        Ok(line_count)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // paru -Qu exits with code 1 when there are no packages to upgrade
        // This is normal behavior, so treat it as 0 packages
        if output.status.code() == Some(1) && stderr.trim().is_empty() {
            Ok(0)
        } else {
            Err(format!("{} -Qu failed: {}", crate::constants::PACKAGE_MANAGER, stderr))
        }
    }
}

/// Update all packages
#[allow(dead_code)]
pub fn update_packages() -> Result<(), String> {
    run_package_command(&["-Syu", "--noconfirm"], &[], "update packages")
}

/// Check if a package is installed
///
/// Uses the package manager to query if a package is currently installed.
/// Returns `Ok(true)` if the package is installed, `Ok(false)` if not installed,
/// or `Err` if there was an error checking the package status.
///
/// # Arguments
/// * `package_name` - The name of the package to check
///
/// # Examples
/// ```
/// let installed = package::is_package_installed("bash")?;
/// if installed {
///     println!("bash is installed");
/// } else {
///     println!("bash is not installed");
/// }
/// ```
pub fn is_package_installed(package_name: &str) -> Result<bool, String> {
    let output = Command::new(crate::constants::PACKAGE_MANAGER)
        .arg("-Q")
        .arg(package_name)
        .output()
        .map_err(|e| format!("Failed to run {} -Q {}: {}", crate::constants::PACKAGE_MANAGER, package_name, e))?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_package_installed() {
        // Test with a package that should be installed
        let result = is_package_installed("bash");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with a package that should not be installed
        let result = is_package_installed("nonexistentpackage12345");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_repo_package() {
        // Test with a known repo package
        let result = is_repo_package("bash");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with a non-existent package
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

    #[test]
    fn test_parse_paru_search_output() {
        let sample_output = r#"aur/jet-bin 0.7.27-1 [+5 ~0.00]
    CLI to transform between JSON, EDN and Transit, powered with a minimal query language.
aur/clang-opencl-headers-minimal-git 21.0.0_r537041.f2e62cfca5e5-1 [+5 ~0.00]
    clang headers & include files for OpenCL, trunk version
extra/texlive-latexextra 2025.2-2 [29.63 MiB 95.69 MiB] (texlive)
    TeX Live - LaTeX additional packages
extra/nim 2.0.8-1 [13.08 MiB 58.55 MiB]
    Imperative, multi-paradigm, compiled programming language"#;

        let results = parse_paru_search_output(sample_output).unwrap();
        assert_eq!(results.len(), 4);

        // Test first result (AUR package)
        assert_eq!(results[0].name, "jet-bin");
        assert_eq!(results[0].repo, "aur");
        assert_eq!(results[0].source, PackageSource::Aur);
        assert_eq!(results[0].ver, "0.7.27-1");
        assert!(results[0].description.contains("CLI to transform"));

        // Test third result (Official repo package)
        assert_eq!(results[2].name, "texlive-latexextra");
        assert_eq!(results[2].repo, "extra");
        assert_eq!(results[2].source, PackageSource::Repo);
    }

    #[test]
    fn test_parse_repo_name() {
        assert_eq!(parse_repo_name("aur/package-name").unwrap(), ("aur", "package-name"));
        assert_eq!(parse_repo_name("extra/bash").unwrap(), ("extra", "bash"));
        assert!(parse_repo_name("invalid-format").is_err());
    }

    #[test]
    fn test_is_header_line() {
        assert!(is_header_line("aur/jet-bin 0.7.27-1 [+5 ~0.00]"));
        assert!(is_header_line("extra/texlive-latexextra 2025.2-2 [29.63 MiB 95.69 MiB] (texlive)"));
        assert!(!is_header_line("    Description line"));
        assert!(!is_header_line("[some other format]"));
    }
}

/// Determine if a package is available in official repositories
pub fn is_repo_package(package_name: &str) -> Result<bool, String> {
    let output = Command::new("pacman")
        .arg("-Si")
        .arg(package_name)
        .output()
        .map_err(|e| format!("Failed to check package info: {}", e))?;

    Ok(output.status.success())
}

/// Categorize packages into repo and AUR lists
pub fn categorize_packages(packages: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    if packages.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Use parallel processing for repo checks
    use rayon::prelude::*;

    let results: Result<Vec<(Option<String>, Option<String>)>, String> = packages
        .par_iter()
        .map(|package| {
            match is_repo_package(package) {
                Ok(true) => Ok((Some(package.clone()), None)),
                Ok(false) => Ok((None, Some(package.clone()))),
                Err(e) => Err(format!("Failed to check {}: {}", package, e)),
            }
        })
        .collect();

    let categorized = results?;
    let (repo_packages, aur_packages): (Vec<String>, Vec<String>) = categorized
        .into_iter()
        .fold((Vec::new(), Vec::new()), |(mut repos, mut aurs), (repo, aur)| {
            if let Some(r) = repo {
                repos.push(r);
            }
            if let Some(a) = aur {
                aurs.push(a);
            }
            (repos, aurs)
        });

    Ok((repo_packages, aur_packages))
}

/// Search packages using paru -Ss --bottomup
pub fn search_packages_paru(terms: &[String]) -> Result<Vec<SearchResult>, String> {
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let output = run_paru_search(terms)?;
    parse_paru_search_output(&output)
}

/// Execute paru search command
fn run_paru_search(terms: &[String]) -> Result<String, String> {
    let mut cmd = Command::new("paru");
    cmd.args(&["-Ss", "--bottomup"]);
    cmd.args(terms);

    let output = cmd.output()
        .map_err(|e| format!("Failed to run paru search: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Paru search failed: {}", stderr))
    }
}

/// Parse paru search output into SearchResult structs
fn parse_paru_search_output(output: &str) -> Result<Vec<SearchResult>, String> {
    let mut results = Vec::new();
    let mut current_result: Option<SearchResult> = None;

    for line in output.lines() {
        let original_line = line;
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        // Check for header lines: repo/name version [flags]
        if is_header_line(trimmed_line) {
            // Flush previous result
            if let Some(result) = current_result.take() {
                results.push(result);
            }

            current_result = Some(parse_header_line(trimmed_line)?);
        } else if original_line.starts_with("    ") {
            // Description lines are indented with 4 spaces
            if let Some(ref mut result) = current_result {
                let desc_part = trimmed_line;
                if result.description.is_empty() {
                    result.description = desc_part.to_string();
                } else {
                    // Handle multi-line descriptions
                    result.description.push(' ');
                    result.description.push_str(desc_part);
                }
            }
        }
    }

    // Don't forget the last result
    if let Some(result) = current_result {
        results.push(result);
    }

    Ok(results)
}

/// Check if a line is a header line (repo/name version format)
fn is_header_line(line: &str) -> bool {
    // Contains "/" and " ", doesn't start with " " or "[", and contains a repo/name pattern
    line.contains('/') && line.contains(' ') &&
    !line.starts_with(' ') && !line.starts_with('[') &&
    line.split_whitespace().next().unwrap_or("").contains('/')
}

/// Parse a header line into a SearchResult
fn parse_header_line(line: &str) -> Result<SearchResult, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty header line".to_string());
    }

    // First part should be "repo/name"
    let repo_name_part = parts[0];
    let (repo, name) = parse_repo_name(repo_name_part)?;

    // Second part is version
    let version = parts.get(1)
        .ok_or("Missing version in header line")?;

    // Check for installed status
    let installed = line.contains("[installed]");

    Ok(SearchResult {
        name: name.to_string(),
        ver: version.to_string(),
        source: if repo == "aur" { PackageSource::Aur } else { PackageSource::Repo },
        repo: repo.to_string(),
        description: String::new(),
        installed,
    })
}

/// Parse "repo/name" into (repo, name)
fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), String> {
    if let Some(slash_pos) = repo_name.find('/') {
        let repo = &repo_name[..slash_pos];
        let name = &repo_name[slash_pos + 1..];
        Ok((repo, name))
    } else {
        Err(format!("Invalid repo/name format: {}", repo_name))
    }
}

/// Run a package manager command with given args and items
fn run_package_command(args: &[&str], items: &[String], operation: &str) -> Result<(), String> {
    let mut cmd = Command::new(crate::constants::PACKAGE_MANAGER);
    cmd.args(args);
    if !items.is_empty() {
        cmd.args(items);
    }

    match cmd.status() {
        Ok(status) if status.success() => {
            if operation.contains("install") {
                println!("{}", crate::colo::green("✓ Packages installed successfully"));
            }
            Ok(())
        }
        Ok(status) => {
            Err(format!(
                "Failed to {} (exit code: {})",
                operation,
                status.code().unwrap_or(-1)
            ))
        }
        Err(e) => {
            Err(format!("Error running {}: {}", crate::constants::PACKAGE_MANAGER, e))
        }
    }
}