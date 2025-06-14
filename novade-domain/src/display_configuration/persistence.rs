// novade-domain/src/display_configuration/persistence.rs
use async_trait::async_trait;
use novade_core::types::display::DisplayConfiguration;
use crate::display_configuration::errors::{Result, DisplayConfigurationError};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
pub trait DisplayPersistence: Send + Sync {
    async fn save_config(&self, config: &DisplayConfiguration) -> Result<()>;
    async fn load_config(&self) -> Result<DisplayConfiguration>;
}

pub struct FileSystemDisplayPersistence {
    config_path: PathBuf,
}

impl FileSystemDisplayPersistence {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    fn ensure_config_dir_exists(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DisplayConfigurationError::IoError(e))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl DisplayPersistence for FileSystemDisplayPersistence {
    async fn save_config(&self, config: &DisplayConfiguration) -> Result<()> {
        self.ensure_config_dir_exists()?;
        let serialized_config = serde_json::to_string_pretty(config)
            .map_err(|e| DisplayConfigurationError::SerdeError(e.to_string()))?;

        let mut file = fs::File::create(&self.config_path).await
            .map_err(|e| DisplayConfigurationError::IoError(e))?;
        file.write_all(serialized_config.as_bytes()).await
            .map_err(|e| DisplayConfigurationError::IoError(e))?;
        Ok(())
    }

    async fn load_config(&self) -> Result<DisplayConfiguration> {
        if !self.config_path.exists() {
            // Return a default configuration or a specific error
            return Err(DisplayConfigurationError::Persistence("Config file not found".to_string()));
        }
        let mut file = fs::File::open(&self.config_path).await
            .map_err(|e| DisplayConfigurationError::IoError(e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await
            .map_err(|e| DisplayConfigurationError::IoError(e))?;

        serde_json::from_str(&contents)
            .map_err(|e| DisplayConfigurationError::SerdeError(e.to_string()))
    }
}
