use novade_domain::notifications::{NotificationService, Notification, NotificationEvent, NotificationError};
use std::sync::Arc;
use gtk::glib;
use tokio::runtime::Handle; // To spawn the listener task
use tracing;

pub struct UINotificationService {
    domain_service: Arc<dyn NotificationService>,
    tokio_handle: Handle,
    // Sender to update the UI (e.g., NotificationCenterPanelWidget)
    // The channel is for Vec<Notification> representing the full current list
    ui_update_sender: glib::Sender<Vec<Notification>>,
}

impl UINotificationService {
    pub fn new(
        domain_service: Arc<dyn NotificationService>, 
        tokio_handle: Handle, 
        ui_update_sender: glib::Sender<Vec<Notification>>
    ) -> Self {
        let service_clone = domain_service.clone();
        let sender_clone = ui_update_sender.clone();
        
        tokio_handle.spawn(async move {
            tracing::info!("UINotificationService: Event listener task started.");
            let mut receiver = service_clone.subscribe_notifications();
            loop {
                match receiver.recv().await {
                    Ok(event) => { 
                        tracing::debug!("UINotificationService: Received NotificationEvent: {:?}", event);
                        // On any domain event (Added or Removed), refetch all notifications to update UI.
                        // This simplifies UI logic for now.
                        match service_clone.get_notifications().await {
                            Ok(notifs) => {
                                if sender_clone.send(notifs).is_err() {
                                    tracing::error!("UINotificationService: UI notifications channel closed! Listener terminating.");
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("UINotificationService: Failed to list notifications after event: {:?}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("UINotificationService: Notification event listener lagged by {} messages. Refetching all.", n);
                        // Refetch all on lag to ensure UI consistency
                        if let Ok(notifs) = service_clone.get_notifications().await {
                             if sender_clone.send(notifs).is_err() { 
                                tracing::error!("UINotificationService: UI notifications channel closed during lag recovery! Listener terminating.");
                                break; 
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("UINotificationService: Notification event channel closed by domain service. Listener terminating.");
                        break;
                    }
                }
            }
            tracing::info!("UINotificationService: Event listener task terminated.");
        });

        Self { domain_service, tokio_handle, ui_update_sender }
    }

    // Fetches current notifications and maps them for the UI.
    // Useful for initial population.
    pub async fn get_initial_notifications(&self) -> Vec<Notification> {
        tracing::debug!("UINotificationService: Fetching initial notifications.");
        self.domain_service.get_notifications().await.unwrap_or_else(|e| {
            tracing::error!("UINotificationService: Failed to get initial notifications: {:?}", e);
            vec![]
        })
    }
    
    // Method to dismiss a notification via the domain service.
    // The UI update will be triggered by the event broadcast from the domain service.
    pub async fn dismiss_ui_notification(&self, id: String) {
        tracing::debug!("UINotificationService: Requesting dismissal of notification ID: {}", id);
        if let Err(e) = self.domain_service.dismiss_notification(id).await {
            tracing::error!("UINotificationService: Failed to dismiss notification via domain service: {:?}", e);
        }
        // UI update will happen via the event broadcast from domain service
        // which is picked up by the listener task in Self::new().
    }
    
    // Expose the Tokio handle for spawning tasks from UI components that use this service
    // (though dismiss_ui_notification is already async and handles spawning if needed)
    pub fn tokio_handle(&self) -> &Handle {
        &self.tokio_handle
    }
}
