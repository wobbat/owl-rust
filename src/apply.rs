/// Get list of AUR packages that can be updated
fn get_aur_updates() -> Result<Vec<String>, String> {
    use std::process::Command;

    let output = Command::new(crate::constants::PACKAGE_MANAGER)
        .arg("-Qua")
        .output()
        .map_err(|e| format!("Failed to check AUR updates: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                // paru -Qua output format: "package-name old-version -> new-version"
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .collect();
        Ok(packages)
    } else {
        // paru -Qua exits with code 1 when no AUR updates available
        if output.status.code() == Some(1) {
            Ok(vec![])
        } else {
            Err(format!(
                "AUR update check failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

/// Count packages that have dotfile configurations
fn count_dotfile_packages(config: &crate::config::Config) -> usize {
    config
        .packages
        .values()
        .filter(|pkg| pkg.config.is_some())
        .count()
}

/// Count total environment variables (package + global)
fn count_environment_variables(config: &crate::config::Config) -> usize {
    let package_env_vars = config
        .packages
        .values()
        .map(|pkg| pkg.env_vars.len())
        .sum::<usize>();
    package_env_vars + config.env_vars.len()
}

/// Combined package operations: install uninstalled packages and update all packages
fn run_combined_package_operations(
    to_install: &[String],
    _package_count: usize,
    had_uninstalled: bool,
    _dotfile_count: usize,
    _env_var_count: usize,
    dry_run: bool,
) {
    // First, handle uninstalled packages
    let (repo_to_install, aur_to_install) = if !to_install.is_empty() {
        match crate::package::categorize_packages(to_install) {
            Ok(result) => result,
            Err(e) => {
                eprintln!(
                    "{}",
                    crate::colo::red(&format!("Failed to categorize packages: {}", e))
                );
                (Vec::new(), Vec::new())
            }
        }
    } else {
        (Vec::new(), Vec::new())
    };

    // Get AUR packages that need updates
    let aur_to_update = if !dry_run {
        match get_aur_updates() {
            Ok(packages) => packages,
            Err(e) => {
                eprintln!(
                    "{}",
                    crate::colo::red(&format!("Failed to check AUR updates: {}", e))
                );
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Combine all AUR operations for confirmation
    let mut all_aur_packages = aur_to_install.clone();
    all_aur_packages.extend(aur_to_update.clone());
    all_aur_packages.sort();
    all_aur_packages.dedup();

    // Install repo packages first (no confirmation needed)
    if !repo_to_install.is_empty() {
        println!(
            "  {} repo packages found: {}",
            crate::colo::yellow(&repo_to_install.len().to_string()),
            repo_to_install.join(", ")
        );
        if dry_run {
            println!(
                "  {} Would install {} from official repositories",
                crate::colo::blue("ℹ"),
                repo_to_install.join(", ")
            );
        } else {
            install_packages(&repo_to_install, "official repositories");
        }
    }

    // Handle all AUR packages together
    if !all_aur_packages.is_empty() {
        // Show detailed breakdown of what will happen
        if !aur_to_install.is_empty() {
            println!(
                "  {} AUR packages to install: {}",
                crate::colo::yellow(&aur_to_install.len().to_string()),
                aur_to_install.join(", ")
            );
        }
        if !aur_to_update.is_empty() {
            println!(
                "  {} AUR packages to update: {}",
                crate::colo::yellow(&aur_to_update.len().to_string()),
                aur_to_update.join(", ")
            );
        }

        if dry_run || crate::ui::confirm_aur_operation(&all_aur_packages, "installing/updating") {
            if dry_run {
                println!(
                    "  {} Would install/update {} from AUR",
                    crate::colo::blue("ℹ"),
                    all_aur_packages.join(", ")
                );
            } else {
                // Install new AUR packages first
                if !aur_to_install.is_empty() {
                    install_packages(&aur_to_install, "AUR");
                }
                // Then update existing AUR packages
                if !aur_to_update.is_empty() {
                    update_aur_packages(&aur_to_update);
                }
            }
        } else {
            println!(
                "  {}",
                crate::colo::blue("AUR package operations cancelled")
            );
        }
    }

    // Add blank line if we installed packages before this
    if had_uninstalled {
        println!();
    }

    // Update repo packages
    if dry_run {
        println!(
            "  {} Would update official repository packages",
            crate::colo::blue("ℹ")
        );
    } else {
        let repo_status = match crate::util::run_command_with_spinner(
            crate::constants::PACKAGE_MANAGER,
            &["--repo", "-Syu", "--noconfirm"],
            "Updating official repository packages (syncing databases and upgrading packages)",
        ) {
            Ok(status) => status,
            Err(err) => {
                eprintln!(
                    "{}",
                    crate::colo::red(&format!("Repo update failed: {}", err))
                );
                apply_dotfiles(dry_run);
                return;
            }
        };

        if repo_status.success() {
            println!("  {} Official repos synced", crate::colo::green("⸎"));
        } else if repo_status.code() == Some(1) {
            // pacman returns 1 when no updates are available, which is not an error
            println!(
                "  {} Packages from main repos have been updated",
                crate::colo::green("⸎")
            );
        } else {
            eprintln!(
                "  {} Repository update failed (exit code: {:?})",
                crate::colo::red("✗"),
                repo_status.code()
            );
        }
    }

    // Apply dotfile synchronization
    apply_dotfiles(dry_run);

    // Handle system section (services + environment)
    handle_system_section(dry_run);
}

/// Install packages from a specific source
fn install_packages(packages: &[String], source: &str) {
    let mut args = vec!["-S", "--noconfirm"];
    args.extend(packages.iter().map(|s| s.as_str()));

    // Run package installation with spinner
    let status = match crate::util::run_command_with_spinner(
        crate::constants::PACKAGE_MANAGER,
        &args,
        &format!("Installing from {}", source),
    ) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("{}", crate::colo::red(&err));
            return; // Don't exit, just return to continue with the rest of the apply command
        }
    };

    if status.success() {
        println!(
            "\r\x1b[2K  {} Package installation from {} completed",
            crate::colo::green("⸎"),
            source
        );
    } else {
        eprintln!("{}", crate::colo::red("package installation failed"));
        // Don't exit here so we can continue with the rest of the apply command
    }
}

/// Update AUR packages
fn update_aur_packages(packages: &[String]) {
    let mut args = vec!["--aur", "-Syu", "--noconfirm"];
    args.extend(packages.iter().map(|s| s.as_str()));

    // Run AUR update with spinner
    let status = match crate::util::run_command_with_spinner(
        crate::constants::PACKAGE_MANAGER,
        &args,
        "Updating AUR packages",
    ) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("{}", crate::colo::red(&err));
            return; // Don't exit, just return to continue with the rest of the apply command
        }
    };

    if status.success() {
        println!(
            "\r\x1b[2K  {} AUR package updates completed",
            crate::colo::green("⸎")
        );
    } else {
        eprintln!("{}", crate::colo::red("AUR package update failed"));
        // Don't exit here so we can continue with the rest of the apply command
    }
}

/// Apply dotfile synchronization
fn apply_dotfiles(dry_run: bool) {
    // Load configuration
    let config = match crate::config::Config::load_all_relevant_config_files() {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Failed to load config: {}", err))
            );
            return;
        }
    };

    // Get dotfile mappings from config
    let mappings = crate::dotfiles::get_dotfile_mappings(&config);

    // Show section header
    println!();
    println!("[{}]", crate::colo::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::colo::blue("ℹ"));
        // Show system section
        let env_var_count = count_environment_variables(&config);
        crate::ui::show_remaining_sections(mappings.len(), env_var_count);
        return;
    }

    // Check if any actions are needed
    let has_actions = match crate::dotfiles::has_actionable_dotfiles(&mappings) {
        Ok(has) => has,
        Err(err) => {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Failed to analyze dotfiles: {}", err))
            );
            return;
        }
    };

    if !has_actions {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::colo::green("➔"),
            mappings.len()
        );
        // Show system section
        let env_var_count = count_environment_variables(&config);
        crate::ui::show_remaining_sections(mappings.len(), env_var_count);
        return;
    }

    // Analyze and apply dotfiles
    let actions = match crate::dotfiles::apply_dotfiles(&mappings, dry_run) {
        Ok(actions) => actions,
        Err(err) => {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Failed to apply dotfiles: {}", err))
            );
            return;
        }
    };

    // Count up-to-date dotfiles
    let up_to_date_count = actions
        .iter()
        .filter(|action| matches!(action.status, crate::dotfiles::DotfileStatus::UpToDate))
        .count();

    // Show summary
    if up_to_date_count > 0 {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::colo::green("➔"),
            up_to_date_count
        );
    }

    // Show individual actions only for changes
    for action in actions {
        match action.status {
            crate::dotfiles::DotfileStatus::Create => {
                if dry_run {
                    println!(
                        "  {} Would create: {} -> {}",
                        crate::colo::blue("ℹ"),
                        action.source,
                        action.destination
                    );
                } else {
                    println!(
                        "  {} Created: {} -> {}",
                        crate::colo::green("➔"),
                        action.source,
                        action.destination
                    );
                }
            }
            crate::dotfiles::DotfileStatus::Update => {
                if dry_run {
                    println!(
                        "  {} Would update: {} -> {}",
                        crate::colo::blue("ℹ"),
                        action.source,
                        action.destination
                    );
                } else {
                    println!(
                        "  {} Updated: {} -> {}",
                        crate::colo::green("➔"),
                        action.source,
                        action.destination
                    );
                }
            }
            crate::dotfiles::DotfileStatus::Conflict => {
                let reason = action
                    .reason
                    .unwrap_or_else(|| "Unknown conflict".to_string());
                println!(
                    "  {} Conflict: {} ({})",
                    crate::colo::red("✗"),
                    action.destination,
                    reason
                );
            }
            crate::dotfiles::DotfileStatus::UpToDate => {
                // Don't show individual up-to-date messages, we show the count above
            }
            crate::dotfiles::DotfileStatus::Skip => {
                // Skip showing skip actions in normal output
            }
        }
    }

    if dry_run {
        println!(
            "  {} Dotfile analysis completed (dry-run mode)",
            crate::colo::blue("ℹ")
        );
    }
}

/// Run the apply command to update packages and system
pub fn run(dry_run: bool) {
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made to the system",
            crate::colo::blue("ℹ")
        );
        println!();
    }

    // Perform analysis with spinner
    let analysis_result = crate::util::run_with_spinner(
        || {
            // Get package count
            let package_count = crate::package::get_package_count()
                .map_err(|e| format!("Failed to get package count: {}", e))?;

            // Load configuration
            let config = crate::config::Config::load_all_relevant_config_files()
                .map_err(|e| format!("Failed to load config: {}", e))?;

            // Load package state
            let state = crate::state::PackageState::load()
                .map_err(|e| format!("Failed to load package state: {}", e))?;

            // Plan package actions (installs and removals)
            let actions = crate::package::plan_package_actions(&config, &state)
                .map_err(|e| format!("Failed to plan package actions: {}", e))?;

            // Calculate dynamic values
            let dotfile_count = count_dotfile_packages(&config);
            let env_var_count = count_environment_variables(&config);
            let service_count = crate::services::get_configured_services(&config).len();
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
        },
        "Analyzing system configuration",
    );

    let (
        package_count,
        _config,
        mut _state,
        actions,
        dotfile_count,
        env_var_count,
        service_count,
        _config_package_count,
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
            crate::package::PackageAction::Install { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    let to_remove: Vec<String> = actions
        .iter()
        .filter_map(|action| match action {
            crate::package::PackageAction::Remove { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    // Save state to disk (skip in dry run)
    if !dry_run {
        if let Err(e) = _state.save() {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Failed to save package state: {}", e))
            );
        }
    }

    crate::ui::generate_apply_output_with_install(
        package_count,
        to_install.len(),
        dotfile_count,
        service_count,
    );

    let had_uninstalled = !to_install.is_empty();

    // Handle removals first
    if !to_remove.is_empty() {
        if dry_run {
            println!("Package cleanup (would remove conflicting packages):");
            for package in &to_remove {
                println!(
                    "  {} Would remove: {}",
                    crate::colo::red("remove"),
                    crate::colo::yellow(package)
                );
            }
            println!(
                "  {} Would remove {} package(s)",
                crate::colo::blue("ℹ"),
                to_remove.len()
            );
        } else {
            if let Err(e) = crate::package::remove_unmanaged_packages(&to_remove, true) {
                eprintln!(
                    "{}",
                    crate::colo::red(&format!("Failed to remove packages: {}", e))
                );
            } else {
                // Remove successfully removed packages from managed list
                for package in &to_remove {
                    _state.remove_managed(package);
                }

                if let Err(e) = _state.save() {
                    eprintln!(
                        "{}",
                        crate::colo::red(&format!("Failed to update package state: {}", e))
                    );
                }
            }
        }
    }

    // Handle all package operations (install + update) in one combined phase
    run_combined_package_operations(
        &to_install,
        package_count,
        had_uninstalled,
        dotfile_count,
        env_var_count,
        dry_run,
    );
}

/// Handle system section (services + environment variables)
fn handle_system_section(dry_run: bool) {
    // Load configuration
    let config = match crate::config::Config::load_all_relevant_config_files() {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{}",
                crate::colo::red(&format!(
                    "Failed to load config for system section: {}",
                    err
                ))
            );
            return;
        }
    };

    // Check if we have services or environment variables
    let services = crate::services::get_configured_services(&config);
    let env_var_count = count_environment_variables(&config);

    if services.is_empty() && env_var_count == 0 {
        return;
    }

    // Show section header
    println!("");
    println!("[{}]", crate::colo::red("system"));

    // Handle services first
    if !services.is_empty() {
        if dry_run {
            println!("  {} Plan:", crate::colo::blue("ℹ"));
            for service in &services {
                println!(
                    "    ✓ Would manage {} (system) [enable, start]",
                    crate::colo::yellow(service)
                );
            }
            println!(
                "  {} Planned {} service(s)",
                crate::colo::blue("ℹ"),
                services.len()
            );
            println!();
        } else {
            // Use spinner for service validation
            let spinner_msg = format!("Validating {} services...", services.len());
            let services_clone = services.clone();
            let result = match crate::util::run_with_spinner(
                move || crate::services::ensure_services_configured(&services_clone),
                &spinner_msg,
            ) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!(
                        "{}",
                        crate::colo::red(&format!("Failed to configure services: {}", err))
                    );
                    return;
                }
            };

            if result.changed {
                println!("  {} Services configured", crate::colo::green("⸎"));
                println!();
                println!(
                    "  {} Managed {} service(s)",
                    crate::colo::green("⸎"),
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
                        crate::colo::red("✗"),
                        result.failed_services.join(", ")
                    );
                }
                println!();
            } else {
                println!("  {} Service state verified", crate::colo::green("⸎"));
            }
        }
    }

    // Handle environment variables
    if env_var_count > 0 {
        if let Err(e) = crate::env::handle_environment_combined(&config, dry_run) {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Environment handling failed: {}", e))
            );
        }
    }
}
