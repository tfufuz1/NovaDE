// ANCHOR: Header
//! # NovaDE Core System D-Bus Interface
//!
//! This module defines the D-Bus interface `org.novade.CoreSystem` for essential
//! inter-component communication within the NovaDE desktop environment.
//!
//! It allows components (agents) to register themselves, query the status of other
//! components, and receive notifications about component lifecycle events.

// ANCHOR: Crates
use zbus::dbus_interface;
use zbus::zvariant::{OwnedObjectPath, Type, Value};
use zbus::Error;
use std::collections::HashMap;

// ANCHOR: ComponentStatusEnum
/// Represents the status of a registered component.
#[derive(Debug, Clone, Type, Value, serde::Serialize, serde::Deserialize)]
pub enum ComponentStatus {
    /// The component is active and running.
    Active,
    /// The component is inactive or stopped.
    Inactive,
    /// The component is in an error state.
    Error(String),
    /// The component is starting up.
    Starting,
    /// The component is shutting down.
    Stopping,
}

// ANCHOR: ComponentInfoStruct
/// Holds information about a registered component.
#[derive(Debug, Clone, Type, Value, serde::Serialize, serde::Deserialize)]
pub struct ComponentInfo {
    /// The unique name of the component (e.g., "org.novade.WindowManager").
    pub name: String,
    /// The D-Bus object path of the component's primary interface.
    pub object_path: OwnedObjectPath,
    /// The current status of the component.
    pub status: ComponentStatus,
    /// Additional metadata about the component (e.g., version, capabilities).
    pub metadata: HashMap<String, String>,
}

// ANCHOR: CoreSystemInterfaceTrait
/// D-Bus Interface: `org.novade.CoreSystem`
///
/// Provides core functionalities for component registration, status querying,
/// and lifecycle notifications within NovaDE.
#[dbus_interface(name = "org.novade.CoreSystem")]
pub trait CoreSystemInterface {
    // ANCHOR: RegisterComponentMethod
    /// Registers a component with the CoreSystem.
    ///
    /// Components should call this method upon startup to make themselves known
    /// to the rest of the NovaDE system.
    ///
    /// # Arguments
    ///
    /// * `name`: A unique, well-known name for the component (e.g., "org.novade.NotificationService").
    /// * `object_path`: The D-Bus object path where the component exposes its primary interface.
    /// * `initial_status`: The initial status of the component.
    /// * `metadata`: A map of key-value pairs providing additional information about the component.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if registration was successful, `Ok(false)` if a component with the
    /// same name is already registered (and replacement is not allowed), or an error
    /// if the registration fails for other reasons.
    async fn register_component(
        &mut self,
        name: String,
        object_path: OwnedObjectPath,
        initial_status: ComponentStatus,
        metadata: HashMap<String, String>,
    ) -> Result<bool, Error>;

    // ANCHOR: UnregisterComponentMethod
    /// Unregisters a component from the CoreSystem.
    ///
    /// Components should call this method before shutting down.
    ///
    /// # Arguments
    ///
    /// * `name`: The unique name of the component to unregister.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if unregistration was successful, `Ok(false)` if the component
    /// was not found, or an error if the unregistration fails.
    async fn unregister_component(&mut self, name: String) -> Result<bool, Error>;

    // ANCHOR: QueryComponentStatusMethod
    /// Queries the current status of a registered component.
    ///
    /// # Arguments
    ///
    /// * `name`: The unique name of the component to query.
    ///
    /// # Returns
    ///
    /// `Ok(ComponentStatus)` if the component is found, or an error if the
    /// component is not registered or if the query fails.
    async fn query_component_status(&self, name: String) -> Result<ComponentStatus, Error>;

    // ANCHOR: GetComponentInfoMethod
    /// Retrieves detailed information about a registered component.
    ///
    /// # Arguments
    ///
    /// * `name`: The unique name of the component.
    ///
    /// # Returns
    ///
    /// `Ok(ComponentInfo)` if the component is found, or an error otherwise.
    async fn get_component_info(&self, name: String) -> Result<ComponentInfo, Error>;

    // ANCHOR: ListComponentsMethod
    /// Lists all currently registered components.
    ///
    /// # Returns
    ///
    /// A vector of `ComponentInfo` structs for all registered components.
    async fn list_components(&self) -> Result<Vec<ComponentInfo>, Error>;

    // ANCHOR: UpdateComponentStatusMethod
    /// Allows a registered component to update its own status.
    ///
    /// # Arguments
    ///
    /// * `name`: The unique name of the component updating its status.
    /// * `new_status`: The new status of the component.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the status was updated successfully, `Ok(false)` if the
    /// component is not found (e.g., if it's trying to update status for another component
    /// or if it's not registered), or an error on failure.
    /// Note: Security considerations are important here; typically, only the component
    /// itself or a privileged service should be able to update its status.
    /// This might require checking the caller's D-Bus unique name.
    async fn update_component_status(
        &mut self,
        name: String, // Could also be implicit based on caller for security
        new_status: ComponentStatus,
    ) -> Result<bool, Error>;

    // ANCHOR: ComponentRegisteredSignal
    /// Signal emitted when a new component successfully registers.
    ///
    /// # Arguments
    ///
    /// * `component_info`: Information about the newly registered component.
    #[dbus_interface(signal)]
    async fn component_registered(&self, component_info: ComponentInfo) -> Result<(), Error>;

    // ANCHOR: ComponentUnregisteredSignal
    /// Signal emitted when a component successfully unregisters.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the component that was unregistered.
    #[dbus_interface(signal)]
    async fn component_unregistered(&self, name: String) -> Result<(), Error>;

    // ANCHOR: ComponentStatusChangedSignal
    /// Signal emitted when a component's status changes.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the component whose status changed.
    /// * `new_status`: The new status of the component.
    #[dbus_interface(signal)]
    async fn component_status_changed(
        &self,
        name: String,
        new_status: ComponentStatus,
    ) -> Result<(), Error>;

    // ANCHOR: PingMethod
    /// A simple ping method to check if the CoreSystem service is alive and responsive.
    /// This is useful for basic health checks by clients.
    ///
    /// # Arguments
    ///
    /// * `message`: An optional message string to be echoed back.
    ///
    /// # Returns
    ///
    /// The echoed message, prefixed with "Pong: ".
    async fn ping(&self, message: String) -> Result<String, Error>;

    // ANCHOR: GetServiceVersionProperty
    /// Property: ServiceVersion (Read-only)
    ///
    /// Returns the current version of the `org.novade.CoreSystem` D-Bus interface.
    #[dbus_interface(property)]
    async fn service_version(&self) -> Result<String, Error>;
}
