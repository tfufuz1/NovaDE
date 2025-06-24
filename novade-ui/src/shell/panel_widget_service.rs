//! Service for managing and providing PanelWidgets.
//!
//! This service allows for the registration of `PanelWidgetProvider` instances
//! and the creation of `PanelWidget` instances. It acts as a central registry
//! for all available panel widgets in the NovaDE shell.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use log::{debug, error, info, warn};
use gtk::glib;

use crate::plugin_api::{PanelWidget, PanelWidgetProvider, PluginError};

/// A unique identifier for a registered PanelWidgetProvider.
/// Can be the plugin ID or another unique name.
type ProviderId = String;

/// A unique identifier for a specific widget type offered by a provider.
type WidgetTypeId = String;

#[derive(Default)]
struct PanelWidgetServiceState {
    /// Stores registered panel widget providers.
    /// The key is a unique ID for the provider (e.g., plugin ID).
    providers: HashMap<ProviderId, Box<dyn PanelWidgetProvider>>,
    // TODO: Potentially cache available widget types for faster lookups.
    // available_widgets: HashMap<WidgetTypeId, ProviderId>,
}

/// Service responsible for managing panel widget providers and creating widget instances.
pub struct PanelWidgetService {
    state: Mutex<PanelWidgetServiceState>,
}

impl PanelWidgetService {
    /// Creates a new `PanelWidgetService`.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(PanelWidgetServiceState::default()),
        })
    }

    /// Registers a `PanelWidgetProvider`.
    ///
    /// # Arguments
    ///
    /// * `provider_id`: A unique identifier for this provider (e.g., the ID of the plugin providing it).
    /// * `provider`: The `PanelWidgetProvider` instance.
    ///
    /// # Returns
    ///
    /// `Ok(())` if registration is successful, or `Err(PluginError)` if a provider with the same ID already exists.
    pub fn register_provider(&self, provider_id: ProviderId, provider: Box<dyn PanelWidgetProvider>) -> Result<(), PluginError> {
        let mut state = self.state.lock().unwrap(); // Handle potential Mutex poisoning

        if state.providers.contains_key(&provider_id) {
            warn!("PanelWidgetProvider with ID '{}' already registered. Ignoring new registration.", provider_id);
            return Err(PluginError::ConfigurationError(format!(
                "Provider ID '{}' already exists.",
                provider_id
            )));
        }

        info!("Registering PanelWidgetProvider: {}", provider_id);
        for widget_type in provider.get_available_widget_types() {
            debug!("Provider '{}' offers panel widget type: {}", provider_id, widget_type);
        }

        state.providers.insert(provider_id, provider);
        Ok(())
    }

    /// Unregisters a `PanelWidgetProvider`.
    ///
    /// # Arguments
    ///
    /// * `provider_id`: The ID of the provider to unregister.
    ///
    /// # Returns
    ///
    /// `Ok(Option<Box<dyn PanelWidgetProvider>>)` with the unregistered provider if found, or `Ok(None)` if not.
    pub fn unregister_provider(&self, provider_id: &ProviderId) -> Result<Option<Box<dyn PanelWidgetProvider>>, PluginError> {
        let mut state = self.state.lock().unwrap();
        let provider = state.providers.remove(provider_id);

        if provider.is_some() {
            info!("Unregistered PanelWidgetProvider: {}", provider_id);
        } else {
            warn!("Attempted to unregister non-existent PanelWidgetProvider: {}", provider_id);
        }
        Ok(provider)
    }

    /// Lists all available panel widget types from all registered providers.
    ///
    /// Each string in the returned vector is a unique `WidgetTypeId`.
    ///
    /// # Returns
    ///
    /// A vector of strings, where each string is a unique `WidgetTypeId`.
    /// The format could be "provider_id/widget_type_id_from_provider" to ensure uniqueness,
    /// or the service can ensure global uniqueness of widget_type_ids.
    /// For now, let's assume widget_type_ids returned by providers are unique enough,
    /// or we select the first provider that offers a given type.
    /// A better approach is to return a list of (ProviderId, WidgetTypeId) or a structured type.
    pub fn get_available_widget_types(&self) -> Vec<WidgetTypeId> {
        let state = self.state.lock().unwrap();
        let mut all_types = Vec::new();
        for (_provider_id, provider) in &state.providers {
            all_types.extend(provider.get_available_widget_types());
        }
        // TODO: Ensure uniqueness if providers might return overlapping type IDs.
        // For now, assumes provider-returned IDs are globally unique or context implies provider.
        all_types.sort();
        all_types.dedup();
        all_types
    }

    /// Creates an instance of a `PanelWidget`.
    ///
    /// # Arguments
    ///
    /// * `widget_type_id`: The unique identifier of the widget type to create.
    /// * `config`: Optional configuration for the widget.
    ///
    /// # Returns
    ///
    /// `Ok(Box<dyn PanelWidget>)` if successful, or `Err(PluginError)` if the widget type is not found
    /// or creation fails.
    ///
    /// This method iterates through providers to find one that can create the widget.
    /// A more optimized version might cache which provider offers which widget type.
    pub fn create_widget(
        &self,
        widget_type_id: &WidgetTypeId,
        config: Option<&glib::Variant>,
    ) -> Result<Box<dyn PanelWidget>, PluginError> {
        let state = self.state.lock().unwrap();

        for (provider_id, provider) in &state.providers {
            // Check if this provider offers this widget_type_id
            if provider.get_available_widget_types().iter().any(|id| id == widget_type_id) {
                debug!("Attempting to create panel widget '{}' using provider '{}'", widget_type_id, provider_id);
                return match provider.create_widget(widget_type_id, config) {
                    Ok(widget_instance) => {
                        info!("Successfully created panel widget '{}' from provider '{}'", widget_type_id, provider_id);
                        Ok(widget_instance)
                    }
                    Err(e) => {
                        error!("Provider '{}' failed to create panel widget '{}': {}", provider_id, widget_type_id, e);
                        Err(e)
                    }
                };
            }
        }

        warn!("No provider found for panel widget type: {}", widget_type_id);
        Err(PluginError::NotImplemented) // Or a more specific "WidgetTypeNotFound" error
    }

    /// Lists all registered provider IDs.
    pub fn list_provider_ids(&self) -> Vec<ProviderId> {
        self.state.lock().unwrap().providers.keys().cloned().collect()
    }
}

// TODO: Add unit tests for PanelWidgetService
// - Test registration and unregistration of providers.
// - Test listing available widget types (empty, single provider, multiple providers).
// - Test creating a widget (success, type not found, provider creation error).
// - Test for provider ID conflicts during registration.
// - Test thread safety (though direct testing of Mutex can be tricky without race conditions).

impl Default for PanelWidgetService {
    fn default() -> Self {
        Arc::try_unwrap(Self::new()).unwrap_or_else(|_| {
            // This case should ideally not happen if Arc::new() is the primary constructor.
            // If it does, it means there's an unexpected Arc clone somewhere.
            // For safety in a default() context, we create a new one, but it's a code smell.
            warn!("PanelWidgetService::default() called on an existing Arc; creating a new instance.");
            Self {
                state: Mutex::new(PanelWidgetServiceState::default()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_api::{PanelWidget, PluginError, Widget}; // Assuming Widget is gtk::Widget
    use gtk::prelude::*; // Required for gtk::Widget methods if any are called directly

    // Mock PanelWidgetProvider
    struct MockPanelWidgetProvider {
        widget_types: Vec<WidgetTypeId>,
        creation_should_fail: bool,
    }

    impl PanelWidgetProvider for MockPanelWidgetProvider {
        fn get_available_widget_types(&self) -> Vec<WidgetTypeId> {
            self.widget_types.clone()
        }

        fn create_widget(
            &self,
            widget_type_id: &str,
            _config: Option<&glib::Variant>,
        ) -> Result<Box<dyn PanelWidget>, PluginError> {
            if self.creation_should_fail {
                return Err(PluginError::OperationFailed("Mock creation failure".to_string()));
            }
            if self.widget_types.contains(&widget_type_id.to_string()) {
                Ok(Box::new(MockPanelWidget {
                    widget_type: widget_type_id.to_string(),
                }))
            } else {
                Err(PluginError::NotImplemented)
            }
        }
    }

    // Mock PanelWidget
    struct MockPanelWidget {
        widget_type: WidgetTypeId,
    }
    impl PanelWidget for MockPanelWidget {
        fn get_widget(&mut self) -> Result<Widget, PluginError> {
            // In a real test, you might want to return an actual gtk::Label or similar.
            // For trait testing, returning Ok with a dummy widget or error is fine.
            // For now, let's assume we can't easily create GTK widgets in this test context without a GTK main loop.
            // So, we'll return an error to signify it's not the focus of this unit test.
            // Or, if Widget is just a struct, we can create it.
            // Let's assume gtk::Label for now and that tests might need `gtk::init()`.
            // For simplicity here, we'll avoid actual GTK widget creation.
            // This part of the test would be better as an integration test if real widgets are needed.
            if gtk::is_initialized() {
                 Ok(gtk::Label::new(Some(&format!("Mock Widget: {}", self.widget_type))).upcast::<Widget>())
            } else {
                // Cannot create GTK widgets if GTK is not initialized.
                // This indicates tests needing GTK widgets should be integration tests or use a test harness.
                Err(PluginError::WidgetRetrievalFailed)
            }
        }
        fn update_data(&mut self, _data: Option<&glib::Variant>) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn ensure_gtk_init() {
        if !gtk::is_initialized() {
            gtk::init().expect("Failed to initialize GTK for tests");
        }
    }


    #[test]
    fn test_register_and_unregister_provider() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();
        let provider_id = "mock_provider_1".to_string();
        let provider = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["widget_A".to_string()],
            creation_should_fail: false,
        });

        assert!(service.register_provider(provider_id.clone(), provider).is_ok());
        assert_eq!(service.list_provider_ids(), vec![provider_id.clone()]);

        let unregistered_provider = service.unregister_provider(&provider_id).unwrap();
        assert!(unregistered_provider.is_some());
        assert!(service.list_provider_ids().is_empty());

        let nonexistent_provider = service.unregister_provider(&"nonexistent".to_string()).unwrap();
        assert!(nonexistent_provider.is_none());
    }

    #[test]
    fn test_register_provider_duplicate_id() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();
        let provider_id = "duplicate_provider".to_string();
        let provider1 = Box::new(MockPanelWidgetProvider { widget_types: vec![], creation_should_fail: false });
        let provider2 = Box::new(MockPanelWidgetProvider { widget_types: vec![], creation_should_fail: false });

        assert!(service.register_provider(provider_id.clone(), provider1).is_ok());
        let result = service.register_provider(provider_id.clone(), provider2);
        assert!(result.is_err());
        match result.err().unwrap() {
            PluginError::ConfigurationError(msg) => {
                assert!(msg.contains("already exists"));
            }
            _ => panic!("Expected ConfigurationError for duplicate provider ID"),
        }
    }

    #[test]
    fn test_get_available_widget_types() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();

        // No providers
        assert!(service.get_available_widget_types().is_empty());

        // One provider
        let provider1 = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["type_A".to_string(), "type_B".to_string()],
            creation_should_fail: false,
        });
        service.register_provider("provider1".to_string(), provider1).unwrap();
        let mut types = service.get_available_widget_types();
        types.sort(); // Ensure order for comparison
        assert_eq!(types, vec!["type_A".to_string(), "type_B".to_string()]);

        // Multiple providers with overlapping and unique types
        let provider2 = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["type_B".to_string(), "type_C".to_string()],
            creation_should_fail: false,
        });
        service.register_provider("provider2".to_string(), provider2).unwrap();

        let mut all_types = service.get_available_widget_types();
        all_types.sort(); // sort due to HashMap iteration order for providers then .dedup()
        // Expected: type_A, type_B, type_C (deduplicated)
        assert_eq!(all_types, vec!["type_A".to_string(), "type_B".to_string(), "type_C".to_string()]);
    }

    #[test]
    fn test_create_widget_success() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();
        let provider = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["clock_widget".to_string()],
            creation_should_fail: false,
        });
        service.register_provider("time_provider".to_string(), provider).unwrap();

        let widget_result = service.create_widget(&"clock_widget".to_string(), None);
        assert!(widget_result.is_ok());
        let mut panel_widget = widget_result.unwrap();
        // Test that get_widget can be called (even if it returns error in mock due to no GTK)
        let gtk_widget_result = panel_widget.get_widget();
        if gtk::is_initialized() { // If GTK is up, we expect Ok.
            assert!(gtk_widget_result.is_ok());
            assert_eq!(gtk_widget_result.unwrap().widget_name(), "GtkLabel");
        } else { // Otherwise, we expect the specific error from our mock.
            assert!(matches!(gtk_widget_result.err().unwrap(), PluginError::WidgetRetrievalFailed));
        }
    }

    #[test]
    fn test_create_widget_type_not_found() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();
        // No providers registered, or provider doesn't have the type
        let provider = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["known_widget".to_string()],
            creation_should_fail: false,
        });
        service.register_provider("provider_empty".to_string(), provider).unwrap();

        let widget_result = service.create_widget(&"unknown_widget_type".to_string(), None);
        assert!(widget_result.is_err());
        match widget_result.err().unwrap() {
            PluginError::NotImplemented => {} // Correct error
            e => panic!("Expected NotImplemented error, got {:?}", e),
        }
    }

    #[test]
    fn test_create_widget_provider_creation_fails() {
        ensure_gtk_init();
        let service = PanelWidgetService::new();
        let provider = Box::new(MockPanelWidgetProvider {
            widget_types: vec!["failing_widget".to_string()],
            creation_should_fail: true, // This provider's create_widget will fail
        });
        service.register_provider("failing_provider".to_string(), provider).unwrap();

        let widget_result = service.create_widget(&"failing_widget".to_string(), None);
        assert!(widget_result.is_err());
        match widget_result.err().unwrap() {
            PluginError::OperationFailed(msg) => {
                assert_eq!(msg, "Mock creation failure");
            }
            e => panic!("Expected OperationFailed error, got {:?}", e),
        }
    }
}
