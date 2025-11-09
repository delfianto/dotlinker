// src/test_utils.rs
//! Shared test utilities for all test modules

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tempfile::{NamedTempFile, TempDir};

static INIT: Once = Once::new();

/// Initialize the test logger. Safe to call multiple times - will only initialize once.
pub fn init_test_logger() {
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .format(crate::logging::format_log)
            .try_init();
    });
}

#[ctor::ctor]
fn init() {
    init_test_logger();
}

// ----------------------
// YAML Template Builders
// ----------------------

/// Builder for creating mod.yml YAML content
pub struct YamlBuilder {
    config_home: Vec<String>,
    home: Vec<String>,
    cache_home: Vec<String>,
    data_home: Vec<String>,
    local_bin: Vec<String>,
    local_dir: Vec<String>,
}

impl YamlBuilder {
    /// Create a new empty YAML builder
    pub fn new() -> Self {
        Self {
            config_home: Vec::new(),
            home: Vec::new(),
            cache_home: Vec::new(),
            data_home: Vec::new(),
            local_bin: Vec::new(),
            local_dir: Vec::new(),
        }
    }

    /// Add entries to config_home section
    pub fn with_config_home(mut self, entries: Vec<&str>) -> Self {
        self.config_home = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entries to home section
    pub fn with_home(mut self, entries: Vec<&str>) -> Self {
        self.home = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entries to cache_home section
    pub fn with_cache_home(mut self, entries: Vec<&str>) -> Self {
        self.cache_home = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entries to data_home section
    pub fn with_data_home(mut self, entries: Vec<&str>) -> Self {
        self.data_home = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entries to local_bin section
    pub fn with_local_bin(mut self, entries: Vec<&str>) -> Self {
        self.local_bin = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entries to local_dir section
    pub fn with_local_dir(mut self, entries: Vec<&str>) -> Self {
        self.local_dir = entries.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the YAML string
    pub fn build(&self) -> String {
        let mut yaml = String::from("mod_info:\n");

        if !self.config_home.is_empty() {
            yaml.push_str("  config_home:\n");
            for entry in &self.config_home {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        if !self.home.is_empty() {
            yaml.push_str("  home:\n");
            for entry in &self.home {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        if !self.cache_home.is_empty() {
            yaml.push_str("  cache_home:\n");
            for entry in &self.cache_home {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        if !self.data_home.is_empty() {
            yaml.push_str("  data_home:\n");
            for entry in &self.data_home {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        if !self.local_bin.is_empty() {
            yaml.push_str("  local_bin:\n");
            for entry in &self.local_bin {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        if !self.local_dir.is_empty() {
            yaml.push_str("  local_dir:\n");
            for entry in &self.local_dir {
                yaml.push_str(&format!("    - {}\n", entry));
            }
        }

        yaml
    }

    /// Create a temporary file with this YAML content
    pub fn to_temp_file(&self) -> Result<NamedTempFile> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(self.build().as_bytes())?;
        Ok(temp_file)
    }

    /// Create a YamlBuilder from an existing YAML string
    /// Useful for working with template functions
    pub fn write_yaml_string_to_file(yaml_content: &str, path: &Path) -> Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(yaml_content.as_bytes())?;
        Ok(())
    }

    /// Create a temp file directly from a YAML string
    pub fn yaml_string_to_temp_file(yaml_content: &str) -> Result<NamedTempFile> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(yaml_content.as_bytes())?;
        Ok(temp_file)
    }
}

impl Default for YamlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------
// Common YAML Templates
// ----------------------

/// Create a simple YAML with basic config_home and home entries
pub fn simple_yaml() -> String {
    YamlBuilder::new()
        .with_config_home(vec!["dunst", "hypr"])
        .with_home(vec![".Xresources"])
        .build()
}

/// Create a complex YAML with multiple sections and entries
pub fn complex_yaml() -> String {
    YamlBuilder::new()
        .with_config_home(vec![
            ":gaming/gamemode.ini",
            ":gaming/MangoHud",
            ":hyprland/dunst",
            ":hyprland/gtk-3.0",
            ":hyprland/gtk-4.0",
            ":hyprland/hypr",
            ":hyprland/matugen",
            ":hyprland/qt6ct",
            ":hyprland/rofi",
            ":hyprland/swaync",
            ":hyprland/wal",
            ":hyprland/waybar",
            ":hyprland/waypaper",
            ":hyprland/wlogout",
            ":hyprland/xdg-desktop-portal",
            ":hyprland/xsettingsd",
        ])
        .with_home(vec![
            ":.Xresources",
            ":.gemrc",
            ":.gitconfig",
            ".docker/cli-plugins:docker/cli-plugins",
            ".docker/config.json:docker/config.json",
        ])
        .build()
}

/// Create YAML with only config_home entries
pub fn config_home_only_yaml() -> String {
    YamlBuilder::new().with_config_home(vec!["hypr"]).build()
}

/// Create YAML with only home entries
pub fn home_only_yaml() -> String {
    YamlBuilder::new().with_home(vec![".Xresources"]).build()
}

/// Create YAML with colon-prefixed entries
pub fn colon_prefix_yaml() -> String {
    YamlBuilder::new()
        .with_config_home(vec![":dunst", ":hypr", "normal_entry"])
        .build()
}

/// Create YAML with mapped entries (link:target syntax)
pub fn mapped_entries_yaml() -> String {
    YamlBuilder::new()
        .with_home(vec![
            ".docker/cli-plugins:docker/cli-plugins",
            ".docker/config.json:docker/config.json",
            "normal_entry",
        ])
        .build()
}

// -------------------------
// Module Directory Fixtures
// -------------------------

/// Create a complete test module directory structure
pub struct ModuleFixture {
    pub temp_dir: TempDir,
    pub module_name: String,
}

impl ModuleFixture {
    /// Create a new module fixture with the given name and YAML content
    pub fn new(module_name: &str, yaml_content: &str) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let module_dir = temp_dir.path().join(module_name);
        fs::create_dir_all(&module_dir)?;

        // Write mod.yml
        let mod_yml = module_dir.join("mod.yml");
        let mut file = fs::File::create(&mod_yml)?;
        file.write_all(yaml_content.as_bytes())?;

        Ok(Self {
            temp_dir,
            module_name: module_name.to_string(),
        })
    }

    /// Create with the simple YAML template
    pub fn simple(module_name: &str) -> Result<Self> {
        Self::new(module_name, &simple_yaml())
    }

    /// Create with the complex YAML template and auto-create source files
    pub fn complex(module_name: &str) -> Result<Self> {
        Self::new(module_name, &complex_yaml())?.with_source_files(vec![
            "gaming/gamemode.ini",
            "gaming/MangoHud",
            "hyprland/dunst",
            "hyprland/gtk-3.0",
            "hyprland/gtk-4.0",
            "hyprland/hypr",
            "hyprland/matugen",
            "hyprland/qt6ct",
            "hyprland/rofi",
            "hyprland/swaync",
            "hyprland/wal",
            "hyprland/waybar",
            "hyprland/waypaper",
            "hyprland/wlogout",
            "hyprland/xdg-desktop-portal",
            "hyprland/xsettingsd",
            "hyprland/.Xresources",
            "hyprland/.gemrc",
            "hyprland/.gitconfig",
            "docker/cli-plugins",
            "docker/config.json",
        ])
    }

    /// Create source files/directories based on YAML entries
    pub fn with_source_files(self, entries: Vec<&str>) -> Result<Self> {
        let module_dir = self.temp_dir.path().join(&self.module_name);

        for entry in entries {
            // Remove colon prefix if present
            let entry = entry.trim_start_matches(':');

            // Handle mapped entries (split on :)
            let source_path = if entry.contains(':') {
                entry.split(':').next().unwrap_or(entry)
            } else {
                entry
            };

            let full_path = module_dir.join(source_path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Create the file or directory
            if source_path.ends_with('/') {
                fs::create_dir_all(&full_path)?;
            } else {
                fs::File::create(&full_path)?;
            }
        }

        Ok(self)
    }

    /// Get the path to the dotfiles directory
    pub fn dotfiles_dir(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Get the path to the module directory
    pub fn module_dir(&self) -> PathBuf {
        self.temp_dir.path().join(&self.module_name)
    }

    /// Get the path to mod.yml
    pub fn mod_yml_path(&self) -> PathBuf {
        self.module_dir().join("mod.yml")
    }
}

// ---------------------------
// Tests for test_utils itself
// ---------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_can_be_called_multiple_times() {
        init_test_logger();
        init_test_logger();
        init_test_logger();
    }

    #[test]
    fn test_yaml_builder_simple() {
        let yaml = YamlBuilder::new()
            .with_config_home(vec!["dunst", "hypr"])
            .build();

        assert!(yaml.contains("mod_info:"));
        assert!(yaml.contains("config_home:"));
        assert!(yaml.contains("- dunst"));
        assert!(yaml.contains("- hypr"));
    }

    #[test]
    fn test_yaml_builder_multiple_sections() {
        let yaml = YamlBuilder::new()
            .with_config_home(vec!["dunst"])
            .with_home(vec![".Xresources"])
            .with_cache_home(vec!["cache_file"])
            .build();

        assert!(yaml.contains("config_home:"));
        assert!(yaml.contains("home:"));
        assert!(yaml.contains("cache_home:"));
    }

    #[test]
    fn test_simple_yaml_template() {
        let yaml = simple_yaml();
        assert!(yaml.contains("dunst"));
        assert!(yaml.contains("hypr"));
        assert!(yaml.contains(".Xresources"));
    }

    #[test]
    fn test_complex_yaml_template() {
        let yaml = complex_yaml();
        assert!(yaml.contains(":gaming/gamemode.ini"));
        assert!(yaml.contains(":hyprland/waybar"));
        assert!(yaml.contains(":.Xresources"));
    }

    #[test]
    fn test_module_fixture_creation() {
        let fixture = ModuleFixture::simple("test_module").unwrap();
        assert!(fixture.mod_yml_path().exists());
        assert!(fixture.module_dir().exists());
    }

    #[test]
    fn test_module_fixture_with_source_files() {
        let fixture = ModuleFixture::simple("test_module")
            .unwrap()
            .with_source_files(vec!["dunst", "hypr", ".Xresources"])
            .unwrap();

        assert!(fixture.module_dir().join("dunst").exists());
        assert!(fixture.module_dir().join("hypr").exists());
        assert!(fixture.module_dir().join(".Xresources").exists());
    }
}
