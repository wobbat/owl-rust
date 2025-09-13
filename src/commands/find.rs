use std::path::Path;

/// Run the find command to find where packages are defined in config files
pub fn run(query: &[String]) {
    if query.is_empty() {
        eprintln!("{}", crate::internal::color::red("Error: find command requires at least one argument"));
        std::process::exit(1);
    }

    // Determine if this is a config syntax query or a package name query
    let is_config_syntax = query.len() > 1 || query[0].starts_with('@') || query[0].starts_with(':');

    let results = if is_config_syntax {
        find_config_syntax_locations(query)
    } else {
        find_package_locations(&query[0])
    };

    match results {
        Ok(locations) => {
            if locations.is_empty() {
                println!(
                    "{}",
                    crate::internal::color::yellow("No matches found for the given query")
                );
            } else {
                display_locations(&locations);
            }
        }
        Err(err) => {
            eprintln!("{}", crate::internal::color::red(&format!("Error: {}", err)));
            std::process::exit(1);
        }
    }
}

/// Find locations where a package name is defined
fn find_package_locations(package_name: &str) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
    let mut locations = Vec::new();
    let config_files = get_all_config_files()?;

    for file_path in config_files {
        let content = std::fs::read_to_string(&file_path)?;
        let file_locations = find_package_in_file(package_name, &content, &file_path)?;
        locations.extend(file_locations);
    }

    Ok(locations)
}

/// Find locations where config syntax is defined
fn find_config_syntax_locations(query: &[String]) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
    let mut locations = Vec::new();
    let config_files = get_all_config_files()?;

    for file_path in config_files {
        let content = std::fs::read_to_string(&file_path)?;
        let file_locations = find_config_syntax_in_file(query, &content, &file_path)?;
        locations.extend(file_locations);
    }

    Ok(locations)
}

/// Find package definitions in a single file
fn find_package_in_file(package_name: &str, content: &str, file_path: &str) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
    let mut locations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Check for @package or @pkg declarations
        if trimmed == format!("@package {}", package_name) || trimmed == format!("@pkg {}", package_name) {
            locations.push(Location {
                file_path: file_path.to_string(),
                line_number: line_num + 1,
                line_content: line.to_string(),
                context: LocationContext::PackageDeclaration,
            });
        }
        // Check for packages in @packages or @pkgs sections
        else if trimmed == package_name {
            // Check if we're in a packages section by looking at previous lines
            if is_in_packages_section(content, line_num) {
                locations.push(Location {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    line_content: line.to_string(),
                    context: LocationContext::PackagesSection,
                });
            }
        }
    }

    Ok(locations)
}

/// Find config syntax definitions in a single file
fn find_config_syntax_in_file(query: &[String], content: &str, file_path: &str) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
    let mut locations = Vec::new();
    let search_term = query.join(" ");

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Direct match
        if trimmed == search_term {
            locations.push(Location {
                file_path: file_path.to_string(),
                line_number: line_num + 1,
                line_content: line.to_string(),
                context: LocationContext::DirectMatch,
            });
        }
        // Handle different types of config syntax searches
        else if query.len() == 1 {
            let search_pattern = &query[0];

            // Search for @env declarations
            if search_pattern == "@env" {
                if trimmed.starts_with("@env ") {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::EnvDeclaration,
                    });
                }
            }
            // Search for :config directives
            else if search_pattern == ":config" {
                if trimmed.starts_with(":config ") {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::ConfigDirective,
                    });
                }
            }
            // Search for :service directives
            else if search_pattern == ":service" {
                if trimmed.starts_with(":service ") {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::ServiceDirective,
                    });
                }
            }
            // Search for @group declarations
            else if search_pattern == "@group" {
                if trimmed.starts_with("@group ") {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::GroupDeclaration,
                    });
                }
            }
            // Search for @package declarations (single argument)
            else if search_pattern.starts_with("@package ") {
                let package_name = search_pattern.strip_prefix("@package ").unwrap();
                if trimmed == format!("@package {}", package_name) || trimmed == format!("@pkg {}", package_name) {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::PackageDeclaration,
                    });
                }
            }
            // Search for @pkg declarations (single argument)
            else if search_pattern.starts_with("@pkg ") {
                let package_name = search_pattern.strip_prefix("@pkg ").unwrap();
                if trimmed == format!("@package {}", package_name) || trimmed == format!("@pkg {}", package_name) {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::PackageDeclaration,
                    });
                }
            }
        }
        // Handle multi-argument searches (existing logic)
        else if query.len() == 2 {
            let directive = &query[0];
            let value = &query[1];

            // Equivalent syntax matches for packages
            if directive == "@package" {
                if trimmed == format!("@pkg {}", value) {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::AlternativeSyntax,
                    });
                }
            }
            else if directive == "@pkg" {
                if trimmed == format!("@package {}", value) {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::AlternativeSyntax,
                    });
                }
            }
            // Check for packages in sections
            else if directive == "@packages" || directive == "@pkgs" {
                if trimmed == *value && is_in_packages_section(content, line_num) {
                    locations.push(Location {
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        context: LocationContext::PackagesSection,
                    });
                }
            }
        }
    }

    Ok(locations)
}

/// Check if a line is within a @packages or @pkgs section
fn is_in_packages_section(content: &str, line_num: usize) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;

    for (i, line) in lines.iter().enumerate() {
        if i >= line_num {
            break;
        }

        let trimmed = line.trim();
        if trimmed == "@packages" || trimmed == "@pkgs" {
            in_section = true;
        } else if trimmed.starts_with('@') && trimmed != "@packages" && trimmed != "@pkgs" {
            in_section = false;
        }
    }

    in_section
}

/// Get all config files from the owl directory
fn get_all_config_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let owl_dir = format!("{}/{}", home, crate::internal::constants::OWL_DIR);

    let mut files = Vec::new();

    // Check main config
    let main_config = format!("{}/main{}", owl_dir, crate::internal::constants::OWL_EXT);
    if Path::new(&main_config).exists() {
        files.push(main_config);
    }

    // Scan hosts directory
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

    // Scan groups directory
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

/// Display the found locations in a formatted way
fn display_locations(locations: &[Location]) {
    if locations.is_empty() {
        return;
    }

    // Group by file
    let mut file_groups: std::collections::HashMap<String, Vec<&Location>> = std::collections::HashMap::new();
    for location in locations {
        file_groups.entry(location.file_path.clone()).or_insert_with(Vec::new).push(location);
    }

    println!(
        "\n{} {} location(s):\n",
        crate::internal::color::bold("Found"),
        locations.len()
    );

    for (file_path, file_locations) in file_groups {
        let friendly_path = file_path.replace(&std::env::var("HOME").unwrap_or_default(), "~");
        println!(
            "{}",
            crate::internal::color::highlight(&friendly_path)
        );

        for location in file_locations {
            let context_indicator = match location.context {
                LocationContext::PackageDeclaration => crate::internal::color::success("[package]"),
                LocationContext::PackagesSection => crate::internal::color::warning("[packages]"),
                LocationContext::DirectMatch => crate::internal::color::success("[direct]"),
                LocationContext::AlternativeSyntax => crate::internal::color::warning("[alt]"),
                LocationContext::EnvDeclaration => crate::internal::color::success("[env]"),
                LocationContext::ConfigDirective => crate::internal::color::success("[config]"),
                LocationContext::ServiceDirective => crate::internal::color::success("[service]"),
                LocationContext::GroupDeclaration => crate::internal::color::success("[group]"),
            };

            println!(
                "  {} {}: {}",
                context_indicator,
                crate::internal::color::dim(&format!("line {}", location.line_number)),
                crate::internal::color::description(&location.line_content)
            );
        }
        println!();
    }
}

#[derive(Debug, Clone)]
struct Location {
    file_path: String,
    line_number: usize,
    line_content: String,
    context: LocationContext,
}

#[derive(Debug, Clone)]
enum LocationContext {
    PackageDeclaration,
    PackagesSection,
    DirectMatch,
    AlternativeSyntax,
    EnvDeclaration,
    ConfigDirective,
    ServiceDirective,
    GroupDeclaration,
}