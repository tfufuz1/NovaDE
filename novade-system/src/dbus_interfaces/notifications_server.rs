// novade-system/src/dbus_interfaces/notifications_server.rs
use std::collections::HashMap;
use std::sync::Arc;

use futures_util::lock::Mutex; // Use futures_util::lock::Mutex for async RwLock/Mutex if needed, or std::sync::RwLock if suitable
use zbus::{dbus_interface, interface, zvariant::Value, SignalContext};
use zbus::zvariant::{ObjectPath, OwnedValue, Str, Type}; // OwnedObjectPath removed, using ObjectPath

use novade_domain::notifications::{
    NotificationService, DefaultNotificationService, // Assuming DefaultNotificationService for instantiation
    Notification, NotificationId, NotificationPriority as DomainPriority, NotificationAction as DomainAction,
};
use novade_domain::notifications::provider::NotificationProvider; // For DefaultNotificationService
use novade_core::error::CoreError; // For handling errors from domain service

// Placeholder for a simple event publisher for DefaultNotificationService
fn dummy_event_publisher(_event: novade_domain::common_events::DomainEvent<novade_domain::notifications::NotificationEvent>) {
    // In a real scenario, this would integrate with a proper event bus
    // or be handled by the UI layer subscribing to signals.
    tracing::debug!("Dummy event publisher received an event.");
}

// Placeholder NotificationProvider for DefaultNotificationService
// The D-Bus server itself is the 'provider' in a sense for external apps.
// How domain notifications are displayed is a separate concern (e.g., UI listening to domain events or D-Bus signals).
#[derive(Debug, Clone)]
struct DummyNotificationProvider;

#[async_trait::async_trait]
impl NotificationProvider for DummyNotificationProvider {
    async fn send_notification(&self, _notification: &Notification) -> novade_domain::error::DomainResult<()> {
        tracing::debug!("DummyNotificationProvider: send_notification called");
        // This provider does nothing as the D-Bus server is the entry point.
        // Actual display should be triggered by events/signals from the NotificationService.
        Ok(())
    }
    async fn update_notification(&self, _notification: &Notification) -> novade_domain::error::DomainResult<()> {
        tracing::debug!("DummyNotificationProvider: update_notification called");
        Ok(())
    }
    async fn dismiss_notification(&self, _notification_id: NotificationId) -> novade_domain::error::DomainResult<()> {
        tracing::debug!("DummyNotificationProvider: dismiss_notification called");
        Ok(())
    }
    async fn perform_action(&self, _notification_id: NotificationId, _action_id: &str) -> novade_domain::error::DomainResult<()> {
        tracing::debug!("DummyNotificationProvider: perform_action called");
        Ok(())
    }
}


pub struct NotificationsServer {
    // Connection to the domain's notification service
    notification_service: Arc<dyn NotificationService>,
    // We might need to store a mapping from our internal NotificationId (UUID) to D-Bus uint32 IDs if necessary.
    // Freedesktop spec uses uint32 for Notify response and CloseNotification.
    // Let's assume for now the domain NotificationId (which seems to be a wrapper around a Uuid)
    // can be mapped or we only deal with D-Bus IDs at the D-Bus layer.
    // For simplicity, let's try to use u32 as IDs when interacting with D-Bus if the spec demands it.
    // The domain NotificationId is a UUID. The D-Bus spec returns a u32 from Notify.
    // We need a way to map these.
    dbus_id_to_domain_id: Arc<Mutex<HashMap<u32, NotificationId>>>,
    next_dbus_id: Arc<Mutex<u32>>,
}

impl NotificationsServer {
    pub fn new(notification_service: Arc<dyn NotificationService>) -> Self {
        Self {
            notification_service,
            dbus_id_to_domain_id: Arc::new(Mutex::new(HashMap::new())),
            next_dbus_id: Arc::new(Mutex::new(1)), // D-Bus notification IDs are typically > 0
        }
    }

    // Helper to get a new D-Bus ID and store the mapping
    async fn get_new_dbus_id(&self, domain_id: NotificationId) -> u32 {
        let mut next_id_guard = self.next_dbus_id.lock().await;
        let dbus_id = *next_id_guard;
        *next_id_guard += 1; // Increment for next use

        let mut mapping_guard = self.dbus_id_to_domain_id.lock().await;
        mapping_guard.insert(dbus_id, domain_id);
        dbus_id
    }

    // Helper to remove a D-Bus ID mapping
    async fn remove_dbus_id_mapping(&self, dbus_id: u32) -> Option<NotificationId> {
        let mut mapping_guard = self.dbus_id_to_domain_id.lock().await;
        mapping_guard.remove(&dbus_id)
    }

    // Helper to get domain_id from dbus_id
    async fn get_domain_id(&self, dbus_id: u32) -> Option<NotificationId> {
        let mapping_guard = self.dbus_id_to_domain_id.lock().await;
        mapping_guard.get(&dbus_id).cloned()
    }
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationsServer {
    async fn GetCapabilities(&self) -> zbus::fdo::Result<Vec<String>> {
        tracing::info!("D-Bus GetCapabilities called");
        Ok(vec![
            "body".to_string(),
            "actions".to_string(),        // Supports actions
            "persistence".to_string(),   // Supports persistence (notifications remain after app quits)
            "body-markup".to_string(),   // Supports Pango markup in body
            "icon-static".to_string(),   // Supports static icons
            // "sound".to_string(),      // If you plan to support sounds
            // "image-data".to_string(), // If you plan to support raw image data
        ])
    }

    async fn Notify(
        &self,
        app_name: String,       // Application name
        replaces_id: u32,       // Notification ID to replace (0 for none)
        app_icon: String,       // Application icon path or name
        summary: String,        // Notification title/summary
        body: String,           // Notification body
        actions: Vec<String>,   // Actions (pairs of action_key, display_name)
        hints: HashMap<String, Value<'_>>, // Hints (e.g., urgency, category)
        expire_timeout: i32,    // Expiration timeout in milliseconds (-1 for default, 0 for persistent)
        #[zbus(signal_context)] _context: SignalContext<'_>, // To emit signals, marked unused for now in tests
    ) -> zbus::fdo::Result<u32> {
        tracing::info!("D-Bus Notify called: app_name={}, summary={}", app_name, summary);

        // TODO: Handle replaces_id. If non-zero, find existing notification by this D-Bus ID,
        // update it using notification_service.update_notification.
        // If zero, create a new one.

        let mut domain_actions = Vec::new();
        for i in (0..actions.len()).step_by(2) {
            if i + 1 < actions.len() {
                domain_actions.push(DomainAction::new(&actions[i], &actions[i+1]));
            }
        }

        // Urgency hint
        let priority = hints.get("urgency")
            .and_then(|v| v.downcast_ref::<u8>())
            .map(|&u| match u {
                0 => DomainPriority::Low,
                1 => DomainPriority::Normal,
                2 => DomainPriority::Critical,
                _ => DomainPriority::Normal,
            })
            .unwrap_or(DomainPriority::Normal);

        // Create domain notification
        // The 'source' could be app_name.
        let mut notification = Notification::new(&summary, &body, &app_name);
        notification.set_priority(priority);
        notification.set_icon(app_icon); // Assuming Notification struct has set_icon
        for action in domain_actions {
            notification.add_action(action);
        }
        // TODO: Handle expire_timeout (Notification struct needs expiration logic)
        // TODO: Handle other hints like category, desktop-entry etc.

        match self.notification_service.create_notification(notification.clone()).await {
            Ok(created_notification) => {
                let domain_id = created_notification.id();
                let dbus_id = self.get_new_dbus_id(domain_id).await;
                tracing::info!("Notification created with domain_id: {:?}, assigned dbus_id: {}", domain_id, dbus_id);
                Ok(dbus_id)
            }
            Err(e) => {
                tracing::error!("Failed to create notification in domain service: {:?}", e);
                // Convert domain error to D-Bus error
                // This needs a proper error mapping. For now, using a generic D-Bus error.
                Err(zbus::fdo::Error::Failed(format!("Failed to create notification: {}", e)))
            }
        }
    }

    async fn CloseNotification(
        &self,
        id: u32, // D-Bus notification ID
        #[zbus(signal_context)] context: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("D-Bus CloseNotification called for D-Bus ID: {}", id);

        if let Some(domain_id) = self.get_domain_id(id).await {
            match self.notification_service.dismiss_notification(domain_id).await {
                Ok(_) => {
                    tracing::info!("Notification with D-Bus ID: {} (Domain ID: {:?}) dismissed.", id, domain_id);
                    self.remove_dbus_id_mapping(id).await; // Clean up mapping

                    // Emit NotificationClosed signal
                    Self::notification_closed(&context, id, 3).await // Reason 3: Dismissed by user
                        .map_err(|e| {
                            tracing::error!("Failed to emit NotificationClosed signal: {}", e);
                            // Non-fatal for the method call itself, but good to log
                            zbus::fdo::Error::Failed("Failed to emit signal".into())
                        })?;
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to dismiss notification (Domain ID: {:?}, D-Bus ID: {}): {:?}", domain_id, id, e);
                    Err(zbus::fdo::Error::Failed(format!("Failed to dismiss notification: {}", e)))
                }
            }
        } else {
            tracing::warn!("CloseNotification called for unknown D-Bus ID: {}", id);
            // Spec says: If the ID is not valid, the server should ignore the request.
            // However, some implementations might return an error. For robustness, returning an error is fine.
            Err(zbus::fdo::Error::InvalidArgs(format!("Notification ID {} not found", id)))
        }
    }

    async fn GetServerInformation(&self) -> zbus::fdo::Result<(String, String, String, String)> {
        tracing::info!("D-Bus GetServerInformation called");
        Ok((
            "NovaDE Notification Server".to_string(), // name
            "NovaDE".to_string(),                   // vendor
            "0.1.0".to_string(),                    // version
            "1.2".to_string(),                      // spec_version (of org.freedesktop.Notifications)
        ))
    }

    // --- Signals ---

    #[dbus_interface(signal)]
    async fn notification_closed(
        context: &SignalContext<'_>,
        id: u32,     // ID of the notification that was closed.
        reason: u32, // Reason for closure (1: expired, 2: dismissed by user, 3: CloseNotification, 4: undefined)
    ) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn action_invoked(
        context: &SignalContext<'_>,
        id: u32,          // ID of the notification.
        action_key: String, // Key of the action invoked.
    ) -> zbus::Result<()>;


    // TODO: Implement ActionInvoked signal emission when an action is performed.
    // This will likely involve the domain service's perform_action method and then
    // mapping back to the D-Bus ID and emitting the signal.
}

// Function to create and run the server (example, might be part of main.rs or a service manager)
// This is a simplified example of how to run the server.
// In a real application, this would be more integrated.
pub async fn run_notifications_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init(); // Basic tracing setup

    // Instantiate the domain's notification service
    let dummy_provider = Arc::new(DummyNotificationProvider);
    // The DefaultNotificationService requires an event publisher.
    // For now, we pass a dummy one. A real implementation might integrate this with D-Bus signals
    // or another event mechanism if the UI is not directly subscribing to domain events.
    let domain_notification_service = Arc::new(DefaultNotificationService::new(
        dummy_provider,
        dummy_event_publisher,
    ));

    let server_logic = NotificationsServer::new(domain_notification_service.clone());

    let _conn = zbus::ConnectionBuilder::session()? // Or system()? Check permissions. Session bus is typical for notifications.
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", server_logic)?
        .build()
        .await?;

    tracing::info!("Notification server listening on org.freedesktop.Notifications");

    // Keep the server running
    std::future::pending::<()>().await;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use novade_domain::notifications::{NotificationService, Notification, NotificationId, NotificationPriority, NotificationAction, NotificationEvent};
    use novade_domain::error::{DomainError, DomainResult}; // For mocking return types
    use std::sync::Arc;
    use zbus::zvariant::Value; // For hints in Notify
    use zbus::SignalContext; // For D-Bus methods requiring it
    use tokio::sync::broadcast; // For mocking subscribe_notifications

    // Create a dummy SignalContext for tests that need it but don't rely on real D-Bus connection for signals.
    // This is a bit of a hack. A proper test SignalContext might require more setup or a test connection.
    async fn create_dummy_signal_context() -> SignalContext<'static> {
        // This is tricky because SignalContext needs a Connection and a Path.
        // For unit tests not running a full D-Bus server, this is hard.
        // We might need to limit tests to logic not involving signal emission,
        // or accept that these tests are more integration-like if they need a real connection.
        // Let's assume for now that if a test *really* needs to emit, it would need more setup.
        // For methods that just take it as input but might not use it if erroring out early,
        // we might get away with a carefully constructed one if zbus allows.
        //
        // A more robust way for testing signal emission would be to use zbus::ConnectionBuilder::session_internal,
        // run a minimal server, and then connect a client to it to check for signals. This is more of an integration test.
        // For unit tests, we'll focus on the logic *before* Self::notification_closed calls.

        // Simplest placeholder if the test doesn't actually emit:
        // This will likely panic if `context.connection()` or `context.path()` is called.
        // So, it's only useful if the tested path doesn't try to emit a signal.
        unsafe {
            // This is highly unsafe and only for making the type checker happy.
            // It will crash if any method is called on it.
            // DO NOT USE THIS IF THE TEST PATH ACTUALLY EMITS SIGNALS.
            let null_ptr = std::ptr::null::<zbus::Connection>();
            let conn_ref: &zbus::Connection = &*null_ptr;
            SignalContext::new(conn_ref, "/org/freedesktop/Notifications").unwrap()
        }
    }


    mock! {
        pub NotificationService {
            // Matched methods from NotificationService trait in domain layer
            async fn create_notification(&self, notification: Notification) -> DomainResult<Notification>;
            async fn get_notification(&self, id: NotificationId) -> DomainResult<Option<Notification>>;
            async fn get_all_notifications(&self) -> DomainResult<Vec<Notification>>;
            async fn update_notification(&self, notification: Notification) -> DomainResult<Notification>;
            async fn dismiss_notification(&self, id: NotificationId) -> DomainResult<()>;
            async fn perform_action(&self, id: NotificationId, action_id: &str) -> DomainResult<()>;
            fn subscribe_notifications(&self) -> broadcast::Receiver<NotificationEvent>;
            // Note: publish_event is not on the trait usually, it's an internal detail of DefaultNotificationService
        }
    }
    // Implement Clone for the mock. This is often needed when Arc<MockService> is used.
    // mockall generates a struct, e.g., MockNotificationService.
    impl Clone for MockNotificationService {
        fn clone(&self) -> Self {
            // Mockall's generated mocks don't automatically derive Clone.
            // A common way is to have the mock store its expectations in an Arc<Mutex<...>>
            // and then cloning is about cloning the Arc.
            // However, for simple test setups where the mock is created per test,
            // this might not be strictly necessary unless the server logic itself clones the Arc.
            // For now, if clone is needed by Arc, we'd need a more complex mock setup.
            // Let's assume for now tests will manage ownership without needing mock to be Clone.
            // If server.clone() is used, then the mock needs to be clonable.
            // The `NotificationsServer` itself is not Clone.
            panic!("MockNotificationService cannot be cloned in this basic setup.");
        }
    }


    #[tokio::test]
    async fn test_get_capabilities() {
        let mock_service = MockNotificationService::new();
        let server = NotificationsServer::new(Arc::new(mock_service));
        let caps = server.GetCapabilities().await.unwrap();
        assert!(caps.contains(&"body".to_string()));
        assert!(caps.contains(&"actions".to_string()));
        assert!(caps.contains(&"persistence".to_string()));
    }

    #[tokio::test]
    async fn test_get_server_information() {
        let mock_service = MockNotificationService::new();
        let server = NotificationsServer::new(Arc::new(mock_service));
        let (name, vendor, version, spec_version) = server.GetServerInformation().await.unwrap();
        assert_eq!(name, "NovaDE Notification Server");
        assert_eq!(vendor, "NovaDE");
        assert_eq!(version, "0.1.0");
        assert_eq!(spec_version, "1.2");
    }

    #[tokio::test]
    async fn test_id_mapping_helpers() {
        let mock_service = MockNotificationService::new(); // Not used directly but needed for server
        let server = NotificationsServer::new(Arc::new(mock_service));
        let domain_id1 = NotificationId::new();
        let domain_id2 = NotificationId::new();

        // Get new D-Bus IDs
        let dbus_id1 = server.get_new_dbus_id(domain_id1).await;
        let dbus_id2 = server.get_new_dbus_id(domain_id2).await;
        assert_ne!(dbus_id1, dbus_id2);
        assert!(dbus_id1 >= 1);
        assert!(dbus_id2 >= 1);

        // Get domain IDs
        assert_eq!(server.get_domain_id(dbus_id1).await, Some(domain_id1));
        assert_eq!(server.get_domain_id(dbus_id2).await, Some(domain_id2));
        assert_eq!(server.get_domain_id(999).await, None); // Non-existent

        // Remove mapping
        assert_eq!(server.remove_dbus_id_mapping(dbus_id1).await, Some(domain_id1));
        assert_eq!(server.get_domain_id(dbus_id1).await, None); // Should be gone
        assert_eq!(server.remove_dbus_id_mapping(dbus_id1).await, None); // Already removed

        // Check next_dbus_id increment
        let domain_id3 = NotificationId::new();
        let dbus_id3 = server.get_new_dbus_id(domain_id3).await;
        assert_eq!(dbus_id3, dbus_id2 + 1); // Assuming sequential IDs from the test start
    }

    #[tokio::test]
    async fn test_notify_basic() {
        let mut mock_service = MockNotificationService::new();
        let expected_domain_id = NotificationId::new();

        mock_service.expect_create_notification()
            .withf(move |n: &Notification| n.summary() == "Test Summary" && n.body() == "Test Body")
            .times(1)
            .returning(move |mut n| {
                n.set_id(expected_domain_id); // Set the ID the real service would
                Ok(n)
            });

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_context = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };


        let dbus_id = server.Notify(
            "TestApp".to_string(),
            0, // replaces_id
            "test-icon".to_string(),
            "Test Summary".to_string(),
            "Test Body".to_string(),
            Vec::new(), // actions
            HashMap::new(), // hints
            -1, // expire_timeout
            dummy_context,
        ).await.unwrap();

        assert!(dbus_id >= 1);
        assert_eq!(server.get_domain_id(dbus_id).await, Some(expected_domain_id));
    }

    #[tokio::test]
    async fn test_notify_with_actions_and_urgency() {
        let mut mock_service = MockNotificationService::new();
        let expected_domain_id = NotificationId::new();

        mock_service.expect_create_notification()
            .withf(move |n: &Notification| {
                n.summary() == "Urgent Test" &&
                n.priority() == DomainPriority::Critical &&
                n.actions().len() == 1 &&
                n.actions()[0].id == "ack" &&
                n.actions()[0].label == "Acknowledge"
            })
            .times(1)
            .returning(move |mut n| {
                n.set_id(expected_domain_id);
                Ok(n)
            });

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_context = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };


        let mut hints = HashMap::new();
        hints.insert("urgency".to_string(), Value::U8(2)); // Critical

        let actions = vec!["ack".to_string(), "Acknowledge".to_string()];

        let dbus_id = server.Notify(
            "TestApp".to_string(),
            0,
            "icon".to_string(),
            "Urgent Test".to_string(),
            "Body here".to_string(),
            actions,
            hints,
            -1,
            dummy_context,
        ).await.unwrap();

        assert!(dbus_id >= 1);
        assert_eq!(server.get_domain_id(dbus_id).await, Some(expected_domain_id));
    }

    #[tokio::test]
    async fn test_notify_domain_service_error() {
        let mut mock_service = MockNotificationService::new();
        mock_service.expect_create_notification()
            .times(1)
            .returning(|_| Err(DomainError::Repository("Failed to save".to_string())));

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_context = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };

        let result = server.Notify(
            "TestApp".to_string(), 0, "icon".to_string(), "Summary".to_string(),
            "Body".to_string(), vec![], HashMap::new(), -1, dummy_context,
        ).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            zbus::fdo::Error::Failed(msg) => {
                assert!(msg.contains("Failed to create notification: Repository(\"Failed to save\")"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[tokio::test]
    async fn test_close_notification_valid_id() {
        let mut mock_service = MockNotificationService::new();
        let domain_id_to_dismiss = NotificationId::new();

        // Setup: Notify first to get a valid D-Bus ID and populate mapping
        mock_service.expect_create_notification()
            .times(1)
            .returning(move |mut n| {
                n.set_id(domain_id_to_dismiss); Ok(n)
            });

        mock_service.expect_dismiss_notification()
            .withf(move |&id| id == domain_id_to_dismiss)
            .times(1)
            .returning(|_| Ok(()));

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_notify_ctx = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };

        let dbus_id = server.Notify( // Assume this Notify works and maps dbus_id to domain_id_to_dismiss
            "TestApp".to_string(), 0, "icon".to_string(), "Summary".to_string(),
            "Body".to_string(), vec![], HashMap::new(), -1, dummy_notify_ctx,
        ).await.unwrap();

        assert_eq!(server.get_domain_id(dbus_id).await, Some(domain_id_to_dismiss));

        // Test CloseNotification
        // For CloseNotification, the SignalContext is used to emit NotificationClosed.
        // This is the tricky part for pure unit tests if we can't mock SignalContext emission.
        // We will use a dummy context and accept that the signal emission itself isn't verified here,
        // but the call to dismiss_notification and map cleanup is.
        let dummy_close_ctx = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };
        let close_result = server.CloseNotification(dbus_id, dummy_close_ctx).await;

        assert!(close_result.is_ok(), "CloseNotification failed: {:?}", close_result.err());
        assert_eq!(server.get_domain_id(dbus_id).await, None); // Mapping should be removed
    }

    #[tokio::test]
    async fn test_close_notification_invalid_id() {
        let mut mock_service = MockNotificationService::new();
        // No calls to mock_service expected for an invalid ID

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_context = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };

        let result = server.CloseNotification(999, dummy_context).await; // 999 is an unknown D-Bus ID

        assert!(result.is_err());
        match result.unwrap_err() {
            zbus::fdo::Error::InvalidArgs(msg) => {
                assert!(msg.contains("Notification ID 999 not found"));
            }
            other_err => panic!("Unexpected error type: {:?}", other_err),
        }
    }

    #[tokio::test]
    async fn test_close_notification_domain_service_error() {
        let mut mock_service = MockNotificationService::new();
        let domain_id_to_fail = NotificationId::new();

        mock_service.expect_create_notification()
            .times(1)
            .returning(move |mut n| { n.set_id(domain_id_to_fail); Ok(n) });

        mock_service.expect_dismiss_notification()
            .withf(move |&id| id == domain_id_to_fail)
            .times(1)
            .returning(|_| Err(DomainError::NotFound("Notification not found in domain".to_string())));

        let server = NotificationsServer::new(Arc::new(mock_service));
        let dummy_notify_ctx = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };

        let dbus_id = server.Notify(
            "TestApp".to_string(), 0, "icon".to_string(), "Summary".to_string(),
            "Body".to_string(), vec![], HashMap::new(), -1, dummy_notify_ctx,
        ).await.unwrap();

        let dummy_close_ctx = unsafe { SignalContext::new(&*(std::ptr::null() as *const zbus::Connection) , "/dummy").unwrap() };
        let close_result = server.CloseNotification(dbus_id, dummy_close_ctx).await;

        assert!(close_result.is_err());
        match close_result.unwrap_err() {
            zbus::fdo::Error::Failed(msg) => {
                assert!(msg.contains("Failed to dismiss notification: NotFound(\"Notification not found in domain\")"));
            }
            other_err => panic!("Unexpected error type: {:?}", other_err),
        }
        // Check that mapping is NOT removed if dismiss fails
        assert_eq!(server.get_domain_id(dbus_id).await, Some(domain_id_to_fail));
    }
}
