use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Base directory mappings for dotfile locations
pub struct BaseDirs {
    mappings: HashMap<String, PathBuf>,
}

#[allow(dead_code)]
impl BaseDirs {
    /// Create base directory mappings
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let mut mappings = HashMap::new();

        mappings.insert("home".to_string(), home.clone());
        mappings.insert("cache_home".to_string(), home.join(".cache"));
        mappings.insert("config_home".to_string(), home.join(".config"));
        mappings.insert("data_home".to_string(), home.join(".local/share"));
        mappings.insert("local_bin".to_string(), home.join(".local/bin"));
        mappings.insert("local_dir".to_string(), home.join(".local"));

        Ok(Self { mappings })
    }

    /// Get the path for a base directory key
    pub fn get(&self, key: &str) -> Option<&PathBuf> {
        self.mappings.get(key)
    }

    /// Get all available base directory keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.mappings.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_base_dirs() {
        let dirs = BaseDirs::new().unwrap();
        assert!(dirs.get("home").is_some());
        assert!(dirs.get("config_home").is_some());
        assert!(dirs.get("invalid_key").is_none());
    }
}
