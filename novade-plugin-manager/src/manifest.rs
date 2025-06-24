//! Defines the structure for the `Plugin.toml` manifest file and provides
//! functionality to load and parse it.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use super::error::PluginManagerError;

/// Represents the overall structure of the `Plugin.toml` file.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub plugin: PluginDetails,
    // Optional sections:
    // pub dependencies: Option<HashMap<String, String>>, // More complex version requirements might need a custom type.
    // pub permissions: Option<HashMap<String, String>>,
    // pub extension_points: Option<ExtensionPoints>, // Or a more structured type
}

/// Contains the core metadata for a plugin, corresponding to the `[plugin]` table.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PluginDetails {
    pub id: String,
    pub name: String,
    pub version: String, // Consider using a semver::Version type for stricter parsing
    pub author: String,
    pub description: String,
    pub license: String,
    pub entry_point: String, // Path to the dynamic library or entry module
    #[serde(default)] // Make it optional
    pub requires_novade_version: Option<String>,
}

// Example for a more structured extension_points section, if needed later.
// #[derive(Deserialize, Debug, Clone)]
// pub struct ExtensionPoints {
//     panel_widgets: Option<Vec<PanelWidgetManifestEntry>>,
//     settings_pages: Option<Vec<SettingsPageManifestEntry>>,
// }
//
// #[derive(Deserialize, Debug, Clone)]
// pub struct PanelWidgetManifestEntry {
//     id: String,
//     name: String,
//     description: Option<String>,
// }
//
// #[derive(Deserialize, Debug, Clone)]
// pub struct SettingsPageManifestEntry {
//     id: String,
//     name: String,
//     icon: Option<String>,
// }


impl PluginManifest {
    /// Loads and parses a `Plugin.toml` file from the given path.
    pub fn load_from_file(path: &Path) -> Result<Self, PluginManagerError> {
        let content = fs::read_to_string(path).map_err(|e| {
            PluginManagerError::ManifestIoError {
                path: path.to_path_buf(),
                source: e,
            }
        })?;

        Self::load_from_string(&content, path)
    }

    /// Parses a `Plugin.toml` string.
    pub fn load_from_string(content: &str, source_path_for_error: &Path) -> Result<Self, PluginManagerError> {
        toml::from_str(content).map_err(|e| {
            PluginManagerError::ManifestParseError {
                path: source_path_for_error.to_path_buf(),
                source: e,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_path() -> PathBuf {
        PathBuf::from("dummy/Plugin.toml")
    }

    #[test]
    fn test_parse_minimal_valid_manifest() {
        let toml_content = r#"
[plugin]
id = "com.example.minimal"
name = "Minimal Plugin"
version = "0.0.1"
author = "Testy McTesterson"
description = "A very minimal plugin."
license = "Unlicense"
entry_point = "minimal_plugin_lib"
"#;
        let manifest = PluginManifest::load_from_string(toml_content, &dummy_path()).unwrap();
        assert_eq!(manifest.plugin.id, "com.example.minimal");
        assert_eq!(manifest.plugin.name, "Minimal Plugin");
        assert_eq!(manifest.plugin.version, "0.0.1");
        assert_eq!(manifest.plugin.author, "Testy McTesterson");
        assert_eq!(manifest.plugin.description, "A very minimal plugin.");
        assert_eq!(manifest.plugin.license, "Unlicense");
        assert_eq!(manifest.plugin.entry_point, "minimal_plugin_lib");
        assert!(manifest.plugin.requires_novade_version.is_none());
    }

    #[test]
    fn test_parse_full_valid_manifest() {
        let toml_content = r#"
[plugin]
id = "com.example.full"
name = "Full Plugin"
version = "1.2.3-alpha+build.123"
author = "A. U. Thor <author@example.com>"
description = "A more complete plugin example."
license = "MIT OR Apache-2.0"
entry_point = "libfull_plugin.so"
requires_novade_version = "0.2.0"

# These sections are not yet strongly typed in PluginManifest struct for simplicity,
# but the parser should not fail if they are present, due to deny_unknown_fields on root.
# To make them parsable, they'd need to be added to PluginManifest struct.
# For now, we test that unknown fields at the root would cause an error if not for deny_unknown_fields.
# With deny_unknown_fields on PluginManifest, top-level unknown tables are an error.
# If we add `dependencies` etc. to `PluginManifest` struct, this test needs adjustment.

# [dependencies]
# "core.networking" = ">=1.0"

# [permissions]
# "network" = "Access internet for updates"
"#;
        // Note: The commented out `[dependencies]` and `[permissions]` would cause a "deny_unknown_fields" error
        // if they were not commented out, because `PluginManifest` does not (yet) define these fields.
        // This is correct behavior. If we want to parse them, we add them to the struct.

        let manifest = PluginManifest::load_from_string(toml_content, &dummy_path()).unwrap();
        assert_eq!(manifest.plugin.id, "com.example.full");
        assert_eq!(manifest.plugin.version, "1.2.3-alpha+build.123");
        assert_eq!(manifest.plugin.requires_novade_version, Some("0.2.0".to_string()));
    }

    #[test]
    fn test_parse_missing_required_field() {
        let toml_content = r#"
[plugin]
# id is missing
name = "Incomplete Plugin"
version = "0.1.0"
author = "Forgot ID"
description = "This plugin is missing its ID."
license = "N/A"
entry_point = "incomplete_lib"
"#;
        let result = PluginManifest::load_from_string(toml_content, &dummy_path());
        assert!(result.is_err());
        match result.err().unwrap() {
            PluginManagerError::ManifestParseError { source, .. } => {
                assert!(source.to_string().contains("missing field `id`"));
            }
            _ => panic!("Expected ManifestParseError for missing field"),
        }
    }

    #[test]
    fn test_parse_unknown_field_in_plugin_table() {
        let toml_content = r#"
[plugin]
id = "com.example.unknown"
name = "Unknown Field Plugin"
version = "0.1.0"
author = "Test Author"
description = "This plugin has an extra field."
license = "MIT"
entry_point = "unknown_lib"
this_is_not_a_valid_field = "some value" # This should cause an error
"#;
        let result = PluginManifest::load_from_string(toml_content, &dummy_path());
        assert!(result.is_err());
        match result.err().unwrap() {
            PluginManagerError::ManifestParseError { source, .. } => {
                 // The error message for unknown fields in TOML can sometimes be "wanted key this_is_not_a_valid_field"
                 // or refer to the table name.
                assert!(source.to_string().contains("unknown field `this_is_not_a_valid_field`"));
            }
            _ => panic!("Expected ManifestParseError for unknown field"),
        }
    }

    #[test]
    fn test_parse_invalid_toml_syntax() {
        let toml_content = "this is not valid toml {";
        let result = PluginManifest::load_from_string(toml_content, &dummy_path());
        assert!(result.is_err());
        match result.err().unwrap() {
            PluginManagerError::ManifestParseError { .. } => {
                // Correct error type
            }
            _ => panic!("Expected ManifestParseError for invalid TOML syntax"),
        }
    }
}
