//! Configuration Loading for NovaDE Core.
//!
//! This module provides the [`ConfigLoader`] struct, which is responsible for
//! loading, parsing, and validating the `CoreConfig` for the NovaDE system.
//! It handles locating the configuration file, deserializing it from TOML format,
//! applying default values, and performing validation checks on the loaded configuration.
//!
//! # Usage
//!
//! The primary way to use this module is through the static `ConfigLoader::load()` method:
//!
//! ```rust,ignore
//! use novade_core::config::ConfigLoader;
//! use novade_core::error::CoreError;
//!
//! match ConfigLoader::load() {
//!     Ok(config) => {
//!         // Use the loaded and validated config
//!         println!("Logging level: {}", config.logging.level);
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to load configuration: {}", e);
//!         // Handle errors, perhaps by using minimal defaults or exiting
//!         // For example, initialize minimal logging:
//!         novade_core::logging::init_minimal_logging();
//!         tracing::error!("Configuration loading failed: {}", e);
//!     }
//! }
//! ```
//!
//! ## Configuration File Location
//!
//! `ConfigLoader::load()` attempts to load `config.toml` from the application-specific
//! configuration directory, as determined by `novade_core::utils::paths::get_app_config_dir()`.
//! If the file is not found, a default configuration is used.
//!
//! ## Validation
//!
//! After loading (or generating defaults), the configuration undergoes validation via
//! the `validate_config` method. This includes:
//! - Normalizing and validating log levels and formats.
//! - Resolving relative log file paths to absolute paths within the application's state directory.
//! - Ensuring necessary parent directories for log files are created.

use std::path::PathBuf;
use std::fs;
use std::env; // For accessing environment variables
use toml::Value; // For TOML manipulation

// Use CoreConfig from the parent module (config/mod.rs)
use crate::config::{
    CoreConfig, LoggingConfig, FeatureFlags, defaults, CompositorConfig, PerformanceConfig, InputConfig, VisualConfig
};
use crate::error::{CoreError, ConfigError};
use crate::utils::paths::{get_app_config_dir, get_app_state_dir, get_system_config_path_with_override}; // Import new path helper
use crate::utils::fs as nova_fs; // Renamed to avoid conflict with std::fs

/// `ConfigLoader` provides static methods to load and validate `CoreConfig`.
///
/// This is an empty struct used as a namespace for configuration loading logic.
/// The main entry point is the `load()` method.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads and validates the `CoreConfig` for the application.
    ///
    /// This method performs the following steps:
    /// 1. Determines the path to the application-specific configuration file (`config.toml`)
    ///    using [`get_app_config_dir`].
    /// 2. Attempts to read the configuration file:
    ///    - If the file is found and read successfully, its TOML content is parsed.
    ///    - If the file is not found (`ErrorKind::NotFound`), a default `CoreConfig` is generated.
    ///      This behavior aligns with the specification: "Wenn die Konfigurationsdatei nicht existiert,
    ///      wird eine Standardkonfiguration verwendet."
    ///    - Other file read errors result in a [`CoreError::Config(ConfigError::ReadError)`].
    /// 3. Parses the TOML content into a [`CoreConfig`]. Parsing errors result in
    ///    [`CoreError::Config(ConfigError::ParseError)`].
    /// 4. Validates the loaded or default configuration using [`Self::validate_config`].
    ///    Validation errors result in [`CoreError::Config(ConfigError::ValidationError)`] or
    ///    filesystem errors from path resolution/creation.
    ///
    /// # Returns
    ///
    /// - `Ok(CoreConfig)`: The loaded and validated configuration.
    /// - `Err(CoreError)`: If any step (directory resolution, file reading, parsing, validation) fails.
    ///   Specific error variants within `CoreError` (like `ConfigError` or `Filesystem`) provide more detail.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // This example assumes a valid environment for path resolution.
    /// // In tests, path resolution might need mocking or careful setup.
    /// match novade_core::config::ConfigLoader::load() {
    ///     Ok(config) => println!("Successfully loaded config with log level: {}", config.logging.level),
    ///     Err(e) => eprintln!("Error loading config: {}", e),
    /// }
    /// ```
    pub fn load() -> Result<CoreConfig, CoreError> {
        // 1. Read System Config
        let system_config_path = get_system_config_path_with_override()
            .map_err(|e| CoreError::Config(e))?; // Convert ConfigError to CoreError

        let system_toml_value = match fs::read_to_string(&system_config_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    None
                } else {
                    Some(content.parse::<Value>().map_err(|e| {
                        CoreError::Config(ConfigError::ParseError(e))
                    })?)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                return Err(CoreError::Config(ConfigError::ReadError {
                    path: system_config_path,
                    source: e,
                }));
            }
        };

        // 2. Read User Config
        let user_config_dir = get_app_config_dir()?;
        let user_config_path = user_config_dir.join("config.toml");
        
        let user_toml_value = match fs::read_to_string(&user_config_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    None
                } else {
                    Some(content.parse::<Value>().map_err(|e| {
                        CoreError::Config(ConfigError::ParseError(e))
                    })?)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                return Err(CoreError::Config(ConfigError::ReadError {
                    path: user_config_path,
                    source: e,
                }));
            }
        };

        // 3. Merge TOML Values
        let merged_toml = Self::merge_toml_values(system_toml_value, user_toml_value);

        // 4. Deserialize to CoreConfig
        let mut final_config: CoreConfig = if let Some(value) = merged_toml {
            toml::from_str(&value.to_string())
                .map_err(|e| CoreError::Config(ConfigError::ParseError(e)))?
        } else {
            CoreConfig::default()
        };

        // 5. Validation
        Self::validate_config(&mut final_config)?;
        Ok(final_config)
    }

    /// Merges two optional TOML values. `override_val` takes precedence.
    fn merge_toml_values(base: Option<Value>, override_val: Option<Value>) -> Option<Value> {
        match (base, override_val) {
            (None, None) => None,
            (Some(b), None) => Some(b),
            (None, Some(o)) => Some(o),
            (Some(Value::Table(mut base_table)), Some(Value::Table(override_table))) => {
                Self::merge_toml_tables(&mut base_table, &override_table);
                Some(Value::Table(base_table))
            }
            (_, Some(o)) => Some(o), // Override takes precedence if types differ or base is not a table
        }
    }

    /// Recursively merges `override_table` into `base_table`.
    fn merge_toml_tables(base_table: &mut toml::map::Map<String, Value>, override_table: &toml::map::Map<String, Value>) {
        for (key, override_item) in override_table {
            match base_table.get_mut(key) {
                Some(base_item) => {
                    if let (Value::Table(bt), Value::Table(ot)) = (base_item, override_item) {
                        Self::merge_toml_tables(bt, ot); // Recursive call for nested tables
                    } else {
                        *base_item = override_item.clone(); // Override non-table or if types differ
                    }
                }
                None => {
                    base_table.insert(key.clone(), override_item.clone()); // Add new key from override
                }
            }
        }
    }


    /// Validates the loaded `CoreConfig` and performs necessary adjustments.
    ///
    /// This internal method is called by `load()` after a configuration is successfully
    /// parsed or a default one is generated.
    ///
    /// Validation steps include:
    /// - Normalizing and validating the logging level (must be one of "trace", "debug", "info", "warn", "error").
    /// - Normalizing and validating the logging format (must be "text" or "json").
    /// - Resolving the logging file path:
    ///   - If `file_path` is `Some` and relative, it's made absolute against the application's state directory
    ///     (obtained via [`get_app_state_dir`]).
    ///   - Parent directories for the log file path are created if they don't exist using
    ///     [`nova_fs::ensure_directory_exists`].
    ///
    /// # Arguments
    ///
    /// * `config`: A mutable reference to the `CoreConfig` to validate and potentially modify (e.g., path resolution).
    ///
    /// # Errors
    ///
    /// Returns a `CoreError` if:
    /// - Validation fails (e.g., invalid log level or format), resulting in [`CoreError::Config(ConfigError::ValidationError)`].
    /// - A required application directory (like the state directory for log files) cannot be determined,
    ///   resulting in [`CoreError::Config(ConfigError::DirectoryUnavailable)`].
    /// - Filesystem operations (like creating log directories) fail, resulting in [`CoreError::Filesystem`].
    fn validate_config(config: &mut CoreConfig) -> Result<(), CoreError> {
        // Validate logging level
        let level_lower = config.logging.level.to_lowercase();
        match level_lower.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {
                config.logging.level = level_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid log level: '{}'. Must be one of trace, debug, info, warn, error.",
                    config.logging.level
                ))));
            }
        }

        // Validate logging format
        let format_lower = config.logging.format.to_lowercase();
        match format_lower.as_str() {
            "text" | "json" => {
                config.logging.format = format_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid log format: '{}'. Must be one of text, json.",
                    config.logging.format
                ))));
            }
        }

        // Handle log file path
        if let Some(relative_path) = &config.logging.file_path {
            if relative_path.is_absolute() {
                // If it's already absolute, use it as is, but ensure parent dir exists
                if let Some(parent_dir) = relative_path.parent() {
                    if !parent_dir.exists() {
                         nova_fs::ensure_dir_exists(parent_dir)?;
                    }
                }
            } else {
                // If relative, make it absolute to the app state directory
                let state_dir = get_app_state_dir()?;
                let absolute_path = state_dir.join(relative_path);
                
                if let Some(parent_dir) = absolute_path.parent() {
                     if !parent_dir.exists() {
                        nova_fs::ensure_dir_exists(parent_dir)?;
                    }
                }
                config.logging.file_path = Some(absolute_path);
            }
        }
        
        // FeatureFlags are part of CoreConfig. No specific validation needed here for them beyond parsing.

        // Validate CompositorConfig
        Self::validate_performance_config(&mut config.compositor.performance)?;
        Self::validate_input_config(&mut config.compositor.input)?;
        Self::validate_visual_config(&mut config.compositor.visual)?;

        Ok(())
    }

    // --- Private validation functions for CompositorConfig sections ---

    fn validate_performance_config(perf_config: &mut PerformanceConfig) -> Result<(), CoreError> {
        // Validate quality_preset
        let quality_lower = perf_config.quality_preset.to_lowercase();
        match quality_lower.as_str() {
            "low" | "medium" | "high" => {
                perf_config.quality_preset = quality_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid quality_preset: '{}'. Must be one of low, medium, high.",
                    perf_config.quality_preset
                ))));
            }
        }

        // Validate power_management_mode
        let power_mode_lower = perf_config.power_management_mode.to_lowercase();
        match power_mode_lower.as_str() {
            "performance" | "balanced" | "powersave" => {
                perf_config.power_management_mode = power_mode_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid power_management_mode: '{}'. Must be one of performance, balanced, powersave.",
                    perf_config.power_management_mode
                ))));
            }
        }
        Ok(())
    }

    fn validate_input_config(input_config: &mut InputConfig) -> Result<(), CoreError> {
        input_config.keyboard_layout = input_config.keyboard_layout.to_lowercase();
        input_config.pointer_acceleration = input_config.pointer_acceleration.to_lowercase();
        // No specific validation for other fields like enable_touch_gestures (bool),
        // multi_device_support (string, but any value is fine for now),
        // or input_profiles (Vec<String>, any list is fine for now).
        Ok(())
    }

    fn validate_visual_config(visual_config: &mut VisualConfig) -> Result<(), CoreError> {
        visual_config.theme_name = visual_config.theme_name.to_lowercase();
        visual_config.color_scheme = visual_config.color_scheme.to_lowercase();

        // Validate font_settings (assuming it's a font name string for now)
        // If font_settings were optional (Option<String>), this would be different.
        // Current struct has `font_settings: String`, so it must be provided by defaults if not in TOML.
        // The default is "system-ui", which is not empty.
        // This check ensures that if a user *provides* it in TOML, it's not empty.
        if visual_config.font_settings.trim().is_empty() {
            return Err(CoreError::Config(ConfigError::ValidationError(
                "VisualConfig.font_settings must not be empty.".to_string(),
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use relevant config structs from crate::config for tests as well
    use crate::config::{LoggingConfig, PerformanceConfig, InputConfig, VisualConfig, CompositorConfig};
    use crate::utils::{self, paths::{get_app_config_dir, get_app_state_dir}}; // For test setup
    use std::env;
    use std::fs::{self, File}; // Added fs for create_temp_config_file and other operations
    use std::io::Write;
    use std::path::{Path, PathBuf}; // Added Path
    use tempfile::TempDir;


    // Helper to create a temporary config file
    fn create_temp_config_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).expect("Failed to write temp config file");
        path
    }

    // Helper to set up a temporary environment for config loading tests
    // This involves overriding where get_app_config_dir() and get_app_state_dir() look.
    // We can do this by setting XDG environment variables that `directories-next` respects.
    struct TestEnv {
        _temp_config_dir: TempDir, // User config
        _temp_state_dir: TempDir,  // For logs etc.
        _temp_system_config_dir: Option<TempDir>, // System config (optional, for specific tests)
        original_xdg_config_home: Option<String>,
        original_xdg_state_home: Option<String>,
        original_system_config_path_env: Option<String>,
    }

    impl TestEnv {
        /// Basic TestEnv for user-level config and state.
        fn new() -> Self {
            Self::new_with_custom_system_path(None)
        }

        /// TestEnv that also sets up a temporary system config path.
        /// If `system_config_filename` is Some, it creates a directory for it and sets NOVADE_TEST_SYSTEM_CONFIG_PATH.
        fn new_with_custom_system_path(system_config_filename: Option<&str>) -> Self {
            let temp_config_dir = TempDir::new().unwrap();
            let temp_state_dir = TempDir::new().unwrap();

            let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
            let original_xdg_state_home = env::var("XDG_STATE_HOME").ok();
            let original_system_config_path_env = env::var("NOVADE_TEST_SYSTEM_CONFIG_PATH").ok();

            env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
            env::set_var("XDG_STATE_HOME", temp_state_dir.path());

            let mut temp_system_config_dir_owner: Option<TempDir> = None;
            if let Some(filename) = system_config_filename {
                let temp_sys_dir = TempDir::new().unwrap();
                let system_config_file_path = temp_sys_dir.path().join(filename);
                env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", system_config_file_path);
                temp_system_config_dir_owner = Some(temp_sys_dir);
            }


            let app_cfg_dir = get_app_config_dir().expect("TestEnv: Failed to resolve app config dir");
            utils::fs::ensure_dir_exists(&app_cfg_dir).expect("TestEnv: Failed to create temp app config dir");

            let app_state_dir = get_app_state_dir().expect("TestEnv: Failed to resolve app state dir");
            utils::fs::ensure_dir_exists(&app_state_dir).expect("TestEnv: Failed to create temp app state dir");

            Self {
                _temp_config_dir: temp_config_dir,
                _temp_state_dir: temp_state_dir,
                _temp_system_config_dir: temp_system_config_dir_owner,
                original_xdg_config_home,
                original_xdg_state_home,
                original_system_config_path_env,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            if let Some(val) = &self.original_xdg_config_home {
                env::set_var("XDG_CONFIG_HOME", val);
            } else {
                env::remove_var("XDG_CONFIG_HOME");
            }
            if let Some(val) = &self.original_xdg_state_home {
                env::set_var("XDG_STATE_HOME", val);
            } else {
                env::remove_var("XDG_STATE_HOME");
            }
            if let Some(val) = &self.original_system_config_path_env {
                env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", val);
            } else {
                env::remove_var("NOVADE_TEST_SYSTEM_CONFIG_PATH");
            }
        }
    }
    
    // Mock get_app_config_dir, get_app_state_dir and get_system_config_path_with_override for consistent testing environments
    // This is complex without a mocking library. For these tests, we'll rely on temp dirs
    // and override the default config path by directly manipulating where ConfigLoader looks,
    // or by ensuring the test environment has these dirs (less ideal for unit tests).
    // The current ConfigLoader::load() is hardcoded to use get_app_config_dir().
    // To make this testable without actual FS side effects in user dirs, we need to
    // either:
    // 1. Mock `get_app_config_dir` and `get_app_state_dir`. (Requires a mocking framework or feature flags)
    // 2. Parameterize `ConfigLoader::load` with the config path (breaks current spec of `load()`).
    // 3. Use environment variables during tests to control `directories-next` (as TestEnv does).
    //
    // For now, these tests will *try* to run, but might fail or have side effects in some CI/local envs
    // if `get_app_config_dir` points to a real but unwriteable location or similar.
    // A robust solution would involve abstracting path resolution for testing.
    // The TestEnv helper is a good step in this direction.

    #[test]
    fn test_load_no_configs_uses_full_defaults() {
        // TestEnv sets up paths but doesn't create files unless we do.
        // We also need to ensure NOVADE_TEST_SYSTEM_CONFIG_PATH is set to a non-existent file path for this test.
        let _test_env = TestEnv::new_with_custom_system_path(Some("non_existent_system.toml"));

        // Ensure user config file also does not exist
        let user_app_config_dir = get_app_config_dir().unwrap();
        let user_config_file_path = user_app_config_dir.join("config.toml");
        if user_config_file_path.exists() {
            fs::remove_file(&user_config_file_path).unwrap();
        }

        // Ensure system config file (pointed to by env var) does not exist
        let system_config_path = get_system_config_path_with_override().unwrap();
        if system_config_path.exists() {
            fs::remove_file(&system_config_path).unwrap();
        }


        let config = ConfigLoader::load().expect("ConfigLoader::load failed for no configs");
        let default_config = CoreConfig::default();

        // Validate against a fully default CoreConfig, which itself has been validated (as per default() impl)
        // Our load function also calls validate_config, so the paths might be absolute.
        // So, we create a default config and validate it for a fair comparison.
        let mut expected_default_config = CoreConfig::default();
        ConfigLoader::validate_config(&mut expected_default_config).unwrap();

        assert_eq!(config, expected_default_config);
    }


    #[test]
    fn test_config_loader_load_success_user_only() { // Renamed from test_config_loader_load_success
        // System config path is set to something non-existent by TestEnv default or specific setup
        let _test_env = TestEnv::new_with_custom_system_path(Some("non_existent_system.toml"));
        let app_config_dir = get_app_config_dir().unwrap();
        
        let user_toml_content = r#"
[logging]
level = "debug"
format = "json"
file_path = "logs/app.log"

[feature_flags]
experimental_feature_x = true

[compositor.performance]
gpu_preference = "nvidia"
quality_preset = "High"
memory_limit_mb = 1024
power_management_mode = "Performance"
adaptive_tuning = false
        "#;
        create_temp_config_file(&app_config_dir, "config.toml", user_toml_content);

        let config = ConfigLoader::load().expect("ConfigLoader::load failed for user-only");

        assert_eq!(config.logging.level, "debug"); // Normalized by validate_config
        assert_eq!(config.logging.format, "json"); // Normalized
        assert!(config.logging.file_path.is_some());
        assert_eq!(config.feature_flags.experimental_feature_x, true);
        assert_eq!(config.compositor.performance.quality_preset, "high"); // Normalized
        assert_eq!(config.compositor.performance.power_management_mode, "performance"); // Normalized
        assert_eq!(config.compositor.performance.gpu_preference, "nvidia");
    }

    #[test]
    fn test_load_only_system_config() {
        let system_config_filename = "system.toml";
        let _test_env = TestEnv::new_with_custom_system_path(Some(system_config_filename));

        let system_config_path = get_system_config_path_with_override().unwrap();
        let system_config_dir = system_config_path.parent().unwrap();
        utils::fs::ensure_dir_exists(system_config_dir).unwrap();


        let system_toml_content = r#"
[logging]
level = "warn"
format = "text"

[compositor.visual]
theme_name = "SystemTheme"
color_scheme = "Dark"
        "#;
        create_temp_config_file(system_config_dir, system_config_filename, system_toml_content);

        // Ensure user config does not exist
        let user_app_config_dir = get_app_config_dir().unwrap();
        let user_config_file_path = user_app_config_dir.join("config.toml");
        if user_config_file_path.exists() {
            fs::remove_file(&user_config_file_path).unwrap();
        }

        let config = ConfigLoader::load().expect("ConfigLoader::load failed for system-only");

        assert_eq!(config.logging.level, "warn");
        assert_eq!(config.logging.format, "text");
        // Fields not in system config should be default
        assert_eq!(config.logging.file_path, defaults::default_log_file_path());
        assert_eq!(config.feature_flags, defaults::default_feature_flags());
        assert_eq!(config.compositor.visual.theme_name, "systemtheme"); // Normalized
        assert_eq!(config.compositor.visual.color_scheme, "dark"); // Normalized
        // Other compositor sections should be default
        assert_eq!(config.compositor.performance, defaults::default_performance_config());
        assert_eq!(config.compositor.input, defaults::default_input_config());
    }


    #[test]
    fn test_load_and_merge_configs_user_overrides_system() {
        let system_config_filename = "system_override.toml";
        let _test_env = TestEnv::new_with_custom_system_path(Some(system_config_filename));

        // Setup System Config
        let system_config_path = get_system_config_path_with_override().unwrap();
        let system_config_dir = system_config_path.parent().unwrap();
        utils::fs::ensure_dir_exists(system_config_dir).unwrap();
        let system_toml_content = r#"
[logging]
level = "info" # System: info
format = "text"
file_path = "system_logs/app.log"

[feature_flags]
experimental_feature_x = false # System: false

[compositor.performance]
quality_preset = "medium" # System: medium
power_management_mode = "balanced"

[compositor.visual]
theme_name = "SystemDefault"
        "#;
        create_temp_config_file(system_config_dir, system_config_filename, system_toml_content);

        // Setup User Config (overrides logging.level and feature_flags.experimental_feature_x)
        let user_app_config_dir = get_app_config_dir().unwrap();
        let user_toml_content = r#"
[logging]
level = "debug" # User: debug (override)

[feature_flags]
experimental_feature_x = true # User: true (override)

[compositor.performance]
quality_preset = "high" # User: high (override)
# power_management_mode is NOT overridden by user
        "#;
        create_temp_config_file(&user_app_config_dir, "config.toml", user_toml_content);

        let config = ConfigLoader::load().expect("ConfigLoader::load failed for merge test");

        // Assertions
        assert_eq!(config.logging.level, "debug"); // User override
        assert_eq!(config.logging.format, "text");  // From system (not in user)
        assert!(config.logging.file_path.is_some()); // From system
        assert!(config.logging.file_path.unwrap().to_string_lossy().contains("system_logs/app.log"));

        assert_eq!(config.feature_flags.experimental_feature_x, true); // User override

        assert_eq!(config.compositor.performance.quality_preset, "high"); // User override
        assert_eq!(config.compositor.performance.power_management_mode, "balanced"); // From system

        assert_eq!(config.compositor.visual.theme_name, "systemdefault"); // From system, normalized
        // Other fields should be their defaults if not specified in either file
        assert_eq!(config.compositor.input, defaults::default_input_config());
    }

    #[test]
    fn test_load_and_merge_configs_partial_override() {
        let system_config_filename = "system_partial.toml";
        let _test_env = TestEnv::new_with_custom_system_path(Some(system_config_filename));

        // Setup System Config (defines logging and visual)
        let system_config_path = get_system_config_path_with_override().unwrap();
        let system_config_dir = system_config_path.parent().unwrap();
        utils::fs::ensure_dir_exists(system_config_dir).unwrap();
        let system_toml_content = r#"
[logging]
level = "error"
format = "json"

[compositor.visual]
theme_name = "Monokai"
color_scheme = "dark"
        "#;
        create_temp_config_file(system_config_dir, system_config_filename, system_toml_content);

        // Setup User Config (defines only compositor.performance and a logging override)
        let user_app_config_dir = get_app_config_dir().unwrap();
        let user_toml_content = r#"
[logging]
level = "trace" # Override system logging level

[compositor.performance]
quality_preset = "low"
power_management_mode = "powersave"
        "#;
        create_temp_config_file(&user_app_config_dir, "config.toml", user_toml_content);

        let config = ConfigLoader::load().expect("ConfigLoader::load failed for partial merge");

        // Logging: level from user, format from system
        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.logging.format, "json");

        // Compositor.Visual: from system
        assert_eq!(config.compositor.visual.theme_name, "monokai");
        assert_eq!(config.compositor.visual.color_scheme, "dark");

        // Compositor.Performance: from user
        assert_eq!(config.compositor.performance.quality_preset, "low");
        assert_eq!(config.compositor.performance.power_management_mode, "powersave");

        // Compositor.Input: should be default as neither config specified it
        assert_eq!(config.compositor.input, defaults::default_input_config());

        // FeatureFlags: should be default
        assert_eq!(config.feature_flags, defaults::default_feature_flags());
    }

    #[test]
    fn test_parse_error_in_system_config() {
        let system_config_filename = "system_invalid.toml";
        let _test_env = TestEnv::new_with_custom_system_path(Some(system_config_filename));

        let system_config_path = get_system_config_path_with_override().unwrap();
        let system_config_dir = system_config_path.parent().unwrap();
        utils::fs::ensure_dir_exists(system_config_dir).unwrap();
        let invalid_toml_content = "this is not valid toml content";
        create_temp_config_file(system_config_dir, system_config_filename, invalid_toml_content);

        // User config can be valid or non-existent, error should still be ParseError from system
        let user_app_config_dir = get_app_config_dir().unwrap();
        let user_toml_content = r#"[logging]\nlevel="info""#; // Valid user config
        create_temp_config_file(&user_app_config_dir, "config.toml", user_toml_content);


        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ParseError(_)) => { /* Expected */ }
            e => panic!("Unexpected error type for system config parse error: {:?}", e),
        }
    }

    #[test]
    fn test_parse_error_in_user_config() {
        // System config is valid or non-existent
        let system_config_filename = "system_valid_for_user_error_test.toml";
        let _test_env = TestEnv::new_with_custom_system_path(Some(system_config_filename));

        let system_config_path = get_system_config_path_with_override().unwrap();
        let system_config_dir = system_config_path.parent().unwrap();
        utils::fs::ensure_dir_exists(system_config_dir).unwrap();
        let system_toml_content = r#"[logging]\nlevel="warn""#; // Valid system config
        create_temp_config_file(system_config_dir, system_config_filename, system_toml_content);

        // User config has a parse error
        let user_app_config_dir = get_app_config_dir().unwrap();
        let invalid_user_toml_content = "invalid_user_config_content = [";
        create_temp_config_file(&user_app_config_dir, "config.toml", invalid_user_toml_content);

        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ParseError(_)) => { /* Expected */ }
            e => panic!("Unexpected error type for user config parse error: {:?}", e),
        }
    }


    // This test is being removed as it's now covered by test_load_no_configs_uses_full_defaults
    // #[test]
    // fn test_config_loader_load_default_when_not_found() {
    //     // ...
    // }


    #[test]
        if config_file_path.exists() {
            fs::remove_file(&config_file_path).unwrap();
        }

        // Also ensure no default log file path exists from a previous test run, if defaults specify one.
        let default_log_path_opt = defaults::default_logging_config().file_path;
        if let Some(default_log_path_segment) = default_log_path_opt {
             let app_state_dir = get_app_state_dir().unwrap();
             let full_default_log_path = app_state_dir.join(default_log_path_segment);
             if full_default_log_path.exists() {
                 fs::remove_file(&full_default_log_path).unwrap();
             }
             if let Some(parent) = full_default_log_path.parent() {
                 if parent.exists() {
                    // We only want to remove the specific log file, not necessarily the whole logs dir
                    // if other tests might use it. But for a clean test of default creation,
                    // ensuring the parent does not pre-exist for the *default* path could be an option.
                    // However, validate_config is meant to *create* it.
                 }
             }
        }


        let result = ConfigLoader::load();
        assert!(result.is_ok(), "ConfigLoader::load failed when config file not found: {:?}", result.err());
        let config = result.unwrap();
        let default_core_config = CoreConfig::default();

        // Check it's the default config (including compositor defaults)
        assert_eq!(config.logging, default_core_config.logging);
        assert_eq!(config.feature_flags, default_core_config.feature_flags);
        assert_eq!(config.compositor.performance, default_core_config.compositor.performance);
        assert_eq!(config.compositor.input, default_core_config.compositor.input);
        assert_eq!(config.compositor.visual, default_core_config.compositor.visual);
        assert_eq!(config.compositor, default_core_config.compositor); // Overall compositor check
        assert_eq!(config, default_core_config); // Full check

        // Specific assertions for default compositor values after validation (some might be normalized)
        assert_eq!(config.compositor.performance.quality_preset, "medium"); // Default is "medium", already lowercase
        assert_eq!(config.compositor.performance.power_management_mode, "balanced"); // Default is "balanced", already lowercase
        assert_eq!(config.compositor.input.keyboard_layout, "us"); // Default is "us", already lowercase
        assert_eq!(config.compositor.visual.theme_name, "novadefault"); // Default is "NovaDefault", normalized
        assert_eq!(config.compositor.visual.color_scheme, "light"); // Default is "light", normalized
        assert_eq!(config.compositor.visual.font_settings, "system-ui"); // Default is "system-ui"

        // If the default config specifies a log file, ensure its parent directory was created by validate_config
        if let Some(ref log_path) = config.logging.file_path {
            assert!(log_path.is_absolute(), "Default log path should be absolute after validation");
            if let Some(parent_dir) = log_path.parent() {
                assert!(parent_dir.exists(), "Parent directory for default log path was not created");
            }
        }
    }


    #[test]
    fn test_config_loader_load_parse_error() {
        let _test_env = TestEnv::new();
        let app_config_dir = get_app_config_dir().unwrap();
        let invalid_toml_content = "this is not valid toml content";
        create_temp_config_file(&app_config_dir, "config.toml", invalid_toml_content);

        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ParseError(_)) => { /* Expected */ }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_config_loader_load_io_error_other_than_not_found() {
        let _test_env = TestEnv::new();
        let app_config_dir = get_app_config_dir().unwrap();
        let config_file_path = app_config_dir.join("config.toml");

        // Create the file, but then make it unreadable (e.g. by making its parent dir unreadable)
        // This is hard to do reliably cross-platform in tests.
        // A simpler IO error to simulate might be if the path is a directory.
        nova_fs::ensure_dir_exists(&config_file_path).unwrap(); // Create config.toml as a directory

        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ReadError { path, source: _ }) => {
                 assert_eq!(path, config_file_path);
            }
            // This might also be CoreError::Io if the read_to_string itself fails due to "is a directory"
            // depending on OS and fs::read_to_string behavior.
            // For now, ReadError is the expectation from the original tests.
            e => panic!("Unexpected error type for ReadError: {:?}", e),
        }
        // Clean up the directory we created
        fs::remove_dir_all(&config_file_path).ok();
    }


    #[test]
    fn test_validate_config_valid_settings() {
        let _test_env = TestEnv::new(); // For state dir resolution
        let mut config = CoreConfig {
            logging: defaults::default_logging_config(),
            feature_flags: defaults::default_feature_flags(),
            compositor: defaults::default_compositor_config(), // Add default compositor config
        };
        config.logging.level = "TRACE".to_string(); // Test case normalization
        config.logging.format = "JSON".to_string(); // Test case normalization
        config.logging.file_path = Some(PathBuf::from("my_app/log.txt")); // Relative path

        // Test normalization for compositor fields
        config.compositor.performance.quality_preset = "HIGH".to_string();
        config.compositor.performance.power_management_mode = "PowerSave".to_string();
        config.compositor.input.keyboard_layout = "FR".to_string();
        config.compositor.visual.theme_name = "MyTheme".to_string();
        config.compositor.visual.color_scheme = "MyScheme".to_string();


        ConfigLoader::validate_config(&mut config).expect("Validation failed for valid settings");

        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.logging.format, "json");
        let log_path = config.logging.file_path.unwrap();
        assert!(log_path.is_absolute());
        assert!(log_path.to_string_lossy().ends_with("my_app/log.txt"));
        assert!(log_path.parent().unwrap().exists());

        // Assert compositor normalizations
        assert_eq!(config.compositor.performance.quality_preset, "high");
        assert_eq!(config.compositor.performance.power_management_mode, "powersave");
        assert_eq!(config.compositor.input.keyboard_layout, "fr");
        assert_eq!(config.compositor.visual.theme_name, "mytheme");
        assert_eq!(config.compositor.visual.color_scheme, "myscheme");
    }

    #[test]
    fn test_validate_config_invalid_log_level() { // Replaced with version from mod.rs
        let mut config = CoreConfig::default();
        config.logging.level = "superlog".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log level: 'superlog'"));
        }
    }

    #[test]
    fn test_validate_config_invalid_log_format() { // Replaced with version from mod.rs
        let mut config = CoreConfig::default();
        config.logging.format = "binary".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log format: 'binary'"));
        }
    }

    #[test]
    fn test_validate_config_absolute_log_path() { // Added from mod.rs
        let _test_env = TestEnv::new();
        let temp_dir_for_log = TempDir::new().unwrap();
        let abs_log_path = temp_dir_for_log.path().join("sub/absolute.log");

        let mut config = CoreConfig::default();
        config.logging.file_path = Some(abs_log_path.clone());

        ConfigLoader::validate_config(&mut config).expect("Validation failed for absolute path");
        
        assert_eq!(config.logging.file_path.unwrap(), abs_log_path);
        assert!(abs_log_path.parent().unwrap().exists());
    }
    
    #[test]
    fn test_validate_config_log_path_is_root_parent() { // Added from mod.rs
        let _test_env = TestEnv::new();
        let mut config = CoreConfig::default();
        
        let log_file_name_only = PathBuf::from("logfile.log");

        config.logging.file_path = Some(log_file_name_only.clone());
        let result = ConfigLoader::validate_config(&mut config);
        assert!(result.is_ok(), "Validation failed for log file in state_dir root: {:?}", result.err());

        let app_state_dir = get_app_state_dir().unwrap();
        let expected_abs_path = app_state_dir.join(log_file_name_only);
        assert_eq!(config.logging.file_path, Some(expected_abs_path.clone()));
        assert!(expected_abs_path.parent().unwrap().exists());
    }

    // Note: test_validate_config_log_file_path_relative from original loader.rs tests
    // is covered by test_validate_config_valid_settings and test_config_loader_load_success,
    // which use relative paths initially and TestEnv for state dir.

    // --- New tests for CompositorConfig validation failures ---

    #[test]
    fn test_validate_compositor_config_invalid_quality_preset() {
        let mut config = CoreConfig::default();
        config.compositor.performance.quality_preset = "ultra-high".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid quality_preset: 'ultra-high'"));
        }
    }

    #[test]
    fn test_validate_compositor_config_invalid_power_mode() {
        let mut config = CoreConfig::default();
        config.compositor.performance.power_management_mode = "turbo".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid power_management_mode: 'turbo'"));
        }
    }

    #[test]
    fn test_validate_compositor_visual_config_empty_font_name() {
        let mut config = CoreConfig::default();
        config.compositor.visual.font_settings = " ".to_string(); // Empty after trim
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("VisualConfig.font_settings must not be empty."));
        }
    }
}
