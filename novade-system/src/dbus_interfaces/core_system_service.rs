// ANCHOR: Header
//! # NovaDE Core System D-Bus Service Implementation
//!
//! This module provides the concrete implementation for the `CoreSystemInterface`.
//! It manages component registration, status tracking, and emits D-Bus signals
//! for component lifecycle events.

// ANCHOR: Crates
use crate::dbus_interfaces::core_system_interface::{
    ComponentInfo, ComponentStatus, CoreSystemInterface,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::zvariant::OwnedObjectPath;
use zbus::{dbus_interface, Error, SignalContext};

// ANCHOR: ServiceVersionConstant
/// The current version of the CoreSystem D-Bus service.
const SERVICE_VERSION: &str = "0.1.0";

// ANCHOR: CoreSystemServiceStruct
/// Implements the `org.novade.CoreSystem` D-Bus interface.
///
/// This struct holds the state of all registered components and provides
/// the logic for the D-Bus methods and properties defined in `CoreSystemInterface`.
#[derive(Debug)]
pub struct CoreSystemService {
    /// Stores information about all registered components.
    /// The key is the component's unique name.
    /// `Arc<Mutex<...>>` is used to allow shared mutable access from async D-Bus methods.
    components: Arc<Mutex<HashMap<String, ComponentInfo>>>,
}

// ANCHOR: DefaultImplementation
impl Default for CoreSystemService {
    fn default() -> Self {
        Self {
            components: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ANCHOR: NewMethod
impl CoreSystemService {
    /// Creates a new instance of `CoreSystemService`.
    pub fn new() -> Self {
        tracing::info!(version = SERVICE_VERSION, "Creating new CoreSystemService instance");
        Default::default()
    }
}

// ANCHOR: DBusInterfaceImplementation
#[async_trait]
#[dbus_interface(name = "org.novade.CoreSystem")]
impl CoreSystemInterface for CoreSystemService {
    // ANCHOR: RegisterComponentMethodImpl
    async fn register_component(
        &mut self,
        name: String,
        object_path: OwnedObjectPath,
        initial_status: ComponentStatus,
        metadata: HashMap<String, String>,
    ) -> Result<bool, Error> {
        let mut components_guard = self.components.lock().await;
        if components_guard.contains_key(&name) {
            tracing::warn!(
                component_name = %name,
                "Attempted to register already existing component"
            );
            return Ok(false); // Component already registered
        }

        let component_info = ComponentInfo {
            name: name.clone(),
            object_path,
            status: initial_status.clone(),
            metadata,
        };

        components_guard.insert(name.clone(), component_info.clone());
        tracing::info!(component_name = %name, path = %component_info.object_path, status = ?initial_status, "Component registered");

        // Emit signal
        // It's important to get the SignalContext from the `self` reference if the trait method provides it,
        // or ensure the struct itself has a way to get it if methods are called outside a direct D-Bus context.
        // For `#[dbus_interface]` methods, zbus provides context implicitly.
        if let Err(e) = CoreSystemInterface::component_registered(self.signal_context(), component_info).await {
            tracing::error!(component_name = %name, "Failed to emit ComponentRegistered signal: {}", e);
        }

        Ok(true)
    }

    // ANCHOR: UnregisterComponentMethodImpl
    async fn unregister_component(&mut self, name: String) -> Result<bool, Error> {
        let mut components_guard = self.components.lock().await;
        if components_guard.remove(&name).is_some() {
            tracing::info!(component_name = %name, "Component unregistered");
            // Emit signal
            if let Err(e) = CoreSystemInterface::component_unregistered(self.signal_context(), name.clone()).await {
                 tracing::error!(component_name = %name, "Failed to emit ComponentUnregistered signal: {}", e);
            }
            Ok(true)
        } else {
            tracing::warn!(component_name = %name, "Attempted to unregister non-existent component");
            Ok(false) // Component not found
        }
    }

    // ANCHOR: QueryComponentStatusMethodImpl
    async fn query_component_status(&self, name: String) -> Result<ComponentStatus, Error> {
        let components_guard = self.components.lock().await;
        match components_guard.get(&name) {
            Some(info) => {
                tracing::debug!(component_name = %name, status = ?info.status, "Queried component status");
                Ok(info.status.clone())
            }
            None => {
                tracing::warn!(component_name = %name, "Queried status for non-existent component");
                Err(Error::UnknownMethod("Component not found".into())) // Or a more specific error
            }
        }
    }

    // ANCHOR: GetComponentInfoMethodImpl
    async fn get_component_info(&self, name: String) -> Result<ComponentInfo, Error> {
        let components_guard = self.components.lock().await;
        match components_guard.get(&name) {
            Some(info) => {
                tracing::debug!(component_name = %name, "Retrieved component info");
                Ok(info.clone())
            }
            None => {
                tracing::warn!(component_name = %name, "Attempted to get info for non-existent component");
                Err(Error::UnknownObject("Component not found".into()))
            }
        }
    }

    // ANCHOR: ListComponentsMethodImpl
    async fn list_components(&self) -> Result<Vec<ComponentInfo>, Error> {
        let components_guard = self.components.lock().await;
        let component_list: Vec<ComponentInfo> = components_guard.values().cloned().collect();
        tracing::debug!(count = component_list.len(), "Listed components");
        Ok(component_list)
    }

    // ANCHOR: UpdateComponentStatusMethodImpl
    async fn update_component_status(
        &mut self,
        name: String,
        new_status: ComponentStatus,
    ) -> Result<bool, Error> {
        let mut components_guard = self.components.lock().await;
        if let Some(info) = components_guard.get_mut(&name) {
            // TODO: Add security check: verify caller is allowed to update this component's status.
            // This might involve getting the caller's unique D-Bus name from the connection.
            // For now, any caller can update any component's status if the component exists.
            info.status = new_status.clone();
            tracing::info!(component_name = %name, status = ?new_status, "Component status updated");

            // Emit signal
            if let Err(e) = CoreSystemInterface::component_status_changed(self.signal_context(), name.clone(), new_status).await {
                tracing::error!(component_name = %name, "Failed to emit ComponentStatusChanged signal: {}", e);
            }
            Ok(true)
        } else {
            tracing::warn!(component_name = %name, "Attempted to update status for non-existent component");
            Ok(false) // Component not found
        }
    }

    // ANCHOR: PingMethodImpl
    async fn ping(&self, message: String) -> Result<String, Error> {
        let response = format!("Pong: {}", message);
        tracing::debug!(incoming_message = %message, response = %response, "Ping method called");
        Ok(response)
    }

    // ANCHOR: ServiceVersionPropertyImpl
    #[dbus_interface(property)]
    async fn service_version(&self) -> Result<String, Error> {
        tracing::debug!(version = SERVICE_VERSION, "ServiceVersion property accessed");
        Ok(SERVICE_VERSION.to_string())
    }

    // ANCHOR: DBusInterfaceSignalContextAccessors
    // The zbus macro should provide `signal_context()` on `&self` within `#[dbus_interface]` methods.
    // If direct signal emission is needed from non-D-Bus methods, the SignalContext
    // would need to be stored or passed around.
}

// ANCHOR: AddToModRs
// Note: Remember to add `pub mod core_system_service;` and `pub use core_system_service::CoreSystemService;`
// to `novade-system/src/dbus_interfaces/mod.rs`.
