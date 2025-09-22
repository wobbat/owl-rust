pub mod analysis;
pub mod dotfiles;
pub mod packages;
pub mod system;

use anyhow::Result;

/// Helper function to handle operation errors with custom message
fn handle_operation_error(operation: &str, result: Result<()>) {
    if let Err(e) = result {
        eprintln!(
            "{}",
            crate::internal::color::red(&format!("Failed to {}: {}", operation, e))
        );
    }
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
    let analysis_result = crate::internal::util::execute_with_progress(
        analysis::analyze_system,
        "Analyzing system configuration",
    );

    let mut analysis = match analysis_result {
        Ok(result) => result,
        Err(err) => {
            crate::error::exit_with_error(anyhow::anyhow!(err));
        }
    };

    // Separate actions into installs and removals
    let to_install: Vec<String> = analysis.actions
        .iter()
        .filter_map(|action| match action {
            crate::core::package::PackageAction::Install { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    let to_remove: Vec<String> = analysis.actions
        .iter()
        .filter_map(|action| match action {
            crate::core::package::PackageAction::Remove { name } => Some(name.clone()),
            _ => None,
        })
        .collect();

    crate::cli::ui::generate_apply_output_with_install(
        analysis.package_count,
        to_install.len(),
        analysis.dotfile_count,
        analysis.service_count,
        to_remove.len(),
        analysis.config_package_count,
    );

    let had_uninstalled = !to_install.is_empty();

    // Handle removals first
    packages::handle_removals(&to_remove, dry_run, &mut analysis.state);

    // Handle all package operations (install + update) in one combined phase
    let package_params = packages::PackageOperationParams {
        dry_run,
        non_interactive,
        had_uninstalled,
    };
    packages::install_and_update_packages(
        &to_install,
        &package_params,
        &analysis.config,
    );

    // After operations, mark newly installed packages as managed (only if installed by our tool)
    if !dry_run {
        let mut changed = false;
        for pkg in &to_install {
            match crate::core::package::is_package_or_group_installed(pkg) {
                Ok(true) => {
                    if !analysis.state.is_managed(pkg) {
                        analysis.state.add_managed(pkg.clone());
                        changed = true;
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    handle_operation_error(&format!("verify installation of {}", pkg), Err(e));
                }
            }
        }

        if changed {
            handle_operation_error("save package state", analysis.state.save());
        }
    }
}
