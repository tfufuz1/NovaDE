// novade-system/src/dbus_integration/mod.rs
use zbus::{Connection, Proxy, Error as ZbusError, SignalReceiver};
use tokio;
use futures_util::stream::StreamExt;

pub mod examples;
pub mod manager;

// TODO: Add D-Bus client logic (connection, proxy, signal handling) here

pub async fn listen_for_name_owner_changes(connection: Connection) -> Result<(), ZbusError> {
    let proxy = Proxy::new(
        connection.clone(),
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus"
    ).await?;
    let mut signal_stream = proxy.receive_signal("NameOwnerChanged").await?;
    println!("Listening for NameOwnerChanged signals...");
    while let Some(signal) = signal_stream.next().await {
        // The actual body type for NameOwnerChanged is (String, String, String)
        // representing name, old_owner, new_owner.
        match signal.body::<(String, String, String)>() {
            Ok(body) => {
                println!("Received NameOwnerChanged signal: name='{}', old_owner='{}', new_owner='{}'", body.0, body.1, body.2);
            }
            Err(e) => {
                eprintln!("Error decoding NameOwnerChanged signal body: {}", e);
            }
        }
    }
    Ok(())
}

pub async fn connect_and_list_names() -> Result<Vec<String>, ZbusError> {
    let connection = Connection::system().await?;

    // Spawn the listener task
    let conn_for_listener = connection.clone();
    tokio::spawn(async move {
        if let Err(e) = listen_for_name_owner_changes(conn_for_listener).await {
            eprintln!("Error in signal listener: {}", e);
        }
    });

    // Continue with existing logic for ListNames
    let proxy = Proxy::new(
        connection.clone(), // Use the original connection
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus"
    ).await?;
    let names: Vec<String> = proxy.call_method("ListNames", &()).await?.body().unwrap_or_else(|_| Vec::new());
    for name in &names {
        println!("Found D-Bus name: {}", name); // Later replace with novade_core::logging if accessible
    }
    Ok(names)
}

pub use manager::{DbusServiceManager, DbusManagerError, Result as DbusManagerResult};

// TODO: Define specific D-Bus interfaces and proxy interactions here
