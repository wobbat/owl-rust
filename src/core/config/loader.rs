use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use super::Config;

impl Config {
    pub fn load_all_relevant_config_files() -> Result<Self> {
        let home = env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;
        Self::load_all_relevant_config_files_from_path(
            Path::new(&home).join(crate::internal::constants::OWL_DIR),
        )
    }

    pub fn load_all_relevant_config_files_from_path<P: AsRef<Path>>(owl_root: P) -> Result<Self> {
        let mut config = Config::new();
        let owl_root = owl_root.as_ref();

        // Load in priority order: main (highest), hostname (medium), groups (lowest)

        // 1. Load main config (highest priority)
        let main_config_path = owl_root.join(crate::internal::constants::MAIN_CONFIG_FILE);
        Self::load_config_if_exists(&mut config, &main_config_path)?;

        // 2. Load host-specific config (medium priority)
        let hostname = crate::internal::constants::get_host_name()?;
        let host_config_path = owl_root
            .join(crate::internal::constants::HOSTS_DIR)
            .join(format!(
                "{}{}",
                hostname,
                crate::internal::constants::OWL_EXT
            ));
        Self::load_config_if_exists(&mut config, &host_config_path)?;

        // 3. Load group configs (lowest priority)
        let groups_path = owl_root.join(crate::internal::constants::GROUPS_DIR);
        if groups_path.exists() && groups_path.is_dir() {
            let mut processed_groups = HashSet::new();
            Self::load_groups_with_precedence(&groups_path, &mut config, &mut processed_groups)?;
        }

        Ok(config)
    }

    fn load_config_if_exists(config: &mut Config, path: &Path) -> Result<()> {
        if path.exists() {
            let loaded_config = Self::parse_file(path)?;
            config.add_if_not_exists(loaded_config);
        }
        Ok(())
    }

    fn load_groups_with_precedence(
        groups_path: &Path,
        config: &mut Config,
        processed_groups: &mut HashSet<String>,
    ) -> Result<()> {
        let mut groups_to_process: Vec<String> = config.groups.clone();

        while let Some(group_name) = groups_to_process.pop() {
            if processed_groups.contains(&group_name) {
                continue;
            }
            processed_groups.insert(group_name.clone());

            let group_file = groups_path.join(format!(
                "{}{}",
                group_name,
                crate::internal::constants::OWL_EXT
            ));
            if group_file.exists() {
                let group_config = Self::parse_file(&group_file)?;
                // Add any new groups found in this group file
                for new_group in &group_config.groups {
                    if !processed_groups.contains(new_group) {
                        groups_to_process.push(new_group.clone());
                    }
                }
                // Add packages from group config only if not already defined
                config.add_if_not_exists(group_config);
            }
        }

        Ok(())
    }

    // Adds packages/env vars from other config only if they don't already exist (respects precedence)
    pub(crate) fn add_if_not_exists(&mut self, other: Self) {
        // Only add packages that don't already exist (higher priority configs win)
        for (name, package) in other.packages {
            self.packages.entry(name).or_insert(package);
        }

        // Add groups (avoid duplicates)
        for group in other.groups {
            if !self.groups.contains(&group) {
                self.groups.push(group);
            }
        }

        // Only add env vars that don't already exist (higher priority configs win)
        for (key, value) in other.env_vars {
            self.env_vars.entry(key).or_insert(value);
        }
    }
}
