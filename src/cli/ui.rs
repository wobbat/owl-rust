use crate::infrastructure::color as colo;
use std::io::Write;

/// Generate the apply command output display with uninstalled package count
pub fn generate_apply_output_with_install(
    package_count: usize,
    uninstalled_count: usize,
    _dotfile_count: usize,
    service_count: usize,
) {
    let host_name = crate::infrastructure::constants::get_host_name().unwrap_or_else(|_| "unknown".to_string());
    println!("[{}]", colo::blue("info"));
    println!("  host: {}", colo::bold(&host_name));
    println!(
        "  packages: {} ({}, {}, {})",
        colo::bold(&(package_count + uninstalled_count).to_string()),
        colo::green(&format!("install {}", uninstalled_count)),
        colo::yellow(&format!("upgrade {}", package_count)),
        colo::red("remove 0")
    );
    println!(
        "  managed pkgs: {}",
        colo::bold(&(package_count + uninstalled_count).to_string())
    );
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
            crate::infrastructure::color::green("➔"),
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
    println!(
        "\n  {} AUR packages require confirmation",
        colo::red("‼")
    );
    println!(
        "  {} AUR packages found: {}",
        colo::yellow(&packages.len().to_string()),
        packages.join(", ")
    );
    let verb = match operation {
        "installing" => "install",
        "updating" => "update",
        "installing/updating" => "install and/or update",
        _ => operation.trim_end_matches("ing"),
    };
    print!("  -> Are you sure you wanna {} AUR packages? (y/N): ", verb);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => matches!(input.trim().to_lowercase().as_str(), "y" | "yes"),
        Err(_) => false,
    }
}
