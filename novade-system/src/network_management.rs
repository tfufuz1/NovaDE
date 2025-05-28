//! Network management module for the NovaDE system layer.
//!
//! This module provides network management functionality for the NovaDE desktop environment,
//! monitoring network connectivity.

use async_trait::async_trait;
use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{Arc}, 
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use zbus::{
    Connection as ZbusConnection, Proxy,
    names::InterfaceName,
    zvariant::{Value, ObjectPath},
};
// Required for mockall with async_trait when pinning futures in expectations
use futures_util::future::BoxFuture;


use crate::error::{SystemError, SystemResult, to_system_error, SystemErrorKind};


// Constants for NetworkManager D-Bus service
const NM_DBUS_SERVICE: &str = "org.freedesktop.NetworkManager";
const NM_DBUS_PATH: &str = "/org/freedesktop/NetworkManager";
const NM_INTERFACE: &str = "org.freedesktop.NetworkManager";
const NM_DEVICE_INTERFACE: &str = "org.freedesktop.NetworkManager.Device";
const NM_ACTIVE_CONN_INTERFACE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const NM_SETTINGS_INTERFACE: &str = "org.freedesktop.NetworkManager.Settings";
const NM_SETTINGS_CONNECTION_INTERFACE: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const NM_ACCESS_POINT_INTERFACE: &str = "org.freedesktop.NetworkManager.AccessPoint";
const NM_DEVICE_WIRELESS_INTERFACE: &str = "org.freedesktop.NetworkManager.Device.Wireless";

// NMConnectivityState enum based on NetworkManager documentation
#[allow(dead_code)] 
enum NMConnectivityState {
    Unknown = 0,
    None = 1,
    Portal = 2,
    Limited = 3,
    Full = 4,
}

impl TryFrom<u32> for NMConnectivityState {
    type Error = SystemError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NMConnectivityState::Unknown),
            1 => Ok(NMConnectivityState::None),
            2 => Ok(NMConnectivityState::Portal),
            3 => Ok(NMConnectivityState::Limited),
            4 => Ok(NMConnectivityState::Full),
            _ => Err(to_system_error(format!("Unknown NMConnectivityState value: {}", value), SystemErrorKind::NetworkManagement)),
        }
    }
}


/// Network connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkConnectionType {
    Wired,
    Wireless,
    Mobile,
    VPN,
    Other,
}

/// Network connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Unknown,
}

/// Trait defining the D-Bus operations needed by NetworkManagerIntegration.
#[cfg_attr(test, mockall::automock)] // Apply automock for testing
#[async_trait]
pub trait NetworkDBusOperations: Send + Sync {
    async fn get_connections(&self) -> SystemResult<Vec<NetworkConnection>>;
    async fn connect(&self, id: &str) -> SystemResult<()>;
    async fn disconnect(&self, id: &str) -> SystemResult<()>;
    async fn has_connectivity(&self) -> SystemResult<bool>;
    async fn start_signal_handlers(&self) -> SystemResult<()>;
}

/// Network connection.
#[derive(Debug, Clone, PartialEq)] // Added PartialEq for test assertions
pub struct NetworkConnection {
    id: String,
    name: String,
    connection_type: NetworkConnectionType,
    state: NetworkConnectionState,
    is_default: bool,
    strength: Option<f64>,
    speed: Option<u32>,
}

impl NetworkConnection {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        connection_type: NetworkConnectionType,
        state: NetworkConnectionState,
        is_default: bool,
        strength: Option<f64>,
        speed: Option<u32>,
    ) -> Self {
        NetworkConnection {
            id: id.into(),
            name: name.into(),
            connection_type,
            state,
            is_default,
            strength,
            speed,
        }
    }

    pub fn id(&self) -> &str { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn connection_type(&self) -> NetworkConnectionType { self.connection_type }
    pub fn state(&self) -> NetworkConnectionState { self.state }
    pub fn is_default(&self) -> bool { self.is_default }
    pub fn strength(&self) -> Option<f64> { self.strength }
    pub fn speed(&self) -> Option<u32> { self.speed }
}

/// Network event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkEventType {
    ConnectionAdded,
    ConnectionRemoved,
    ConnectionStateChanged,
    DefaultConnectionChanged,
    ConnectivityChanged,
}

/// Network event.
#[derive(Debug, Clone, PartialEq)] // Added PartialEq for test assertions
pub struct NetworkEvent {
    pub event_type: NetworkEventType,
    pub connection_id: Option<String>,
    pub state: Option<NetworkConnectionState>,
    pub has_connectivity: Option<bool>,
}

/// Network manager interface.
#[async_trait]
pub trait NetworkManager: Send + Sync {
    async fn get_connections(&self) -> SystemResult<Vec<NetworkConnection>>;
    async fn get_connection(&self, id: &str) -> SystemResult<NetworkConnection>;
    async fn get_default_connection(&self) -> SystemResult<NetworkConnection>;
    async fn connect(&self, id: &str) -> SystemResult<()>;
    async fn disconnect(&self, id: &str) -> SystemResult<()>;
    async fn has_connectivity(&self) -> SystemResult<bool>;
    async fn subscribe(&self) -> SystemResult<Receiver<NetworkEvent>>;
}


pub struct NetworkManagerIntegration<T: NetworkDBusOperations + ?Sized = NetworkDBusConnection> {
    connection: Arc<T>,
    connection_cache: Arc<tokio::sync::Mutex<HashMap<String, NetworkConnection>>>,
    client_event_sender: Sender<NetworkEvent>,
}

impl NetworkManagerIntegration<NetworkDBusConnection> {
    pub fn new_production() -> SystemResult<Self> {
        let (dbus_internal_event_tx, dbus_internal_event_rx) = mpsc::channel(100);
        let dbus_connection = Arc::new(NetworkDBusConnection::new(dbus_internal_event_tx)?);
        Self::new(dbus_connection, dbus_internal_event_rx)
    }
}

impl<T: NetworkDBusOperations + Send + Sync + 'static> NetworkManagerIntegration<T> {
    pub fn new(dbus_ops_provider: Arc<T>, mut internal_event_rx: Receiver<NetworkEvent>) -> SystemResult<Self> {
        let (client_event_tx, _) = mpsc::channel(100);
        
        let integration = NetworkManagerIntegration {
            connection: dbus_ops_provider.clone(),
            connection_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            client_event_sender: client_event_tx,
        };
        
        tokio::spawn(async move {
            if let Err(e) = dbus_ops_provider.start_signal_handlers().await {
                eprintln!("NetworkDBusOperations: start_signal_handlers failed: {}", e);
            }
        });
        
        integration.start_event_forwarding_loop(internal_event_rx);
        Ok(integration)
    }
    
    fn start_event_forwarding_loop(&self, mut internal_event_receiver: Receiver<NetworkEvent>) {
        let client_event_sender_clone = self.client_event_sender.clone();
        let connection_cache_clone = self.connection_cache.clone();

        tokio::spawn(async move {
            while let Some(event) = internal_event_receiver.recv().await {
                let mut cache = connection_cache_clone.lock().await;
                match event.event_type {
                    NetworkEventType::ConnectionStateChanged => {
                        if let Some(id) = &event.connection_id {
                            if let Some(conn) = cache.get_mut(id) {
                                if let Some(new_state) = event.state {
                                    conn.state = new_state;
                                }
                            }
                        }
                    }
                    NetworkEventType::ConnectionAdded => {
                        // A full cache refresh might be more robust here unless event carries full data.
                        // For now, this event signals a change, rely on `update_cache` being called.
                    }
                    NetworkEventType::ConnectionRemoved => {
                        if let Some(id) = &event.connection_id {
                            cache.remove(id);
                        }
                    }
                    NetworkEventType::DefaultConnectionChanged => {
                        // Requires re-evaluating `is_default` for all. `update_cache` handles this.
                    }
                    NetworkEventType::ConnectivityChanged => {
                        // Global state, not directly cached per connection here.
                    }
                }
                drop(cache); 

                if client_event_sender_clone.send(event).await.is_err() {
                    eprintln!("No active subscribers to NetworkManagerIntegration client channel, or channel closed.");
                }
            }
        });
    }
    
    async fn update_cache(&self) -> SystemResult<()> {
        let connections = self.connection.get_connections().await?;
        let mut cache = self.connection_cache.lock().await; 
        cache.clear();
        for connection in connections {
            cache.insert(connection.id().to_string(), connection);
        }
        Ok(())
    }
}

#[async_trait]
impl<T: NetworkDBusOperations + ?Sized + Send + Sync + 'static> NetworkManager for NetworkManagerIntegration<T> {
    async fn get_connections(&self) -> SystemResult<Vec<NetworkConnection>> {
        self.update_cache().await?;
        let cache = self.connection_cache.lock().await;
        Ok(cache.values().cloned().collect())
    }
    
    async fn get_connection(&self, id: &str) -> SystemResult<NetworkConnection> {
        self.update_cache().await?; // Ensures cache is fresh before specific lookup
        let cache = self.connection_cache.lock().await;
        cache.get(id)
            .cloned()
            .ok_or_else(|| to_system_error(format!("Network connection not found: {}", id), SystemErrorKind::NetworkManagement))
    }
    
    async fn get_default_connection(&self) -> SystemResult<NetworkConnection> {
        self.update_cache().await?;
        let cache = self.connection_cache.lock().await;
        cache.values()
            .find(|c| c.is_default())
            .cloned()
            .ok_or_else(|| to_system_error("No default network connection found", SystemErrorKind::NetworkManagement))
    }
    
    async fn connect(&self, id: &str) -> SystemResult<()> { self.connection.connect(id).await }
    async fn disconnect(&self, id: &str) -> SystemResult<()> { self.connection.disconnect(id).await }
    async fn has_connectivity(&self) -> SystemResult<bool> { self.connection.has_connectivity().await }
    async fn subscribe(&self) -> SystemResult<Receiver<NetworkEvent>> { Ok(self.client_event_sender.subscribe()) }
}

pub struct NetworkDBusConnection { 
    connection: ZbusConnection,
    nm_proxy: Proxy<'static>, 
    event_sender: Sender<NetworkEvent>, 
}

impl NetworkDBusConnection {
    async fn get_property_generic_priv<'a, V>(proxy: &Proxy<'_>, prop_name: &str) -> SystemResult<V>
    where V: TryFrom<Value<'a>> + Send + Sync + 'static, 
          V::Error: std::fmt::Display, 
    {
        proxy.get_property::<V>(prop_name)
            .await
            .map_err(|e| to_system_error(format!("Failed to get property '{}': {}", prop_name, e), SystemErrorKind::DBus))?
            .ok_or_else(|| to_system_error(format!("Property '{}' not found or None for proxy {}", prop_name, proxy.path()), SystemErrorKind::DBusPropertyNotFound))
    }

    async fn get_active_connections_paths_priv(&self) -> SystemResult<Vec<ObjectPath<'static>>> {
        Self::get_property_generic_priv(&self.nm_proxy, "ActiveConnections").await
    }

    async fn get_connection_details_priv(&self, active_conn_path: ObjectPath<'_>) -> SystemResult<Option<NetworkConnection>> {
        let active_conn_proxy = Proxy::new(
            self.connection.clone(), NM_DBUS_SERVICE, active_conn_path.clone(), NM_ACTIVE_CONN_INTERFACE,
        ).map_err(|e| to_system_error(format!("Failed to create active connection proxy for '{}': {}", active_conn_path, e), SystemErrorKind::DBus))?;

        let id: String = Self::get_property_generic_priv(&active_conn_proxy, "Id").await?;
        let uuid: String = Self::get_property_generic_priv(&active_conn_proxy, "Uuid").await?;
        let conn_type_str: String = Self::get_property_generic_priv(&active_conn_proxy, "Type").await?;
        let state_u32: u32 = Self::get_property_generic_priv(&active_conn_proxy, "State").await?;
        let default_bool: bool = Self::get_property_generic_priv(&active_conn_proxy, "Default").await?;
        let devices_paths: Vec<ObjectPath> = Self::get_property_generic_priv(&active_conn_proxy, "Devices").await?;

        let connection_type = match conn_type_str.as_str() {
            "802-3-ethernet" => NetworkConnectionType::Wired,
            "802-11-wireless" => NetworkConnectionType::Wireless,
            "vpn" => NetworkConnectionType::VPN, "gsm" | "cdma" => NetworkConnectionType::Mobile,
            _ => NetworkConnectionType::Other,
        };
        let state = match state_u32 {
            0 => NetworkConnectionState::Unknown, 1 => NetworkConnectionState::Connecting,
            2 => NetworkConnectionState::Connected, 3 => NetworkConnectionState::Disconnecting,
            4 => NetworkConnectionState::Disconnected, _ => NetworkConnectionState::Unknown,
        };
        
        let mut strength: Option<f64> = None; let mut speed: Option<u32> = None;
        if !devices_paths.is_empty() {
            let device_path = &devices_paths[0];
            let gen_device_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, device_path.clone(), NM_DEVICE_INTERFACE)
                .map_err(|e| to_system_error(format!("Dev proxy fail {}: {}", device_path, e), SystemErrorKind::DBus))?;
            if let Ok(s) = Self::get_property_generic_priv::<u32>(&gen_device_proxy, "Speed").await { speed = Some(s); }
            if connection_type == NetworkConnectionType::Wireless {
                 let wireless_device_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, device_path.clone(), NM_DEVICE_WIRELESS_INTERFACE)
                    .map_err(|e| to_system_error(format!("Wireless dev proxy fail {}: {}", device_path, e), SystemErrorKind::DBus))?;
                if let Ok(active_ap_path) = Self::get_property_generic_priv::<ObjectPath>(&wireless_device_proxy, "ActiveAccessPoint").await {
                    if active_ap_path.as_str() != "/" {
                        let ap_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, active_ap_path.clone(), NM_ACCESS_POINT_INTERFACE)
                           .map_err(|e| to_system_error(format!("AP proxy fail {}: {}", active_ap_path, e), SystemErrorKind::DBus))?;
                        if let Ok(strength_u8) = Self::get_property_generic_priv::<u8>(&ap_proxy, "Strength").await { strength = Some(f64::from(strength_u8) / 100.0); }
                    }
                }
                 if let Ok(bitrate_u32) = Self::get_property_generic_priv::<u32>(&wireless_device_proxy, "Bitrate").await { speed = Some(bitrate_u32 / 1000); }
            }
        }
        Ok(Some(NetworkConnection::new(uuid, id, connection_type, state, default_bool, strength, speed)))
    }
}

#[async_trait]
impl NetworkDBusOperations for NetworkDBusConnection {
    async fn get_connections(&self) -> SystemResult<Vec<NetworkConnection>> {
        let active_conn_paths = self.get_active_connections_paths_priv().await?;
        let mut connections = Vec::new();
        for path in active_conn_paths {
            match self.get_connection_details_priv(path.into_owned()).await {
                Ok(Some(conn_details)) => connections.push(conn_details),
                Ok(None) => {}, Err(e) => eprintln!("Error fetching details for connection: {}", e),
            }
        }
        Ok(connections)
    }

    async fn connect(&self, uuid: &str) -> SystemResult<()> {
        let settings_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, "/org/freedesktop/NetworkManager/Settings", NM_SETTINGS_INTERFACE)
            .map_err(|e| to_system_error(format!("Failed to create NM Settings proxy: {}", e), SystemErrorKind::DBus))?;
        let all_connections_paths: Vec<ObjectPath> = settings_proxy.call_method("ListConnections", ()).await
            .map_err(|e| to_system_error(format!("Failed to list connections: {}", e), SystemErrorKind::DBus))?.0; 
        let mut found_conn_path: Option<ObjectPath> = None;
        for conn_path_cow in all_connections_paths {
            let conn_path = conn_path_cow.into_owned();
            let conn_settings_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, conn_path.clone(), NM_SETTINGS_CONNECTION_INTERFACE)
                .map_err(|e| to_system_error(format!("Conn settings proxy fail {}: {}", conn_path, e), SystemErrorKind::DBus))?;
            let settings_map: HashMap<String, HashMap<String, Value>> = conn_settings_proxy.call_method("GetSettings", ()).await
                .map_err(|e| to_system_error(format!("GetSettings fail {}: {}", conn_path, e), SystemErrorKind::DBus))?.0;
            if let Some(connection_settings) = settings_map.get("connection") {
                if let Some(Value::Str(s_uuid)) = connection_settings.get("uuid") { if s_uuid.as_str() == uuid { found_conn_path = Some(conn_path); break; } }
            }
        }
        if let Some(conn_to_activate) = found_conn_path {
            self.nm_proxy.call_method("ActivateConnection", (conn_to_activate, ObjectPath::from_static_str_unchecked("/"), ObjectPath::from_static_str_unchecked("/"))).await
                .map_err(|e| to_system_error(format!("Activate fail UUID '{}': {}", uuid, e), SystemErrorKind::DBus))?;
            Ok(())
        } else { Err(to_system_error(format!("Conn profile UUID '{}' not found.", uuid), SystemErrorKind::NotFound)) }
    }
    
    async fn disconnect(&self, uuid_or_active_path: &str) -> SystemResult<()> {
        let active_conn_path = if uuid_or_active_path.starts_with("/org/freedesktop/NetworkManager/ActiveConnection/") {
            ObjectPath::try_from(uuid_or_active_path).map_err(|e| to_system_error(format!("Invalid active conn path '{}': {}", uuid_or_active_path, e), SystemErrorKind::InvalidInput))?
        } else {
            let active_connections = self.get_active_connections_paths_priv().await?;
            let mut found_path: Option<ObjectPath> = None;
            for path_cow in active_connections {
                let path = path_cow.into_owned();
                let active_conn_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, path.clone(), NM_ACTIVE_CONN_INTERFACE)
                    .map_err(|e| to_system_error(format!("Active conn proxy fail '{}': {}", path, e), SystemErrorKind::DBus))?;
                let current_uuid: String = Self::get_property_generic_priv(&active_conn_proxy, "Uuid").await?;
                if current_uuid == uuid_or_active_path { found_path = Some(path); break; }
            }
            found_path.ok_or_else(|| to_system_error(format!("No active conn with UUID '{}'", uuid_or_active_path), SystemErrorKind::NotFound))?
        };
        self.nm_proxy.call_method("DeactivateConnection", (active_conn_path,)).await
            .map_err(|e| to_system_error(format!("Deactivate fail '{}': {}", uuid_or_active_path, e), SystemErrorKind::DBus))?;
        Ok(())
    }
    
    async fn has_connectivity(&self) -> SystemResult<bool> {
        let connectivity_u32: u32 = Self::get_property_generic_priv(&self.nm_proxy, "Connectivity").await?;
        let connectivity_state = NMConnectivityState::try_from(connectivity_u32)?;
        match connectivity_state {
            NMConnectivityState::Full | NMConnectivityState::Limited | NMConnectivityState::Portal => Ok(true), 
            _ => Ok(false),
        }
    }

    async fn start_signal_handlers(&self) -> SystemResult<()> {
        use futures_util::StreamExt; // Ensure StreamExt is in scope for .next()
        let state_changed_stream = self.nm_proxy.receive_signal("StateChanged").await
            .map_err(|e| to_system_error(format!("Failed to subscribe to NM StateChanged: {}", e), SystemErrorKind::DBus))?;
        let sender_clone1 = self.event_sender.clone();
        tokio::spawn(async move {
            futures_util::pin_mut!(state_changed_stream); 
            while let Some(signal) = state_changed_stream.next().await {
                if let Ok(new_nm_state_u32) = signal.body::<u32>() {
                    let has_connectivity = new_nm_state_u32 >= 70; 
                    if sender_clone1.send(NetworkEvent { event_type: NetworkEventType::ConnectivityChanged, connection_id: None, state: None, has_connectivity: Some(has_connectivity) }).await.is_err() {
                        eprintln!("Failed to send ConnectivityChanged event"); break; 
                    }
                }
            }
        });

        let nm_props_proxy = Proxy::new(self.connection.clone(), NM_DBUS_SERVICE, NM_DBUS_PATH, "org.freedesktop.DBus.Properties")
            .map_err(|e| to_system_error(format!("NM Props proxy fail: {}", e), SystemErrorKind::DBus))?;
        let props_changed_stream = nm_props_proxy.receive_signal("PropertiesChanged").await
            .map_err(|e| to_system_error(format!("Subscribe to NM PropertiesChanged fail: {}", e), SystemErrorKind::DBus))?;
        let sender_clone2 = self.event_sender.clone();
        tokio::spawn(async move {
            futures_util::pin_mut!(props_changed_stream);
            while let Some(signal) = props_changed_stream.next().await {
                if let Ok((iface, changed, _invalidated)) = signal.body::<(String, HashMap<String, Value>, Vec<String>)>() {
                    if iface == NM_INTERFACE && (changed.contains_key("ActiveConnections") || changed.contains_key("PrimaryConnection") || changed.contains_key("Connectivity")) {
                        if sender_clone2.send(NetworkEvent { event_type: NetworkEventType::DefaultConnectionChanged, connection_id: None, state: None, has_connectivity: None }).await.is_err() {
                            eprintln!("Failed to send DefaultConnectionChanged/ConnectivityChanged event"); break;
                        }
                    }
                }
            }
        });
        
        let device_added_stream = self.nm_proxy.receive_signal("DeviceAdded").await
            .map_err(|e| to_system_error(format!("Failed to subscribe to NM DeviceAdded: {}", e), SystemErrorKind::DBus))?;
        tokio::spawn(async move {
            futures_util::pin_mut!(device_added_stream);
            while let Some(signal) = device_added_stream.next().await {
                if let Ok(device_path) = signal.body::<ObjectPath<'_>>() { println!("NetworkManager D-Bus signal: DeviceAdded - Path: {}", device_path); }
            }
        });

        let device_removed_stream = self.nm_proxy.receive_signal("DeviceRemoved").await
            .map_err(|e| to_system_error(format!("Failed to subscribe to NM DeviceRemoved: {}", e), SystemErrorKind::DBus))?;
        tokio::spawn(async move {
            futures_util::pin_mut!(device_removed_stream);
            while let Some(signal) = device_removed_stream.next().await {
                if let Ok(device_path) = signal.body::<ObjectPath<'_>>() { println!("NetworkManager D-Bus signal: DeviceRemoved - Path: {}", device_path); }
            }
        });
        println!("NetworkManager D-Bus signal handlers started."); 
        Ok(())
    }
}

impl NetworkDBusConnection {
    pub fn new(event_sender: Sender<NetworkEvent>) -> SystemResult<Self> {
        let conn = ZbusConnection::system().map_err(|e| to_system_error(format!("Failed to connect to D-Bus system bus: {}", e), SystemErrorKind::DBus))?;
        let nm_proxy = Proxy::new(conn.clone(), NM_DBUS_SERVICE, NM_DBUS_PATH, NM_INTERFACE)
            .map_err(|e| to_system_error(format!("Failed to create NetworkManager proxy: {}", e), SystemErrorKind::DBus))?;
        Ok(Self { connection: conn, nm_proxy, event_sender })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc as tokio_mpsc; 
    use futures_util::StreamExt; // For stream.next() on signal streams
    use mockall::predicate; // For argument matching in mockall
    use std::time::Duration;

    // Apply automock directly to the trait NetworkDBusOperations
    // This generates MockNetworkDBusOperations
    // #[automock] // This is on the trait itself now
    // #[async_trait]
    // pub trait NetworkDBusOperations: Send + Sync { ... }
    
    // Helper to create a test NetworkManagerIntegration with a mock
    fn setup_test_integration_with_mock(
        mock_dbus_ops: MockNetworkDBusOperations, 
    ) -> (
        NetworkManagerIntegration<MockNetworkDBusOperations>, 
        tokio_mpsc::Sender<NetworkEvent>, 
    ) {
        let (dbus_internal_event_tx, dbus_internal_event_rx) = tokio_mpsc::channel(100);
        let integration = NetworkManagerIntegration::new(
            Arc::new(mock_dbus_ops), 
            dbus_internal_event_rx
        ).expect("Failed to create NetworkManagerIntegration for test");
        (integration, dbus_internal_event_tx)
    }

    #[tokio::test]
    async fn test_nmi_get_connections_empty_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new(); 
        mock_dbus.expect_get_connections()
            .times(1)
            .returning(|| Box::pin(async { Ok(Vec::new()) })); // Use Box::pin for async closures in mockall
        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));

        let (integration, _dbus_event_tx) = setup_test_integration_with_mock(mock_dbus);
        
        let connections = integration.get_connections().await.unwrap();
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_nmi_get_connections_with_data_and_cache_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new();
        let sample_connections = vec![
            NetworkConnection::new("conn1_uuid", "Conn1", NetworkConnectionType::Wired, NetworkConnectionState::Connected, true, None, None),
            NetworkConnection::new("conn2_uuid", "Conn2", NetworkConnectionType::Wireless, NetworkConnectionState::Disconnected, false, Some(0.8), None),
        ];
        
        let expected_connections_clone1 = sample_connections.clone();
        mock_dbus.expect_get_connections()
            .times(1) 
            .returning(move || Box::pin(async move { Ok(expected_connections_clone1.clone()) }));
        
        let expected_connections_clone2 = sample_connections.clone();
        mock_dbus.expect_get_connections()
            .times(1) 
            .returning(move || Box::pin(async move { Ok(expected_connections_clone2.clone()) }));

        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));

        let (integration, _dbus_event_tx) = setup_test_integration_with_mock(mock_dbus);

        let connections = integration.get_connections().await.unwrap();
        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].id(), "conn1_uuid");
        
        let conn1 = integration.get_connection("conn1_uuid").await.unwrap();
        assert_eq!(conn1.name(), "Conn1");
    }
    
    #[tokio::test]
    async fn test_nmi_connect_disconnect_success_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new();
        mock_dbus.expect_connect()
            .with(predicate::eq("test_id"))
            .times(1)
            .returning(|_| Box::pin(async {Ok(())}));
        mock_dbus.expect_disconnect()
            .with(predicate::eq("test_id"))
            .times(1)
            .returning(|_| Box::pin(async {Ok(())}));
        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));

        let (integration, _dbus_event_tx) = setup_test_integration_with_mock(mock_dbus);

        assert!(integration.connect("test_id").await.is_ok());
        assert!(integration.disconnect("test_id").await.is_ok());
    }

    #[tokio::test]
    async fn test_nmi_has_connectivity_true_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new();
        mock_dbus.expect_has_connectivity()
            .times(1)
            .returning(|| Box::pin(async {Ok(true)}));
        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));
        
        let (integration, _dbus_event_tx) = setup_test_integration_with_mock(mock_dbus);
        assert_eq!(integration.has_connectivity().await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_nmi_event_forwarding_and_cache_update_on_state_change_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new();
        
        let initial_connections = vec![
            NetworkConnection::new("conn1_uuid", "Conn1", NetworkConnectionType::Wired, NetworkConnectionState::Disconnected, false, None, None),
        ];
        let initial_connections_clone = initial_connections.clone();
        mock_dbus.expect_get_connections()
            .times(1) 
            .returning(move |_| Box::pin(async move { Ok(initial_connections_clone.clone()) }));
        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));

        let (integration, dbus_internal_event_tx) = setup_test_integration_with_mock(mock_dbus);
        let mut client_event_rx = integration.subscribe().await.unwrap();

        let initial_conn_state = integration.get_connection("conn1_uuid").await.unwrap().state();
        assert_eq!(initial_conn_state, NetworkConnectionState::Disconnected);

        let event_to_send = NetworkEvent {
            event_type: NetworkEventType::ConnectionStateChanged,
            connection_id: Some("conn1_uuid".to_string()),
            state: Some(NetworkConnectionState::Connected),
            has_connectivity: None,
        };
        dbus_internal_event_tx.send(event_to_send.clone()).await.unwrap();

        match tokio::time::timeout(Duration::from_millis(200), client_event_rx.recv()).await {
            Ok(Some(received_event)) => {
                assert_eq!(received_event, event_to_send);
            }
            _ => panic!("Timeout or error receiving ConnectionStateChanged event"),
        }
        
        tokio::time::sleep(Duration::from_millis(50)).await; 
        let updated_conn_state = integration.connection_cache.lock().await.get("conn1_uuid").unwrap().state();
        assert_eq!(updated_conn_state, NetworkConnectionState::Connected);
    }
    
    #[tokio::test]
    async fn test_nmi_event_forwarding_for_connectivity_changed_with_mock() {
        let mut mock_dbus = MockNetworkDBusOperations::new();
        mock_dbus.expect_start_signal_handlers().times(1).returning(|| Box::pin(async {Ok(())}));
        mock_dbus.expect_get_connections().returning(|| Box::pin(async {Ok(vec![])})). Mtimes(0..);

        let (integration, dbus_internal_event_tx) = setup_test_integration_with_mock(mock_dbus);
        let mut client_event_rx = integration.subscribe().await.unwrap();

        let event_to_send = NetworkEvent {
            event_type: NetworkEventType::ConnectivityChanged,
            connection_id: None, state: None, has_connectivity: Some(true),
        };
        dbus_internal_event_tx.send(event_to_send.clone()).await.unwrap();

        match tokio::time::timeout(Duration::from_millis(100), client_event_rx.recv()).await {
            Ok(Some(received_event)) => { assert_eq!(received_event, event_to_send); }
            _ => panic!("Failed to receive ConnectivityChanged event or timeout"),
        }
    }

    // --- Original integration tests (marked ignore, renamed for clarity) ---
    async fn create_test_dbus_connection_live() -> Option<Arc<NetworkDBusConnection>> { 
        let (tx, _) = tokio_mpsc::channel(10); 
        NetworkDBusConnection::new(tx).ok().map(Arc::new)
    }

    #[tokio::test]
    #[ignore] 
    async fn test_dbus_get_connections_live() { 
        if let Some(dbus_conn) = create_test_dbus_connection_live().await {
            match dbus_conn.get_connections().await {
                Ok(connections) => {
                    println!("Found {} active connections:", connections.len());
                    for conn in connections {
                        println!("  ID: {}, Name: {}, Type: {:?}, State: {:?}, Default: {}, Strength: {:?}, Speed: {:?}", 
                                 conn.id(), conn.name(), conn.connection_type(), conn.state(), conn.is_default(), conn.strength(), conn.speed());
                    }
                }
                Err(e) => panic!("Failed to get connections: {}", e),
            }
        } else { println!("Skipping test_dbus_get_connections_live: D-Bus connection failed."); }
    }

    #[tokio::test]
    #[ignore] 
    async fn test_dbus_has_connectivity_live() { 
        if let Some(dbus_conn) = create_test_dbus_connection_live().await {
            match dbus_conn.has_connectivity().await {
                Ok(has_conn) => println!("System has connectivity: {}", has_conn),
                Err(e) => panic!("Failed to check connectivity: {}", e),
            }
        } else { println!("Skipping test_dbus_has_connectivity_live: D-Bus connection failed."); }
    }
    
    #[tokio::test]
    #[ignore] 
    async fn test_networkmanager_integration_init_and_subscribe_live() { 
        match NetworkManagerIntegration::new_production() { 
            Ok(manager) => {
                println!("NetworkManagerIntegration initialized successfully.");
                let mut rx = manager.subscribe().await.unwrap();
                tokio::spawn(async move {
                    for _ in 0..2 { 
                        tokio::select! {
                            event = rx.recv() => {
                                if let Some(ev) = event { println!("Received event via subscribe: {:?}", ev); } 
                                else { println!("Subscriber channel closed."); break; }
                            }
                            _ = tokio::time::sleep(Duration::from_secs(5)) => { println!("Timeout waiting for event via subscribe."); break; }
                        }
                    }
                });
                tokio::time::sleep(Duration::from_secs(12)).await;
            }
            Err(e) => { println!("NetworkManagerIntegration init failed (may be expected in CI): {}", e); }
        }
    }
}
