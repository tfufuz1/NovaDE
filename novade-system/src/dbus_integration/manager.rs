//! # D-Bus Integration Manager
//!
//! This module provides the [`DbusServiceManager`], a utility struct for managing
//! D-Bus connections (both system and session bus), requesting service names,
//! serving D-Bus interface implementations, and creating proxies to interact with
//! other D-Bus services.
//!
//! It aims to simplify common D-Bus tasks within the NovaDE system.

use zbus::{object_server::{Guard as ObjectServerGuard, ObjectServer}, Connection, Error as ZbusError, Result as ZbusResult};
use zbus::names::WellKnownName;
use std::sync::Arc;
use crate::dbus_interfaces::notifications_server::NotificationsServer;
use novade_domain::notifications::NotificationService as DomainNotificationService;
use thiserror::Error;
use tokio::sync::Mutex; // Using tokio's Mutex if the manager itself needs to be shared across async tasks that modify it.
                       // For read-only access to Connection, Arc<Connection> is fine.

/// Errors that can occur during D-Bus operations managed by [`DbusServiceManager`].
#[derive(Debug, Error)]
pub enum DbusManagerError {
    /// Failed to establish a connection to the D-Bus.
    #[error("D-Bus connection failed: {0}")]
    ConnectionFailed(#[from] ZbusError),
    /// Failed to request a well-known name on the D-Bus.
    #[error("Failed to request D-Bus service name '{name}': {source}")]
    NameRequestFailed {
        /// The name that was being requested.
        name: String,
        /// The underlying D-Bus error.
        source: ZbusError,
    },
    /// Failed to serve a D-Bus object at a specified path.
    #[error("Failed to serve D-Bus object at path '{path}': {source}")]
    ServeAtFailed {
        /// The path at which serving the object was attempted.
        path: String,
        /// The underlying D-Bus error.
        source: ZbusError,
    },
    /// Indicates that the object server associated with the D-Bus connection was not available.
    /// This typically should not happen with active connections.
    #[error("Object server not available.")]
    ObjectServerNotAvailable,
    // ANCHOR [Task ID: ProxyCreationFailedError] Added error variant for proxy creation failures.
    /// Failed to create a D-Bus proxy for a remote service.
    #[error("Failed to create D-Bus proxy for service '{service}' at path '{path}': {source}")]
    ProxyCreationFailed {
        /// The destination service name for the proxy.
        service: String,
        /// The object path for the proxy.
        path: String,
        /// The underlying D-Bus error.
        source: ZbusError,
    },
}

/// A specialized `Result` type for `DbusServiceManager` operations.
pub type Result<T> = std::result::Result<T, DbusManagerError>;

/// Manages D-Bus connections (system or session), service name requests,
/// object serving, and proxy creation.
///
/// `DbusServiceManager` simplifies common D-Bus tasks by providing a unified
/// interface. It holds an `Arc<Connection>`, allowing the manager itself to be
/// cloned and shared across asynchronous tasks if needed. Served objects are
/// kept alive by `ObjectServerGuard`s stored internally.
///
/// # Examples
///
/// ```no_run
/// use novade_system::dbus_integration::DbusServiceManager;
/// use std::sync::Arc;
/// use zbus::Connection;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Connect to the session bus
///     let manager = Arc::new(DbusServiceManager::new_session().await?);
///
///     // Request a service name
///     manager.request_name("org.novade.ExampleService").await?;
///
///     // ... serve interfaces and create proxies ...
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct DbusServiceManager {
    /// The active D-Bus connection (either system or session).
    /// Field name `system_bus` is a misnomer if `new_session` was used; prefer using `connection()` accessor.
    // TODO: Rename `system_bus` field to `connection` or `bus_connection` in a future refactor.
    system_bus: Arc<Connection>,
    /// Guards for D-Bus objects served by this manager. When a guard is dropped,
    /// the corresponding object is unregistered from the D-Bus.
    server_guards: Mutex<Vec<ObjectServerGuard<'static>>>,
}

impl DbusServiceManager {
    /// Creates a new `DbusServiceManager` and connects to the **system D-Bus**.
    ///
    /// This is typically used for services that need system-wide privileges or access.
    ///
    /// # Errors
    ///
    /// Returns `DbusManagerError::ConnectionFailed` if the connection to the system bus cannot be established.
    #[tracing::instrument(skip_all)]
    pub async fn new() -> Result<Self> {
        tracing::info!("Initializing DbusServiceManager and connecting to system bus...");
        let system_bus_conn = Connection::system().await?;
        tracing::info!("Successfully connected to D-Bus system bus. Unique name: {}", system_bus_conn.unique_name().map_or_else(|| "<unknown>".to_string(), |n| n.to_string()));
        Ok(Self {
            system_bus: Arc::new(system_bus_conn),
            server_guards: Mutex::new(Vec::new()),
        })
    }

    // ANCHOR: NewSessionMethod
    /// Creates a new `DbusServiceManager` and connects to the **session D-Bus**.
    ///
    /// This is often preferred for user-specific services, applications, and for testing.
    ///
    /// # Errors
    ///
    /// Returns `DbusManagerError::ConnectionFailed` if the connection to the session bus cannot be established.
    #[tracing::instrument(skip_all)]
    pub async fn new_session() -> Result<Self> {
        tracing::info!("Initializing DbusServiceManager and connecting to session bus...");
        let session_bus_conn = Connection::session().await?;
        tracing::info!("Successfully connected to D-Bus session bus. Unique name: {}", session_bus_conn.unique_name().map_or_else(|| "<unknown>".to_string(), |n| n.to_string()));
        Ok(Self {
            system_bus: Arc::new(session_bus_conn),
            server_guards: Mutex::new(Vec::new()),
        })
    }

    /// Returns a clone of the Arc-wrapped D-Bus [`Connection`].
    ///
    /// This connection could be to either the system or session bus, depending on how
    /// the `DbusServiceManager` was instantiated (`new()` vs `new_session()`).
    /// Useful for creating proxies or for services that need direct connection access.
    pub fn connection(&self) -> Arc<Connection> {
        self.system_bus.clone()
    }

    /// Requests a well-known name on the D-Bus (e.g., "org.freedesktop.Notifications").
    ///
    /// This allows other D-Bus clients to find and interact with the service using this name.
    ///
    /// # Arguments
    ///
    /// * `name`: The well-known name to request. This typically follows reverse domain name notation.
    ///
    /// # Errors
    ///
    /// Returns `DbusManagerError::NameRequestFailed` if the name cannot be parsed or
    /// if the D-Bus broker denies the request (e.g., name already taken without allowance for replacement).
    #[tracing::instrument(skip_all, fields(name = %name))]
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
    /// with the connection's object server. The served object is kept alive as long
    /// as the returned `ObjectServerGuard` (stored internally by the manager) is kept.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the object to serve. It must implement `zbus::Interface`, `Send`, `Sync`,
    ///   and be `'static`.
    ///
    /// # Arguments
    ///
    /// * `object`: An `Arc<T>` pointing to the object instance that implements one or more D-Bus interfaces.
    ///   `zbus` will handle dispatching calls to the correct interface methods on this object.
    /// * `path`: The D-Bus object path (e.g., "/org/novade/MyService/Object1") at which to serve the object.
    ///
    /// # Errors
    ///
    /// Returns `DbusManagerError::ServeAtFailed` if the path is invalid or if the object
    /// cannot be registered with the object server.
    // ANCHOR [Task ID: GenericServeAtMethod] Implemented generic serve_at method.
    // ANCHOR [Task ID: ServeAtAcceptsArc] Modified serve_at to accept Arc<T>.
    #[tracing::instrument(skip_all, fields(path = %path))]
    pub async fn serve_at<T>(&self, object: Arc<T>, path: &str) -> Result<()>
    where
        T: zbus::Interface + Send + Sync + 'static,
    {
        tracing::info!("Serving D-Bus object (Arc<T>) at path: {}", path);
        let object_path = zbus::zvariant::ObjectPath::try_from(path)
            .map_err(|e| DbusManagerError::ServeAtFailed { path: path.to_string(), source: ZbusError::Path(e) })?;

        match self.system_bus.object_server().at(object_path.clone(), object).await {
            Ok(guard) => {
                let mut guards_vec = self.server_guards.lock().await;
                guards_vec.push(guard);
                tracing::info!("Successfully served D-Bus object at path '{}'. Total guards: {}", path, guards_vec.len());
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to serve D-Bus object at path '{}': {}", path, e);
                Err(DbusManagerError::ServeAtFailed { path: path.to_string(), source: e })
            }
        }
    }

    // ANCHOR [Task ID: InstrumentServeNotificationsServer] Added tracing instrument.
    #[tracing::instrument(skip_all)]
    pub async fn serve_notifications_server(
        &self,
        domain_notification_service: Arc<dyn DomainNotificationService>, // Use imported trait
    ) -> Result<()> {
        tracing::info!("Preparing to serve NotificationsServer on D-Bus...");

        self.request_name("org.freedesktop.Notifications").await?;

        let notification_server_logic = NotificationsServer::new(domain_notification_service);
        tracing::debug!("NotificationsServer logic instance created, attempting to serve.");

        // ANCHOR [Task ID: RefactorServeNotifications] Refactored to use generic serve_at.
        // Now that serve_at takes Arc<T>, we need to wrap notification_server_logic in Arc.
        self.serve_at(Arc::new(notification_server_logic), "/org/freedesktop/Notifications").await?; // This uses self.system_bus which is now generic

        tracing::info!("NotificationsServer successfully served via generic serve_at method.");

        Ok(())
    }

    // ANCHOR [Task ID: CreateProxyMethod] Implemented typed proxy creation method.
    /// Creates a typed D-Bus proxy for interacting with a remote D-Bus service.
    ///
    /// This allows calling methods, getting/setting properties, and receiving signals
    /// from the remote object in a type-safe manner.
    ///
    /// # Type Parameters
    ///
    /// * `P`: The type of the proxy to create. This type is typically generated by
    ///   `zbus::dbus_proxy!` macro and implements `zbus::InterfaceProxy`.
    ///   It must also be `Send + Sync + 'static`.
    ///
    /// # Arguments
    ///
    /// * `destination`: The well-known D-Bus name of the target service (e.g., "org.freedesktop.UPower").
    /// * `path`: The object path on the target service (e.g., "/org/freedesktop/UPower").
    ///
    /// # Errors
    ///
    /// Returns `DbusManagerError::ProxyCreationFailed` if the proxy cannot be built,
    /// for example, due to invalid destination or path, or if the connection is lost.
    #[tracing::instrument(skip_all, fields(destination = %destination, path = %path))]
    pub async fn create_proxy<P>(
        &self,
        destination: WellKnownName<'static>,
        path: zbus::zvariant::ObjectPath<'static>,
    ) -> Result<P>
    where
        P: for<'a> zbus::InterfaceProxy<'a> + Send + Sync + 'static,
    {
        tracing::info!(
            "Creating D-Bus proxy for service '{}' at path '{}' using connection: {}",
            destination,
            path,
            self.system_bus.unique_name().map_or_else(|| "<unknown>", |n| n.as_str())
        );
        P::builder(&self.connection()) // Use the generic connection() method
            .destination(destination.clone())
            .map_err(|e| DbusManagerError::ProxyCreationFailed {
                service: destination.to_string(),
                path: path.to_string(),
                source: e,
            })?
            .path(path.clone())
            .map_err(|e| DbusManagerError::ProxyCreationFailed {
                service: destination.to_string(),
                path: path.to_string(),
                source: e,
            })?
            .build()
            .await
            .map_err(|e| DbusManagerError::ProxyCreationFailed {
                service: destination.to_string(),
                path: path.to_string(),
                source: e,
            })
    }
}
