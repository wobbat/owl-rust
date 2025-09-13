use crate::core::pm::PackageManager;

/// Get list of AUR packages that can be updated
pub fn get_aur_updates() -> Result<Vec<String>, String> {
    crate::core::pm::ParuPacman::new().get_aur_updates()
}

/// Count packages that have dotfile configurations
pub fn count_dotfile_packages(config: &crate::core::config::Config) -> usize {
    config
        .packages
        .values()
        .filter(|pkg| pkg.config.is_some())
        .count()
}

/// Count total environment variables (package + global)
pub fn count_environment_variables(config: &crate::core::config::Config) -> usize {
    let package_env_vars = config
        .packages
        .values()
        .map(|pkg| pkg.env_vars.len())
        .sum::<usize>();
    package_env_vars + config.env_vars.len()
}

pub type Analysis = (
    usize,                                    // package_count
    crate::core::config::Config,              // config
    crate::core::state::PackageState,         // state
    Vec<crate::core::package::PackageAction>, // actions
    usize,                                    // dotfile_count
    usize,                                    // env_var_count
    usize,                                    // service_count
    usize,                                    // config_package_count
);

pub fn analyze_system() -> Result<Analysis, String> {
    use std::thread;

    // Run independent, potentially slow operations in parallel
    // 1) Count upgradable packages
    let count_handle = thread::spawn(|| crate::core::package::get_package_count());
    // 2) Load config files
    let config_handle = thread::spawn(|| {
        crate::core::config::Config::load_all_relevant_config_files().map_err(|e| e.to_string())
    });
    // 3) Load package state from disk
    let state_handle = thread::spawn(|| crate::core::state::PackageState::load());
    // 4) Prewarm installed package cache to avoid repeated -Q calls later
    let installed_warm_handle = thread::spawn(|| {
        let _ = crate::core::package::get_installed_packages();
        Ok::<(), String>(())
    });

    // Join results
    let package_count = count_handle
        .join()
        .map_err(|_| "Failed to join package count thread".to_string())?
        .map_err(|e| format!("Failed to get package count: {}", e))?;

    let mut state = state_handle
        .join()
        .map_err(|_| "Failed to join state loader thread".to_string())?
        .map_err(|e| format!("Failed to load package state: {}", e))?;

    let config = config_handle
        .join()
        .map_err(|_| "Failed to join config loader thread".to_string())?
        .map_err(|e| format!("Failed to load config: {}", e))?;

    // Ensure installed cache warm-up finished (best-effort)
    let _ = installed_warm_handle.join();

    // Seed managed state with currently installed packages that are present in config.
    // This ensures future removals are detected only for packages user explicitly managed via config.
    if seed_managed_with_desired_installed(&config, &mut state)? {
        // Best-effort save; don't fail analysis if saving state fails.
        if let Err(e) = state.save() {
            eprintln!(
                "{}",
                crate::internal::color::red(&format!("Failed to save seeded package state: {}", e))
            );
        }
    }

    // Plan package actions (installs and removals)
    let actions = crate::core::package::plan_package_actions(&config, &state)
        .map_err(|e| format!("Failed to plan package actions: {}", e))?;

    // Calculate dynamic values
    let dotfile_count = count_dotfile_packages(&config);
    let env_var_count = count_environment_variables(&config);
    let service_count = crate::core::services::get_configured_services(&config).len();
    let config_package_count = config.packages.len();

    Ok((
        package_count,
        config,
        state,
        actions,
        dotfile_count,
        env_var_count,
        service_count,
        config_package_count,
    ))
}

/// Ensure packages that are currently in the config and installed are marked as managed
pub fn seed_managed_with_desired_installed(
    config: &crate::core::config::Config,
    state: &mut crate::core::state::PackageState,
) -> Result<bool, String> {
    let mut changed = false;
    for pkg in config.packages.keys() {
        if !state.is_managed(pkg) {
            match crate::core::package::is_package_or_group_installed(pkg) {
                Ok(true) => {
                    state.add_managed(pkg.to_string());
                    changed = true;
                }
                Ok(false) => {}
                Err(e) => {
                    eprintln!(
                        "{}",
                        crate::internal::color::red(&format!(
                            "Failed to verify installation of {}: {}",
                            pkg, e
                        ))
                    );
                }
            }
        }
    }
    Ok(changed)
}
