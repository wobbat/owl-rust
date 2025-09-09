/// Handle system section (services + environment variables)
pub fn handle_system_section_with_config(config: &crate::core::config::Config, dry_run: bool) {
    // no-op placeholder kept for potential future use

    // Check if we have services or environment variables
    let services = crate::core::services::get_configured_services(&config);
    let env_var_count = super::analysis::count_environment_variables(&config);

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