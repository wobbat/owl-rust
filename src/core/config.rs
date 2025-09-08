use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;



#[derive(Debug, Clone, serde::Serialize)]
pub struct Package {
    pub config: Option<String>,
    pub service: Option<String>,
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Config {
    pub packages: HashMap<String, Package>,
    pub groups: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            packages: HashMap::new(),
            groups: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }



    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Config::new();
        let mut current_package: Option<String> = None;
        let mut in_packages_section = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            Self::parse_line(
                &mut config,
                &mut current_package,
                &mut in_packages_section,
                line,
            )?;
        }

        Ok(config)
    }

    fn parse_line(
        config: &mut Config,
        current_package: &mut Option<String>,
        in_packages_section: &mut bool,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if line.starts_with("@package ") || line.starts_with("@pkg ") {
            Self::parse_package_declaration(config, current_package, in_packages_section, line);
        } else if line == "@packages" || line == "@pkgs" {
            Self::parse_packages_section(in_packages_section, current_package);
        } else if line.starts_with(":config ") {
            Self::parse_config_directive(config, current_package, line)?;
        } else if line.starts_with(":service ") {
            Self::parse_service_directive(config, current_package, line)?;
        } else if line.starts_with(":env ") {
            Self::parse_package_env_directive(config, current_package, line)?;
        } else if line.starts_with("@env ") {
            Self::parse_global_env_directive(config, line)?;
        } else if line.starts_with("@group ") {
            Self::parse_group_declaration(config, current_package, line);
        } else if !line.starts_with('@') && !line.starts_with(':') && *in_packages_section {
            Self::parse_package_in_section(config, line);
        }
        // Ignore unknown lines
        Ok(())
    }

    fn parse_package_declaration(
        config: &mut Config,
        current_package: &mut Option<String>,
        in_packages_section: &mut bool,
        line: &str,
    ) {
        *in_packages_section = false;
        let name = if let Some(name) = line.strip_prefix("@package ") {
            name.trim().to_string()
        } else if let Some(name) = line.strip_prefix("@pkg ") {
            name.trim().to_string()
        } else {
            // This shouldn't happen since we check the prefix in parse_line
            line.trim().to_string()
        };
        *current_package = Some(name.clone());
        config.packages.insert(
            name.clone(),
            Package {
                config: None,
                service: None,
                env_vars: HashMap::new(),
            },
        );
    }

    fn parse_packages_section(
        in_packages_section: &mut bool,
        current_package: &mut Option<String>,
    ) {
        *in_packages_section = true;
        *current_package = None;
    }

    fn parse_group_declaration(
        config: &mut Config,
        current_package: &mut Option<String>,
        line: &str,
    ) {
        config
            .groups
            .push(line.strip_prefix("@group ").unwrap().trim().to_string());
        *current_package = None;
    }

    fn parse_package_in_section(config: &mut Config, line: &str) {
        let package_name = line.trim().to_string();
        config.packages.insert(
            package_name.clone(),
            Package {
                config: None,
                service: None,
                env_vars: HashMap::new(),
            },
        );
    }

    #[allow(clippy::collapsible_if)]
    fn parse_config_directive(
        config: &mut Config,
        current_package: &Option<String>,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rest = line.strip_prefix(":config ").unwrap();
        if let Some((source, sink)) = rest.split_once(" -> ") {
            if let Some(pkg_name) = current_package {
                if let Some(package) = config.packages.get_mut(pkg_name) {
                    // Store the full source -> destination mapping
                    package.config = Some(format!("{} -> {}", source.trim(), sink.trim()));
                }
            }
        } else {
            // Handle configs without explicit source (assume source is same as destination filename)
            if let Some(pkg_name) = current_package {
                if let Some(package) = config.packages.get_mut(pkg_name) {
                    package.config = Some(rest.trim().to_string());
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    fn parse_service_directive(
        config: &mut Config,
        current_package: &Option<String>,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service_part = line.strip_prefix(":service ").unwrap();
        let service_name = service_part
            .split('[')
            .next()
            .unwrap_or(service_part)
            .trim();
        if let Some(pkg_name) = current_package {
            if let Some(package) = config.packages.get_mut(pkg_name) {
                package.service = Some(service_name.to_string());
            }
        }
        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    fn parse_package_env_directive(
        config: &mut Config,
        current_package: &Option<String>,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let env_part = line.strip_prefix(":env ").unwrap();
        if let Some((key, value)) = env_part.split_once('=') {
            if let Some(pkg_name) = current_package {
                if let Some(package) = config.packages.get_mut(pkg_name) {
                    package
                        .env_vars
                        .insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        Ok(())
    }

    fn parse_global_env_directive(
        config: &mut Config,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let env_part = line.strip_prefix("@env ").unwrap();
        if let Some((key, value)) = env_part.split_once('=') {
            config
                .env_vars
                .insert(key.trim().to_string(), value.trim().to_string());
        }
        Ok(())
    }

    pub fn load_all_relevant_config_files() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_all_relevant_config_files_from_path(
            Path::new(&env::var("HOME")?).join(crate::internal::constants::OWL_DIR),
        )
    }

    pub fn load_all_relevant_config_files_from_path<P: AsRef<Path>>(
        owl_root: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Config::new();
        let owl_root = owl_root.as_ref();

        // Load in priority order: main (highest), hostname (medium), groups (lowest)

        // 1. Load main config (highest priority)
        let main_config_path = owl_root.join(crate::internal::constants::MAIN_CONFIG_FILE);
        Self::load_config_if_exists(&mut config, &main_config_path)?;

        // 2. Load host-specific config (medium priority)
        let hostname = fs::read_to_string("/etc/hostname")?.trim().to_string();
        let host_config_path = owl_root.join("hosts").join(format!("{}.owl", hostname));
        Self::load_config_if_exists(&mut config, &host_config_path)?;

        // 3. Load group configs (lowest priority)
        let groups_path = owl_root.join("groups");
        if groups_path.exists() && groups_path.is_dir() {
            let mut processed_groups = std::collections::HashSet::new();
            Self::load_groups_with_precedence(&groups_path, &mut config, &mut processed_groups)?;
        }

        Ok(config)
    }

    fn load_config_if_exists(
        config: &mut Config,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if path.exists() {
            let loaded_config = Self::parse_file(path)?;
            config.merge_with_precedence(loaded_config);
        }
        Ok(())
    }

    fn load_groups_with_precedence(
        groups_path: &Path,
        config: &mut Config,
        processed_groups: &mut std::collections::HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut groups_to_process: Vec<String> = config.groups.clone();

        while let Some(group_name) = groups_to_process.pop() {
            if processed_groups.contains(&group_name) {
                continue;
            }
            processed_groups.insert(group_name.clone());

            let group_file = groups_path.join(format!("{}.owl", group_name));
            if group_file.exists() {
                let group_config = Self::parse_file(&group_file)?;
                // Add any new groups found in this group file
                for new_group in &group_config.groups {
                    if !processed_groups.contains(new_group) {
                        groups_to_process.push(new_group.clone());
                    }
                }
                // Use precedence merge for groups (lowest priority)
                config.merge_with_precedence(group_config);
            }
        }

        Ok(())
    }

    // New function that implements precedence (higher priority completely replaces lower priority)
    fn merge_with_precedence(&mut self, other: Self) {
        // Replace packages completely (higher priority wins)
        for (name, package) in other.packages {
            self.packages.insert(name, package);
        }

        // Merge groups (avoid duplicates)
        for group in other.groups {
            if !self.groups.contains(&group) {
                self.groups.push(group);
            }
        }

        // Replace global env vars (higher priority wins)
        self.env_vars.extend(other.env_vars);
    }
}

/// Validate a provided .owl config file can be parsed
pub fn run_configcheck(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Err(format!("Config file not found: {}", path));
    }
    match Config::parse_file(p) {
        Ok(_) => {
            println!(
                "{} {}",
                crate::internal::color::green("✓"),
                crate::internal::color::bold(&format!("Config valid: {}", path))
            );
            Ok(())
        }
        Err(e) => Err(format!("Failed to parse {}: {}", path, e)),
    }
}

/// Validate and print the full config chain (main, hostname, groups)
pub fn run_full_configcheck() -> Result<(), String> {
    let owl_root = std::path::Path::new(&std::env::var("HOME").map_err(|_| "HOME not set".to_string())?).join(crate::internal::constants::OWL_DIR);
    println!("Loading config from: {}", owl_root.display());

    // Check main config
    let main_config_path = owl_root.join(crate::internal::constants::MAIN_CONFIG_FILE);
    println!("Main config: {} (exists: {})", main_config_path.display(), main_config_path.exists());

    // Check host config
    let hostname = std::fs::read_to_string("/etc/hostname").unwrap_or_else(|_| "unknown".to_string()).trim().to_string();
    let host_config_path = owl_root.join("hosts").join(format!("{}.owl", hostname));
    println!("Host config: {} (exists: {})", host_config_path.display(), host_config_path.exists());

    // Check groups
    let groups_path = owl_root.join("groups");
    println!("Groups dir: {} (exists: {})", groups_path.display(), groups_path.exists());
    if groups_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&groups_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  Group file: {} (exists: {})", entry.path().display(), entry.path().exists());
                }
            }
        }
    }

    match Config::load_all_relevant_config_files() {
        Ok(config) => {
            println!(
                "{}",
                crate::internal::color::green("✓ Full config chain loaded successfully")
            );
            println!("{}", serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?);

            // Print summary
            let package_count = config.packages.len();
            let dotfile_count = config.packages.values().filter(|pkg| pkg.config.is_some()).count();
            let service_count = config.packages.values().filter(|pkg| pkg.service.is_some()).count();
            let env_var_count = config.packages.values().map(|pkg| pkg.env_vars.len()).sum::<usize>() + config.env_vars.len();
            let group_count = config.groups.len();

            println!();
            println!("Summary:");
            println!("  Packages: {}", package_count);
            println!("  Dotfiles: {}", dotfile_count);
            println!("  Services: {}", service_count);
            println!("  Environment variables: {}", env_var_count);
            println!("  Groups: {}", group_count);

            Ok(())
        }
        Err(e) => Err(format!("Failed to load full config: {}", e)),
    }
}

/// Show the host-specific config path for this machine
pub fn run_confighost() -> Result<(), String> {
    let hostname = crate::internal::constants::get_host_name().unwrap_or_else(|_| "unknown".to_string());
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let path = std::path::Path::new(&home)
        .join(crate::internal::constants::OWL_DIR)
        .join("hosts")
        .join(format!("{}.owl", hostname));
    println!(
        "Host config: {}",
        crate::internal::color::bold(&path.to_string_lossy())
    );
    Ok(())
}

/// Return list of packages declared in config that are not installed
#[cfg(test)]
pub fn get_uninstalled_packages(config: &Config) -> Result<Vec<String>, String> {
    let installed = crate::core::package::get_installed_packages()?;
    let mut missing = Vec::new();
    for name in config.packages.keys() {
        if !installed.contains(name) {
            missing.push(name.clone());
        }
    }
    Ok(missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let content = "@package test\n:config test -> ~/.config/test";

        let config = Config::parse(content).unwrap();
        assert!(config.packages.contains_key("test"));
    }

    #[test]
    fn test_parse_packages_section() {
        let content = "@packages\npackage1\npackage2\npackage3";
        let config = Config::parse(content).unwrap();

        assert!(config.packages.contains_key("package1"));
        assert!(config.packages.contains_key("package2"));
        assert!(config.packages.contains_key("package3"));

        // keys serve as package names
    }

    #[test]
    fn test_parse_service_directive() {
        let content = "@package test-service\n:service test-service";
        let config = Config::parse(content).unwrap();

        let package = &config.packages["test-service"];
        assert_eq!(package.service.as_ref().unwrap(), "test-service");
    }

    #[test]
    fn test_parse_env_directive() {
        let content = "@package test-env\n:env TEST_VAR=test_value";
        let config = Config::parse(content).unwrap();

        let package = &config.packages["test-env"];
        assert_eq!(package.env_vars.get("TEST_VAR").unwrap(), "test_value");
    }

    #[test]
    fn test_parse_global_env_directive() {
        let content = "@env GLOBAL_VAR=global_value";
        let config = Config::parse(content).unwrap();

        assert_eq!(config.env_vars.get("GLOBAL_VAR").unwrap(), "global_value");
    }

    #[test]
    fn test_parse_group_directive() {
        let content = "@group test-group";
        let config = Config::parse(content).unwrap();

        assert!(config.groups.contains(&"test-group".to_string()));
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = r#"@package fish
:config fish -> ~/.config/fish

@packages
eza
vi

@package htop

@env EDITOR=vim
@group core"#;

        let config = Config::parse(content).unwrap();

        // Check @package entries
        assert!(config.packages.contains_key("fish"));
        assert!(config.packages.contains_key("htop"));

        // Check @packages entries
        assert!(config.packages.contains_key("eza"));
        assert!(config.packages.contains_key("vi"));

        // Check config directive
        assert_eq!(
            config.packages["fish"].config.as_ref().unwrap(),
            "fish -> ~/.config/fish"
        );

        // Check global env
        assert_eq!(config.env_vars.get("EDITOR").unwrap(), "vim");

        // Check group
        assert!(config.groups.contains(&"core".to_string()));
    }

    #[test]
    fn test_parse_comments_and_empty_lines() {
        let content = r#"# This is a comment

@package test

# Another comment
:config test -> ~/.config/test

@packages
# Comment in packages section
package1
package2"#;

        let config = Config::parse(content).unwrap();

        assert!(config.packages.contains_key("test"));
        assert!(config.packages.contains_key("package1"));
        assert!(config.packages.contains_key("package2"));
        assert_eq!(
            config.packages["test"].config.as_ref().unwrap(),
            "test -> ~/.config/test"
        );
    }

    #[test]
    fn test_parse_empty_config() {
        let content = "";
        let config = Config::parse(content).unwrap();

        assert!(config.packages.is_empty());
        assert!(config.groups.is_empty());
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_parse_invalid_directive() {
        let content = "@package test\n:invalid directive";
        // Should not panic, just ignore unknown directives
        let config = Config::parse(content).unwrap();
        assert!(config.packages.contains_key("test"));
    }

    #[test]
    fn test_merge_with_precedence() {
        let mut config1 = Config::new();
        config1.packages.insert(
            "test".to_string(),
            Package {
                config: Some("config1".to_string()),
                service: None,
                env_vars: std::collections::HashMap::new(),
            },
        );

        let mut config2 = Config::new();
        config2.packages.insert(
            "test".to_string(),
            Package {
                config: Some("config2".to_string()),
                service: Some("service2".to_string()),
                env_vars: std::collections::HashMap::new(),
            },
        );

        config1.merge_with_precedence(config2);

        let package = &config1.packages["test"];
        assert_eq!(package.config.as_ref().unwrap(), "config2"); // Higher priority wins
        assert_eq!(package.service.as_ref().unwrap(), "service2"); // Added from higher priority
    }

    #[test]
    fn test_get_uninstalled_packages() {
        let mut config = Config::new();

        // Add some packages to the config
        config.packages.insert(
            "installed-package".to_string(),
            Package {
                config: None,
                service: None,
                env_vars: std::collections::HashMap::new(),
            },
        );

        config.packages.insert(
            "uninstalled-package".to_string(),
            Package {
                config: None,
                service: None,
                env_vars: std::collections::HashMap::new(),
            },
        );

        // Note: This test assumes that "installed-package" exists and "uninstalled-package" doesn't
        // In a real test environment, you might want to mock the package installation check
        // For now, we'll just test that the function runs without error
        let result = get_uninstalled_packages(&config);
        assert!(result.is_ok());

        let _uninstalled = result.unwrap();
        // The result will depend on what's actually installed on the system
        // We just verify that the function runs without error
    }

    #[test]
    fn test_parse_pkg_alternative_syntax() {
        let content = "@pkg test-package\n:config test -> ~/.config/test";
        let config = Config::parse(content).unwrap();

        assert!(config.packages.contains_key("test-package"));
        let package = &config.packages["test-package"];
        assert_eq!(package.config.as_ref().unwrap(), "test -> ~/.config/test");
    }

    #[test]
    fn test_parse_pkgs_alternative_syntax() {
        let content = "@pkgs\npackage1\npackage2\npackage3";
        let config = Config::parse(content).unwrap();

        assert!(config.packages.contains_key("package1"));
        assert!(config.packages.contains_key("package2"));
        assert!(config.packages.contains_key("package3"));

        // keys serve as package names
    }

    #[test]
    fn test_parse_mixed_alternative_syntax() {
        let content = r#"@pkg fish
:config fish -> ~/.config/fish

@pkgs
eza
vi

@package htop

@env EDITOR=vim
@group core"#;

        let config = Config::parse(content).unwrap();

        // Check @pkg entries
        assert!(config.packages.contains_key("fish"));
        assert!(config.packages.contains_key("htop"));

        // Check @pkgs entries
        assert!(config.packages.contains_key("eza"));
        assert!(config.packages.contains_key("vi"));

        // Check config directive
        assert_eq!(
            config.packages["fish"].config.as_ref().unwrap(),
            "fish -> ~/.config/fish"
        );

        // Check global env
        assert_eq!(config.env_vars.get("EDITOR").unwrap(), "vim");

        // Check group
        assert!(config.groups.contains(&"core".to_string()));
    }
}
