/// Get list of AUR packages that can be updated
fn get_aur_updates() -> Result<Vec<String>, String> {
    crate::core::pm::ParuPacman::new().get_aur_updates()
}

/// Count packages that have dotfile configurations
fn count_dotfile_packages(config: &crate::core::config::Config) -> usize {
    config
        .packages
        .values()
        .filter(|pkg| pkg.config.is_some())
        .count()
}

/// Count total environment variables (package + global)
fn count_environment_variables(config: &crate::core::config::Config) -> usize {
    let package_env_vars = config
        .packages
        .values()
        .map(|pkg| pkg.env_vars.len())
        .sum::<usize>();
    package_env_vars + config.env_vars.len()
}

type Analysis = (
    usize,                          // package_count
    crate::core::config::Config,          // config
    crate::core::state::PackageState,     // state
    Vec<crate::core::package::PackageAction>, // actions
    usize,                          // dotfile_count
    usize,                          // env_var_count
    usize,                          // service_count
    usize,                          // config_package_count
);

fn analyze_system() -> Result<Analysis, String> {
    use std::thread;

    // Run independent, potentially slow operations in parallel
    // 1) Count upgradable packages
    let count_handle = thread::spawn(|| crate::core::package::get_package_count());
    // 2) Load config files
    let config_handle = thread::spawn(|| {
        crate::core::config::Config::load_all_relevant_config_files()
            .map_err(|e| e.to_string())
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
fn seed_managed_with_desired_installed(
    config: &crate::core::config::Config,
    state: &mut crate::core::state::PackageState,
) -> Result<bool, String> {
    let mut changed = false;
    for pkg in config.packages.keys() {
        if !state.is_managed(pkg) {
            match crate::core::package::is_package_installed(pkg) {
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

fn handle_removals(to_remove: &[String], dry_run: bool, state: &mut crate::core::state::PackageState) {
    if to_remove.is_empty() {
        return;
    }

    if dry_run {
        println!("Package cleanup (would remove conflicting packages):");
        for package in to_remove {
            println!(
                "  {} Would remove: {}",
                crate::internal::color::red("remove"),
                crate::internal::color::yellow(package)
            );
        }
        println!(
            "  {} Would remove {} package(s)",
            crate::internal::color::blue("info:"),
            to_remove.len()
        );
        return;
    }

    // Ask for explicit confirmation before removing packages
    if !crate::cli::ui::confirm_remove_operation(to_remove) {
        println!(
            "  {}",
            crate::internal::color::blue("Package removal cancelled")
        );
        return;
    }

    if let Err(e) = crate::core::package::remove_unmanaged_packages(to_remove, true) {
        eprintln!(
            "{}",
            crate::internal::color::red(&format!("Failed to remove packages: {}", e))
        );
        return;
    }

    // Remove successfully removed packages from managed list
    for package in to_remove {
        state.remove_managed(package);
    }

    if let Err(e) = state.save() {
        eprintln!("{}", crate::internal::color::red(&format!("Failed to update package state: {}", e)));
    }
}

/// Combined package operations: install uninstalled packages and update all packages
fn run_combined_package_operations(
    to_install: &[String],
    _package_count: usize,
    had_uninstalled: bool,
    _dotfile_count: usize,
    _env_var_count: usize,
    dry_run: bool,
    non_interactive: bool,
    config: &crate::core::config::Config,
) {
    // First, handle uninstalled packages
    let (repo_to_install, aur_to_install) = categorize_install_sets(to_install);

    // Get AUR packages that need updates
    let aur_to_update = compute_aur_updates(dry_run);

    // Combine all AUR operations for confirmation
    let mut all_aur_packages = aur_to_install.clone();
    all_aur_packages.extend(aur_to_update.clone());
    all_aur_packages.sort();
    all_aur_packages.dedup();

    // Install repo packages first (no confirmation needed)
    install_repo_packages(&repo_to_install, dry_run);

    // Handle all AUR packages together
    if !all_aur_packages.is_empty() {
        // Show detailed breakdown of what will happen
        if !aur_to_install.is_empty() {
            println!(
                "  {} AUR packages to install: {}",
                crate::internal::color::yellow(&aur_to_install.len().to_string()),
                aur_to_install.join(", ")
            );
        }
        if !aur_to_update.is_empty() {
            println!(
                "  {} AUR packages to update: {}",
                crate::internal::color::yellow(&aur_to_update.len().to_string()),
                aur_to_update.join(", ")
            );
        }

        handle_aur_operations(&all_aur_packages, &aur_to_install, &aur_to_update, dry_run, non_interactive);
    }

    // Add blank line if we installed packages before this
    if had_uninstalled {
        println!();
    }

    // Update repo packages
    update_repo_packages(dry_run);

    // Apply dotfile synchronization
    apply_dotfiles_with_config(config, dry_run);

    // Handle system section (services + environment)
    handle_system_section_with_config(config, dry_run);
}

fn categorize_install_sets(to_install: &[String]) -> (Vec<String>, Vec<String>) {
    if to_install.is_empty() {
        return (Vec::new(), Vec::new());
    }
    match crate::core::package::categorize_packages(to_install) {
        Ok(result) => result,
        Err(e) => {
            eprintln!(
                "{}",
                crate::internal::color::red(&format!("Failed to categorize packages: {}", e))
            );
            (Vec::new(), Vec::new())
        }
    }
}

fn compute_aur_updates(dry_run: bool) -> Vec<String> {
    if dry_run {
        return Vec::new();
    }
    match get_aur_updates() {
        Ok(packages) => packages,
        Err(e) => {
            eprintln!(
                "{}",
            crate::internal::color::red(&format!("Failed to check AUR updates: {}", e))
            );
            Vec::new()
        }
    }
}

fn install_repo_packages(repo_to_install: &[String], dry_run: bool) {
    if repo_to_install.is_empty() {
        return;
    }
    println!(
        "  {} repo packages found: {}",
        crate::internal::color::yellow(&repo_to_install.len().to_string()),
        repo_to_install.join(", ")
    );
    if dry_run {
        println!(
            "  {} Would install {} from official repositories",
            crate::internal::color::blue("info:"),
            repo_to_install.join(", ")
        );
    } else {
        if let Err(e) = crate::core::pm::ParuPacman::new().install_repo(repo_to_install) {
            eprintln!("{}", crate::internal::color::red(&e));
        }
    }
}

fn handle_aur_operations(
    all_aur_packages: &[String],
    aur_to_install: &[String],
    aur_to_update: &[String],
    dry_run: bool,
    non_interactive: bool,
) {
    if dry_run || non_interactive || crate::cli::ui::confirm_aur_operation(all_aur_packages, "installing/updating") {
        if dry_run {
            println!(
                "  {} Would install/update {} from AUR",
                crate::internal::color::blue("info:"),
                all_aur_packages.join(", ")
            );
            return;
        }
        if !aur_to_install.is_empty() {
            if let Err(e) = crate::core::pm::ParuPacman::new().install_aur(aur_to_install) {
                eprintln!("{}", crate::internal::color::red(&e));
            }
        }
        if !aur_to_update.is_empty() {
            if let Err(e) = crate::core::pm::ParuPacman::new().update_aur(aur_to_update) {
                eprintln!("{}", crate::internal::color::red(&e));
            }
        }
    } else {
        println!(
            "  {}",
            crate::internal::color::blue("AUR package operations cancelled")
        );
    }
}

fn update_repo_packages(dry_run: bool) {
    if dry_run {
        println!(
            "  {} Would update official repository packages",
            crate::internal::color::blue("info:")
        );
        return;
    }
    if let Err(err) = crate::core::pm::ParuPacman::new().update_repo() {
        eprintln!(
            "{}",
            crate::internal::color::red(&format!("Repo update failed: {}", err))
        );
    }
}

/// Install packages from a specific source
// install_packages replaced by PackageManager::install_repo/install_aur

/// Update AUR packages
// update_aur_packages removed in favor of PackageManager::update_aur

/// Apply dotfile synchronization
fn apply_dotfiles_with_config(config: &crate::core::config::Config, dry_run: bool) {
    // Config is provided from earlier analysis

    // Get dotfile mappings from config
    let mappings = crate::core::dotfiles::get_dotfile_mappings(config);

    // Show section header
    println!();
    println!("[{}]", crate::internal::color::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::internal::color::blue("info:"));
        return;
    }

    // Check if any actions are needed
    let has_actions = match crate::core::dotfiles::has_actionable_dotfiles(&mappings) {
        Ok(has) => has,
        Err(err) => {
            eprintln!(
                "{}",
            crate::internal::color::red(&format!("Failed to analyze dotfiles: {}", err))
            );
            return;
        }
    };

    if !has_actions {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::internal::color::green("➔"),
            mappings.len()
        );
        return;
    }

    // Analyze and apply dotfiles
    let actions = match crate::core::dotfiles::apply_dotfiles(&mappings, dry_run) {
        Ok(actions) => actions,
        Err(err) => {
            eprintln!(
                "{}",
                crate::internal::color::red(&format!("Failed to apply dotfiles: {}", err))
            );
            return;
        }
    };

    crate::core::dotfiles::print_actions(&actions, dry_run);
}

/// Run the apply command to update packages and system
#[allow(clippy::collapsible_if)]
pub fn run(opts: &crate::cli::handler::CliOptions) {
    let dry_run = opts.global.dry_run;
    let non_interactive = opts.global.non_interactive;
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made to the system",
            crate::internal::color::blue("info:")
        );
        println!();
    }

    // Perform analysis with spinner
    let analysis_result = crate::internal::util::run_with_spinner(|| analyze_system(), "Analyzing system configuration");

    let (
        package_count,
        config,
        mut state,
        actions,
        dotfile_count,
        env_var_count,
        service_count,
        config_package_count,
    ) = match analysis_result {
        Ok(result) => result,
        Err(err) => {
            crate::error::exit_with_error(&err);
        }
    };

    // Separate actions into installs and removals
    let to_install: Vec<String> = actions
        .iter()
        .filter_map(|action| match action {
            crate::core::package::PackageAction::Install { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    let to_remove: Vec<String> = actions
        .iter()
        .filter_map(|action| match action {
            crate::core::package::PackageAction::Remove { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    crate::cli::ui::generate_apply_output_with_install(
        package_count,
        to_install.len(),
        dotfile_count,
        service_count,
        to_remove.len(),
        config_package_count,
    );

    let had_uninstalled = !to_install.is_empty();

    // Handle removals first
    handle_removals(&to_remove, dry_run, &mut state);

    // Handle all package operations (install + update) in one combined phase
    run_combined_package_operations(
        &to_install,
        package_count,
        had_uninstalled,
        dotfile_count,
        env_var_count,
        dry_run,
        non_interactive,
        &config,
    );

    // After operations, mark newly installed packages as managed (only if installed by our tool)
    if !dry_run {
        let mut changed = false;
        for pkg in &to_install {
            match crate::core::package::is_package_installed(pkg) {
                Ok(true) => {
                    if !state.is_managed(pkg) {
                        state.add_managed(pkg.clone());
                        changed = true;
                    }
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

        if changed {
            if let Err(e) = state.save() {
                eprintln!(
                    "{}",
                    crate::internal::color::red(&format!("Failed to save package state: {}", e))
                );
            }
        }
    }
}

/// Handle system section (services + environment variables)
fn handle_system_section_with_config(config: &crate::core::config::Config, dry_run: bool) {
    // no-op placeholder kept for potential future use

    // Check if we have services or environment variables
    let services = crate::core::services::get_configured_services(&config);
    let env_var_count = count_environment_variables(&config);

    if services.is_empty() && env_var_count == 0 {
        return;
    }

    // Show section header
    println!();
    println!("[{}]", crate::internal::color::red("system"));

    // Handle services first
    if !services.is_empty() {
        if dry_run {
            println!("  {} Plan:", crate::internal::color::blue("info:"));
            for service in &services {
                println!(
                    "    ✓ Would manage {} (system) [enable, start]",
                    crate::internal::color::yellow(service)
                );
            }
            println!(
                "  {} Planned {} service(s)",
                crate::internal::color::blue("info:"),
                services.len()
            );
            println!();
        } else {
            // Use spinner for service validation
            let spinner_msg = format!("Validating {} services...", services.len());
            let services_clone = services.clone();
            let result = match crate::internal::util::run_with_spinner(
                move || crate::core::services::ensure_services_configured(&services_clone),
                &spinner_msg,
            ) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!(
                        "{}",
                        crate::internal::color::red(&format!("Failed to configure services: {}", err))
                    );
                    return;
                }
            };

            if result.changed {
                println!("  {} Services configured", crate::internal::color::green("⸎"));
                println!();
                println!(
                    "  {} Managed {} service(s)",
                    crate::internal::color::green("⸎"),
                    services.len()
                );

                if !result.enabled_services.is_empty() {
                    println!("    Enabled: {}", result.enabled_services.join(", "));
                }
                if !result.started_services.is_empty() {
                    println!("    Started: {}", result.started_services.join(", "));
                }
                if !result.failed_services.is_empty() {
                    println!(
                        "    {} Failed: {}",
                        crate::internal::color::red("✗"),
                        result.failed_services.join(", ")
                    );
                }
                println!();
            } else {
                println!("  {} Service state verified", crate::internal::color::green("⸎"));
            }
        }
    }

    // Handle environment variables
    if env_var_count > 0 {
        match crate::core::env::handle_environment_combined(&config, dry_run) {
            Ok(()) => {}
            Err(e) => {
                eprintln!(
                    "{}",
                    crate::internal::color::red(&format!("Environment handling failed: {}", e))
                );
            }
        }
    }
}
use crate::core::pm::PackageManager;
