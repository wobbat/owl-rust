use crate::core::pm::PackageManager;

pub fn handle_removals(to_remove: &[String], dry_run: bool, state: &mut crate::core::state::PackageState) {
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
pub fn run_combined_package_operations(
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
    super::dotfiles::apply_dotfiles_with_config(config, dry_run);

    // Handle system section (services + environment)
    super::system::handle_system_section_with_config(config, dry_run);
}

pub fn categorize_install_sets(to_install: &[String]) -> (Vec<String>, Vec<String>) {
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

pub fn compute_aur_updates(dry_run: bool) -> Vec<String> {
    if dry_run {
        return Vec::new();
    }
    match super::analysis::get_aur_updates() {
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

pub fn install_repo_packages(repo_to_install: &[String], dry_run: bool) {
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

pub fn handle_aur_operations(
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

pub fn update_repo_packages(dry_run: bool) {
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