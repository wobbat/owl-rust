/// Apply dotfile synchronization
pub fn apply_dotfiles_with_config(config: &crate::core::config::Config, dry_run: bool) {
    // Config is provided from earlier analysis

    // Get dotfile mappings from config
    let mappings = crate::core::dotfiles::get_dotfile_mappings(config);

    // Show section header
    println!();
    println!("[{}]", crate::internal::color::green("config"));

    if mappings.is_empty() {
        println!("  {} No dotfiles configured", crate::internal::color::blue("info:"));
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
            return;
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
            return;
        }
    };

    crate::core::dotfiles::print_actions(&actions, dry_run);
}