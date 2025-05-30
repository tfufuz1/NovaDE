// novade-ui/src/notification_client/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use zbus::{Connection, Proxy, Result as ZbusResult, Error as ZbusError};
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
#[derive(Debug, serde::Deserialize, zbus::SignalArgs, Clone)]
pub struct ActionInvokedArgs {
    pub id: u32,
    pub action_key: String,
}

// Struct for NotificationClosed signal from org.freedesktop.Notifications
#[derive(Debug, serde::Deserialize, zbus::SignalArgs, Clone)]
pub struct NotificationClosedArgs {
    pub id: u32,
    pub reason: u32,
}


#[derive(Clone)]
pub struct NotificationClient {
    connection: Arc<Connection>, // Arc allows cloning the client cheaply
    proxy: Arc<Proxy<'static>>, // Proxy can be cloned if connection is Arc'd
}

impl NotificationClient {
    const NOTIFICATION_SERVICE: &'static str = "org.freedesktop.Notifications";
    const NOTIFICATION_PATH: &'static str = "/org/freedesktop/Notifications";
    const NOTIFICATION_INTERFACE: &'static str = "org.freedesktop.Notifications";

    pub async fn new() -> ClientResult<Self> {
        debug!("Creating new NotificationClient");
        let connection = Connection::session().await?; // Usually on session bus
        let proxy = Proxy::new(
            &connection,
            Self::NOTIFICATION_SERVICE,
            Self::NOTIFICATION_PATH,
            Self::NOTIFICATION_INTERFACE,
        )
        .await
        .map_err(NotificationClientError::Proxy)?;
        
        info!("NotificationClient connected to D-Bus session bus and proxy created for {}", Self::NOTIFICATION_SERVICE);
        Ok(Self {
            connection: Arc::new(connection),
            proxy: Arc::new(proxy), // Store as Arc<Proxy>
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
        // Other hints can be added here: category, desktop-entry, image-data, sound-file, suppress-sound etc.

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
            .body()?; // Ensure body is ().
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
    
    // --- Signal Listening ---

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
                    // Decide if this is fatal or if we should try to re-establish
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

    // Example of monitoring server availability (optional, advanced)
    pub async fn monitor_server_availability<F_available, F_unavailable>(
        &self,
        mut on_available: F_available,
        mut on_unavailable: F_unavailable,
    ) -> ClientResult<()>
    where
        F_available: FnMut() + Send + 'static,
        F_unavailable: FnMut() + Send + 'static,
    {
        debug!("Setting up server availability monitor for {}", Self::NOTIFICATION_SERVICE);
        let dbus_proxy = Proxy::new(
            &self.connection,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        )
        .await
        .map_err(NotificationClientError::Proxy)?;

        let mut initial_owner_res: ZbusResult<String> = dbus_proxy.call_method("GetNameOwner", &(Self::NOTIFICATION_SERVICE,)).await
            .map_err(|e| NotificationClientError::MethodCall{method_name: "GetNameOwner".to_string(), source: e})?
            .body();
        
        if initial_owner_res.is_ok() && !initial_owner_res.as_ref().unwrap().is_empty() {
            info!("Notification server {} is initially available.", Self::NOTIFICATION_SERVICE);
            on_available();
        } else {
            warn!("Notification server {} is initially unavailable.", Self::NOTIFICATION_SERVICE);
            on_unavailable();
        }

        let mut name_owner_changed_stream = dbus_proxy
            .receive_signal_with_args::<NameOwnerChangedSignal>("NameOwnerChanged")
            .await
            .map_err(NotificationClientError::Proxy)?;
        
        info!("Monitoring NameOwnerChanged signals for {}", Self::NOTIFICATION_SERVICE);
        while let Some(signal_result) = name_owner_changed_stream.next().await {
             match signal_result {
                Ok(signal) => {
                    match signal.args() {
                        Ok(args) => {
                            if args.name == Self::NOTIFICATION_SERVICE {
                                if args.new_owner.is_empty() && !args.old_owner.is_empty() {
                                    warn!("Notification server {} became unavailable ({} released by {}).", Self::NOTIFICATION_SERVICE, args.name, args.old_owner);
                                    on_unavailable();
                                } else if !args.new_owner.is_empty() && args.old_owner.is_empty() {
                                    info!("Notification server {} became available ({} acquired by {}).", Self::NOTIFICATION_SERVICE, args.name, args.new_owner);
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
        warn!("NameOwnerChanged signal stream ended for {}.", Self::NOTIFICATION_SERVICE);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc; // For sending results from signal handlers
    use tokio;

    // Helper to run a basic test server (simplified version of the actual server)
    // This is very basic and only for testing client calls.
    struct TestNotificationServer {
        // last_received_summary: Arc<Mutex<Option<String>>>,
    }

    #[zbus::dbus_interface(name = "org.freedesktop.Notifications")]
    impl TestNotificationServer {
        async fn Notify(
            &self,
            app_name: String,
            replaces_id: u32,
            app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, Value<'_>>,
            expire_timeout: i32,
        ) -> zbus::fdo::Result<u32> {
            debug!("[TestServer] Notify called: summary={}", summary);
            // let mut guard = self.last_received_summary.lock().await;
            // *guard = Some(summary);
            Ok(12345) // Return a dummy ID
        }

        async fn CloseNotification(&self, id: u32) -> zbus::fdo::Result<()> {
            debug!("[TestServer] CloseNotification called for id={}", id);
            Ok(())
        }
        
        async fn GetCapabilities(&self) -> zbus::fdo::Result<Vec<String>> {
            Ok(vec!["body".to_string(), "actions".to_string()])
        }

        async fn GetServerInformation(&self) -> zbus::fdo::Result<(String, String, String, String)> {
            Ok(("TestServer".into(), "TestVendor".into(), "1.0".into(), "1.2".into()))
        }

        // Signals for testing client's listening capabilities
        #[dbus_interface(signal)]
        async fn action_invoked(context: &zbus::SignalContext<'_>, id: u32, action_key: String) -> zbus::Result<()>;
        #[dbus_interface(signal)]
        async fn notification_closed(context: &zbus::SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;
    }


    // Note: These tests require a D-Bus session bus to be available.
    // They will attempt to connect to a real or test notification server.
    // For CI, these might need to be conditional or use a mocked D-Bus environment.

    #[tokio::test]
    #[ignore] // Ignored by default as it needs a D-Bus server (either real or the test one below)
    async fn test_notification_client_notify_and_close() {
        // This test would ideally spawn a temporary TestNotificationServer
        // For now, it assumes some server is running or will fail.
        let client = NotificationClient::new().await;
        if let Err(e) = &client {
            warn!("Failed to create NotificationClient (is a D-Bus session bus available?): {:?}. Skipping test.", e);
            return;
        }
        let client = client.unwrap();

        let notify_result = client.notify(
            "TestApp",
            0,
            "test-icon",
            "Test Summary from Client",
            "Test body from client.",
            vec![UIAction::new("id1", "Action 1")],
            UIPriority::Normal,
            5000,
        ).await;

        assert!(notify_result.is_ok(), "Notify failed: {:?}", notify_result.err());
        let notification_id = notify_result.unwrap();
        assert!(notification_id > 0);

        let close_result = client.close_notification(notification_id).await;
        assert!(close_result.is_ok(), "CloseNotification failed: {:?}", close_result.err());
    }
    
    #[tokio::test]
    #[ignore] // Ignored by default
    async fn test_get_capabilities_and_info() {
        let client = NotificationClient::new().await;
         if let Err(e) = &client {
            warn!("Failed to create NotificationClient: {:?}. Skipping test.", e);
            return;
        }
        let client = client.unwrap();

        let caps_result = client.get_capabilities().await;
        assert!(caps_result.is_ok(), "GetCapabilities failed: {:?}", caps_result.err());
        // assert!(caps_result.unwrap().contains(&"body".to_string())); // Depends on actual server

        let info_result = client.get_server_information().await;
        assert!(info_result.is_ok(), "GetServerInformation failed: {:?}", info_result.err());
        // let (name, vendor, version, spec_version) = info_result.unwrap();
        // assert_eq!(name, "SomeServerName"); // Depends on actual server
    }

    // Test for signal listening - requires a server that can emit signals
    // This test is more complex to set up correctly without a full mock framework or a real server.
    #[tokio::test]
    #[ignore] // Ignored by default
    async fn test_listen_for_action_invoked() {
        // Setup: Spawn a test server that can emit ActionInvoked
        let server_logic = TestNotificationServer {};
        let conn_build_res = zbus::ConnectionBuilder::session()
            .unwrap()
            .name("org.freedesktop.Notifications.TestClient") // Unique name for test server
            .unwrap()
            .serve_at("/org/freedesktop/Notifications", server_logic)
            .unwrap()
            .build()
            .await;
        
        if conn_build_res.is_err() {
            warn!("Failed to start test D-Bus server: {:?}. Skipping signal test.", conn_build_res.err());
            return;
        }
        let server_conn = conn_build_res.unwrap();
        let server_signal_context = zbus::SignalContext::new(&server_conn, "/org/freedesktop/Notifications").unwrap();


        // Client part
        let client = NotificationClient::new().await;
        if let Err(e) = &client {
            warn!("Failed to create NotificationClient: {:?}. Skipping signal test.", e);
            return;
        }
        let client = client.unwrap();
        
        let (tx, rx) = mpsc::channel::<ActionInvokedArgs>();

        let client_clone = client.clone();
        tokio::spawn(async move {
            let _ = client_clone.receive_action_invoked_signals(move |args| {
                info!("[TestClient] Received ActionInvoked: {:?}", args);
                tx.send(args).expect("Failed to send ActionInvokedArgs via mpsc");
            }).await;
        });

        // Give listener a moment to connect
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Trigger the signal from the test server
        debug!("[TestSetup] Emitting ActionInvoked signal from test server...");
        let emit_res = TestNotificationServer::action_invoked(&server_signal_context, 123, "test_action".to_string()).await;
        assert!(emit_res.is_ok(), "Failed to emit signal from test server: {:?}", emit_res.err());
        debug!("[TestSetup] Signal emitted.");

        // Check if the client received it
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(args) => {
                assert_eq!(args.id, 123);
                assert_eq!(args.action_key, "test_action");
            }
            Err(e) => {
                panic!("Did not receive ActionInvoked signal in time: {}", e);
            }
        }
    }
}
