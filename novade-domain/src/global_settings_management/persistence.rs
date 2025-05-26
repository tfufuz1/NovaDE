use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn; // For logging warnings

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::GlobalDesktopSettings;
use super::errors::GlobalSettingsError;
use super::persistence_iface::SettingsPersistenceProvider;
// SettingPath is not directly used here for root errors, but good to have if specific field errors were to be mapped.
// use super::paths::SettingPath; 

/// A persistence provider that loads and saves global desktop settings
/// from/to the filesystem using a TOML format.
#[derive(Debug)]
pub struct FilesystemSettingsProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    config_file_path: PathBuf,
}

impl FilesystemSettingsProvider {
    /// Creates a new `FilesystemSettingsProvider`.
    ///
    /// # Arguments
    /// * `config_service` - An `Arc` to a `ConfigServiceAsync` implementation.
    /// * `config_file_path` - The `PathBuf` to the configuration file.
    pub fn new(
        config_service: Arc<dyn ConfigServiceAsync>,
        config_file_path: PathBuf,
    ) -> Self {
        Self {
            config_service,
            config_file_path,
        }
    }
}

#[async_trait]
impl SettingsPersistenceProvider for FilesystemSettingsProvider {
    async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError> {
        match self.config_service.load_config_file_content_async(self.config_file_path.to_str().ok_or_else(|| GlobalSettingsError::InternalError("Konfigurationspfad enthält ungültige UTF-8 Zeichen.".to_string()))?).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    GlobalSettingsError::DeserializationError {
                        // path: None, // Path is not relevant for a full deserialization error of the file
                        source_message: e.to_string(),
                    }
                })
            }
            Err(core_error) => {
                // Attempt to check if the error is a "Not Found" type.
                // This depends on the structure of CoreError.
                // Assuming CoreError::NotFound(path) exists or similar.
                // Or, if CoreError wraps std::io::Error, check io_error.kind().
                // For this example, let's assume a hypothetical is_not_found method or variant match.
                // If CoreError is an enum: `if matches!(core_error, CoreError::NotFound(_)) { ... }`
                // If CoreError has a kind(): `if core_error.kind() == SomeErrorKind::NotFound { ... }`
                // For now, a simple string check on the error description if nothing more specific.
                // This part is highly dependent on `novade_core::errors::CoreError` definition.
                let is_not_found_error = match &core_error {
                    CoreError::NotFound(_) => true,
                    // Example if CoreError wraps std::io::Error:
                    // CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => true,
                    _ => false, // Default: assume not a "not found" error
                };

                if is_not_found_error {
                    warn!(
                        "Einstellungsdatei unter {:?} nicht gefunden. Standardeinstellungen werden verwendet.",
                        self.config_file_path
                    );
                    Ok(GlobalDesktopSettings::default())
                } else {
                    Err(GlobalSettingsError::persistence_error_with_core_source(
                        "load",
                        format!("Fehler beim Lesen der Einstellungen von {:?}", self.config_file_path),
                        core_error,
                    ))
                }
            }
        }
    }

    async fn save_global_settings(
        &self,
        settings: &GlobalDesktopSettings,
    ) -> Result<(), GlobalSettingsError> {
        let serialized_content = toml::to_string_pretty(settings).map_err(|e| {
            // For a global serialization error, a specific path isn't always relevant.
            // We might need a new variant in GlobalSettingsError or use a placeholder.
            // Using a descriptive string for now.
            GlobalSettingsError::SerializationError{
                path_description: None, // Error is for the entire settings object
                source_message: e.to_string()
            }
            // A more general variant might be:
            // GlobalSettingsError::SerializationError {
            //     context: "GlobalDesktopSettings".to_string(), // Or some other context
            //     source_message: e.to_string(),
            // }
        })?;

        self.config_service
            .save_config_file_content_async(self.config_file_path.to_str().ok_or_else(|| GlobalSettingsError::InternalError("Konfigurationspfad enthält ungültige UTF-8 Zeichen.".to_string()))?, &serialized_content)
            .await
            .map_err(|core_error| {
                GlobalSettingsError::persistence_error_with_core_source(
                    "save",
                    format!("Fehler beim Schreiben der Einstellungen nach {:?}", self.config_file_path),
                    core_error,
                )
            })
    }
}
