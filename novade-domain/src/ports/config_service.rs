use async_trait::async_trait;
use novade_core::CoreError; // Corrected path, assumes CoreError is pub use'd from novade_core
use std::path::{Path, PathBuf};

/// Trait for a service that can read and write configuration strings by a key or path.
/// This is an abstraction expected by some domain services (like FilesystemConfigProvider
/// for workspaces) to interact with the underlying configuration persistence mechanism.
#[async_trait]
pub trait ConfigServiceAsync: Send + Sync {
    /// Reads a configuration file identified by a key (e.g., "workspaces.toml")
    /// and returns its content as a string.
    /// The implementation would typically resolve this key to a full path.
    async fn read_config_file_string(&self, key: &str) -> Result<String, CoreError>;

    /// Writes the given content string to a configuration file identified by a key.
    async fn write_config_file_string(&self, key: &str, content: String) -> Result<(), CoreError>;
    
    /// Reads an arbitrary file to a string given its full path.
    async fn read_file_to_string(&self, path: &Path) -> Result<String, CoreError>;

    /// Lists files in a directory, optionally filtering by extension.
    async fn list_files_in_dir(&self, dir_path: &Path, extension: Option<&str>) -> Result<Vec<PathBuf>, CoreError>;

    /// Gets the application-specific configuration directory.
    async fn get_config_dir(&self) -> Result<PathBuf, CoreError>; // Changed to Result

    /// Gets the application-specific data directory.
    async fn get_data_dir(&self) -> Result<PathBuf, CoreError>; // Changed to Result
}
