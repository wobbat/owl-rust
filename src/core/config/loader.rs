use std::collections::HashSet;
use std::env;
use std::path::Path;

use super::Config;

impl Config {
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
        processed_groups: &mut HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                // Use precedence merge for groups (lowest priority)
                config.merge_with_precedence(group_config);
            }
        }

        Ok(())
    }

    // New function that implements precedence (higher priority completely replaces lower priority)
    pub(crate) fn merge_with_precedence(&mut self, other: Self) {
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
