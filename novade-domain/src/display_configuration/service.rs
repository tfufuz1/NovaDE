// novade-domain/src/display_configuration/service.rs
use async_trait::async_trait;
use novade_core::types::display::{Display, DisplayConfiguration, DisplayLayout};
use crate::display_configuration::errors::{Result, DisplayConfigurationError};
use crate::display_configuration::persistence::DisplayPersistence;
use std::sync::Arc;

#[async_trait]
pub trait DisplayConfigService: Send + Sync {
    /// Retrieves the current full display configuration.
    async fn get_display_configuration(&self) -> Result<DisplayConfiguration>;

    /// Applies a new display configuration.
    /// This might involve validating the configuration and instructing the system layer.
    async fn apply_display_configuration(&self, config: &DisplayConfiguration) -> Result<()>;

    /// Updates the configuration for a single display.
    async fn update_single_display_config(&self, display_id: &str, config: &Display) -> Result<()>;

    /// Sets the display layout (e.g., Mirrored, Extended).
    async fn set_layout(&self, layout: DisplayLayout) -> Result<()>;

    /// Persists the current configuration.
    async fn save_configuration(&self) -> Result<()>;

    /// Loads the configuration from persistence.
    async fn load_configuration(&self) -> Result<DisplayConfiguration>;
}

pub struct DefaultDisplayConfigService {
    persistence: Arc<dyn DisplayPersistence>,
    current_config: tokio::sync::RwLock<DisplayConfiguration>,
    // May need a way to communicate with novade-system's DisplayManager
    // This could be a trait object or a specific type passed during construction.
    // For now, we'll focus on domain logic.
}

impl DefaultDisplayConfigService {
    pub async fn new(persistence: Arc<dyn DisplayPersistence>) -> Result<Self> {
        let loaded_config = persistence.load_config().await.unwrap_or_else(|_| {
            // Default configuration if loading fails
            DisplayConfiguration {
                displays: vec![],
                layout: DisplayLayout::Single,
            }
        });
        Ok(Self {
            persistence,
            current_config: tokio::sync::RwLock::new(loaded_config),
        })
    }
}

#[async_trait]
impl DisplayConfigService for DefaultDisplayConfigService {
    async fn get_display_configuration(&self) -> Result<DisplayConfiguration> {
        Ok(self.current_config.read().await.clone())
    }

    async fn apply_display_configuration(&self, config: &DisplayConfiguration) -> Result<()> {
        // 1. Validate configuration (e.g., no overlapping displays in extended mode, valid modes)
        // Placeholder for validation logic
        if config.displays.is_empty() && config.layout != DisplayLayout::Single {
             //return Err(DisplayConfigurationError::Validation("Cannot have non-single layout with no displays".to_string()));
        }
        // More validation needed here...

        // 2. Update internal state
        let mut current_config_lock = self.current_config.write().await;
        *current_config_lock = config.clone();

        // 3. TODO: Instruct novade-system layer to apply these settings.
        // This is a critical part that will be expanded later.
        // For now, we assume this service is called *after* system layer has new info,
        // or this service will *trigger* system layer changes.

        // 4. Persist changes (optional, could be explicit save)
        // self.persistence.save_config(config).await?;
        Ok(())
    }

    async fn update_single_display_config(&self, display_id: &str, updated_display_config: &Display) -> Result<()> {
        let mut current_config_lock = self.current_config.write().await;
        if let Some(display) = current_config_lock.displays.iter_mut().find(|d| d.id == display_id) {
            *display = updated_display_config.clone();
            // TODO: Trigger system layer update for this single display
            // self.persistence.save_config(&*current_config_lock).await?;
            Ok(())
        } else {
            Err(DisplayConfigurationError::DisplayNotFound(display_id.to_string()))
        }
    }

    async fn set_layout(&self, layout: DisplayLayout) -> Result<()> {
        let mut current_config_lock = self.current_config.write().await;
        current_config_lock.layout = layout;
        // TODO: Trigger system layer update for layout change
        // self.persistence.save_config(&*current_config_lock).await?;
        Ok(())
    }

    async fn save_configuration(&self) -> Result<()> {
        let config = self.current_config.read().await;
        self.persistence.save_config(&config).await
    }

    async fn load_configuration(&self) -> Result<DisplayConfiguration> {
        let config = self.persistence.load_config().await?;
        let mut current_config_lock = self.current_config.write().await;
        *current_config_lock = config.clone();
        Ok(config)
    }
}
