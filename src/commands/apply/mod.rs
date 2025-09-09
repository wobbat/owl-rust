pub mod analysis;
pub mod packages;
pub mod dotfiles;
pub mod system;

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
    let analysis_result = crate::internal::util::run_with_spinner(|| analysis::analyze_system(), "Analyzing system configuration");

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
    packages::handle_removals(&to_remove, dry_run, &mut state);

    // Handle all package operations (install + update) in one combined phase
    packages::run_combined_package_operations(
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
            match crate::core::package::is_package_or_group_installed(pkg) {
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