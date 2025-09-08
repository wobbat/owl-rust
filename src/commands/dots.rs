/// Run the dots command to apply dotfile synchronization
pub fn run(opts: &crate::cli::handler::CliOptions) {
    let dry_run = opts.global.dry_run;
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made to the system",
            crate::internal::color::blue("ℹ")
        );
        println!();
    }

    // Load configuration
    let config = match crate::core::config::Config::load_all_relevant_config_files_with_pest(opts.global.use_pest) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{}",
            crate::internal::color::red(&format!("Failed to load config: {}", err))
            );
            std::process::exit(1);
        }
    };

    // Get dotfile mappings from config
    let mappings = crate::core::dotfiles::get_dotfile_mappings(&config);

    // Show section header
    println!();
    println!("[{}]", crate::internal::color::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::internal::color::blue("ℹ"));
        return;
    }

    // Check if any actions are needed
    let has_actions = match crate::core::dotfiles::has_actionable_dotfiles(&mappings) {
        Ok(has) => has,
        Err(err) => {
            eprintln!(
                "{}",
            crate::internal::color::red(&format!("Failed to analyze dotfiles: {}", err))
            );
            std::process::exit(1);
        }
    };

    if !has_actions {
        println!(
            "  {} Up to date: {} dotfiles",
            crate::internal::color::green("➔"),
            mappings.len()
        );
        return;
    }

    // Analyze and apply dotfiles
    let actions = match crate::core::dotfiles::apply_dotfiles(&mappings, dry_run) {
        Ok(actions) => actions,
        Err(err) => {
            eprintln!(
                "{}",
            crate::internal::color::red(&format!("Failed to apply dotfiles: {}", err))
            );
            std::process::exit(1);
        }
    };

    crate::core::dotfiles::print_actions(&actions, dry_run);
}
