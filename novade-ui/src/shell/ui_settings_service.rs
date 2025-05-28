use novade_domain::settings::ui_provider::{UISettingsProvider, SettingsError}; // Adjusted path
use std::sync::Arc;
use gtk::glib; // For GObject signals if we were to make UISettingsService a GObject
use tokio::runtime::Handle; // For tokio_handle
use tracing;

// For this iteration, UISettingsService is a simple struct.
// If it were a GObject, it would have GObject boilerplate and signals.
pub struct UISettingsService {
    provider: Arc<dyn UISettingsProvider>,
    tokio_handle: Handle,
    // If UISettingsService were to emit GObject signals, they'd be defined here.
    // For example, using async-broadcaster or similar mechanisms bridged to GObject signals.
    // For now, UI components will fetch initial state and then send updates.
    // Reactivity from domain changes back to UI components (other than the one making the change)
    // would typically involve the UI component subscribing to domain events (via this service)
    // or this service emitting GObject signals.
}

impl UISettingsService {
    pub fn new(provider: Arc<dyn UISettingsProvider>, tokio_handle: Handle) -> Self {
        // In a more complex scenario, this service might immediately subscribe to
        // provider.subscribe_to_changes() and then emit its own GObject signals
        // or Rust-level events for broader UI consumption.
        Self { provider, tokio_handle }
    }

    // Methods for specific settings
    pub async fn get_dark_mode(&self) -> bool {
        match self.provider.get_bool_setting("ui.dark_mode").await {
            Ok(val) => val,
            Err(e) => {
                tracing::warn!("Failed to get dark_mode setting, defaulting to false: {:?}", e);
                false // Default value
            }
        }
    }

    pub async fn set_dark_mode(&self, active: bool) {
        tracing::debug!("UISettingsService: Setting dark_mode to {}", active);
        if let Err(e) = self.provider.set_bool_setting("ui.dark_mode", active).await {
            tracing::error!("UISettingsService: Failed to set dark_mode: {:?}", e);
        }
    }

    pub async fn get_volume(&self) -> f64 {
        match self.provider.get_f64_setting("audio.volume").await {
            Ok(val) => val,
            Err(e) => {
                tracing::warn!("Failed to get volume setting, defaulting to 50.0: {:?}", e);
                50.0 // Default value
            }
        }
    }

    pub async fn set_volume(&self, value: f64) {
        let clamped_value = value.clamp(0.0, 100.0); // Ensure value is within typical range
        tracing::debug!("UISettingsService: Setting volume to {}", clamped_value);
        if let Err(e) = self.provider.set_f64_setting("audio.volume", clamped_value).await {
            tracing::error!("UISettingsService: Failed to set volume: {:?}", e);
        }
    }
    
    // Expose the Tokio handle for spawning tasks from UI components that use this service
    pub fn tokio_handle(&self) -> &Handle {
        &self.tokio_handle
    }
}

// To make UISettingsService usable with Rc (as planned for UI components)
// it doesn't need to be Clone itself if it's wrapped in Rc/Arc.
// If UISettingsService were a GObject, it would be inherently reference-counted.
