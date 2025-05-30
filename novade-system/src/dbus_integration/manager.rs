// novade-system/src/dbus_integration/manager.rs

use zbus::{Connection, object_server::{ObjectServer, Guard as ObjectServerGuard}, Error as ZbusError, Result as ZbusResult};
use zbus::names::WellKnownName;
use std::sync::Arc;
use crate::dbus_interfaces::notifications_server::NotificationsServer;
use novade_domain::notifications::NotificationService as DomainNotificationService;
use thiserror::Error;
use tokio::sync::Mutex; // Using tokio's Mutex if the manager itself needs to be shared across async tasks that modify it.
                       // For read-only access to Connection, Arc<Connection> is fine.

// Use a more specific error type for the manager, or re-use/adapt from examples.rs
#[derive(Debug, Error)]
pub enum DbusManagerError {
    #[error("D-Bus connection failed: {0}")]
    ConnectionFailed(#[from] ZbusError),
    #[error("Failed to request D-Bus service name '{name}': {source}")]
    NameRequestFailed { name: String, source: ZbusError },
    #[error("Failed to serve D-Bus object at path '{path}': {source}")]
    ServeAtFailed { path: String, source: ZbusError },
    #[error("Object server not available.")]
    ObjectServerNotAvailable,
}

pub type Result<T> = std::result::Result<T, DbusManagerError>;

/// Manages the D-Bus connection and service/object registration for NovaDE system services.
#[derive(Clone)] // Clone if DbusServiceManager itself needs to be shared widely via Arc.
                 // Connection is already Arc-like.
pub struct DbusServiceManager {
    // System bus connection, shared via Arc for multiple proxies/services.
    system_bus: Arc<Connection>,
    server_guards: Mutex<Vec<ObjectServerGuard<'static>>>,
}

impl DbusServiceManager {
    /// Creates a new `DbusServiceManager` and connects to the system D-Bus.
    /// This method is async because connecting to D-Bus is an async operation.
    pub async fn new() -> Result<Self> {
        tracing::info!("Initializing DbusServiceManager and connecting to system bus...");
        let system_bus_conn = Connection::system().await?;
        tracing::info!("Successfully connected to D-Bus system bus. Unique name: {}", system_bus_conn.unique_name().map_or_else(|| "<unknown>".to_string(), |n| n.to_string()));
        Ok(Self {
            system_bus: Arc::new(system_bus_conn),
            server_guards: Mutex::new(Vec::new()),
        })
    }

    /// Returns a clone of the Arc-wrapped system D-Bus connection.
    /// Useful for creating proxies or for services that need direct connection access.
    pub fn system_bus(&self) -> Arc<Connection> {
        self.system_bus.clone()
    }

    /// Requests a well-known name on the D-Bus.
    ///
    /// # Arguments
    /// * `name`: The well-known name to request (e.g., "org.freedesktop.Notifications").
    pub async fn request_name(&self, name: &str) -> Result<()> {
        let well_known_name = WellKnownName::try_from(name)
            .map_err(|e| DbusManagerError::NameRequestFailed { name: name.to_string(), source: ZbusError::Name(e) })?;

        tracing::info!("Requesting D-Bus name: {}", name);
        self.system_bus.request_name(well_known_name).await.map_err(|e| {
            DbusManagerError::NameRequestFailed { name: name.to_string(), source: e }
        })?;
        tracing::info!("Successfully requested D-Bus name: {}", name);
        Ok(())
    }

    /// Serves a D-Bus object at a given path.
    ///
    /// This method registers an object implementing a D-Bus interface
    /// with the connection's object server.
    ///
    /// # Arguments
    /// * `object`: The object to serve. It must implement a zbus interface trait.
    /// * `path`: The D-Bus object path (e.g., "/org/freedesktop/Notifications").
    ///
    /// Note: The lifetime of the served object is tied to the `ObjectServerGuard`
    // pub async fn serve_at<T>(&self, object: T, path: &str) -> Result<()> // Removed
    // where
    //     T: zbus::Interface + Send + Sync + 'static,
    // {
    //     // ... implementation removed ...
    // }

    pub async fn serve_notifications_server(
        &self,
        domain_notification_service: Arc<dyn DomainNotificationService>, // Use imported trait
    ) -> Result<()> {
        tracing::info!("Preparing to serve NotificationsServer on D-Bus...");

        self.request_name("org.freedesktop.Notifications").await?;

        let notification_server_logic = NotificationsServer::new(domain_notification_service);
        tracing::debug!("NotificationsServer logic instance created.");

        let guard = self.system_bus
            .object_server()
            .at("/org/freedesktop/Notifications", notification_server_logic)
            .await
            .map_err(|e| DbusManagerError::ServeAtFailed {
                path: "/org/freedesktop/Notifications".to_string(),
                source: e,
            })?;

        tracing::info!(
            "NotificationsServer successfully registered with D-Bus object server at /org/freedesktop/Notifications. Object path: {:?}",
            guard.path()
        );

        let mut guards_vec = self.server_guards.lock().await;
        guards_vec.push(guard);
        tracing::debug!("NotificationsServer ObjectServerGuard stored in DbusServiceManager. Total guards: {}", guards_vec.len());

        Ok(())
    }

    // Method to create a typed proxy for convenience
    // pub async fn create_proxy<'a, P: zbus::InterfaceProxy<'a>>(
    //     &self,
    //     dest: impl Into<WellKnownName<'static>>, // Or ServiceName
    //     path: impl TryInto<ObjectPath<'static>>, // ObjectPath
    // ) -> Result<P> {
    //     P::builder(&self.system_bus)
    //         .destination(dest)?
    //         .path(path)?
    //         .build()
    //         .await
    //         .map_err(|e| DbusManagerError::ProxyCreationFailed { source: e }) // Assuming ProxyCreationFailed can take generic zbus::Error
    // }
}
