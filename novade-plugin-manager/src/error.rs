//! Error types for the NovaDE Plugin Manager.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("Plugin discovery failed: {0}")]
    DiscoveryError(String),

    #[error("Manifest parsing error in '{path}': {source}")]
    ManifestParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("I/O error related to manifest file '{path}': {source}")]
    ManifestIoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Plugin loading failed for '{plugin_id}': {reason}")]
    LoadingError { plugin_id: String, reason: String },

    #[error("Plugin initialization failed for '{plugin_id}': {reason}")]
    InitializationError { plugin_id: String, reason: String },

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Incompatible plugin ABI or version for '{plugin_id}'. Expected {expected}, found {found}")]
    IncompatibleAbiVersion {
        plugin_id: String,
        expected: String,
        found: String,
    },

    #[error("Plugin entry point symbol '{symbol_name}' not found in library '{library_path}' for plugin '{plugin_id}'")]
    SymbolNotFound {
        plugin_id: String,
        library_path: PathBuf,
        symbol_name: String,
    },

    #[error("Attempted to operate on a plugin '{plugin_id}' that is not in the correct state. Current state: {state}")]
    InvalidPluginState { plugin_id: String, state: String },

    #[error("Configuration error for plugin '{plugin_id}': {message}")]
    ConfigurationError { plugin_id: String, message: String },

    #[error("An internal plugin manager error occurred: {0}")]
    InternalError(String),
}

// Allows easy conversion from libloading::Error if that feature is enabled.
// Example:
// #[cfg(feature = "dynamic_loading")]
// impl From<libloading::Error> for PluginManagerError {
//     fn from(err: libloading::Error) -> Self {
//         PluginManagerError::LoadingError {
//             plugin_id: "unknown".to_string(), // Might need context to fill this properly
//             reason: err.to_string(),
//         }
//     }
// }
