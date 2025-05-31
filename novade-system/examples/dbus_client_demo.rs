use novade_system::dbus_integration;
use tokio;

#[tokio::main]
async fn main() {
    println!("Starting D-Bus simple client demo...");
    if let Err(e) = dbus_integration::connect_and_list_names().await {
        eprintln!("Demo failed during connect_and_list_names: {}", e);
    }
    // Keep the demo running for a bit to allow signals to be received.
    // In a real application, the main loop would keep things alive.
    // For a simple demo, a short sleep can help observe signals.
    println!("Demo running. Listening for NameOwnerChanged signals for 30 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    println!("Demo finished.");
}
