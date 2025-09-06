use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource { Repo, Aur }

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
}

pub struct ParuPacman;
impl ParuPacman { pub fn new() -> Self { Self } }

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
            if !name.is_empty() { installed.insert(name.to_string()); }
        }
        Ok(installed)
    }

    fn batch_repo_available(&self, packages: &[String]) -> Result<HashSet<String>, String> {
        if packages.is_empty() { return Ok(HashSet::new()); }
        let mut cmd = Command::new("pacman");
        cmd.arg("-Si");
        cmd.args(packages);
        let output = cmd.output().map_err(|e| format!("Failed to check package info: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut repo_names = HashSet::new();
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Name") {
                if let Some(idx) = rest.find(':') {
                    let value = rest[idx + 1..].trim();
                    if !value.is_empty() { repo_names.insert(value.to_string()); }
                }
            }
        }
        Ok(repo_names)
    }

    fn upgrade_count(&self) -> Result<usize, String> {
        let output = Command::new(crate::internal::constants::PACKAGE_MANAGER)
            .args(["-Qu", "-q"])
            .output()
            .map_err(|e| format!(
                "Failed to run {} -Qu: {}",
                crate::internal::constants::PACKAGE_MANAGER,
                e
            ))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.lines().count())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(1) && stderr.trim().is_empty() { Ok(0) }
            else { Err(format!("{} -Qu failed: {}", crate::internal::constants::PACKAGE_MANAGER, stderr)) }
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
                    if l.is_empty() { return None; }
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
        if packages.is_empty() { return Ok(()); }
        let args = ["--repo", "-S", "--noconfirm"];
        let status = crate::internal::util::run_command_with_spinner(
            crate::internal::constants::PACKAGE_MANAGER,
            &args,
            &format!("Installing {} repo packages", packages.len()),
        )?;
        if !status.success() { return Err("Repository install failed".into()); }
        Ok(())
    }

    fn install_aur(&self, packages: &[String]) -> Result<(), String> {
        if packages.is_empty() { return Ok(()); }
        let mut args = vec!["--aur", "-S", "--noconfirm"];
        args.extend(packages.iter().map(|s| s.as_str()));
        let status = crate::internal::util::run_command_with_spinner(
            crate::internal::constants::PACKAGE_MANAGER,
            &args,
            &format!("Installing {} AUR packages", packages.len()),
        )?;
        if !status.success() { return Err("AUR install failed".into()); }
        Ok(())
    }

    fn update_repo(&self) -> Result<(), String> {
        let (status, _stderr_out) = crate::internal::util::run_command_with_spinner_capture(
            crate::internal::constants::PACKAGE_MANAGER,
            &["--repo", "-Syu", "--noconfirm"],
            "Updating official repository packages (syncing databases and upgrading packages)",
        )?;
        if status.success() {
            println!("  {} Official repos synced", crate::internal::color::green("⸎"));
            Ok(())
        } else if status.code() == Some(1) {
            println!("  {} Packages from main repos have been updated", crate::internal::color::green("⸎"));
            Ok(())
        } else {
            Err(format!("Repository update failed (exit code: {:?})", status.code()))
        }
    }

    fn update_aur(&self, packages: &[String]) -> Result<(), String> {
        if packages.is_empty() { return Ok(()); }
        let mut args = vec!["--aur", "-Syu", "--noconfirm"];
        args.extend(packages.iter().map(|s| s.as_str()));
        let (status, stderr_out) = crate::internal::util::run_command_with_spinner_capture(
            crate::internal::constants::PACKAGE_MANAGER,
            &args,
            "Updating AUR packages",
        ).map_err(|e| e.to_string())?;
        if status.success() {
            println!("\r\x1b[2K  {} AUR package updates completed", crate::internal::color::green("⸎"));
            Ok(())
        } else {
            let err = stderr_out.trim();
            if !err.is_empty() {
                let lines: Vec<&str> = err.lines().collect();
                let take = 30usize;
                let start = lines.len().saturating_sub(take);
                for line in &lines[start..] { eprintln!("  {}", line); }
            }
            Err("AUR package update failed".to_string())
        }
    }

    fn remove_packages(&self, packages: &[String], quiet: bool) -> Result<(), String> {
        if packages.is_empty() { return Ok(()); }
        let mut cmd = Command::new(crate::internal::constants::PACKAGE_MANAGER);
        cmd.arg("-Rns");
        if quiet { cmd.arg("--noconfirm"); }
        cmd.args(packages);
        let status = cmd.status().map_err(|e| format!("Failed to remove packages: {}", e))?;
        if status.success() {
            println!("  {} Removed {} package(s)", crate::internal::color::green("✓"), packages.len());
            Ok(())
        } else {
            Err("Package removal failed".to_string())
        }
    }

    fn search_packages(&self, terms: &[String]) -> Result<Vec<SearchResult>, String> {
        if terms.is_empty() { return Ok(Vec::new()); }
        let mut cmd = Command::new("paru");
        cmd.args(["-Ss", "--bottomup"]);
        cmd.args(terms);
        let output = cmd.output().map_err(|e| format!("Failed to run paru search: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Paru search failed: {}", stderr));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        parse_paru_search_output(&text)
    }
}

fn is_header_line(line: &str) -> bool {
    line.contains('/') && line.contains(' ') && !line.starts_with(' ') && !line.starts_with('[') && line.split_whitespace().next().unwrap_or("").contains('/')
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), String> {
    if let Some(slash_pos) = repo_name.find('/') {
        let repo = &repo_name[..slash_pos];
        let name = &repo_name[slash_pos + 1..];
        Ok((repo, name))
    } else {
        Err(format!("Invalid repo/name format: {}", repo_name))
    }
}

fn parse_header_line(line: &str) -> Result<SearchResult, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() { return Err("Empty header line".to_string()); }
    let repo_name_part = parts[0];
    let (repo, name) = parse_repo_name(repo_name_part)?;
    let version = parts.get(1).ok_or("Missing version in header line")?;
    let installed = line.contains("[installed]");
    Ok(SearchResult { name: name.to_string(), ver: version.to_string(), source: if repo == "aur" { PackageSource::Aur } else { PackageSource::Repo }, repo: repo.to_string(), description: String::new(), installed })
}

fn parse_paru_search_output(output: &str) -> Result<Vec<SearchResult>, String> {
    let mut results = Vec::new();
    let mut current_result: Option<SearchResult> = None;
    for line in output.lines() {
        let original_line = line;
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() { continue; }
        if is_header_line(trimmed_line) {
            if let Some(result) = current_result.take() { results.push(result); }
            current_result = Some(parse_header_line(trimmed_line)?);
        } else if original_line.starts_with("    ") {
            if let Some(ref mut result) = current_result {
                let desc_part = trimmed_line;
                if result.description.is_empty() { result.description = desc_part.to_string(); }
                else { result.description.push(' '); result.description.push_str(desc_part); }
            }
        }
    }
    if let Some(result) = current_result { results.push(result); }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(results[0].name, "jet-bin");
        assert_eq!(results[0].repo, "aur");
        assert_eq!(results[0].source, PackageSource::Aur);
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
