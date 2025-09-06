use std::collections::HashMap;
use std::env as std_env;
use std::fs;
use std::path::Path;

/// Get the Owl directory path
fn owl_dir() -> Result<std::path::PathBuf, String> {
    let home = std_env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(Path::new(&home).join(".owl"))
}

/// Get bash environment file path
fn env_file_bash() -> Result<std::path::PathBuf, String> {
    Ok(owl_dir()?.join(crate::infrastructure::constants::ENV_BASH_FILE))
}

/// Get fish environment file path
fn env_file_fish() -> Result<std::path::PathBuf, String> {
    Ok(owl_dir()?.join(crate::infrastructure::constants::ENV_FISH_FILE))
}

/// Ensure Owl directories exist
fn ensure_owl_directories() -> Result<(), String> {
    let dir = owl_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create Owl directory: {}", e))?;
    }
    Ok(())
}

/// Read existing environment variables from bash file
#[allow(clippy::collapsible_if)]
fn read_existing_env_vars() -> HashMap<String, String> {
    let mut existing = HashMap::new();
    let bash_file = match env_file_bash() {
        Ok(path) => path,
        Err(_) => return existing,
    };

    if !bash_file.exists() {
        return existing;
    }

    let content = match fs::read_to_string(&bash_file) {
        Ok(content) => content,
        Err(_) => return existing,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("export ") && line.contains('=') {
            let export_removed = &line[7..]; // Remove "export "
            if let Some(eq_index) = export_removed.find('=') {
                if eq_index > 0 {
                    let key = &export_removed[..eq_index];
                    let mut value = export_removed[eq_index + 1..].to_string();

                    // Remove quotes if present
                    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
                        value = value[1..value.len() - 1].to_string();
                        // Unescape bash quotes
                        value = value.replace("\\'", "'");
                    }

                    existing.insert(key.to_string(), value);
                }
            }
        }
    }

    existing
}

/// Environment comparison result
#[derive(Debug, Clone)]
pub struct EnvComparison {
    pub added: Vec<(String, String)>,
    pub updated: Vec<(String, String)>,
    pub unchanged: Vec<(String, String)>,
    pub removed: Vec<String>,
}

/// Compare environment variables and return categorized results
pub fn compare_env_vars(new_envs: &[(String, String)]) -> EnvComparison {
    let existing = read_existing_env_vars();
    let mut result = EnvComparison {
        added: Vec::new(),
        updated: Vec::new(),
        unchanged: Vec::new(),
        removed: Vec::new(),
    };

    // Check new/updated vars
    for env in new_envs {
        let key = &env.0;
        let value = &env.1;

        if let Some(existing_value) = existing.get(key) {
            if existing_value == value {
                result.unchanged.push(env.clone());
            } else {
                result.updated.push(env.clone());
            }
        } else {
            result.added.push(env.clone());
        }
    }

    // Check removed vars
    let new_keys: std::collections::HashSet<_> = new_envs.iter().map(|(k, _)| k).collect();

    for key in existing.keys() {
        if !new_keys.contains(key) {
            result.removed.push(key.clone());
        }
    }

    result
}

enum ShellStyle {
    Bash,
    Fish,
}

fn render_env_content(envs: &[(String, String)], style: ShellStyle) -> String {
    match style {
        ShellStyle::Bash => {
            let mut content = String::from("#!/bin/bash\n");
            content.push_str("# This file is managed by Owl package manager\n");
            content.push_str("# Manual changes may be overwritten\n");
            if !envs.is_empty() {
                content.push('\n');
                for (key, value) in envs {
                    let escaped = value.replace("'", "'\\''");
                    content.push_str(&format!("export {}='{}'\n", key, escaped));
                }
            }
            content
        }
        ShellStyle::Fish => {
            let mut content = String::from("# This file is managed by Owl package manager\n");
            content.push_str("# Manual changes may be overwritten");
            if !envs.is_empty() {
                content.push_str("\n\n");
                for (key, value) in envs {
                    let escaped = value.replace("'", "\\'");
                    content.push_str(&format!("set -x {} '{}'\n", key, escaped));
                }
            }
            content
        }
    }
}

fn write_env_file(path: &Path, envs: &[(String, String)], style: ShellStyle) -> Result<(), String> {
    let content = render_env_content(envs, style);
    fs::write(path, content)
        .map_err(|e| format!("Failed to write environment file {}: {}", path.display(), e))
}

/// Write bash environment file
fn write_env_bash(envs: &[(String, String)]) -> Result<(), String> {
    let path = env_file_bash()?;
    write_env_file(&path, envs, ShellStyle::Bash)
}

/// Write fish environment file
fn write_env_fish(envs: &[(String, String)]) -> Result<(), String> {
    let path = env_file_fish()?;
    write_env_file(&path, envs, ShellStyle::Fish)
}

/// Set environment variables by writing both shell files
pub fn set_environment_variables(envs: &[(String, String)]) -> Result<(), String> {
    ensure_owl_directories()?;
    write_env_bash(envs)?;
    write_env_fish(envs)?;
    Ok(())
}

/// Collect all environment variables from config (package + global)
pub fn collect_all_env_vars(config: &crate::domain::config::Config) -> Vec<(String, String)> {
    let mut all_env_vars = Vec::new();

    // Add global environment variables
    for (key, value) in &config.env_vars {
        all_env_vars.push((key.clone(), value.clone()));
    }

    // Add package environment variables
    for package in config.packages.values() {
        for (key, value) in &package.env_vars {
            all_env_vars.push((key.clone(), value.clone()));
        }
    }

    all_env_vars
}

// Handle environment variables for apply command
// Removed unused handle_environment; environment is handled via handle_environment_combined

/// Handle environment variables combined with system section (no separate header)
pub fn handle_environment_combined(
    config: &crate::domain::config::Config,
    dry_run: bool,
) -> Result<(), String> {
    let all_env_vars = collect_all_env_vars(config);
    if all_env_vars.is_empty() {
        return Ok(());
    }

    if dry_run {
        show_environment_dry_run_content(&all_env_vars);
    } else {
        let comparison = compare_env_vars(&all_env_vars);
        set_environment_variables(&all_env_vars)?;
        show_environment_changes_content(&comparison);
    }

    Ok(())
}

/// Show environment dry-run content (without header)
fn show_environment_dry_run_content(env_vars: &[(String, String)]) {
    println!(
        "  {} Environment variables to set:",
        crate::infrastructure::color::blue("ℹ")
    );

    for (key, value) in env_vars {
        println!(
            "    {} Would set: {}={}",
            crate::infrastructure::color::blue("✓"),
            crate::infrastructure::color::yellow(key),
            crate::infrastructure::color::green(value)
        );
    }
}

/// Show environment changes content (without header)
pub fn show_environment_changes_content(comparison: &EnvComparison) {
    if !comparison.added.is_empty() {
        for (key, value) in &comparison.added {
            println!(
                "  {} Set: {}={}",
                crate::infrastructure::color::green("➔"),
                crate::infrastructure::color::yellow(key),
                crate::infrastructure::color::green(value)
            );
        }
    }

    if !comparison.updated.is_empty() {
        for (key, value) in &comparison.updated {
            println!(
                "  {} Updated: {}={}",
                crate::infrastructure::color::green("➔"),
                crate::infrastructure::color::yellow(key),
                crate::infrastructure::color::green(value)
            );
        }
    }

    if !comparison.removed.is_empty() {
        for key in &comparison.removed {
            println!(
                "  {} Removed: {}",
                crate::infrastructure::color::green("➔"),
                crate::infrastructure::color::yellow(key)
            );
        }
    }

    if comparison.added.is_empty() && comparison.updated.is_empty() && comparison.removed.is_empty()
    {
        if !comparison.unchanged.is_empty() {
            println!(
                "  {} Environment variables maintained ({} unchanged)",
                crate::infrastructure::color::green("➔"),
                crate::infrastructure::color::blue(&comparison.unchanged.len().to_string())
            );
        }
    } else if !comparison.unchanged.is_empty() {
        println!(
            "  {} ({} environment variables unchanged)",
            crate::infrastructure::color::dim(""),
            crate::infrastructure::color::blue(&comparison.unchanged.len().to_string())
        );
    }
}
