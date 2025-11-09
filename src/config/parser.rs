use anyhow::{Context, Result};
use saphyr::{LoadableYamlNode, Yaml};
use std::path::Path;

#[derive(Debug)]
pub struct ModInfo {
    pub config_home: Option<Vec<String>>,
    pub home: Option<Vec<String>>,
    pub cache_home: Option<Vec<String>>,
    pub data_home: Option<Vec<String>>,
    pub local_bin: Option<Vec<String>>,
    pub local_dir: Option<Vec<String>>,
}

impl ModInfo {
    /// Create ModInfo from a YAML mapping node
    fn from_yaml(yaml: &Yaml) -> Result<Self> {
        if !yaml.is_mapping() {
            anyhow::bail!("mod_info must be a mapping/hash");
        }

        let home = Self::parse_string_array_by_key(yaml, "home");
        let config_home = Self::parse_string_array_by_key(yaml, "config_home");
        let cache_home = Self::parse_string_array_by_key(yaml, "cache_home");
        let data_home = Self::parse_string_array_by_key(yaml, "data_home");
        let local_bin = Self::parse_string_array_by_key(yaml, "local_bin");
        let local_dir = Self::parse_string_array_by_key(yaml, "local_dir");

        // Log parsed entries
        if let Some(ref entries) = home {
            log::debug!("Parsed 'home' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        if let Some(ref entries) = config_home {
            log::debug!("Parsed 'config_home' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        if let Some(ref entries) = cache_home {
            log::debug!("Parsed 'cache_home' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        if let Some(ref entries) = data_home {
            log::debug!("Parsed 'data_home' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        if let Some(ref entries) = local_bin {
            log::debug!("Parsed 'local_bin' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        if let Some(ref entries) = local_dir {
            log::debug!("Parsed 'local_dir' section: {} entries", entries.len());
            for entry in entries {
                log::debug!("  - {}", entry);
            }
        }

        Ok(Self {
            home,
            config_home,
            cache_home,
            data_home,
            local_bin,
            local_dir,
        })
    }

    /// Safely get a string array from a mapping by key name
    fn parse_string_array_by_key(yaml: &Yaml, key: &str) -> Option<Vec<String>> {
        // Pattern match to get the mapping
        match yaml {
            Yaml::Mapping(map) => {
                // Create a Yaml::Value key to look up in the map
                let key_yaml = Yaml::value_from_str(key);

                // Use .get() which returns Option
                if let Some(value) = map.get(&key_yaml) {
                    Self::parse_string_array(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Parse a YAML node into a Vec<String> if it's an array/sequence
    fn parse_string_array(node: &Yaml) -> Option<Vec<String>> {
        // If the node is bad or null, return None
        if node.is_badvalue() || node.is_null() {
            return None;
        }

        if !node.is_sequence() {
            return None;
        }

        let mut result = Vec::new();

        // Iterate over the sequence
        for item in node.as_vec().unwrap_or(&vec![]) {
            if let Some(s) = Self::yaml_to_string(item) {
                result.push(s);
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Convert a YAML node to a string
    fn yaml_to_string(yaml: &Yaml) -> Option<String> {
        // For strings, use as_str()
        if let Some(s) = yaml.as_str() {
            return Some(s.to_string());
        }

        // For integers, use as_integer()
        if let Some(i) = yaml.as_integer() {
            return Some(i.to_string());
        }

        // For booleans, check with is_boolean() then convert
        if yaml.is_boolean() {
            // Booleans don't have as_boolean(), but we can compare with known values
            if yaml.as_str() == Some("true") {
                return Some("true".to_string());
            } else if yaml.as_str() == Some("false") {
                return Some("false".to_string());
            }
        }

        // For real numbers, they're stored as strings internally
        // Try to parse as f64 to detect real numbers
        if let Some(s) = yaml.as_str() {
            if s.parse::<f64>().is_ok() {
                return Some(s.to_string());
            }
        }

        None
    }
}

/// Parse a mod.yml file and extract the mod_info section
pub fn parse_mod_file(path: &Path) -> Result<ModInfo> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read YAML file: {:?}", path))?;

    log::debug!("Parsing YAML file: {:?}", path);

    // Parse YAML - returns a Vec of documents
    let docs = Yaml::load_from_str(&contents).context("Failed to parse YAML")?;

    // Get the first document
    let doc = docs.get(0).context("YAML file is empty")?;

    // Check that root is a mapping
    if !doc.is_mapping() {
        anyhow::bail!("Root YAML node must be a mapping/hash");
    }

    // Extract the mod_info section safely
    let mod_info_node = match doc {
        Yaml::Mapping(map) => {
            let key = Yaml::value_from_str("mod_info");
            map.get(&key).context("Missing 'mod_info' key in YAML")?
        }
        _ => anyhow::bail!("Root node is not a mapping"),
    };

    ModInfo::from_yaml(mod_info_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        YamlBuilder, colon_prefix_yaml, complex_yaml, config_home_only_yaml, home_only_yaml,
        mapped_entries_yaml, simple_yaml,
    };
    use test_log::test;

    #[test]
    fn test_parse_mod_file() {
        // Use simple_yaml() template
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&simple_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();
        assert!(mod_info.config_home.is_some());
        let config_home = mod_info.config_home.unwrap();
        assert_eq!(config_home.len(), 2);
        assert_eq!(config_home[0], "dunst");
        assert_eq!(config_home[1], "hypr");
    }

    #[test]
    fn test_parse_home_entry() {
        // Use home_only_yaml() template
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&home_only_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();
        assert!(mod_info.home.is_some());
        let home = mod_info.home.unwrap();
        assert_eq!(home.len(), 1);
        assert_eq!(home[0], ".Xresources");
    }

    #[test]
    fn test_empty_sections() {
        // Use config_home_only_yaml() template
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&config_home_only_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();
        assert!(mod_info.config_home.is_some());
        assert!(mod_info.home.is_none());
        assert!(mod_info.cache_home.is_none());
    }

    #[test]
    fn test_parse_complex_yaml() {
        // Use complex_yaml() template - one line instead of 25!
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&complex_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();

        // Test config_home section
        assert!(mod_info.config_home.is_some());
        let config_home = mod_info.config_home.unwrap();
        assert_eq!(config_home.len(), 16);
        assert_eq!(config_home[0], ":gaming/gamemode.ini");
        assert_eq!(config_home[1], ":gaming/MangoHud");
        assert_eq!(config_home[2], ":hyprland/dunst");
        assert_eq!(config_home[15], ":hyprland/xsettingsd");

        // Test home section
        assert!(mod_info.home.is_some());
        let home = mod_info.home.unwrap();
        assert_eq!(home.len(), 5);
        assert_eq!(home[0], ":.Xresources");
        assert_eq!(home[1], ":.gemrc");
        assert_eq!(home[2], ":.gitconfig");
        assert_eq!(home[3], ".docker/cli-plugins:docker/cli-plugins");
        assert_eq!(home[4], ".docker/config.json:docker/config.json");

        // Test that other sections are None
        assert!(mod_info.cache_home.is_none());
        assert!(mod_info.data_home.is_none());
        assert!(mod_info.local_bin.is_none());
        assert!(mod_info.local_dir.is_none());
    }

    #[test]
    fn test_parse_colon_prefix_entries() {
        // Use colon_prefix_yaml() template
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&colon_prefix_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();
        assert!(mod_info.config_home.is_some());
        let config_home = mod_info.config_home.unwrap();
        assert_eq!(config_home.len(), 3);

        // The parser should keep the colon prefix as-is
        assert_eq!(config_home[0], ":dunst");
        assert_eq!(config_home[1], ":hypr");
        assert_eq!(config_home[2], "normal_entry");
    }

    #[test]
    fn test_parse_mapped_entries() {
        // Use mapped_entries_yaml() template
        let temp_file = YamlBuilder::yaml_string_to_temp_file(&mapped_entries_yaml()).unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();
        assert!(mod_info.home.is_some());
        let home = mod_info.home.unwrap();
        assert_eq!(home.len(), 3);

        // The parser should keep the mapping syntax as-is
        assert_eq!(home[0], ".docker/cli-plugins:docker/cli-plugins");
        assert_eq!(home[1], ".docker/config.json:docker/config.json");
        assert_eq!(home[2], "normal_entry");
    }

    #[test]
    fn test_parse_all_sections() {
        // This test is unique - it tests ALL six sections at once
        // Keep using YamlBuilder directly
        let temp_file = YamlBuilder::new()
            .with_config_home(vec!["config_item"])
            .with_home(vec!["home_item"])
            .with_cache_home(vec!["cache_item"])
            .with_data_home(vec!["data_item"])
            .with_local_bin(vec!["bin_item"])
            .with_local_dir(vec!["dir_item"])
            .to_temp_file()
            .unwrap();

        let mod_info = parse_mod_file(temp_file.path()).unwrap();

        assert_eq!(mod_info.config_home.unwrap()[0], "config_item");
        assert_eq!(mod_info.home.unwrap()[0], "home_item");
        assert_eq!(mod_info.cache_home.unwrap()[0], "cache_item");
        assert_eq!(mod_info.data_home.unwrap()[0], "data_item");
        assert_eq!(mod_info.local_bin.unwrap()[0], "bin_item");
        assert_eq!(mod_info.local_dir.unwrap()[0], "dir_item");
    }
}
