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
    Ok(owl_dir()?.join(crate::internal::constants::ENV_BASH_FILE))
}

/// Get fish environment file path
fn env_file_fish() -> Result<std::path::PathBuf, String> {
    Ok(owl_dir()?.join(crate::internal::constants::ENV_FISH_FILE))
}

pub fn collect_all_env_vars(config: &crate::core::config::Config) -> Vec<(String, String)> {
    let mut vars: HashMap<String, String> = HashMap::new();
    // Global first
    for (k, v) in &config.env_vars { vars.insert(k.clone(), v.clone()); }
    // Package-level, override globals
    for (_name, pkg) in &config.packages {
        for (k, v) in &pkg.env_vars { vars.insert(k.clone(), v.clone()); }
    }
    let mut v: Vec<(String, String)> = vars.into_iter().collect();
    v.sort_by(|a,b| a.0.cmp(&b.0));
    v
}

pub fn handle_environment_combined(config: &crate::core::config::Config, dry_run: bool) -> Result<(), String> {
    let vars = collect_all_env_vars(config);
    if vars.is_empty() { return Ok(()); }

    if dry_run {
        println!("  {} Plan:", crate::internal::color::blue("info:"));
        for (k, v) in &vars {
            println!("    ✓ Would export {}={} (shells)", crate::internal::color::yellow(k), crate::internal::color::green(v));
        }
        return Ok(());
    }

    // Write bash
    let bash_path = env_file_bash()?;
    let mut bash = String::new();
    for (k, v) in &vars { bash.push_str(&format!("export {}=\"{}\"\n", k, v)); }
    fs::write(&bash_path, bash).map_err(|e| format!("Failed to write {}: {}", bash_path.display(), e))?;

    // Write fish
    let fish_path = env_file_fish()?;
    let mut fish = String::new();
    for (k, v) in &vars { fish.push_str(&format!("set -x {} \"{}\"\n", k, v)); }
    fs::write(&fish_path, fish).map_err(|e| format!("Failed to write {}: {}", fish_path.display(), e))?;

    println!("  {} Environment exported (bash, fish)", crate::internal::color::green("⸎"));
    Ok(())
}
