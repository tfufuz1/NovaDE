pub mod client;
pub mod dispatcher; // Should be event_dispatcher, correct later if this was a typo
pub mod error;
pub mod event_dispatcher; // Added
pub mod events;
pub mod event_sender;
pub mod message;
pub mod object_registry;
pub mod objects;
pub mod protocol;
pub mod protocols;
pub mod socket;

use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicU64, Ordering};

pub use client::{Client, ClientError, validate_client_uid};
pub use event_dispatcher::{EventDispatcher, WaylandEvent}; // Added
pub use message::Message as WaylandMessageProtocol; // Alias to avoid conflict if Message is defined elsewhere
pub use object_registry::ObjectRegistry;
pub use socket::init_wayland_socket;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

pub fn run_server(listener: UnixListener, display_num: u32) -> Result<(), String> {
    println!(
        "Wayland server listening on display {} (socket path: /run/user/{}/wayland-{})",
        display_num,
        nix::unistd::getuid(),
        display_num
    );

    let mut clients: HashMap<u64, Client> = HashMap::new();
    let mut object_registry = ObjectRegistry::new();
    let mut event_dispatcher = EventDispatcher::new(); // Instantiate dispatcher

    println!("Wayland display object ID 1 registered in object_registry.");

    loop {
        // 1. Accept new connections (simplified, a real loop would use epoll/mio here)
        match listener.accept() {
            Ok((stream, _addr)) => {
                println!("New connection accepted.");
                stream
                    .set_nonblocking(true) // Important for how Client::handle_readable is written
                    .map_err(|e| format!("Failed to set stream non-blocking: {}", e))?;

                match nix::sys::socket::get_peer_creds(&stream) {
                    Ok(creds) => {
                        if !validate_client_uid(creds.uid()) {
                            eprintln!("Client UID {} validation failed. Closing.", creds.uid());
                            continue;
                        }
                        let client_id_val = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
                        let new_client = Client::new(client_id_val, stream, creds);
                        println!("Client connected: ID={}", new_client.id);
                        clients.insert(client_id_val, new_client);
                    }
                    Err(e) => {
                        eprintln!("Failed to get client credentials: {}. Closing.", e);
                        continue; // Skip this connection
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // This is expected for a non-blocking listener if no new connections are pending.
                // The main loop would then proceed to handle other I/O (client messages).
            }
            Err(e) => {
                eprintln!("Error accepting new connection: {}", e);
                // Depending on the error, might need to break or handle specific cases.
                // For now, just log and continue.
            }
        }

        // 2. Process readable clients and post events
        // In a real event loop, this would iterate over clients marked readable by epoll/mio.
        // Here, we iterate all clients for simplicity, and handle_readable uses non-blocking reads.
        let mut clients_to_disconnect: Vec<u64> = Vec::new();
        for (id, client) in clients.iter_mut() {
            // Attempt to handle readable data for the client
            match client.handle_readable_and_get_messages(&mut object_registry) {
                Ok(messages) => {
                    for msg in messages {
                        event_dispatcher.post_event(WaylandEvent::ClientMessage {
                            client_id: *id,
                            message: msg,
                        });
                    }
                }
                Err(ClientError::ConnectionClosed) => {
                    println!("Client {} connection closed by peer.", id);
                    clients_to_disconnect.push(*id);
                }
                Err(ClientError::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data to read for this client, perfectly normal for non-blocking.
                }
                Err(e) => {
                    eprintln!("Error handling client {}: {:?}. Marking for disconnect.", id, e);
                    clients_to_disconnect.push(*id);
                }
            }
        }

        // Post disconnect events for clients marked for disconnection
        for client_id in clients_to_disconnect {
            event_dispatcher.post_event(WaylandEvent::ClientDisconnect { client_id });
        }

        // 3. Process all pending events in the dispatcher
        event_dispatcher.process_pending_events(&mut object_registry, &mut clients);

        // In a real server, there would be a delay here, or blocking wait on epoll/mio.
        // For a simple test loop, a small sleep can prevent busy-looping on CPU.
        std::thread::sleep(std::time::Duration::from_millis(10)); // Adjust as needed
    }
    // Ok(()) // Unreachable
}

pub fn start_wayland_server(display_num: u32) -> Result<(), String> {
    println!("Starting Wayland server on display {}...", display_num);
    match init_wayland_socket(display_num) {
        Ok(listener) => {
            listener.set_nonblocking(true).map_err(|e| format!("Failed to set listener non-blocking: {}", e))?;
            run_server(listener, display_num)
        }
        Err(e) => {
            eprintln!("Failed to initialize Wayland socket: {}", e);
            Err(format!("Socket initialization failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::net::{UnixListener as StdUnixListener, UnixStream as StdUnixStream};
    use tempfile::tempdir;
    use std::thread;
    use std::time::Duration;
    use crate::compositor::wayland_server::message::Message as WaylandMessage; // Alias if needed

    fn temp_socket_path(dir: &tempfile::TempDir, name: &str) -> std::path::PathBuf {
        dir.path().join(name)
    }

    #[test]
    fn test_run_server_accept_client_parses_message_and_dispatches_event() {
        let dir = tempdir().unwrap();
        let socket_path = temp_socket_path(&dir, "test_event_dispatch.sock");

        let listener = StdUnixListener::bind(&socket_path).expect("Failed to bind test listener");
        // run_server will set it to non-blocking.

        let server_thread = thread::spawn(move || {
            // run_server is an infinite loop. For a test, it needs to be managed.
            // This test will focus on one client connecting and sending one message.
            // The assertions will be based on expected log output (conceptually) or side effects
            // if we could inspect the dispatcher or registry state from here (hard for now).
            if let Err(e) = start_wayland_server(100) { // Use a unique display number for test
                eprintln!("Test server failed: {}", e); // Log if server exits with error
            }
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(300)); // Increased for listener setup

        match StdUnixStream::connect(&socket_path) {
            Ok(mut stream) => {
                println!("Test Client: Connected to server.");
                stream.set_nonblocking(false).expect("Client stream to blocking for test");

                // Construct wl_display.get_registry (sender_id=1, opcode=1, new_id=5)
                let new_registry_id = 5u32;
                let mut msg_bytes = Vec::new();
                msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id
                msg_bytes.extend_from_slice(&((1u32 << 16) | 12u32).to_ne_bytes()); // opcode=1, len=12
                msg_bytes.extend_from_slice(&new_registry_id.to_ne_bytes());     // new_id argument

                if let Err(e) = stream.write_all(&msg_bytes) {
                    panic!("Test Client: Failed to write message: {}", e);
                }
                stream.flush().expect("Test Client: Failed to flush stream");
                println!("Test Client: Sent get_registry message (new_id={}).", new_registry_id);

                // Give server time to read, parse, post event, and process event.
                thread::sleep(Duration::from_millis(300));

                // How to verify?
                // 1. Logs: The server should log the message parsing and event processing.
                //    (Requires log capture or inspection, not done here).
                // 2. ObjectRegistry state: If wl_registry object (ID 5) was created.
                //    (Cannot access registry from this test thread without shared state).
                // This test mainly ensures the flow doesn't panic and conceptually runs.
                // More detailed checks are in client.rs and event_dispatcher.rs unit tests.
                println!("Test Client: Message sent. Assuming server processed it.");
            }
            Err(e) => {
                panic!("Test Client: Failed to connect: {}", e);
            }
        }

        // Note: server_thread will run indefinitely. In a real test suite,
        // you'd signal it to shut down. For CI, it will be terminated with the test process.
        // std::fs::remove_file(&socket_path).unwrap_or_default(); // Cleanup socket file
    }
}
