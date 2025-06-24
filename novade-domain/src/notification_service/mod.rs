// novade-domain/src/notification_service/mod.rs
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

// Potentially use zbus::zvariant::Value directly if that simplifies D-Bus layer,
// or define a domain-specific type for hints. For now, let's use a generic Value placeholder or skip complex hints.
// For simplicity in this step, we'll use String for hints for now.
// Actual D-Bus hints are more complex (HashMap<String, zvariant::Value>).
// This can be refined when implementing the D-Bus layer.

pub mod policies;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,     // ID of notification to replace, or 0 if new
    pub app_icon: String,     // Path or name
    pub summary: String,      // Single line summary
    pub body: String,         // Multi-line body
    pub actions: Vec<String>, // Alternating action_key, display_name
    pub hints: HashMap<String, String>, // Placeholder for common hints like urgency, category.
    // Real hints are zvariant::Value. This will need mapping.
    pub timeout: i32, // Milliseconds, or -1 for default, 0 for persistent
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            replaces_id: 0,
            app_icon: String::new(),
            summary: String::new(),
            body: String::new(),
            actions: Vec::new(),
            hints: HashMap::new(),
            timeout: -1,
        }
    }
}

#[derive(Debug, thiserror::Error)] // Assuming thiserror is used or can be added to novade-domain
pub enum Error {
    #[error("Notification with ID {0} not found")]
    NotFound(u32),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("Internal service error: {0}")]
    Internal(String),
    // Potentially: #[error(transparent)] DbusError(#[from] zbus::Error) if we want to wrap zbus errors here
}

pub trait NotificationManager: Send + Sync {
    fn notify(&mut self, notification: Notification) -> Result<u32, Error>;
    fn close_notification(&mut self, id: u32) -> Result<(), Error>;
    fn get_capabilities(&self) -> Result<Vec<String>, Error>;
    fn get_server_information(&self) -> Result<(String, String, String, String), Error>; // name, vendor, version, spec_version
}

#[derive(Debug, Default)]
pub struct DefaultNotificationManager {
    notifications: std::collections::HashMap<u32, Notification>,
    next_id: AtomicU32, // For generating unique IDs
                        // Potentially add a sender for a channel to communicate with UI/logging later
}

impl DefaultNotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: std::collections::HashMap::new(),
            next_id: AtomicU32::new(1), // Start IDs from 1
        }
    }

    fn generate_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl NotificationManager for DefaultNotificationManager {
    fn notify(&mut self, mut notification: Notification) -> Result<u32, Error> {
        // If client provides an ID that it wants to reuse (replaces_id),
        // and we don't have it, it's arguably an error or we just assign a new one.
        // The spec says: "If the replaces_id is 0, a new notification is created.
        // If replaces_id is not 0, then the notification with that ID is replaced with the new notification."
        // It doesn't explicitly state what to do if replaces_id is non-zero but non-existent.
        // For now, we'll allow replacing existing or creating new with the given ID if it doesn't clash badly.
        // Or, always use generated ID if replaces_id is not found.
        // Let's simplify: if replaces_id is set and exists, we replace. Otherwise, new ID.

        let final_id = if notification.replaces_id != 0
            && self.notifications.contains_key(&notification.replaces_id)
        {
            notification.replaces_id
        } else {
            self.generate_id() // Ensure a truly new ID if replaces_id is bogus or 0
        };
        notification.replaces_id = final_id; // Ensure the stored notification knows its actual ID.

        println!(
            "Domain: Storing notification (id: {}): {:?}",
            final_id, notification.summary
        );
        self.notifications.insert(final_id, notification);
        Ok(final_id)
    }

    fn close_notification(&mut self, id: u32) -> Result<(), Error> {
        if self.notifications.remove(&id).is_some() {
            println!("Domain: Closed notification with ID: {}", id);
            // Here, we'd emit a signal in a real scenario, or notify other parts of the system.
            Ok(())
        } else {
            Err(Error::NotFound(id))
        }
    }

    fn get_capabilities(&self) -> Result<Vec<String>, Error> {
        Ok(vec![
            "body".to_string(),
            "actions".to_string(), // Assuming we'll support them
            "persistence".to_string(), // If notifications can persist until explicitly closed
                                   // "sound".to_string(), // Example of another capability
        ])
    }

    fn get_server_information(&self) -> Result<(String, String, String, String), Error> {
        Ok((
            "NovaDE Domain Notification Manager".to_string(), // name
            "NovaDE Project".to_string(),                     // vendor
            "0.1.0".to_string(),                              // version
            "1.2".to_string(), // spec_version (of freedesktop standard)
        ))
    }
}
