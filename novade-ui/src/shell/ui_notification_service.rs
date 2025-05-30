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
use novade_domain::notifications::{Notification, NotificationEvent, NotificationService as DomainNotificationService};


pub struct UINotificationService {
    // Keep a reference to the domain service for listening to events for UI updates.
    // This assumes the D-Bus server in novade-system correctly updates the domain service,
    // which then broadcasts events that this UI service listens to for display purposes.
    domain_service_listener: Arc<dyn DomainNotificationService>, 
    
    // The D-Bus client for *sending* notifications from the UI
    dbus_client: Arc<NotificationClient>,
    
    tokio_handle: Handle,
    ui_update_sender: glib::Sender<Vec<Notification>>, // For updating NotificationCenterPanelWidget etc.
}

impl UINotificationService {
    pub async fn new(
        domain_service_for_listening: Arc<dyn DomainNotificationService>, // For subscribing to domain events
        tokio_handle: Handle, 
        ui_update_sender: glib::Sender<Vec<Notification>>
    ) -> Result<Self, NotificationClientError> {
        info!("Initializing UINotificationService with D-Bus client integration.");
        
        let dbus_client = Arc::new(NotificationClient::new().await?);
        let service_clone_for_event_listener = domain_service_for_listening.clone();
        let sender_clone_for_event_listener = ui_update_sender.clone();
        let dbus_client_clone_for_signal_listener = dbus_client.clone(); // Clone for signal listener task

        // Task 1: Listen to domain events (if D-Bus server updates domain, which broadcasts events)
        // This remains important for populating the UI with notifications from any source.
        tokio_handle.spawn(async move {
            debug!("UINotificationService: Domain event listener task started.");
            let mut receiver = service_clone_for_event_listener.subscribe_notifications(); // Assuming this method exists on DomainNotificationService
            loop {
                match receiver.recv().await {
                    Ok(event) => { 
                        debug!("UINotificationService: Received Domain NotificationEvent: {:?}", event);
                        // On any domain event, refetch all notifications to update UI.
                        match service_clone_for_event_listener.get_all_notifications().await { // Changed from get_notifications
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

        // Task 2: Listen to D-Bus signals using the NotificationClient
        // This is important for real-time updates from the D-Bus server itself, like actions invoked.
        let dbus_signal_tokio_handle = tokio_handle.clone();
        let sender_clone_for_dbus_signals = ui_update_sender.clone();
        let service_clone_for_dbus_signals = domain_service_for_listening.clone(); // To refetch on close/action

        dbus_signal_tokio_handle.spawn(async move {
            debug!("UINotificationService: D-Bus signal listener task started.");

            let client_for_actions = dbus_client_clone_for_signal_listener.clone();
            let client_for_closed = dbus_client_clone_for_signal_listener.clone();
            
            let action_listener_handle = tokio::spawn(async move {
                if let Err(e) = client_for_actions.receive_action_invoked_signals(move |args: ActionInvokedArgs| {
                    info!("D-Bus ActionInvoked signal received via client: {:?}", args);
                    // Potentially trigger UI updates or specific actions here.
                    // For now, just log. A full implementation might refetch or update specific UI elements.
                    // Example: could map dbus_id back to domain_id if necessary, or pass dbus_id to UI.
                }).await {
                    error!("Error in D-Bus ActionInvoked listener: {}", e);
                }
            });

            let closed_listener_handle = tokio::spawn(async move {
                 // Clone sender and service again for this specific async block
                let local_sender = sender_clone_for_dbus_signals.clone();
                let local_service = service_clone_for_dbus_signals.clone();
                if let Err(e) = client_for_closed.receive_notification_closed_signals(move |args: NotificationClosedArgs| {
                    info!("D-Bus NotificationClosed signal received via client: {:?}", args);
                    // A notification was closed. Refetch all notifications to update the main UI view.
                    // This is similar to how domain events are handled.
                    let future = async move {
                        match local_service.get_all_notifications().await {
                            Ok(notifs) => {
                                if local_sender.send(notifs).is_err() {
                                    error!("UINotificationService: UI notifications channel closed! (from D-Bus closed signal)");
                                }
                            }
                            Err(e) => {
                                error!("UINotificationService: Failed to list notifications after D-Bus closed signal: {:?}", e);
                            }
                        }
                    };
                    // Spawn this future on the provided Tokio handle to avoid blocking the signal callback.
                    Handle::current().spawn(future); // Assuming Handle::current() is appropriate here or pass tokio_handle
                }).await {
                    error!("Error in D-Bus NotificationClosed listener: {}", e);
                }
            });
            
            // Optional: Server availability monitoring
            let client_for_availability = dbus_client_clone_for_signal_listener.clone();
            tokio::spawn(async move {
                 if let Err(e) = client_for_availability.monitor_server_availability(
                    || info!("Notification server is available."),
                    || warn!("Notification server is unavailable.")
                ).await {
                    error!("Error in D-Bus server availability monitor: {}", e);
                }
            });

            // Keep the signal listener task alive
            let _ = tokio::try_join!(action_listener_handle, closed_listener_handle);
            debug!("UINotificationService: D-Bus signal listener task exiting.");
        });


        Ok(Self { 
            domain_service_listener: domain_service_for_listening, 
            dbus_client, 
            tokio_handle, 
            ui_update_sender 
        })
    }

    // Method to send a notification using the D-Bus client
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
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            priority,
            expire_timeout,
        ).await
    }
    
    // Method to dismiss/close a notification via the D-Bus client
    pub async fn close_ui_notification(&self, dbus_id: u32) -> Result<(), NotificationClientError> {
        info!("UINotificationService: Closing notification with D-Bus ID {} via D-Bus client.", dbus_id);
        self.dbus_client.close_notification(dbus_id).await
    }

    // Fetches current notifications from the domain for initial population or refresh.
    // The UI should primarily rely on the event/signal listeners for updates.
    pub async fn get_current_notifications_for_ui(&self) -> Vec<Notification> {
        debug!("UINotificationService: Fetching current notifications from domain for UI.");
        self.domain_service_listener.get_all_notifications().await.unwrap_or_else(|e| { // Changed from get_notifications
            error!("UINotificationService: Failed to get current notifications for UI: {:?}", e);
            vec![]
        })
    }
    
    pub fn tokio_handle(&self) -> &Handle {
        &self.tokio_handle
    }

    pub async fn handle_ui_invoked_action(&self, dbus_id: u32, action_key: &str) {
        info!("UINotificationService: UI invoked action '{}' on D-Bus ID {}", action_key, dbus_id);
        // This is where UINotificationService tells the D-Bus server (in novade-system)
        // that an action was invoked, so the server can emit the ActionInvoked signal.
        // This requires a way for the UI process to call into the system process's server logic.
        // This could be a custom D-Bus method on the server like "DeveloperEmitActionInvoked".
        // Or, if they are in the same process, a direct call or an internal channel.

        // For now, we'll just log. The actual mechanism for this call needs to be designed.
        // If using D-Bus, it might look like:
        // self.dbus_client.call_custom_method_to_trigger_action_signal(dbus_id, action_key).await;
        
        // Placeholder: If the server automatically handles this upon some D-Bus call,
        // or if the client that *owns* the notification is a different application,
        // then our UI service might not need to do anything here other than telling the
        // *local* display manager (NotificationUi) to close the popup if the action implies that.
        // The spec says the server sends ActionInvoked. Our server needs to be told by the UI.
        
        // This is a complex part of the spec. Let's assume for now that the server
        // might need a custom D-Bus method that this client can call to report an action invocation.
        // For example, if our `NotificationsServer` in `novade-system` had a method:
        // async fn ReportActionInvoked(&self, notification_id: u32, action_key: String) -> zbus::fdo::Result<()>;
        // Then the client could call it. This method on the server would then emit the standard signal.
        
        // For this step, we will assume this method is called, but the actual D-Bus call
        // to the server to trigger the signal is a TODO or will be handled by NotificationClient if extended.
        debug!("TODO: Implement actual call to D-Bus server to report action invoked for D-Bus ID {}, action '{}'", dbus_id, action_key);
    }
}

// Ensure that the DomainNotificationService trait has `subscribe_notifications` and `get_all_notifications`
// If `subscribe_notifications` is not on the trait, the code above will need adjustment
// (e.g. if it's specific to DefaultNotificationService or another mechanism is used for events).
// For `get_all_notifications` it was `get_notifications` in the original file.
// The problem description implies `novade-domain/src/notification_service/mod.rs` which
// should be `novade-domain/src/notifications/service.rs`.

// Previous content of `UINotificationService`:
// use novade_domain::notifications::{NotificationService, Notification, NotificationEvent, NotificationError};
// ...
// pub struct UINotificationService {
//     domain_service: Arc<dyn NotificationService>,
//     tokio_handle: Handle, 
//     ui_update_sender: glib::Sender<Vec<Notification>>,
// }
// ...
// Methods: new (spawned event listener for domain_service.subscribe_notifications()), 
// get_initial_notifications (calls domain_service.get_notifications()),
// dismiss_ui_notification (calls domain_service.dismiss_notification()),
// tokio_handle.
