use std::collections::HashMap;

pub mod loader;
pub mod parser;
pub mod validator;

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
        let result = validator::get_uninstalled_packages(&config);
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
