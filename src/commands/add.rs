/// Add items (packages) to configuration files
///
/// # Arguments
/// * `items` - List of package names to search for and add
/// * `search_mode` - Whether to search for packages first (always true now)
pub fn run(items: &[String], _search_mode: bool) {
    run_search_mode(items);
}



/// Search and select mode - add to config instead of installing
fn run_search_mode(terms: &[String]) {
    match crate::core::package::search_packages(terms) {
        Ok(results) => {
            if results.is_empty() {
                println!("{}", crate::internal::color::yellow("No packages found matching the search terms"));
                return;
            }

            display_search_results(&results);
            let selection = prompt_package_selection(&results);

            match selection {
                Some(package_name) => {
                    if let Err(err) = add_package_to_config(&package_name) {
                        crate::error::exit_with_error(&err);
                    }
                }
                None => {
                    println!("{}", crate::internal::color::yellow("No package selected"));
                }
            }
        }
        Err(e) => {
            crate::error::exit_with_error(&format!("Search failed: {}", e));
        }
    }
}



/// Display search results in a formatted way
// use crate::domain::package; // no direct uses
use crate::core::pm::{PackageSource, SearchResult};
fn display_search_results(results: &[SearchResult]) {
    println!("\n{} {} package(s):\n",
        crate::internal::color::bold("Found"),
        results.len());

    for (i, result) in results.iter().enumerate() {
        let num_str = number_brackets((results.len() - 1 - i) as i32);
        let name = crate::internal::color::highlight(&result.name);
        let version = crate::internal::color::success(&result.ver);

        let tag = match result.source {
            PackageSource::Aur => {
                crate::internal::color::warning(&format!("[{}]", result.repo))
            }
            PackageSource::Repo => {
                crate::internal::color::repository(&format!("[{}]", result.repo))
            }
        };

        let status = if result.installed {
            format!(" {}", crate::internal::color::success("installed"))
        } else {
            String::new()
        };

        let desc = if !result.description.is_empty() {
            format!(" - {}", crate::internal::color::description(&result.description))
        } else {
            String::new()
        };

        println!("{}{} {}{} {}{}",
            num_str, name, version, tag, status, desc);
    }
    println!();
}

/// Prompt user to select a package from search results
fn prompt_package_selection(results: &[SearchResult]) -> Option<String> {
    if results.is_empty() {
        return None;
    }

    loop {
        print!("Select package (0-{}, or 'c' to cancel): ", results.len() - 1);
        std::io::Write::flush(&mut std::io::stdout()).ok()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let input = input.trim();

        if input == "c" || input == "cancel" {
            return None;
        }

        match input.parse::<usize>() {
            Ok(num) if num < results.len() => {
                let index = results.len() - 1 - num;
                return Some(results[index].name.clone());
            }
            _ => {
                println!("{}", crate::internal::color::red("Invalid selection. Please try again."));
            }
        }
    }
}

/// Format a number in brackets like [1], [2], etc.
fn number_brackets(num: i32) -> String {
    format!("[{}]", num)
}

/// Add a package to the appropriate configuration file
fn add_package_to_config(package_name: &str) -> Result<(), String> {
    let mut config_files = get_relevant_config_files()?;

    if config_files.is_empty() {
        // Use main config if no relevant files found
        let main_config = get_main_config_path()?;
        add_package_to_file(package_name, &main_config)?;
        println!("{}", crate::internal::color::success(&format!("Added '{}' to {}", package_name, main_config)));
        return Ok(());
    }

    if config_files.len() == 1 {
        let file_path = &config_files[0];
        add_package_to_file(package_name, file_path)?;
        println!("{}", crate::internal::color::success(&format!("Added '{}' to {}", package_name, file_path)));
        return Ok(());
    }

    // Reverse the order so main appears at the bottom
    config_files.reverse();

    // Multiple files - prompt for selection
    println!("\n{} {} config file(s):\n",
        crate::internal::color::bold("Found"),
        config_files.len());

    for (i, file) in config_files.iter().enumerate() {
        let num_str = number_brackets((config_files.len() - 1 - i) as i32);
        let friendly = file.replace(&std::env::var("HOME").unwrap_or_default(), "~");
        println!("{} {}", num_str, crate::internal::color::highlight(&friendly));
    }
    println!();

    let selection = prompt_file_selection(config_files.len());
    match selection {
        Some(index) => {
            let file_path = &config_files[index];
            add_package_to_file(package_name, file_path)?;
            println!("{}", crate::internal::color::success(&format!("Added '{}' to {}", package_name, file_path)));
            Ok(())
        }
        None => {
            println!("{}", crate::internal::color::yellow("No config file selected"));
            Ok(())
        }
    }
}

/// Get relevant config files for the current system
#[allow(clippy::collapsible_if)]
fn get_relevant_config_files() -> Result<Vec<String>, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let owl_dir = format!("{}/{}", home, crate::internal::constants::OWL_DIR);

    let mut files = Vec::new();

    // Check main config
    let main_config = format!("{}/main{}", owl_dir, crate::internal::constants::OWL_EXT);
    if std::path::Path::new(&main_config).exists() {
        files.push(main_config);
    }

    // Scan all files in hosts directory
    let hosts_dir = format!("{}/{}", owl_dir, crate::internal::constants::HOSTS_DIR);
    if let Ok(entries) = std::fs::read_dir(&hosts_dir) {
        for entry in entries.flatten() {
            if let Some(path) = entry.path().to_str() {
                if path.ends_with(crate::internal::constants::OWL_EXT) {
                    files.push(path.to_string());
                }
            }
        }
    }

    // Scan all files in groups directory
    let groups_dir = format!("{}/{}", owl_dir, crate::internal::constants::GROUPS_DIR);
    if let Ok(entries) = std::fs::read_dir(&groups_dir) {
        for entry in entries.flatten() {
            if let Some(path) = entry.path().to_str() {
                if path.ends_with(crate::internal::constants::OWL_EXT) {
                    files.push(path.to_string());
                }
            }
        }
    }

    Ok(files)
}

/// Get the main config file path
fn get_main_config_path() -> Result<String, String> {
    use std::path::PathBuf;
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let path = PathBuf::from(home)
        .join(crate::internal::constants::OWL_DIR)
        .join(crate::internal::constants::MAIN_CONFIG_FILE);
    Ok(path.to_string_lossy().into_owned())
}

/// Add a package to a config file
fn add_package_to_file(package_name: &str, file_path: &str) -> Result<(), String> {
    use std::fs;

    // Read existing content
    let content = if std::path::Path::new(file_path).exists() {
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?
    } else {
        String::new()
    };

    // Check if package already exists
    if content.lines().any(|line| line.trim() == package_name) {
        return Err(format!("Package '{}' already exists in {}", package_name, file_path));
    }

    // Add package to @packages section or create one
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut added = false;

    // Look for @packages section
    for i in 0..lines.len() {
        if lines[i].trim() == "@packages" || lines[i].trim() == "@pkgs" {
            // Add after the @packages line
            lines.insert(i + 1, package_name.to_string());
            added = true;
            break;
        }
    }

    // If no @packages section, add one at the end
    if !added {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new()); // Add blank line
        }
        lines.push("@packages".to_string());
        lines.push(package_name.to_string());
    }

    // Write back to file
    let new_content = lines.join("\n") + "\n";
    fs::write(file_path, new_content)
        .map_err(|e| format!("Failed to write to config file: {}", e))?;

    Ok(())
}

/// Prompt user to select a config file from search results
fn prompt_file_selection(count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    loop {
        print!("Select config file (0-{}, or 'c' to cancel): ", count - 1);
        std::io::Write::flush(&mut std::io::stdout()).ok()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let input = input.trim();

        if input == "c" || input == "cancel" {
            return None;
        }

        match input.parse::<usize>() {
            Ok(num) if num < count => {
                let index = count - 1 - num;
                return Some(index);
            }
            _ => {
                println!("{}", crate::internal::color::red("Invalid selection. Please try again."));
            }
        }
    }
}
