use anyhow::Result;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Represents a parsed entry with source and destination paths
#[derive(Debug, Clone)]
pub struct LinkEntry {
    /// Path in the dotfiles repository (source)
    pub source: PathBuf,
    /// Path where the symlink should be created (destination)
    pub destination: PathBuf,
}

/// Parse an entry string and resolve it to source and destination paths
pub fn parse_entry(
    entry: &str,
    module_name: &str,
    module_dir: &Path,
    base_dir: &Path,
) -> Result<LinkEntry> {
    // Strip leading colon if present
    let entry = entry.strip_prefix(':').unwrap_or(entry);

    // Check if entry contains a mapping (link_name:dest_name)
    if entry.contains(':') {
        parse_mapped_entry(entry, module_name, module_dir, base_dir)
    } else {
        parse_simple_entry(entry, module_name, module_dir, base_dir)
    }
}

/// Parse a simple entry (e.g., "dunst", "$", "*item")
fn parse_simple_entry(
    entry: &str,
    module_name: &str,
    module_dir: &Path,
    base_dir: &Path,
) -> Result<LinkEntry> {
    // Replace $ symbol with module name
    let entry = if entry == "$" { module_name } else { entry };

    // Strip leading asterisk if present
    let entry = entry.strip_prefix('*').unwrap_or(entry);

    // Source path: module_dir/entry
    let source = module_dir.join(entry);

    // Destination path: base_dir/entry
    let destination = base_dir.join(entry);

    Ok(LinkEntry {
        source,
        destination,
    })
}

/// Parse a mapped entry (e.g., "link_name:dest_name")
fn parse_mapped_entry(
    entry: &str,
    module_name: &str,
    module_dir: &Path,
    base_dir: &Path,
) -> Result<LinkEntry> {
    let parts: Vec<&str> = entry.splitn(2, ':').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid mapped entry format: {}", entry);
    }

    let link_name = parts[0];
    let dest_name = parts[1];

    // Handle $ substitution in both parts
    let link_name = substitute_module_name(link_name, module_name);
    let dest_name = substitute_module_name(dest_name, module_name);

    // Strip asterisks
    let link_name = link_name.strip_prefix('*').unwrap_or(&link_name);
    let dest_name = dest_name.strip_prefix('*').unwrap_or(&dest_name);

    // Source path: module_dir/link_name
    let source = module_dir.join(link_name);

    // Destination path: base_dir/dest_name
    let destination = base_dir.join(dest_name);

    Ok(LinkEntry {
        source,
        destination,
    })
}

/// Substitute $ with module name in a string
fn substitute_module_name(s: &str, module_name: &str) -> String {
    if s == "$" {
        module_name.to_string()
    } else {
        let re = Regex::new(r"^\$").unwrap();
        re.replace(s, module_name).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use test_log::test;

    #[test]
    fn test_parse_simple_entry() {
        let module_dir = PathBuf::from("/dotfiles/hypr");
        let base_dir = PathBuf::from("/home/user/.config");
        let entry = parse_entry("hyprland.conf", "hypr", &module_dir, &base_dir).unwrap();

        assert_eq!(entry.source, PathBuf::from("/dotfiles/hypr/hyprland.conf"));
        assert_eq!(
            entry.destination,
            PathBuf::from("/home/user/.config/hyprland.conf")
        );
    }

    #[test]
    fn test_parse_entry_with_dollar() {
        let module_dir = PathBuf::from("/dotfiles/hypr");
        let base_dir = PathBuf::from("/home/user/.config");
        let entry = parse_entry("$", "hypr", &module_dir, &base_dir).unwrap();

        assert_eq!(entry.source, PathBuf::from("/dotfiles/hypr/hypr"));
        assert_eq!(entry.destination, PathBuf::from("/home/user/.config/hypr"));
    }

    #[test]
    fn test_parse_mapped_entry() {
        let module_dir = PathBuf::from("/dotfiles/hypr");
        let base_dir = PathBuf::from("/home/user/.config");
        let entry = parse_entry("config:hypr.conf", "hypr", &module_dir, &base_dir).unwrap();

        assert_eq!(entry.source, PathBuf::from("/dotfiles/hypr/config"));
        assert_eq!(
            entry.destination,
            PathBuf::from("/home/user/.config/hypr.conf")
        );
    }

    #[test]
    fn test_parse_entry_strip_asterisk() {
        let module_dir = PathBuf::from("/dotfiles/hypr");
        let base_dir = PathBuf::from("/home/user/.config");
        let entry = parse_entry("*special", "hypr", &module_dir, &base_dir).unwrap();

        assert_eq!(entry.source, PathBuf::from("/dotfiles/hypr/special"));
        assert_eq!(
            entry.destination,
            PathBuf::from("/home/user/.config/special")
        );
    }
}
