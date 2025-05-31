// novade-ui/src/shell/ui_notification_service.rs
use std::sync::Arc;
use gtk::glib;
use tokio::runtime::Handle;
use tracing::{self, debug, error, info, warn};

// Use the new D-Bus client
use crate::notification_client::{
    NotificationClient,
    NotificationClientError,
    UIPriority, // Re-exported from domain by client
    UIAction,   // Re-exported from domain by client
    ActionInvokedArgs,
    NotificationClosedArgs,
};

// Still need the domain Notification for the UI update sender and event listening
use novade_domain::notifications::{Notification, NotificationEvent, NotificationService as DomainNotificationServiceTrait};


pub struct UINotificationService {
    domain_service_listener: Arc<dyn DomainNotificationServiceTrait>,
    dbus_client: Arc<NotificationClient>,
    tokio_handle: Handle,
    ui_update_sender: glib::Sender<Vec<Notification>>,
}

impl UINotificationService {
    pub async fn new(
        domain_service_for_listening: Arc<dyn DomainNotificationServiceTrait>,
        tokio_handle: Handle, 
        ui_update_sender: glib::Sender<Vec<Notification>>
    ) -> Result<Self, NotificationClientError> {
        info!("Initializing UINotificationService with D-Bus client integration.");

        // In production, NotificationClient::new() would be used.
        // For testing, this might be injected or NotificationClient::new() needs to be mockable if it does I/O.
        // Assuming NotificationClient::new() is okay for this structure, if it fails, new() fails.
        let dbus_client = Arc::new(NotificationClient::new().await?);

        let service_clone_for_event_listener = domain_service_for_listening.clone();
        let sender_clone_for_event_listener = ui_update_sender.clone();

        // Task 1: Listen to domain events
        tokio_handle.spawn(async move {
            debug!("UINotificationService: Domain event listener task started.");
            let mut receiver = service_clone_for_event_listener.subscribe_notifications();
            loop {
                match receiver.recv().await {
                    Ok(event) => { 
                        debug!("UINotificationService: Received Domain NotificationEvent: {:?}", event);
                        match service_clone_for_event_listener.get_all_notifications().await {
                            Ok(notifs) => {
                                if sender_clone_for_event_listener.send(notifs).is_err() {
                                    error!("UINotificationService: UI notifications channel closed! Domain event listener terminating.");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("UINotificationService: Failed to list notifications after domain event: {:?}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("UINotificationService: Domain event listener lagged by {} messages. Refetching all.", n);
                        if let Ok(notifs) = service_clone_for_event_listener.get_all_notifications().await {
                             if sender_clone_for_event_listener.send(notifs).is_err() {
                                error!("UINotificationService: UI notifications channel closed during lag recovery! Domain event listener terminating.");
                                break; 
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("UINotificationService: Domain event channel closed. Domain event listener terminating.");
                        break;
                    }
                }
            }
            debug!("UINotificationService: Domain event listener task terminated.");
        });

        let dbus_client_clone_for_signals = dbus_client.clone();
        let sender_clone_for_dbus_signals = ui_update_sender.clone();
        let service_clone_for_dbus_signals = domain_service_for_listening.clone();

        // Task 2: Listen to D-Bus signals
        tokio_handle.spawn(async move {
            debug!("UINotificationService: D-Bus signal listener task started.");

            let client_for_actions = dbus_client_clone_for_signals.clone();
            let client_for_closed = dbus_client_clone_for_signals.clone();
            let client_for_availability = dbus_client_clone_for_signals.clone(); // Added this clone

            let action_listener_handle = tokio::spawn(async move {
                if let Err(e) = client_for_actions.receive_action_invoked_signals(move |args: ActionInvokedArgs| {
                    info!("D-Bus ActionInvoked signal received via client: {:?}", args);
                }).await {
                    // Avoid panic in task, just log error.
                    if !format!("{}", e).contains("Connection refused") && !format!("{}",e).contains("proxy has been dropped"){ // Common if server not there or test ends
                         error!("Error in D-Bus ActionInvoked listener: {}", e);
                    }
                }
            });

            let closed_listener_handle = tokio::spawn(async move {
                let local_sender = sender_clone_for_dbus_signals.clone();
                let local_service = service_clone_for_dbus_signals.clone();
                if let Err(e) = client_for_closed.receive_notification_closed_signals(move |args: NotificationClosedArgs| {
                    info!("D-Bus NotificationClosed signal received via client: {:?}", args);
                    let future = async move {
                        match local_service.get_all_notifications().await {
                            Ok(notifs) => {
                                if local_sender.send(notifs).is_err() {
                                    error!("UINotificationService: UI notifications channel closed! (from D-Bus closed signal)");
                                }
                            }
                            Err(e_fetch) => { // Renamed variable to avoid conflict
                                error!("UINotificationService: Failed to list notifications after D-Bus closed signal: {:?}", e_fetch);
                            }
                        }
                    };
                    Handle::current().spawn(future);
                }).await {
                     if !format!("{}", e).contains("Connection refused") && !format!("{}",e).contains("proxy has been dropped"){
                        error!("Error in D-Bus NotificationClosed listener: {}", e);
                    }
                }
            });

            let availability_handle = tokio::spawn(async move { // Added this handle
                 if let Err(e) = client_for_availability.monitor_server_availability(
                    || info!("Notification server is available."),
                    || warn!("Notification server is unavailable.")
                ).await {
                     if !format!("{}", e).contains("Connection refused") && !format!("{}",e).contains("proxy has been dropped"){
                        error!("Error in D-Bus server availability monitor: {}", e);
                    }
                }
            });

            let _ = tokio::try_join!(action_listener_handle, closed_listener_handle, availability_handle); // join all tasks
            debug!("UINotificationService: D-Bus signal listener task (or one of its sub-tasks) exiting.");
        });

        Ok(Self {
            domain_service_listener: domain_service_for_listening,
            dbus_client,
            tokio_handle,
            ui_update_sender
        })
    }

    pub async fn send_ui_notification(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<UIAction>,
        priority: UIPriority,
        expire_timeout: i32,
    ) -> Result<u32, NotificationClientError> {
        info!("UINotificationService: Sending notification '{}' via D-Bus client.", summary);
        self.dbus_client.notify(
            app_name, replaces_id, app_icon, summary, body, actions, priority, expire_timeout,
        ).await
    }

    pub async fn close_ui_notification(&self, dbus_id: u32) -> Result<(), NotificationClientError> {
        info!("UINotificationService: Closing notification with D-Bus ID {} via D-Bus client.", dbus_id);
        self.dbus_client.close_notification(dbus_id).await
    }

    pub async fn get_current_notifications_for_ui(&self) -> Vec<Notification> {
        debug!("UINotificationService: Fetching current notifications from domain for UI.");
        self.domain_service_listener.get_all_notifications().await.unwrap_or_else(|e| {
            error!("UINotificationService: Failed to get current notifications for UI: {:?}", e);
            vec![]
        })
    }
    
    pub fn tokio_handle(&self) -> &Handle {
        &self.tokio_handle
    }

    pub async fn handle_ui_invoked_action(&self, dbus_id: u32, action_key: &str) {
        info!("UINotificationService: UI invoked action '{}' on D-Bus ID {}", action_key, dbus_id);
        debug!("TODO: Implement actual call to D-Bus server to report action invoked for D-Bus ID {}, action '{}'", dbus_id, action_key);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification_client::{NotificationClient, NotificationClientError, ActionInvokedArgs, NotificationClosedArgs, UIAction, UIPriority};
    use novade_domain::notifications::{Notification, NotificationId, NotificationEvent, NotificationService as DomainNotificationServiceTrait};
    use novade_domain::error::DomainResult as ActualDomainResult;
    use tokio::runtime::{Handle, Runtime};
    use tokio::sync::broadcast::{channel as broadcast_channel, error::RecvError};
    use std::sync::mpsc::{channel as std_mpsc_channel, TryRecvError}; // For glib sender
    use mockall::mock;
    use zbus::Error as ZbusError; // For creating mock errors


    // Mock for NotificationClient
    // Note: new() is a static method on the real NotificationClient.
    // We can't mock it directly on an instance.
    // Instead, for tests requiring UINotificationService::new(), we'd need to ensure
    // NotificationClient::new() can be controlled, e.g. by having it connect to a test D-Bus server.
    // For other methods, we mock them on the instance.
    mock! {
        NotificationClient {
            // Static methods like new() are hard to mock on an instance basis.
            // pub async fn new() -> Result<NotificationClient, NotificationClientError>;
            // pub async fn new_with_custom_name(...) -> Result<NotificationClient, NotificationClientError>;

            // Instance methods
            pub async fn notify(&self, app_name: &str, replaces_id: u32, app_icon: &str, summary: &str, body: &str, actions: Vec<UIAction>, priority: UIPriority, expire_timeout: i32) -> Result<u32, NotificationClientError>;
            pub async fn close_notification(&self, id: u32) -> Result<(), NotificationClientError>;
            pub async fn get_capabilities(&self) -> Result<Vec<String>, NotificationClientError>;
            pub async fn get_server_information(&self) -> Result<(String, String, String, String), NotificationClientError>;

            // For signal listening, we need to mock the method that sets up the listener.
            // The callback makes it tricky. We can check if the method is called.
            // The actual callback testing will be indirect via ui_update_sender.
            #[allow(clippy::type_complexity)] // Allow complex types for mockall FnMut
            pub async fn receive_action_invoked_signals<F>(&self, callback: F) -> Result<(), NotificationClientError>
            where F: FnMut(ActionInvokedArgs) + Send + 'static;

            #[allow(clippy::type_complexity)]
            pub async fn receive_notification_closed_signals<F>(&self, callback: F) -> Result<(), NotificationClientError>
            where F: FnMut(NotificationClosedArgs) + Send + 'static;

            #[allow(clippy::type_complexity)]
            pub async fn monitor_server_availability<F_available, F_unavailable>(&self, on_available: F_available, on_unavailable: F_unavailable) -> Result<(), NotificationClientError>
            where F_available: FnMut() + Send + 'static, F_unavailable: FnMut() + Send + 'static;
        }
    }
    // If Arc<MockNotificationClient> is used, and it needs to be Clone for some reason by the code under test.
    // impl Clone for MockNotificationClient { fn clone(&self) -> Self { panic!("MockNotificationClient basic clone not supported/needed") } }


    // Mock for DomainNotificationServiceTrait
    mock! {
        DomainNotificationService {
            async fn create_notification(&self, notification: Notification) -> ActualDomainResult<Notification>;
            async fn get_notification(&self, id: NotificationId) -> ActualDomainResult<Option<Notification>>;
            async fn get_all_notifications(&self) -> ActualDomainResult<Vec<Notification>>;
            async fn update_notification(&self, notification: Notification) -> ActualDomainResult<Notification>;
            async fn dismiss_notification(&self, id: NotificationId) -> ActualDomainResult<()>;
            async fn perform_action(&self, id: NotificationId, action_id: &str) -> ActualDomainResult<()>;
            fn subscribe_notifications(&self) -> tokio::sync::broadcast::Receiver<NotificationEvent>;
        }
    }
    // If Arc<MockDomainNotificationService> needs to be Clone.
    // impl Clone for MockDomainNotificationService { fn clone(&self) -> Self { panic!("MockDomainNotificationService basic clone not supported/needed") } }

    // Helper to create a Tokio runtime for tests that need to spawn tasks
    fn get_test_tokio_runtime() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    // Helper for glib::Sender and mpsc::Receiver
    fn setup_glib_sender_receiver() -> (glib::Sender<Vec<Notification>>, std::sync::mpsc::Receiver<Vec<Notification>>) {
        let (mpsc_tx, mpsc_rx) = std_mpsc_channel::<Vec<Notification>>();
        let glib_sender = glib::Sender::new(move |data| {
            mpsc_tx.send(data).expect("Failed to send from glib::Sender to mpsc::Receiver");
            glib::Continue(true)
        });
        (glib_sender, mpsc_rx)
    }


    #[tokio::test]
    async fn test_uinotificationservice_send_notification() {
        let rt_handle = Handle::current();
        let (glib_sender, _mpsc_receiver) = setup_glib_sender_receiver();

        let mut mock_dbus_client = MockNotificationClient::new();
        mock_dbus_client.expect_notify()
            .withf(|app_name, _rid, _icon, summary, ..| app_name == "TestApp" && summary == "Test Summary")
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(123)); // Return a dummy D-Bus ID

        let (tx_domain_event, _rx_domain_event) = broadcast_channel::<NotificationEvent>(16);
        let mut mock_domain_service = MockDomainNotificationService::new();
        mock_domain_service.expect_subscribe_notifications().return_return(tx_domain_event.subscribe());
        // Other domain service expectations if new() spawns tasks that use them immediately

        // For new() method, NotificationClient::new() is called.
        // This test focuses on send_ui_notification, so we construct UINotificationService directly with mocks.
        let service = UINotificationService {
            domain_service_listener: Arc::new(mock_domain_service),
            dbus_client: Arc::new(mock_dbus_client),
            tokio_handle: rt_handle,
            ui_update_sender: glib_sender,
        };

        let result = service.send_ui_notification(
            "TestApp", 0, "icon", "Test Summary", "Body", vec![], UIPriority::Normal, -1
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }

    #[tokio::test]
    async fn test_uinotificationservice_close_notification() {
        let rt_handle = Handle::current();
        let (glib_sender, _mpsc_receiver) = setup_glib_sender_receiver();

        let mut mock_dbus_client = MockNotificationClient::new();
        mock_dbus_client.expect_close_notification()
            .withf(|&id| id == 123)
            .times(1)
            .returning(|_| Ok(()));

        let (tx_domain_event, _rx_domain_event) = broadcast_channel::<NotificationEvent>(16);
        let mut mock_domain_service = MockDomainNotificationService::new();
        mock_domain_service.expect_subscribe_notifications().return_return(tx_domain_event.subscribe());

        let service = UINotificationService {
            domain_service_listener: Arc::new(mock_domain_service),
            dbus_client: Arc::new(mock_dbus_client),
            tokio_handle: rt_handle,
            ui_update_sender: glib_sender,
        };

        let result = service.close_ui_notification(123).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_ui_invoked_action_logs_todo() {
        // This test is basic as the method itself is a TODO.
        // It mainly ensures the method can be called.
        let rt_handle = Handle::current();
        let (glib_sender, _mpsc_receiver) = setup_glib_sender_receiver();
        let mock_dbus_client = MockNotificationClient::new(); // No expectations needed for this test

        let (tx_domain_event, _rx_domain_event) = broadcast_channel::<NotificationEvent>(16);
        let mut mock_domain_service = MockDomainNotificationService::new();
        mock_domain_service.expect_subscribe_notifications().return_return(tx_domain_event.subscribe());

        let service = UINotificationService {
            domain_service_listener: Arc::new(mock_domain_service),
            dbus_client: Arc::new(mock_dbus_client),
            tokio_handle: rt_handle,
            ui_update_sender: glib_sender,
        };

        // Just call it. If it panics or has type errors, test fails.
        service.handle_ui_invoked_action(123, "action_id_test").await;
        // In a real scenario with logging capture, we could assert the TODO message.
    }

    #[tokio::test]
    async fn test_domain_event_triggers_ui_update() {
        let rt = get_test_tokio_runtime();
        let rt_handle = rt.handle().clone();
        let (glib_sender, mpsc_receiver) = setup_glib_sender_receiver();

        let (tx_domain_event, rx_domain_event_for_service) = broadcast_channel::<NotificationEvent>(16);

        let mut mock_domain_service = MockDomainNotificationService::new();
        mock_domain_service.expect_subscribe_notifications().return_once(|| rx_domain_event_for_service);

        let expected_notifications = vec![Notification::new("Test", "Body", "App")];
        let expected_notifications_clone = expected_notifications.clone();
        mock_domain_service.expect_get_all_notifications()
            .times(1) // Expect it to be called once after event
            .returning(move || Ok(expected_notifications_clone.clone()));

        // For UINotificationService::new, which calls NotificationClient::new()
        // This is part of the challenge of mocking things called inside `new`.
        // We are testing the *listener task* behavior here.
        // So, we construct UINotificationService directly with a mock client.
        let mut mock_dbus_client = MockNotificationClient::new();
        // Setup default expectations for signal listeners if new() tries to start them,
        // even if not the focus of *this* test.
        mock_dbus_client.expect_receive_action_invoked_signals().returning(|_| Ok(()));
        mock_dbus_client.expect_receive_notification_closed_signals().returning(|_| Ok(()));
        mock_dbus_client.expect_monitor_server_availability().returning(|_,_| Ok(()));


        let _service = UINotificationService {
            domain_service_listener: Arc::new(mock_domain_service),
            dbus_client: Arc::new(mock_dbus_client), // Pass the mock_dbus_client
            tokio_handle: rt_handle.clone(),
            ui_update_sender: glib_sender,
        };
        // The listener task is spawned in new(). We need to give it a moment to start.
        // Then send an event.

        rt_handle.spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await; // Allow listener to start
            let test_event = NotificationEvent::Added(NotificationId::new());
            tx_domain_event.send(test_event).unwrap();
        }).await.unwrap();


        // Check if glib_sender (via mpsc_receiver) received the notifications
        match mpsc_receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(received_notifs) => {
                assert_eq!(received_notifs.len(), expected_notifications.len());
                // Further checks if Notification had PartialEq
            }
            Err(e) => panic!("UI update sender did not receive notifications in time: {}", e),
        }
    }

    #[tokio::test]
    async fn test_dbus_closed_signal_triggers_ui_update() {
        let rt = get_test_tokio_runtime();
        let rt_handle = rt.handle().clone();
        let (glib_sender, mpsc_receiver) = setup_glib_sender_receiver();

        let mut mock_dbus_client = MockNotificationClient::new();
        let mut mock_domain_service = MockDomainNotificationService::new();

        // Mock subscribe_notifications for the domain event listener task in new()
        let (tx_domain_event_dummy, rx_domain_event_dummy) = broadcast_channel::<NotificationEvent>(1);
        mock_domain_service.expect_subscribe_notifications().return_return(rx_domain_event_dummy);
        drop(tx_domain_event_dummy); // Drop sender so it closes immediately, listener task exits.

        let expected_notifications = vec![Notification::new("After D-Bus Close", "Body", "App")];
        let expected_notifications_clone = expected_notifications.clone();
        mock_domain_service.expect_get_all_notifications()
            .times(1) // Expect it after the D-Bus signal
            .returning(move || Ok(expected_notifications_clone.clone()));

        // Mock setup for receive_notification_closed_signals
        // We need to capture the callback and invoke it.
        let (signal_cb_tx, mut signal_cb_rx) = tokio::sync::mpsc::channel::<Box<dyn FnMut(NotificationClosedArgs) + Send + 'static>>(1);

        mock_dbus_client.expect_receive_notification_closed_signals()
            .times(1)
            .returning(move |f| {
                signal_cb_tx.try_send(Box::new(f)).expect("Failed to send callback");
                Ok(())
            });
        // Default expectations for other signal listeners in new()
        mock_dbus_client.expect_receive_action_invoked_signals().returning(|_| Ok(()));
        mock_dbus_client.expect_monitor_server_availability().returning(|_,_| Ok(()));


        let _service = UINotificationService {
            domain_service_listener: Arc::new(mock_domain_service),
            dbus_client: Arc::new(mock_dbus_client),
            tokio_handle: rt_handle.clone(),
            ui_update_sender: glib_sender,
        };
        // Listener tasks are spawned in new().

        rt_handle.spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await; // Allow listeners to start
            if let Some(mut callback) = signal_cb_rx.recv().await {
                let closed_args = NotificationClosedArgs { id: 123, reason: 2 };
                callback(closed_args); // Invoke the captured callback
            } else {
                panic!("Failed to capture the D-Bus signal callback.");
            }
        }).await.unwrap();


        match mpsc_receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(received_notifs) => {
                assert_eq!(received_notifs.len(), expected_notifications.len());
            }
            Err(e) => panic!("UI update sender did not receive notifications after D-Bus signal: {}", e),
        }
    }
}
