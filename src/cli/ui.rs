use crate::internal::color as colo;
use std::io::Write;

fn confirm_operation(
    packages: &[String],
    header_icon: &str,
    header_text: &str,
    detail_label: &str,
    prompt: &str,
) -> bool {
    println!("\n  {} {}", colo::red(header_icon), header_text);
    println!(
        "  {} {}: {}",
        colo::yellow(&packages.len().to_string()),
        detail_label,
        packages.join(", ")
    );
    print!("  -> {} ", prompt);
    std::io::stdout().flush().ok();

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => matches!(input.trim().to_lowercase().as_str(), "y" | "yes"),
        Err(_) => false,
    }
}

/// Generate the apply command output display with uninstalled package count
pub fn generate_apply_output_with_install(
    package_count: usize,
    uninstalled_count: usize,
    _dotfile_count: usize,
    service_count: usize,
    remove_count: usize,
    managed_count: usize,
) {
    let host_name =
        crate::internal::constants::get_host_name().unwrap_or_else(|_| "unknown".to_string());
    println!("[{}]", colo::blue("info"));
    println!("  host: {}", colo::bold(&host_name));
    println!(
        "  packages: {} ({}, {}, {})",
        colo::bold(&(package_count + uninstalled_count).to_string()),
        colo::green(&format!("install {}", uninstalled_count)),
        colo::yellow(&format!("upgrade {}", package_count)),
        colo::red(&format!("remove {}", remove_count))
    );
    println!("  managed pkgs: {}", colo::bold(&managed_count.to_string()));
    if service_count > 0 {
        println!("  services: {}", colo::bold(&service_count.to_string()));
    }
    println!();
    println!("[{}]", colo::yellow("packages"));
    if package_count > 0 {
        println!(
            "  {} packages can be upgraded",
            colo::yellow(&package_count.to_string())
        );
    } else {
        println!(
            "  {} {}",
            crate::internal::color::green("➔"),
            colo::dim("no packages to upgrade")
        );
    }
    if uninstalled_count > 0 {
        println!(
            "  {} packages can be installed",
            colo::green(&uninstalled_count.to_string())
        );
    }
}

/// Prompt user for AUR package confirmation
pub fn confirm_aur_operation(packages: &[String], operation: &str) -> bool {
    let verb = match operation {
        "installing" => "install",
        "updating" => "update",
        "installing/updating" => "install and/or update",
        _ => operation.trim_end_matches("ing"),
    };
    confirm_operation(
        packages,
        "‼",
        "AUR packages require confirmation",
        "AUR packages found",
        &format!("Are you sure you wanna {} AUR packages? (y/N):", verb),
    )
}

/// Prompt user for removal confirmation
pub fn confirm_remove_operation(packages: &[String]) -> bool {
    confirm_operation(
        packages,
        "‼",
        "Package removals require confirmation",
        "packages to remove",
        "Are you sure you want to remove these packages? (y/N):",
    )
}
