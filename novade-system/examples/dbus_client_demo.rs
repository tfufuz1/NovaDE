// novade-system/examples/dbus_client_demo.rs

// Use the published library novade_system
use novade_system::dbus_integration::{self, DBusIntegrationError};
use novade_core; // Use novade_core directly as a dependency of the novade-system package.

use zbus::Connection; // Import Connection for signal listening setup

#[tokio::main]
async fn main() {
    println!("NovaDE System D-Bus Integration Demo");
    println!("====================================");

    // Initialize logging from novade_core
    // This requires novade_core to be accessible.
    match novade_core::logging::init_minimal_logging() {
        Ok(_) => println!("Minimal logging initialized."),
        Err(e) => eprintln!("Failed to initialize logging: {}", e),
    }
    
    println!("\n--- Testing connect_and_list_names ---");
    match dbus_integration::connect_and_list_names().await {
        Ok(_) => println!("connect_and_list_names executed successfully."),
        Err(e) => eprintln!("Error in connect_and_list_names: {}", e),
    }

    println!("\n--- Testing listen_for_name_owner_changed ---");
    let connection_result = Connection::system().await;

    match connection_result {
        Ok(connection) => {
            println!("Successfully connected to D-Bus for signal listening demo.");
            let listen_handle = tokio::spawn(async move {
                match dbus_integration::listen_for_name_owner_changed(&connection).await {
                    Ok(_) => println!("listen_for_name_owner_changed finished (likely received a signal or completed its task)."),
                    Err(DBusIntegrationError::SignalStreamEnded) => {
                        println!("listen_for_name_owner_changed: Signal stream ended (no specific signal received or timeout if applicable). This is often normal.");
                    }
                    Err(e) => {
                        eprintln!("Error in listen_for_name_owner_changed: {}", e);
                    }
                }
            });

            println!("Listening for NameOwnerChanged for a few seconds...");
            tokio::select! {
                _ = listen_handle => {
                    println!("Signal listener task completed.");
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                    println!("Finished 10-second listening period for NameOwnerChanged signals.");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to D-Bus for signal listening demo: {}", e);
            eprintln!("Skipping listen_for_name_owner_changed test.");
        }
    }
    
    println!("\n--- D-Bus Integration Demo Finished ---");
}
