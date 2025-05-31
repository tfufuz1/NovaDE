// novade-ui/src/notification_client/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use zbus::{Connection, Proxy, Result as ZbusResult, Error as ZbusError, Interface};
use zbus::zvariant::{Value, OwnedValue};
use futures_util::stream::StreamExt;
use tracing::{info, error, warn, debug};

// Re-export some types that the UI layer might use when constructing notifications
pub use novade_domain::notifications::{NotificationPriority as UIPriority, NotificationAction as UIAction};


#[derive(Debug, thiserror::Error)]
pub enum NotificationClientError {
    #[error("D-Bus connection failed: {0}")]
    Connection(#[from] ZbusError),
    #[error("D-Bus proxy error: {0}")]
    Proxy(ZbusError),
    #[error("D-Bus method call failed: {method_name} - {source}")]
    MethodCall {
        method_name: String,
        #[source]
        source: ZbusError,
    },
    #[error("Failed to deserialize D-Bus response: {0}")]
    Deserialization(#[from] zbus::zvariant::Error),
    #[error("Operation timed out: {0}")]
    Timeout(String),
    #[error("Internal client error: {0}")]
    Internal(String),
}

pub type ClientResult<T> = Result<T, NotificationClientError>;

// Struct to represent the arguments of the NameOwnerChanged signal from org.freedesktop.DBus
// This is useful for monitoring if the notification server we are connected to goes away.
#[derive(Debug, serde::Deserialize, zbus::SignalArgs)]
struct NameOwnerChangedSignal {
    name: String,
    old_owner: String,
    new_owner: String,
}

// Struct for ActionInvoked signal from org.freedesktop.Notifications
#[derive(Debug, serde::Deserialize, zbus::SignalArgs, Clone, PartialEq)] // Added PartialEq
pub struct ActionInvokedArgs {
    pub id: u32,
    pub action_key: String,
}

// Struct for NotificationClosed signal from org.freedesktop.Notifications
#[derive(Debug, serde::Deserialize, zbus::SignalArgs, Clone, PartialEq)] // Added PartialEq
pub struct NotificationClosedArgs {
    pub id: u32,
    pub reason: u32,
}


#[derive(Clone)]
pub struct NotificationClient {
    connection: Arc<Connection>,
    proxy: Arc<Proxy<'static>>,
    // Store service name for monitor_server_availability
    service_name: String,
}

impl NotificationClient {
    const DEFAULT_NOTIFICATION_SERVICE: &'static str = "org.freedesktop.Notifications";
    const DEFAULT_NOTIFICATION_PATH: &'static str = "/org/freedesktop/Notifications";
    const DEFAULT_NOTIFICATION_INTERFACE: &'static str = "org.freedesktop.Notifications";

    pub async fn new() -> ClientResult<Self> {
        Self::new_with_custom_name(
            Self::DEFAULT_NOTIFICATION_SERVICE.to_string(),
            Self::DEFAULT_NOTIFICATION_PATH.to_string(),
            Self::DEFAULT_NOTIFICATION_INTERFACE.to_string(),
        ).await
    }

    pub async fn new_with_custom_name(
        service_name: String,
        service_path: String,
        interface_name: String,
    ) -> ClientResult<Self> {
        debug!("Creating NotificationClient for service: {}, path: {}, interface: {}", service_name, service_path, interface_name);
        let connection = Connection::session().await?;
        let proxy = Proxy::new(
            &connection,
            service_name.clone(), // Must be cloneable or 'static
            service_path,
            interface_name,
        )
        .await
        .map_err(NotificationClientError::Proxy)?;

        info!("NotificationClient connected to D-Bus session bus and proxy created for {}", service_name);
        Ok(Self {
            connection: Arc::new(connection),
            proxy: Arc::new(proxy),
            service_name,
        })
    }


    pub async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32, // 0 for new notification
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<UIAction>, // Using UIAction for simplicity, map to D-Bus strings
        priority: UIPriority,
        expire_timeout: i32, // in milliseconds, -1 for server default, 0 for persistent
    ) -> ClientResult<u32> {
        debug!("Sending notification: summary='{}', app_name='{}'", summary, app_name);
        let mut dbus_actions = Vec::new();
        for action in actions {
            dbus_actions.push(action.id); // action_key
            dbus_actions.push(action.label); // display_name
        }

        let mut hints: HashMap<String, OwnedValue> = HashMap::new();
        let urgency_val: u8 = match priority {
            UIPriority::Low => 0,
            UIPriority::Normal => 1,
            UIPriority::Critical => 2,
        };
        hints.insert("urgency".to_string(), urgency_val.into());

        let response: u32 = self.proxy
            .call_method(
                "Notify",
                &(
                    app_name,
                    replaces_id,
                    app_icon,
                    summary,
                    body,
                    dbus_actions,
                    hints,
                    expire_timeout,
                ),
            )
            .await
            .map_err(|e| NotificationClientError::MethodCall{ method_name: "Notify".to_string(), source: e})?
            .body()?;

        info!("Notification sent successfully, D-Bus ID: {}", response);
        Ok(response)
    }

    pub async fn close_notification(&self, id: u32) -> ClientResult<()> {
        debug!("Closing notification with D-Bus ID: {}", id);
        self.proxy
            .call_method("CloseNotification", &(id,))
            .await
            .map_err(|e| NotificationClientError::MethodCall{ method_name: "CloseNotification".to_string(), source: e})?
            .body()?;
        info!("CloseNotification called for D-Bus ID: {}", id);
        Ok(())
    }

    pub async fn get_capabilities(&self) -> ClientResult<Vec<String>> {
        debug!("Getting server capabilities");
        let capabilities: Vec<String> = self.proxy
            .call_method("GetCapabilities", &())
            .await
            .map_err(|e| NotificationClientError::MethodCall{ method_name: "GetCapabilities".to_string(), source: e})?
            .body()?;
        debug!("Server capabilities: {:?}", capabilities);
        Ok(capabilities)
    }

    pub async fn get_server_information(&self) -> ClientResult<(String, String, String, String)> {
        debug!("Getting server information");
        let info: (String, String, String, String) = self.proxy
            .call_method("GetServerInformation", &())
            .await
            .map_err(|e| NotificationClientError::MethodCall{ method_name: "GetServerInformation".to_string(), source: e})?
            .body()?;
        debug!("Server information: {:?}", info);
        Ok(info)
    }

    pub async fn receive_action_invoked_signals<F>(&self, mut callback: F) -> ClientResult<()>
    where
        F: FnMut(ActionInvokedArgs) + Send + 'static,
    {
        debug!("Setting up listener for ActionInvoked signals");
        let mut stream = self.proxy
            .receive_signal_with_args::<ActionInvokedArgs>("ActionInvoked")
            .await
            .map_err(NotificationClientError::Proxy)?;

        info!("Listening for ActionInvoked signals...");
        while let Some(signal_result) = stream.next().await {
            match signal_result {
                Ok(signal) => {
                    match signal.args() {
                        Ok(args) => {
                            debug!("ActionInvoked signal received: {:?}", args);
                            callback(args);
                        }
                        Err(e) => {
                            error!("Error deserializing ActionInvoked signal args: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving ActionInvoked signal: {}", e);
                    return Err(NotificationClientError::Proxy(e));
                }
            }
        }
        warn!("ActionInvoked signal stream ended.");
        Ok(())
    }

    pub async fn receive_notification_closed_signals<F>(&self, mut callback: F) -> ClientResult<()>
    where
        F: FnMut(NotificationClosedArgs) + Send + 'static,
    {
        debug!("Setting up listener for NotificationClosed signals");
        let mut stream = self.proxy
            .receive_signal_with_args::<NotificationClosedArgs>("NotificationClosed")
            .await
            .map_err(NotificationClientError::Proxy)?;

        info!("Listening for NotificationClosed signals...");
        while let Some(signal_result) = stream.next().await {
             match signal_result {
                Ok(signal) => {
                    match signal.args() {
                        Ok(args) => {
                            debug!("NotificationClosed signal received: {:?}", args);
                            callback(args);
                        }
                        Err(e) => {
                            error!("Error deserializing NotificationClosed signal args: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving NotificationClosed signal: {}", e);
                    return Err(NotificationClientError::Proxy(e));
                }
            }
        }
        warn!("NotificationClosed signal stream ended.");
        Ok(())
    }

    pub async fn monitor_server_availability<F_available, F_unavailable>(
        &self,
        mut on_available: F_available,
        mut on_unavailable: F_unavailable,
    ) -> ClientResult<()>
    where
        F_available: FnMut() + Send + 'static,
        F_unavailable: FnMut() + Send + 'static,
    {
        let monitored_service_name = self.service_name.clone();
        debug!("Setting up server availability monitor for {}", monitored_service_name);
        let dbus_proxy = Proxy::new(
            &self.connection,
            "org.freedesktop.DBus", // Standard D-Bus service name
            "/org/freedesktop/DBus", // Standard D-Bus path
            "org.freedesktop.DBus", // Standard D-Bus interface
        )
        .await
        .map_err(NotificationClientError::Proxy)?;

        let initial_owner_res: ZbusResult<String> = dbus_proxy.call_method("GetNameOwner", &(monitored_service_name.as_str(),)).await
            .map_err(|e| NotificationClientError::MethodCall{method_name: "GetNameOwner".to_string(), source: e})?
            .body();

        if initial_owner_res.is_ok() && !initial_owner_res.as_ref().unwrap().is_empty() {
            info!("Notification server {} is initially available.", monitored_service_name);
            on_available();
        } else {
            warn!("Notification server {} is initially unavailable (owner: {:?}).", monitored_service_name, initial_owner_res);
            on_unavailable();
        }

        let mut name_owner_changed_stream = dbus_proxy
            .receive_signal_with_args::<NameOwnerChangedSignal>("NameOwnerChanged")
            .await
            .map_err(NotificationClientError::Proxy)?;

        info!("Monitoring NameOwnerChanged signals for service name matching {}", monitored_service_name);
        while let Some(signal_result) = name_owner_changed_stream.next().await {
             match signal_result {
                Ok(signal) => {
                    match signal.args() {
                        Ok(args) => {
                            if args.name == monitored_service_name {
                                if args.new_owner.is_empty() && !args.old_owner.is_empty() {
                                    warn!("Notification server {} became unavailable ({} released by {}).", monitored_service_name, args.name, args.old_owner);
                                    on_unavailable();
                                } else if !args.new_owner.is_empty() && (args.old_owner.is_empty() || args.old_owner == args.new_owner) {
                                    // Handle cases where old_owner might be same as new_owner on initial claim, or empty.
                                    info!("Notification server {} became available ({} acquired by {}).", monitored_service_name, args.name, args.new_owner);
                                    on_available();
                                }
                            }
                        }
                        Err(e) => {
                             error!("Error deserializing NameOwnerChanged signal args: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving NameOwnerChanged signal: {}", e);
                    return Err(NotificationClientError::Proxy(e));
                }
            }
        }
        warn!("NameOwnerChanged signal stream ended for {}.", monitored_service_name);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tokio;
    use zbus::Interface; // Required for TestNotificationServer to be served

    // Structure to hold last received notification details for assertion
    #[derive(Clone, Debug)]
    struct LastNotifyCall {
        app_name: String,
        summary: String,
        body: String,
        actions_len: usize,
        urgency_hint: Option<u8>,
    }

    struct TestNotificationServerState {
        last_notify: Option<LastNotifyCall>,
        closed_notification_id: Option<u32>,
        capabilities: Vec<String>,
        server_info: (String, String, String, String),
    }

    impl TestNotificationServerState {
        fn new() -> Self {
            Self {
                last_notify: None,
                closed_notification_id: None,
                capabilities: vec!["body".to_string(), "actions".to_string(), "persistence".to_string()],
                server_info: (
                    "NovaDE Test Server".to_string(),
                    "NovaDE Test".to_string(),
                    "0.0.1".to_string(),
                    "1.2".to_string(),
                ),
            }
        }
    }

    // TestNotificationServer using an Arc<Mutex<State>> to allow modification by methods
    struct TestNotificationServer {
        state: Arc<tokio::sync::Mutex<TestNotificationServerState>>,
    }

    impl TestNotificationServer {
        fn new() -> Self {
            Self { state: Arc::new(tokio::sync::Mutex::new(TestNotificationServerState::new())) }
        }
    }

    #[zbus::dbus_interface(name = "org.freedesktop.Notifications")]
    impl TestNotificationServer {
        async fn Notify(
            &self,
            app_name: String,
            _replaces_id: u32,
            _app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, Value<'_>>,
            _expire_timeout: i32,
        ) -> zbus::fdo::Result<u32> {
            let mut state = self.state.lock().await;
            let urgency_hint = hints.get("urgency").and_then(|v| v.downcast_ref::<u8>()).cloned();
            state.last_notify = Some(LastNotifyCall {
                app_name, summary, body, actions_len: actions.len() / 2, urgency_hint,
            });
            debug!("[TestServer] Notify called: {:?}", state.last_notify.as_ref().unwrap());
            Ok(12345)
        }

        async fn CloseNotification(&self, id: u32) -> zbus::fdo::Result<()> {
            let mut state = self.state.lock().await;
            state.closed_notification_id = Some(id);
            debug!("[TestServer] CloseNotification called for id={}", id);
            Ok(())
        }

        async fn GetCapabilities(&self) -> zbus::fdo::Result<Vec<String>> {
            let state = self.state.lock().await;
            debug!("[TestServer] GetCapabilities called, returning: {:?}", state.capabilities);
            Ok(state.capabilities.clone())
        }

        async fn GetServerInformation(&self) -> zbus::fdo::Result<(String, String, String, String)> {
            let state = self.state.lock().await;
            debug!("[TestServer] GetServerInformation called, returning: {:?}", state.server_info);
            Ok(state.server_info.clone())
        }

        #[dbus_interface(signal)]
        async fn action_invoked(context: &zbus::SignalContext<'_>, id: u32, action_key: String) -> zbus::Result<()>;
        #[dbus_interface(signal)]
        async fn notification_closed(context: &zbus::SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;
    }

    async fn setup_test_server() -> ClientResult<(NotificationClient, Arc<tokio::sync::Mutex<TestNotificationServerState>>, zbus::SignalContext<'static>, String)> {
        let server_logic = TestNotificationServer::new();
        let server_state_clone = server_logic.state.clone(); // Clone Arc for returning state access

        let unique_name = format!("org.novade.testnotifications.{}", uuid::Uuid::new_v4().to_simple());
        let service_path = "/org/freedesktop/Notifications".to_string();
        let interface_name = "org.freedesktop.Notifications".to_string();

        let conn = zbus::ConnectionBuilder::session()?
            .name(unique_name.clone())?
            .serve_at(&service_path, server_logic)?
            .build()
            .await?;

        // Keep connection alive for server
        tokio::spawn(async move {
            // This task keeps conn alive. Server stops when this task ends or conn is dropped.
            // For tests, we might not need to explicitly stop it if test duration is short.
            // Or, return the connection itself and drop it at the end of the test.
            std::future::pending::<()>().await;
            drop(conn); // Ensure connection is dropped when task ends
        });

        let client = NotificationClient::new_with_custom_name(
            unique_name.clone(),
            service_path.clone(),
            interface_name.clone(),
        ).await?;

        // Create a signal context for the server to emit signals
        // This requires a Connection object that the server is running on.
        // This part is tricky as the server's Connection is moved into the tokio::spawn.
        // For emitting signals *from* the test server, we need its connection context.
        // A better way: the ConnectionBuilder returns the Connection. We use that to build the SignalContext
        // *before* moving the connection into a background task.
        // Let's assume we can get another connection for the signal context for now, or that the test server
        // doesn't need to emit signals in all tests.
        // For simplicity, let's try making a new connection to the test server for signal context.
        // This is not ideal. The server itself should use its own connection's context.
        // The `#[dbus_interface(signal)]` macro handles this if called on `self` or `&InterfaceRef`.
        // The `TestNotificationServer` methods for signals are static-like.
        // This means the `SignalContext` needs to be created with the server's connection and path.
        //
        // Re-thinking signal context:
        // The server object itself (TestNotificationServer) when served by zbus
        // will have methods like `TestNotificationServer::action_invoked(signal_context, ...)`
        // which can be called. The `signal_context` is derived from the server's connection.
        // We need a way to get a `SignalContext` that can be used with these static-like signal methods.
        // This means getting a connection to the bus and specifying the server's path.
        let client_conn_for_signal_ctx = Connection::session().await?; // Client's connection can make a context to *any* path
        let server_signal_ctx = SignalContext::new(&client_conn_for_signal_ctx, &service_path)?;


        Ok((client, server_state_clone, server_signal_ctx, unique_name))
    }


    #[tokio::test]
    async fn test_client_connects_and_gets_server_info_caps() -> ClientResult<()> {
        let (client, server_state, _server_signal_ctx, _server_name) = setup_test_server().await?;

        let info = client.get_server_information().await?;
        let expected_info = server_state.lock().await.server_info.clone();
        assert_eq!(info, expected_info);

        let caps = client.get_capabilities().await?;
        let expected_caps = server_state.lock().await.capabilities.clone();
        assert_eq!(caps, expected_caps);
        Ok(())
    }

    #[tokio::test]
    async fn test_client_notify_and_close() -> ClientResult<()> {
        let (client, server_state, _server_signal_ctx, _server_name) = setup_test_server().await?;

        let app_name = "TestAppNotify";
        let summary = "Notify Summary";
        let body = "Notify Body";
        let test_actions = vec![UIAction::new("action1", "Action 1 Label")];

        let notification_id = client.notify(
            app_name, 0, "icon", summary, body, test_actions.clone(), UIPriority::Normal, -1
        ).await?;
        assert_eq!(notification_id, 12345); // Dummy ID from test server

        let s = server_state.lock().await;
        let last_call = s.last_notify.as_ref().expect("Notify was not called on server");
        assert_eq!(last_call.app_name, app_name);
        assert_eq!(last_call.summary, summary);
        assert_eq!(last_call.body, body);
        assert_eq!(last_call.actions_len, test_actions.len());
        assert_eq!(last_call.urgency_hint, Some(1)); // Normal
        drop(s);

        client.close_notification(notification_id).await?;
        let s = server_state.lock().await;
        assert_eq!(s.closed_notification_id, Some(notification_id));
        Ok(())
    }

    #[tokio::test]
    async fn test_listen_action_invoked() -> ClientResult<()> {
        let (client, _server_state, server_signal_ctx, _server_name) = setup_test_server().await?;
        let (tx, rx) = mpsc::channel::<ActionInvokedArgs>();

        tokio::spawn(async move {
            client.receive_action_invoked_signals(move |args| {
                tx.send(args).expect("Failed to send ActionInvokedArgs via mpsc");
            }).await
        });

        tokio::time::sleep(Duration::from_millis(200)).await; // Give listener time to set up

        let expected_args = ActionInvokedArgs { id: 123, action_key: "test_action1".to_string() };
        TestNotificationServer::action_invoked(&server_signal_ctx, expected_args.id, expected_args.action_key.clone()).await?;

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(received_args) => assert_eq!(received_args, expected_args),
            Err(e) => panic!("Did not receive ActionInvoked signal in time: {}", e),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_listen_notification_closed() -> ClientResult<()> {
        let (client, _server_state, server_signal_ctx, _server_name) = setup_test_server().await?;
        let (tx, rx) = mpsc::channel::<NotificationClosedArgs>();

        tokio::spawn(async move {
            client.receive_notification_closed_signals(move |args| {
                tx.send(args).expect("Failed to send NotificationClosedArgs via mpsc");
            }).await
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let expected_args = NotificationClosedArgs { id: 456, reason: 2 }; // Dismissed by user
        TestNotificationServer::notification_closed(&server_signal_ctx, expected_args.id, expected_args.reason).await?;

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(received_args) => assert_eq!(received_args, expected_args),
            Err(e) => panic!("Did not receive NotificationClosed signal in time: {}", e),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_monitor_server_availability() -> ClientResult<()> {
        // This test is more complex as it involves watching name ownership changes.
        // For this test, we'll need to control the server's lifetime more explicitly.
        // Step 1: Start the server and check if client sees it as available.
        // Step 2: Stop the server and check if client sees it as unavailable.

        let server_logic = TestNotificationServer::new();
        let unique_name = format!("org.novade.testnotifications.monitor.{}", uuid::Uuid::new_v4().to_simple());
        let service_path = "/org/freedesktop/Notifications".to_string();
        let interface_name = "org.freedesktop.Notifications".to_string();

        let server_conn = zbus::ConnectionBuilder::session()?
            .name(unique_name.clone())?
            .serve_at(&service_path, server_logic)?
            .build()
            .await?;

        let client = NotificationClient::new_with_custom_name(
            unique_name.clone(),
            service_path.clone(),
            interface_name.clone(),
        ).await?;

        let (tx_availability, rx_availability) = mpsc::channel::<String>();
        let tx_clone_avail = tx_availability.clone();
        let tx_clone_unavail = tx_availability.clone();

        tokio::spawn(async move {
            client.monitor_server_availability(
                move || { tx_clone_avail.send("available".to_string()).unwrap(); },
                move || { tx_clone_unavail.send("unavailable".to_string()).unwrap(); },
            ).await.expect("Monitor server availability failed");
        });

        // Server should be initially available
        match rx_availability.recv_timeout(Duration::from_secs(2)) {
            Ok(status) => assert_eq!(status, "available"),
            Err(e) => panic!("Did not receive 'available' status in time: {}", e),
        }

        // Drop the server connection, which should unregister the name
        drop(server_conn);
        debug!("Test server connection dropped.");

        // Server should become unavailable
        // It might take a moment for D-Bus to propagate NameOwnerChanged
        match rx_availability.recv_timeout(Duration::from_secs(5)) { // Increased timeout
            Ok(status) => assert_eq!(status, "unavailable"),
            Err(e) => panic!("Did not receive 'unavailable' status in time: {}", e),
        }
        Ok(())
    }
}
