use std::sync::Arc;
use novade_system::dbus_menu_provider::{DBusMenuProvider, DBusMenuError}; // Ensure correct path
use gtk4_gio::gio; // Use gtk4_gio for gio::MenuModel
use tracing;

pub struct AppMenuService {
    provider: Arc<dyn DBusMenuProvider>,
}

impl AppMenuService {
    pub fn new(provider: Arc<dyn DBusMenuProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_menu_for_app(&self, app_id: Option<String>) -> Option<gio::MenuModel> {
        match app_id {
            Some(id_str) => {
                tracing::debug!("AppMenuService: Fetching menu for app_id: {}", id_str);
                match self.provider.fetch_menu_model(&id_str).await {
                    Ok(Some(menu_model)) => {
                        tracing::debug!("AppMenuService: Menu model found for app_id: {}", id_str);
                        Some(menu_model)
                    }
                    Ok(None) => {
                        tracing::debug!("AppMenuService: No menu model found by provider for app_id: {}", id_str);
                        None
                    }
                    Err(e) => {
                        tracing::warn!("AppMenuService: Error fetching menu model for app_id {}: {:?}", id_str, e);
                        None
                    }
                }
            }
            None => {
                tracing::debug!("AppMenuService: No app_id provided, returning no menu.");
                None
            }
        }
    }
}
