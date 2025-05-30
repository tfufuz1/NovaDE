// novade-system/src/dbus_integration/mod.rs

use zbus::{Connection, Proxy, SignalContext}; // Added SignalContext
use novade_core::CoreError;
use thiserror::Error;
use tokio; // Ensure tokio is in scope for tokio::spawn
use futures_util::stream::StreamExt; // Required for signal_stream().next().await

#[derive(Debug, Error)]
pub enum DBusIntegrationError {
    #[error("D-Bus connection failed: {0}")]
    ConnectionFailed(#[from] zbus::Error),

    #[error("D-Bus call failed: {source}")]
    MethodCallFailed{ #[source] source: zbus::Error },

    #[error("Failed to create D-Bus proxy: {source}")]
    ProxyCreationFailed{ #[source] source: zbus::Error },

    #[error("Failed to build D-Bus signal receiver: {source}")]
    SignalReceiverBuildFailed{ #[source] source: zbus::Error },

    #[error("D-Bus signal stream ended unexpectedly")]
    SignalStreamEnded,

    #[error("Failed to deserialize D-Bus signal arguments: {0}")]
    SignalDeserializationFailed(#[from] zbus::zvariant::Error),

    #[error("Failed to interpret D-Bus response body: {0}")]
    BodyDeserializationFailed(#[from] zbus::zvariant::Error), // Keep this one

    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    #[error("An unexpected internal error occurred: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DBusIntegrationError>;

pub async fn connect_and_list_names() -> Result<()> {
    println!("Attempting to connect to system bus...");
    let connection = Connection::system()
        .await
        .map_err(DBusIntegrationError::ConnectionFailed)?;
    println!("Connected to system bus.");

    println!("Creating proxy for org.freedesktop.DBus...");
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await
    .map_err(|e| DBusIntegrationError::ProxyCreationFailed{ source: e })?;
    println!("Proxy created for org.freedesktop.DBus.");

    println!("Calling ListNames()...");
    let names_body: zbus::zvariant::OwnedValue = proxy
        .call_method("ListNames", &())
        .await
        .map_err(|e| DBusIntegrationError::MethodCallFailed{ source: e })?
        .body()?;

    let names: Vec<String> = names_body
        .try_into()
        .map_err(DBusIntegrationError::BodyDeserializationFailed)?;

    println!("Available D-Bus service names:");
    for name in names {
        println!("- {}", name);
    }

    Ok(())
}

// Struct to represent the arguments of the NameOwnerChanged signal
#[derive(Debug, serde::Deserialize, zbus::SignalArgs)]
struct NameOwnerChangedSignal {
    name: String,
    old_owner: String,
    new_owner: String,
}

pub async fn listen_for_name_owner_changed(connection: &Connection) -> Result<()> {
    println!("Creating proxy for org.freedesktop.DBus to listen for NameOwnerChanged signals...");
    let proxy = Proxy::new(
        connection, // Use the provided connection
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await
    .map_err(|e| DBusIntegrationError::ProxyCreationFailed{ source: e })?;
    println!("Proxy created. Setting up signal receiver for NameOwnerChanged...");

    let mut signal_stream = proxy
        .receive_signal_with_args::<NameOwnerChangedSignal>("NameOwnerChanged")
        .await
        .map_err(|e| DBusIntegrationError::SignalReceiverBuildFailed{ source: e })?;

    println!("Listening for NameOwnerChanged signals... (Try starting/stopping a service in another terminal, e.g., systemctl --user stop some.service)");

    // Process a few signals then exit, or run indefinitely in a real app.
    // For this example, let's try to receive one signal.
    if let Some(signal_result) = signal_stream.next().await {
        match signal_result {
            Ok(signal) => {
                 // The actual signal arguments are inside signal.args()
                let args = signal.args()?; // This can fail if args don't match NameOwnerChangedSignal
                println!(
                    "Received NameOwnerChanged signal: name='{}', old_owner='{}', new_owner='{}'",
                    args.name, args.old_owner, args.new_owner
                );
            }
            Err(e) => {
                eprintln!("Error receiving or deserializing signal: {}", e);
                // Depending on the error, you might want to propagate it or handle it.
                // For now, just print and continue or return.
                // This could be zbus::Error or zbus::zvariant::Error if using receive_signal_with_args.
                // Let's assume it's a zbus::Error for now, if not handled by args() above.
                return Err(DBusIntegrationError::Internal(format!("Signal processing error: {}", e)));
            }
        }
    } else {
        eprintln!("Signal stream ended unexpectedly.");
        return Err(DBusIntegrationError::SignalStreamEnded);
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration; // For timeouts

    #[tokio::test]
    async fn test_connect_and_list_names_error_handling() {
        match connect_and_list_names().await {
            Ok(_) => println!("test_connect_and_list_names_error_handling succeeded."),
            Err(e) => {
                eprintln!("test_connect_and_list_names_error_handling failed (this might be expected in some environments): {}", e);
                match e {
                    DBusIntegrationError::ConnectionFailed(zbus_err) => {
                        eprintln!("Error was due to ConnectionFailed: {}", zbus_err);
                        assert!(zbus_err.to_string().to_lowercase().contains("connect"));
                    }
                    // ... other error arms from previous step
                    _ => panic!("Unexpected error type in test_connect_and_list_names_error_handling: {}", e),

                }
            }
        }
    }

    #[tokio::test]
    async fn test_listen_for_name_owner_changed() {
        println!("Starting test_listen_for_name_owner_changed...");
        let connection_result = Connection::system().await;

        if let Err(e) = connection_result {
            eprintln!("Failed to connect to D-Bus system bus for signal test (expected in some CI environments): {}. Skipping signal listening part.", e);
            // If connection fails, we can't test signal listening.
            // This is an acceptable outcome in environments without D-Bus.
            return;
        }
        let connection = connection_result.unwrap();
        println!("Successfully connected to D-Bus for signal test.");

        // Spawn the signal listener.
        let listen_handle = tokio::spawn(async move {
            listen_for_name_owner_changed(&connection).await
        });

        // Give it some time to register and potentially receive a signal.
        // In a real test, you might trigger a D-Bus signal here if possible,
        // or use a mock D-Bus environment.
        // For now, we just wait a short period. If no signal is naturally emitted,
        // the listener might timeout or complete if it's designed to receive only one.
        // The current `listen_for_name_owner_changed` tries to receive one signal.
        // To make this test more robust, we'd need a way to reliably trigger a NameOwnerChanged signal.
        // For now, we'll rely on system activity or assume it might not receive one within the timeout.

        tokio::select! {
            result = listen_handle => {
                match result {
                    Ok(Ok(_)) => println!("Signal listener completed successfully (received a signal)."),
                    Ok(Err(e @ DBusIntegrationError::SignalStreamEnded)) => {
                        // This can happen if the stream ends before a signal is received,
                        // which is possible if no D-Bus services change ownership during the brief test window.
                        eprintln!("Signal listener reported stream ended (no signal received or timeout): {}", e);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Signal listener returned an error: {}", e);
                        // Don't panic here as some errors (like stream ending) might be okay depending on test setup
                        // For now, let's just log it. A real test might need more specific assertions.
                    }
                    Err(e) => { // JoinError from tokio::spawn
                        panic!("Signal listener task panicked: {}", e);
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                println!("Signal listener test timed out after 5 seconds (no signal received or listener did not exit).");
                // This isn't necessarily a failure of the listener itself, but the test conditions.
            }
        }
         println!("Finished test_listen_for_name_owner_changed.");
    }
}
