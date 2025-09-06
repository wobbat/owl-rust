/// Run the dots command to apply dotfile synchronization
pub fn run(dry_run: bool) {
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made to the system",
            crate::infrastructure::color::blue("ℹ")
        );
        println!();
    }

    // Load configuration
    let config = match crate::domain::config::Config::load_all_relevant_config_files() {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{}",
            crate::infrastructure::color::red(&format!("Failed to load config: {}", err))
            );
            std::process::exit(1);
        }
    };

    // Get dotfile mappings from config
    let mappings = crate::domain::dotfiles::get_dotfile_mappings(&config);

    // Show section header
    println!();
    println!("[{}]", crate::infrastructure::color::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::infrastructure::color::blue("ℹ"));
        return;
    }

    // Check if any actions are needed
    let has_actions = match crate::domain::dotfiles::has_actionable_dotfiles(&mappings) {
        Ok(has) => has,
        Err(err) => {
            eprintln!(
                "{}",
            crate::infrastructure::color::red(&format!("Failed to analyze dotfiles: {}", err))
            );
            std::process::exit(1);
        }
    };

    if !has_actions {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::infrastructure::color::green("➔"),
            mappings.len()
        );
        return;
    }

    // Analyze and apply dotfiles
    let actions = match crate::domain::dotfiles::apply_dotfiles(&mappings, dry_run) {
        Ok(actions) => actions,
        Err(err) => {
            eprintln!(
                "{}",
            crate::infrastructure::color::red(&format!("Failed to apply dotfiles: {}", err))
            );
            std::process::exit(1);
        }
    };

    crate::domain::dotfiles::print_actions(&actions, dry_run);
}
