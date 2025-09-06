/// Run the dots command to apply dotfile synchronization
pub fn run(dry_run: bool) {
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made to the system",
            crate::colo::blue("ℹ")
        );
        println!();
    }

    // Load configuration
    let config = match crate::config::Config::load_all_relevant_config_files() {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{}",
                crate::colo::red(&format!("Failed to load config: {}", err))
            );
            std::process::exit(1);
        }
    };

    // Get dotfile mappings from config
    let mappings = crate::dotfiles::get_dotfile_mappings(&config);

    // Show section header
    println!();
    println!("[{}]", crate::colo::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::colo::blue("ℹ"));
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
            std::process::exit(1);
        }
    };

    if !has_actions {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::colo::green("➔"),
            mappings.len()
        );
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
            std::process::exit(1);
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