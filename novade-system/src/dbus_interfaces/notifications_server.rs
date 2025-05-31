use std::collections::HashMap;
use zbus::{dbus_interface, zvariant::Value, SignalContext, fdo::Result as FdoResult};
use novade_domain::notification_service::{
    Notification as DomainNotification,
    NotificationManager,
    Error as DomainError
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NotificationsDBusService {
    manager: Arc<Mutex<dyn NotificationManager>>,
}

impl NotificationsDBusService {
    pub fn new(manager: Arc<Mutex<dyn NotificationManager>>) -> Self {
        Self { manager }
    }
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationsDBusService {
    // Methods
    async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints_dbus: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> FdoResult<u32> {
        println!("D-Bus Notify received in NotificationsDBusService, forwarding to domain manager...");

        let mut domain_hints = HashMap::new();
        for (k, v_wrapper) in hints_dbus {
            if let Some(s) = v_wrapper.downcast_ref::<String>() {
                 domain_hints.insert(k, s.clone());
            } else if let Some(s_ref) = v_wrapper.downcast_ref::<&str>() {
                 domain_hints.insert(k, s_ref.to_string());
            } else {
                println!("Warning: Hint '{}' has an unhandled zbus::Value type, storing as string representation.", k);
                domain_hints.insert(k, format!("{:?}", v_wrapper));
            }
        }

        let domain_notification = DomainNotification {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints: domain_hints,
            timeout: expire_timeout,
        };

        let mut manager_guard = self.manager.lock().await;
        manager_guard.notify(domain_notification).map_err(|e| {
            match e {
                DomainError::NotFound(id) => zbus::fdo::Error::UnknownMethod(format!("Notification {} not found", id)), // Or a more specific error
                DomainError::InvalidArguments(s) => zbus::fdo::Error::InvalidArgs(s),
                DomainError::Internal(s) => zbus::fdo::Error::Failed(s),
            }
        })
    }

    async fn close_notification(&mut self, id: u32) -> FdoResult<()> {
        println!("D-Bus CloseNotification received for ID: {}, forwarding to domain manager...", id);
        let mut manager_guard = self.manager.lock().await;
        manager_guard.close_notification(id).map_err(|e| {
            match e {
                DomainError::NotFound(_id) => zbus::fdo::Error::InvalidArgs("Notification ID not found.".to_string()), // Spec is loose here, InvalidArgs is common
                DomainError::InvalidArguments(s) => zbus::fdo::Error::InvalidArgs(s), // Should not happen for close
                DomainError::Internal(s) => zbus::fdo::Error::Failed(s),
            }
        })
    }

    async fn get_capabilities(&self) -> FdoResult<Vec<String>> {
        println!("D-Bus GetCapabilities called, forwarding to domain manager...");
        let manager_guard = self.manager.lock().await; // Note: `get_capabilities` in trait is &self, so lock might be &mut self
                                                       // This might need adjustment if manager methods are &self.
                                                       // For `Arc<Mutex<T>>`, `lock()` provides `MutexGuard<T>`, which derefs to `T`.
                                                       // If `NotificationManager` methods become `&self`, then `manager_guard.get_capabilities()` works.
                                                       // Current trait methods are `&mut self` or `&self`. `get_capabilities` is `&self`.
        manager_guard.get_capabilities().map_err(|e| {
            match e {
                DomainError::Internal(s) => zbus::fdo::Error::Failed(s),
                _ => zbus::fdo::Error::Failed("An unexpected error occurred in GetCapabilities".to_string()),
            }
        })
    }

    async fn get_server_information(
        &self,
    ) -> FdoResult<(String, String, String, String)> {
        println!("D-Bus GetServerInformation called, forwarding to domain manager...");
        let manager_guard = self.manager.lock().await;
        manager_guard.get_server_information().map_err(|e| {
             match e {
                DomainError::Internal(s) => zbus::fdo::Error::Failed(s),
                _ => zbus::fdo::Error::Failed("An unexpected error occurred in GetServerInformation".to_string()),
            }
        })
    }

    // Signals (definitions remain the same, emission is TBD)
    #[dbus_interface(signal)]
    async fn notification_closed(signal_ctxt: &SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn action_invoked(signal_ctxt: &SignalContext<'_>, id: u32, action_key: String) -> zbus::Result<()>;
}
