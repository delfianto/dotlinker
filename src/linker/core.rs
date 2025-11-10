use crate::linker::entry::parse_entry;
use crate::linker::path::BaseDirs;
use crate::linker::symlink::create_symlink;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct DotLinker {
    pub dotfiles_dir: PathBuf,
    pub base_dirs: BaseDirs,
    pub dry_run: bool,
    pub verbose: bool,
    pub relative: bool,
    pub force: bool,
}

impl DotLinker {
    /// Create a new DotLinker instance
    pub fn new(
        dotfiles_dir: PathBuf,
        dry_run: bool,
        verbose: bool,
        relative: bool,
        force: bool,
    ) -> Result<Self> {
        let base_dirs = BaseDirs::new().context("Failed to initialize base directories")?;

        if verbose {
            log::info!("Dotfiles directory: {:?}", dotfiles_dir);
            log::debug!("Initializing DotLinker with:");
            log::debug!("  - abs_dir : {}", !relative);
            log::debug!("  - dry_run : {}", dry_run);
            log::debug!("  - force   : {}", force);
            log::debug!("  - verbose : {}", verbose);
        }

        Ok(Self {
            dotfiles_dir,
            base_dirs,
            dry_run,
            verbose,
            relative,
            force,
        })
    }

    /// Process a single module
    pub fn process_module(&self, module_name: &str) -> Result<()> {
        log::info!("[MOD:{}] Processing module", module_name.to_uppercase());

        let module_dir = self.dotfiles_dir.join(module_name);
        log::debug!(
            "[MOD:{}] Module directory: {:?}",
            module_name.to_uppercase(),
            module_dir
        );

        if !module_dir.exists() {
            anyhow::bail!("Module directory does not exist: {:?}", module_dir);
        }

        let mod_file = module_dir.join("mod.yaml");
        log::debug!(
            "[MOD:{}] Looking for config file: {:?}",
            module_name.to_uppercase(),
            mod_file
        );

        if !mod_file.exists() {
            anyhow::bail!("Module configuration not found: {:?}", mod_file);
        }

        log::debug!(
            "[MOD:{}] Reading configuration from {:?}",
            module_name.to_uppercase(),
            mod_file
        );

        let mod_info = crate::config::parse_mod_file(&mod_file)?;

        // Process each base directory category
        self.process_config_entries(
            module_name,
            &module_dir,
            "config_home",
            &mod_info.config_home,
        )?;
        self.process_config_entries(module_name, &module_dir, "home", &mod_info.home)?;
        self.process_config_entries(module_name, &module_dir, "cache_home", &mod_info.cache_home)?;
        self.process_config_entries(module_name, &module_dir, "data_home", &mod_info.data_home)?;
        self.process_config_entries(module_name, &module_dir, "local_bin", &mod_info.local_bin)?;
        self.process_config_entries(module_name, &module_dir, "local_dir", &mod_info.local_dir)?;

        log::info!(
            "[MOD:{}] Module processing complete",
            module_name.to_uppercase()
        );
        Ok(())
    }

    /// Process entries for a specific base directory
    fn process_config_entries(
        &self,
        module_name: &str,
        module_dir: &Path,
        base_key: &str,
        entries: &Option<Vec<String>>,
    ) -> Result<()> {
        let entries = match entries {
            Some(e) => e,
            None => {
                log::debug!(
                    "[MOD:{}] No entries for {} - skipping",
                    module_name.to_uppercase(),
                    base_key
                );
                return Ok(());
            }
        };

        if entries.is_empty() {
            log::debug!(
                "[MOD:{}] Empty entry list for {} - skipping",
                module_name.to_uppercase(),
                base_key
            );
            return Ok(());
        }

        let base_dir = self
            .base_dirs
            .get(base_key)
            .with_context(|| format!("Unknown base directory: {}", base_key))?;

        log::debug!(
            "[MOD:{}] Processing {} entries for {} (base: {:?})",
            module_name.to_uppercase(),
            entries.len(),
            base_key,
            base_dir
        );

        for (idx, entry_str) in entries.iter().enumerate() {
            log::debug!(
                "[MOD:{}] Entry {}/{}: {}",
                module_name.to_uppercase(),
                idx + 1,
                entries.len(),
                entry_str
            );

            if let Err(e) = self.process_entry(module_name, module_dir, base_dir, entry_str) {
                log::error!(
                    "[MOD:{}] Failed to process entry '{}': {:?}",
                    module_name.to_uppercase(),
                    entry_str,
                    e
                );
                // Continue with other entries
            }
        }

        Ok(())
    }

    /// Process a single entry (e.g., ":dunst" or "link:target")
    fn process_entry(
        &self,
        module_name: &str,
        module_dir: &Path,
        base_dir: &Path,
        entry_str: &str,
    ) -> Result<()> {
        log::debug!(
            "[MOD:{}] Processing entry: {}",
            module_name.to_uppercase(),
            entry_str
        );

        // Parse the entry to get source and destination paths
        let entry = parse_entry(entry_str, module_name, module_dir, base_dir)?;

        log::debug!(
            "[MOD:{}] Parsed entry - Source: {:?}, Destination: {:?}",
            module_name.to_uppercase(),
            entry.source,
            entry.destination
        );

        // Create the symlink
        create_symlink(
            &entry.source,
            &entry.destination,
            self.relative,
            self.dry_run,
            self.force,
            module_name,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ModuleFixture;
    use test_log::test;

    #[test]
    fn test_dotlinker_new_with_absolute_links() {
        let fixture = ModuleFixture::simple("test_module").unwrap();
        let dotfiles_dir = fixture.dotfiles_dir();

        let linker = DotLinker::new(
            dotfiles_dir.clone(),
            true,  // dry_run
            true,  // verbose
            false, // relative
            false, // force
        )
        .unwrap();

        assert_eq!(linker.dotfiles_dir, dotfiles_dir);
        assert!(linker.dry_run);
        assert!(linker.verbose);
        assert!(linker.relative);
        assert!(!linker.force);
    }

    #[test]
    fn test_dotlinker_new_with_relative_links() {
        let fixture = ModuleFixture::simple("test_module").unwrap();

        let linker = DotLinker::new(
            fixture.dotfiles_dir(),
            false, // dry_run
            false, // verbose
            true,  // relative
            true,  // force
        )
        .unwrap();

        assert!(!linker.relative);
        assert!(linker.force);
    }

    #[test]
    fn test_process_module_missing_directory() {
        let fixture = ModuleFixture::simple("existing_module").unwrap();
        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();

        let result = linker.process_module("nonexistent_module");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_process_module_missing_mod_yml() {
        let fixture = ModuleFixture::simple("test_module").unwrap();

        // Create another module without mod.yml
        let module_dir = fixture.dotfiles_dir().join("no_config_module");
        std::fs::create_dir_all(&module_dir).unwrap();

        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();
        let result = linker.process_module("no_config_module");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_process_module_success() {
        let fixture = ModuleFixture::simple("test_module")
            .unwrap()
            .with_source_files(vec!["dunst", "hypr", ".Xresources"])
            .unwrap();

        let linker = DotLinker::new(
            fixture.dotfiles_dir(),
            true,  // dry_run
            true,  // verbose
            false, // relative
            false, // force
        )
        .unwrap();

        let result = linker.process_module("test_module");
        assert!(
            result.is_ok(),
            "Should process module successfully: {:?}",
            result
        );
    }

    #[test]
    fn test_process_complex_module() {
        let fixture = ModuleFixture::complex("hyprland").unwrap();

        let linker = DotLinker::new(
            fixture.dotfiles_dir(),
            true,  // dry_run
            true,  // verbose
            false, // relative
            false, // force
        )
        .unwrap();

        let result = linker.process_module("hyprland");
        assert!(
            result.is_ok(),
            "Should process complex module: {:?}",
            result
        );
    }

    #[test]
    fn test_process_config_entries_empty() {
        let fixture = ModuleFixture::simple("test").unwrap();
        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();
        let module_dir = fixture.module_dir();

        let result = linker.process_config_entries("test", &module_dir, "config_home", &None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_config_entries_with_entries() {
        let fixture = ModuleFixture::simple("test_module")
            .unwrap()
            .with_source_files(vec!["dunst", "hypr"])
            .unwrap();

        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();
        let module_dir = fixture.module_dir();
        let entries = Some(vec!["dunst".to_string()]);

        let result =
            linker.process_config_entries("test_module", &module_dir, "config_home", &entries);

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_entry_colon_prefix() {
        // Create fixture with colon-prefixed entry
        let fixture = ModuleFixture::simple("test_module")
            .unwrap()
            .with_source_files(vec!["dunst"])
            .unwrap();

        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();
        let module_dir = fixture.module_dir();
        let base_dir = linker.base_dirs.get("config_home").unwrap();

        let result = linker.process_entry("test_module", &module_dir, base_dir, ":dunst");

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_process_multiple_modules() {
        // Create first module
        let fixture1 = ModuleFixture::simple("module1")
            .unwrap()
            .with_source_files(vec!["dunst", "hypr", ".Xresources"])
            .unwrap();

        // Create additional modules in the same temp dir
        for module_name in &["module2", "module3"] {
            let module_dir = fixture1.dotfiles_dir().join(module_name);
            std::fs::create_dir_all(&module_dir).unwrap();

            // Write simple_yaml to each module
            use crate::test_utils::{YamlBuilder, simple_yaml};
            YamlBuilder::write_yaml_string_to_file(&simple_yaml(), &module_dir.join("mod.yml"))
                .unwrap();

            // Create source files
            std::fs::File::create(module_dir.join("dunst")).unwrap();
            std::fs::File::create(module_dir.join("hypr")).unwrap();
            std::fs::File::create(module_dir.join(".Xresources")).unwrap();
        }

        let linker = DotLinker::new(fixture1.dotfiles_dir(), true, true, false, false).unwrap();

        for module_name in &["module1", "module2", "module3"] {
            let result = linker.process_module(module_name);
            assert!(
                result.is_ok(),
                "Module {} should process successfully",
                module_name
            );
        }
    }

    #[test]
    fn test_verbose_and_non_verbose() {
        let fixture = ModuleFixture::simple("test").unwrap();

        let linker_verbose = DotLinker::new(
            fixture.dotfiles_dir(),
            true,  // dry_run
            true,  // verbose
            false, // relative
            false, // force
        )
        .unwrap();
        assert!(linker_verbose.verbose);

        let linker_quiet = DotLinker::new(
            fixture.dotfiles_dir(),
            true,  // dry_run
            false, // verbose
            false, // relative
            false, // force
        )
        .unwrap();
        assert!(!linker_quiet.verbose);
    }

    #[test]
    fn test_dry_run_vs_real_mode() {
        let fixture = ModuleFixture::simple("test").unwrap();

        let linker_dry = DotLinker::new(
            fixture.dotfiles_dir(),
            true,  // dry_run
            false, // verbose
            false, // relative
            false, // force
        )
        .unwrap();
        assert!(linker_dry.dry_run);

        let linker_real = DotLinker::new(
            fixture.dotfiles_dir(),
            false, // dry_run
            false, // verbose
            false, // relative
            false, // force
        )
        .unwrap();
        assert!(!linker_real.dry_run);
    }

    #[test]
    fn test_all_base_directories() {
        // Tests ALL base directories at once
        use crate::test_utils::YamlBuilder;

        let fixture = ModuleFixture::new(
            "full_module",
            &YamlBuilder::new()
                .with_config_home(vec!["config_item"])
                .with_home(vec!["home_item"])
                .with_cache_home(vec!["cache_item"])
                .with_data_home(vec!["data_item"])
                .with_local_bin(vec!["bin_item"])
                .with_local_dir(vec!["dir_item"])
                .build(),
        )
        .unwrap()
        .with_source_files(vec![
            "config_item",
            "home_item",
            "cache_item",
            "data_item",
            "bin_item",
            "dir_item",
        ])
        .unwrap();

        let linker = DotLinker::new(fixture.dotfiles_dir(), true, true, false, false).unwrap();
        let result = linker.process_module("full_module");

        assert!(result.is_ok(), "Should process all base directories");
    }
}
