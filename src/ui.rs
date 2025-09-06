use crate::colo;
use std::io::Write;

/// Print usage information for the CLI
pub fn print_usage() {
    eprintln!("{}", colo::bold("Usage: owl [OPTIONS] <COMMAND>"));
    eprintln!("{}", colo::green("Commands:"));
    eprintln!("  apply");
    eprintln!("  edit {} {}", colo::dim("<type>"), colo::dim("<argument>"));
    eprintln!(
        "    de {}    {}",
        colo::dim("<argument>"),
        colo::dim("(alias for edit dots)")
    );
    eprintln!(
        "    ce {}    {}",
        colo::dim("<argument>"),
        colo::dim("(alias for edit config)")
    );
    eprintln!("  add {}", colo::dim("<items...>"));
    eprintln!("{}", colo::blue("Options:"));
    eprintln!(
        "  {}   {}",
        colo::bold("-v, --verbose"),
        colo::dim(":Enable verbose logging")
    );
}

/// Generate the apply command output display
pub fn generate_apply_output(package_count: usize) {
    let host_name = crate::constants::get_host_name().unwrap_or_else(|_| "unknown".to_string());
    println!("[{}]", colo::red("info"));
    println!("  host: {}", colo::bold(&host_name));
    println!(
        "  packages: {} ({}, {}, {})",
        colo::bold(&package_count.to_string()),
        colo::green("install +1"),
        colo::yellow(&format!("upgrade {}", package_count)),
        colo::red("remove 0")
    );
    println!("  dotfiles: {}", colo::bold("0"));
    println!();
    println!("[{}]", colo::yellow("packages"));
    println!("  checking for package upgrades...");
    if package_count > 0 {
        println!(
            "  {} packages can be upgraded",
            colo::yellow(&package_count.to_string())
        );
    } else {
        println!("  {}", colo::blue("nothing to do"));
    }
}

/// Generate the apply command output display with uninstalled package count
pub fn generate_apply_output_with_install(
    package_count: usize,
    uninstalled_count: usize,
    _dotfile_count: usize,
    service_count: usize,
) {
    let host_name = crate::constants::get_host_name().unwrap_or_else(|_| "unknown".to_string());
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
            crate::colo::green("➔"),
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

/// Show remaining sections after package update
pub fn show_remaining_sections(_dotfile_count: usize, _env_var_count: usize) {
    // This function is now deprecated - environment variables are handled in the system section
}

/// Print package update completion message
pub fn print_update_complete() {
    println!("\r\x1b[2K  {} Package update completed", colo::green("⸎"));
}

/// Prompt user for AUR package confirmation
pub fn confirm_aur_operation(packages: &[String], operation: &str) -> bool {
    println!(
        "\n  {}{}",
        colo::red("‼"),
        " AUR packages require confirmation"
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
