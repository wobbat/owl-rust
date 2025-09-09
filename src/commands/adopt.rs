use crate::internal::color as colo;

pub fn run(items: &[String], all: bool) {
    // Determine target packages to adopt
    let targets: Vec<String> = if all {
        match crate::core::config::Config::load_all_relevant_config_files() {
            Ok(cfg) => cfg.packages.keys().cloned().collect(),
            Err(e) => {
                eprintln!("{}", colo::red(&format!("Failed to load config: {}", e)));
                return;
            }
        }
    } else {
        items.to_vec()
    };

    if targets.is_empty() {
        println!("{}", colo::yellow("No packages to adopt"));
        return;
    }

    let mut state = match crate::core::state::PackageState::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", colo::red(&format!("Failed to load state: {}", e)));
            return;
        }
    };

    let mut adopted = Vec::new();
    let mut skipped_not_installed = Vec::new();
    let mut skipped_already = Vec::new();

    for pkg in targets {
        if state.is_managed(&pkg) {
            skipped_already.push(pkg);
            continue;
        }
        match crate::core::package::is_package_installed(&pkg) {
            Ok(true) => {
                state.add_managed(pkg.clone());
                adopted.push(pkg);
            }
            Ok(false) => skipped_not_installed.push(pkg),
            Err(e) => {
                eprintln!("{}", colo::red(&format!("Failed to check {}: {}", pkg, e)));
            }
        }
    }

    if let Err(e) = state.save() {
        eprintln!("{}", colo::red(&format!("Failed to save state: {}", e)));
        return;
    }

    if !adopted.is_empty() {
        println!(
            "{} Adopted {} package(s): {}",
            colo::green("✓"),
            adopted.len(),
            adopted.join(", ")
        );
    }
    if !skipped_already.is_empty() {
        println!(
            "{} Already managed: {}",
            colo::blue("info:"),
            skipped_already.join(", ")
        );
    }
    if !skipped_not_installed.is_empty() {
        println!(
            "{} Not installed (skipped): {}",
            colo::yellow("!"),
            skipped_not_installed.join(", ")
        );
    }
}
