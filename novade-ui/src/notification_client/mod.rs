use zbus::{Proxy, Connection, Error as ZbusError, zvariant::{Value, Dict}};
use std::collections::HashMap;
use futures_util::stream::StreamExt; // For signal stream
use std::sync::Arc;

pub struct NotificationClient {
    connection: Arc<Connection>, // Store Arc<Connection>
}

impl NotificationClient {
    pub async fn new() -> Result<Self, ZbusError> {
        let connection = Connection::session().await?;
        Ok(Self { connection: Arc::new(connection) })
    }

    // Helper to get a proxy, reduces repetition
    async fn get_proxy(&self) -> Result<Proxy<'static>, ZbusError> {
         Proxy::new(
            self.connection.clone(), // Clone Arc, not the connection itself
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
        ).await
    }

    pub async fn send_notification(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>, // e.g. ["default", "Open Link", "cancel", "Dismiss"]
        hints: HashMap<&str, Value<'_>>, // Allow flexible hints
        expire_timeout: i32,
    ) -> Result<u32, ZbusError> {
        let proxy = self.get_proxy().await?;

        let actions_string_vec: Vec<String> = actions.iter().map(|s| s.to_string()).collect();

        // The D-Bus method expects hints as HashMap<String, Value<'_>> which zbus::zvariant::Dict can represent.
        // Keys in Dict must be String or &str that can be converted.
        let mut dbus_hints = Dict::new();
        for (k,v) in hints {
            // We need to ensure v is owned if Value constructor needs owned,
            // but Value itself can hold references. For sending, it's often simpler if Value owns its data
            // or the data outlives the call. Given Value<'_>, it implies it can borrow.
            // Dict::insert will handle the key conversion if k is &str.
            dbus_hints.insert(k, v)?;
        }
        
        proxy.call_method(
            "Notify",
            &(app_name, replaces_id, app_icon, summary, body, actions_string_vec, dbus_hints, expire_timeout),
        )
        .await?
        .body::<u32>() // Expects a u32 in the reply body
    }

    pub async fn receive_notification_closed<F>(&self, mut callback: F) -> Result<(), ZbusError>
    where
        F: FnMut(u32, u32) + Send + 'static, // id, reason
    {
        let proxy = self.get_proxy().await?; // This proxy is fine for receiving signals too
        let mut stream = proxy.receive_signal("NotificationClosed").await?;
        while let Some(signal) = stream.next().await {
            match signal.body::<(u32, u32)>() {
                Ok((id, reason)) => {
                    callback(id, reason);
                }
                Err(e) => {
                    // Handle or log the error in deserializing the signal body
                    eprintln!("Error deserializing NotificationClosed signal body: {:?}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn receive_action_invoked<F>(&self, mut callback: F) -> Result<(), ZbusError>
    where
        F: FnMut(u32, String) + Send + 'static, // id, action_key
    {
        let proxy = self.get_proxy().await?;
        let mut stream = proxy.receive_signal("ActionInvoked").await?;
        while let Some(signal) = stream.next().await {
             match signal.body::<(u32, String)>() {
                Ok((id, action_key)) => {
                    callback(id, action_key);
                }
                Err(e) => {
                    eprintln!("Error deserializing ActionInvoked signal body: {:?}", e);
                }
            }
        }
        Ok(())
    }
}
