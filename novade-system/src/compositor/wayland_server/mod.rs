pub mod client;
// pub mod dispatcher;
pub mod error;
pub mod event_dispatcher;
pub mod events;
pub mod protocol_spec; // Added module
pub mod event_sender;
pub mod message;
pub mod object_registry;
pub mod objects;
pub mod protocol;
pub mod protocols;
pub mod socket;

// Imports for signal handling and server lifecycle
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::sync::{
    atomic::{AtomicBool, Ordering as AtomicOrdering},
    Arc,
};
use crate::compositor::wayland_server::socket::cleanup_socket_path_from_display_num;

// Standard library and crate imports
use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicU64, Ordering as StdOrdering};
use std::os::unix::io::AsRawFd; // For Poller
use std::time::Duration; // For Poller timeout

// Polling crate imports
use polling::{Event, Poller};

// Re-export types from submodules
pub use client::{Client, ClientError, validate_client_uid};
pub use event_dispatcher::{EventDispatcher, WaylandEvent};
pub use message::Message as WaylandMessageProtocol;
pub use object_registry::ObjectRegistry;
pub use socket::init_wayland_socket;
// Protocol related items might also be re-exported if needed widely
pub use protocol_spec::ProtocolManager;


// Global state
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

// Token for the listener socket in the Poller
const LISTENER_TOKEN: usize = 0;
const CLIENT_TIMEOUT_DURATION: std::time::Duration = std::time::Duration::from_secs(60);


pub fn run_server(listener_stream: UnixListener, display_num: u32) -> Result<(), String> {
    println!(
        "Wayland server listening on display {} (socket path: /run/user/{}/wayland-{})",
        display_num,
        nix::unistd::getuid(),
        display_num
    );

    // Ensure listener is non-blocking for Poller. init_wayland_socket should ideally do this,
    // but it's also set in start_wayland_server before calling run_server.
    // If it was already set, this is a no-op.
    listener_stream.set_nonblocking(true)
        .map_err(|e| format!("run_server: Failed to set listener to non-blocking: {}", e))?;

    let poller = Poller::new().map_err(|e| format!("Failed to create Poller: {}", e))?;

    // Register the main listener socket for readability.
    // The key (LISTENER_TOKEN) is used to identify events from the listener.
    // SAFETY: `add` requires the fd to be valid and not closed while poller is using it.
    // `listener_stream` owns the fd and remains in scope for the lifetime of `poller.add` usage.
    poller.add(listener_stream.as_raw_fd(), Event::readable(LISTENER_TOKEN))
        .map_err(|e| format!("Failed to register listener with Poller: {}", e))?;

    let mut clients: HashMap<u64, Client> = HashMap::new();
    let mut object_registry = ObjectRegistry::new(); // wl_display (ID 1) is created in ObjectRegistry::new()
    let mut event_dispatcher = EventDispatcher::new();
    let mut poller_events = Vec::new(); // Buffer for Poller events

    // Initialize ProtocolManager and load core protocols
    let mut protocol_manager = ProtocolManager::new();
    protocol_spec::load_core_protocols(&mut protocol_manager);
    // Convert to Arc for shared access if ProtocolManager needs to be shared (e.g. with other threads).
    // For single-threaded run_server, direct reference is fine if it's not moved.
    // However, client.handle_readable_and_get_messages will need it.
    // If client methods are on Client struct, it might need a reference.
    // Let's pass it by reference for now.

    println!("Wayland display object (ID 1) registered. Protocol manager initialized.");

    while !SHUTDOWN_REQUESTED.load(AtomicOrdering::SeqCst) {
        poller_events.clear(); // Clear events from the previous iteration

        // Wait for events with a timeout to periodically check SHUTDOWN_REQUESTED.
        match poller.wait(&mut poller_events, Some(Duration::from_millis(100))) {
            Ok(_) => { /* Events were received or timeout occurred */ }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is expected, continue to check shutdown flag and loop again.
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // Interrupted by a signal (e.g. SIGINT/SIGTERM handled by our signal thread)
                // The SHUTDOWN_REQUESTED flag should be set by the signal handler.
                // Log and continue to check the flag.
                println!("Poller::wait interrupted, likely by signal. Checking shutdown status.");
                continue;
            }
            Err(e) => {
                // Other Poller errors are critical.
                return Err(format!("Poller::wait failed: {}", e));
            }
        }

        let mut clients_to_disconnect_ids: Vec<u64> = Vec::new();

        for event in poller_events.iter() {
            if event.key == LISTENER_TOKEN {
                // Event on the listener socket: new client connection(s) waiting.
                if event.readable {
                    loop { // Accept multiple connections if available (edge-triggered behavior)
                        match listener_stream.accept() {
                            Ok((stream, _addr)) => {
                                // Client::new now ensures stream is non-blocking.
                                // If that failed, it logged an error. Here we proceed.
                                match nix::sys::socket::get_peer_creds(&stream) {
                                    Ok(creds) => {
                                        if !validate_client_uid(creds.uid()) {
                                            eprintln!("Client UID {} validation failed. Closing connection.", creds.uid());
                                            // Stream is dropped, connection closed.
                                            continue;
                                        }
                                        let client_id_val = NEXT_CLIENT_ID.fetch_add(1, StdOrdering::Relaxed);
                                        let new_client = Client::new(client_id_val, stream, creds);
                                        let client_fd = new_client.stream.as_raw_fd();

                                        println!("Client connected: ID={} (fd={})", new_client.id, client_fd);

                                        // Register the new client's stream with the Poller.
                                        // Key for client is its ID. Event for readability.
                                        // SAFETY: Client struct (and its stream) stored in `clients` map.
                                        // FD valid as long as client is in map and not yet explicitly closed.
                                        if let Err(e) = poller.add(client_fd, Event::readable(client_id_val as usize)) {
                                            eprintln!("Failed to register client {} (fd={}) with Poller: {}. Closing connection.", client_id_val, client_fd, e);
                                            // Client struct `new_client` is dropped, closing its stream.
                                            continue;
                                        }
                                        clients.insert(client_id_val, new_client);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to get client credentials: {}. Closing connection attempt.", e);
                                        // `stream` from accept is dropped, closing it.
                                        continue;
                                    }
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // No more pending connections to accept on the listener.
                                break;
                            }
                            Err(e) => {
                                // Error accepting a connection. Log and continue.
                                // This might be a transient error or a problem with a specific connection.
                                eprintln!("Error accepting new connection: {}", e);
                                // Avoid busy-looping on accept errors if not WouldBlock.
                                std::thread::sleep(Duration::from_millis(50));
                                break;
                            }
                        }
                    }
                    // Re-register listener if it's edge-triggered (though `add` is often level-triggered for readability)
                    // For level-triggered, this is not strictly necessary after every accept batch,
                    // but good practice if behavior is uncertain or if using `Poller::modify` for other reasons.
                    // poller.modify(&listener_stream, Event::readable(LISTENER_TOKEN))
                    //    .map_err(|e| format!("Failed to re-register listener: {}", e))?;
                }
            } else {
                // Event for an existing client socket.
                let client_id = event.key as u64;

                if let Some(client) = clients.get_mut(&client_id) {
                    if event.readable || event.error || event.hangup {
                        // Attempt to read and process messages.
                        // Pass ProtocolManager and ObjectRegistry for dynamic parsing.
                        match client.handle_readable_and_get_messages(&protocol_manager, &object_registry) {
                            Ok(messages) => {
                                for msg in messages {
                                    event_dispatcher.post_event(WaylandEvent::ClientMessage {
                                        client_id,
                                        message: msg,
                                    });
                                }
                                // If after processing, client is still fine, re-register for next readable event.
                                // With level-triggering this might not be strictly needed if data is still buffered,
                                // but for edge-triggering or to be safe:
                                // poller.modify(client.stream.as_raw_fd(), Event::readable(client_id as usize))
                                //    .unwrap_or_else(|e| eprintln!("Failed to re-register client {}: {}", client_id, e));
                            }
                            Err(ClientError::ConnectionClosed) => {
                                println!("Client {} disconnected (reported by handle_readable).", client_id);
                                clients_to_disconnect_ids.push(client_id);
                            }
                            Err(ClientError::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // This means no complete message was parsed, but connection is fine.
                                // Poller will notify again when more data arrives.
                            }
                            Err(e) => {
                                eprintln!("Error handling client {}: {:?}. Marking for disconnect.", client_id, e);
                                clients_to_disconnect_ids.push(client_id);
                            }
                        }
                    }
                    // Note: `polling` crate might consolidate hangup/error into readable returning 0/error.
                    // Explicitly checking event.error or event.hangup might be useful for some Poller backends.
                    if event.hangup || event.error { // Ensure hangup or error also leads to disconnect
                        if !clients_to_disconnect_ids.contains(&client_id) { // Avoid double-adding
                            println!("Client {} hangup/error event from Poller. Marking for disconnect.", client_id);
                            clients_to_disconnect_ids.push(client_id);
                        }
                    }
                } else {
                    // Should not happen if poller and clients map are consistent.
                    eprintln!("Poller event for unknown client key: {}. Ignoring.", event.key);
                    // Potentially try to deregister this unknown key from poller if it persists.
                    // poller.delete_by_key(event.key).unwrap_or_default();
                }
            }
        }

        // Process disconnections
        for client_id in &clients_to_disconnect_ids {
            if let Some(client_to_remove) = clients.remove(client_id) {
                println!("Disconnecting client {}.", client_id);
                // Deregister from Poller. client_to_remove.stream.as_raw_fd() is now the focus.
                // The client struct is removed, its stream will be closed upon drop.
                // It's important to deregister before the FD becomes invalid.
                if let Err(e) = poller.delete(client_to_remove.stream.as_raw_fd()) {
                    // Log error if delete fails, but proceed with client removal.
                    // This might happen if FD was already closed or is invalid.
                    eprintln!("Failed to deregister client {} (fd={}) from Poller: {}", client_id, client_to_remove.stream.as_raw_fd(), e);
                }
                event_dispatcher.post_event(WaylandEvent::ClientDisconnect { client_id: *client_id });
                println!("Client {} (fd={}) removed and deregistered.", client_id, client_to_remove.stream.as_raw_fd());
            }
        }
        clients_to_disconnect_ids.clear();


        // Process all pending Wayland events
        event_dispatcher.process_pending_events(&mut object_registry, &mut clients);

        // Check for client timeouts
        let now = std::time::SystemTime::now();
        let mut clients_timed_out_ids: Vec<u64> = Vec::new();

        for (id, client) in clients.iter() { // Iterate immutably first
            if now.duration_since(client.last_activity_ts).unwrap_or_default() > CLIENT_TIMEOUT_DURATION {
                println!("[Server] Client {} timed out. Initiating disconnect.", client.id);
                clients_timed_out_ids.push(*id);
            }
        }

        for client_id_to_timeout in clients_timed_out_ids {
            if let Some(client_to_remove) = clients.get(&client_id_to_timeout) { // Get client to find its FD
                 // Deregister from Poller before posting disconnect event or removing from map.
                if let Err(e) = poller.delete(client_to_remove.stream.as_raw_fd()) {
                    eprintln!("[Server] Error deregistering timed-out client {} (fd={}) from poller: {}",
                        client_id_to_timeout, client_to_remove.stream.as_raw_fd(), e);
                } else {
                    println!("[Server] Deregistered timed-out client {} (fd={}) from poller.",
                        client_id_to_timeout, client_to_remove.stream.as_raw_fd());
                }
            }
            // Post a ClientDisconnect event. The EventDispatcher will handle removal from the `clients` map
            // and resource cleanup via ObjectRegistry.
            event_dispatcher.post_event(WaylandEvent::ClientDisconnect { client_id: client_id_to_timeout });
        }
        // Note: Actual removal from `clients` map and resource cleanup happens when ClientDisconnect is processed.

        // No explicit sleep needed here as Poller::wait() with a timeout handles blocking.
    }

    println!("Shutdown requested, exiting run_server for display {}.", display_num);
    // Cleanup remaining clients and the listener from Poller
    println!("Deregistering listener (fd={}) from Poller.", listener_stream.as_raw_fd());
    poller.delete(listener_stream.as_raw_fd()).unwrap_or_else(|e| {
        eprintln!("Error deregistering listener from Poller: {}", e);
    });

    println!("Closing all remaining client connections for display {}...", display_num);
    for (id, client) in clients.iter() {
         println!("Deregistering client {} (fd={}) from Poller during shutdown.", id, client.stream.as_raw_fd());
        poller.delete(client.stream.as_raw_fd()).unwrap_or_else(|e| {
            eprintln!("Error deregistering client {} (fd={}) from Poller during shutdown: {}", id, client.stream.as_raw_fd(), e);
        });
    }
    clients.clear(); // Drops all Client instances, closing their streams.
    println!("All client connections for display {} closed and deregistered.", display_num);
    Ok(())
}

pub fn start_wayland_server(display_num: u32) -> Result<(), String> {
    println!("Attempting to start Wayland server on display {}...", display_num);
    SHUTDOWN_REQUESTED.store(false, AtomicOrdering::SeqCst);

    let signals_display_num_clone = display_num;
    let mut signals = match Signals::new(&[SIGINT, SIGTERM]) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to setup signal iterator for display {}: {}", display_num, e)),
    };

    let signal_handler_thread = std::thread::Builder::new()
        .name(format!("signal-handler-disp-{}", signals_display_num_clone))
        .spawn(move || {
            println!("[SignalHandler-{}] Listening for SIGINT/SIGTERM...", signals_display_num_clone);
            for sig in signals.forever() {
                println!("[SignalHandler-{}] Received signal: {:?}. Requesting shutdown.", signals_display_num_clone, sig);
                SHUTDOWN_REQUESTED.store(true, AtomicOrdering::SeqCst);
                break;
            }
            println!("[SignalHandler-{}] Signal handling thread exiting.", signals_display_num_clone);
        });

    if let Err(e) = signal_handler_thread {
        return Err(format!("Failed to spawn signal handler thread for display {}: {}", display_num, e));
    }

    match init_wayland_socket(display_num) {
        Ok(listener) => {
            // Ensure listener is non-blocking. init_wayland_socket should ideally handle this.
            // Redundant if already set, but safe.
            if let Err(e) = listener.set_nonblocking(true) {
                cleanup_socket_path_from_display_num(display_num)
                    .unwrap_or_else(|cleanup_err| eprintln!("Error during socket cleanup after listener setup failure for display {}: {}", display_num, cleanup_err));
                return Err(format!("Failed to set listener to non-blocking for display {}: {}", display_num, e));
            }
            println!("Wayland server socket initialized for display {}.", display_num);

            let run_server_result = run_server(listener, display_num);

            println!("Server loop for display {} finished. Cleaning up socket.", display_num);
            if let Err(e) = cleanup_socket_path_from_display_num(display_num) {
                eprintln!("Error during socket cleanup for display {}: {}", display_num, e);
            } else {
                println!("Socket for display {} cleaned up successfully.", display_num);
            }
            run_server_result
        }
        Err(e) => {
            eprintln!("Failed to initialize Wayland socket for display {}: {}", display_num, e);
            cleanup_socket_path_from_display_num(display_num)
                .unwrap_or_else(|cleanup_err| eprintln!("Error during socket cleanup after init failure for display {}: {}", display_num, cleanup_err));
            Err(format!("Socket initialization failed for display {}: {}", display_num, e))
        }
    }
}

// Comment out the problematic test as requested by the subtask.
/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::net::{UnixListener as StdUnixListener, UnixStream as StdUnixStream};
    use tempfile::tempdir;
    use std::thread;
    // use std::time::Duration; // Already imported via top-level `use std::time::Duration`
    use crate::compositor::wayland_server::message::Message as WaylandMessage; // Alias if needed

    fn temp_socket_path(dir: &tempfile::TempDir, name: &str) -> std::path::PathBuf {
        dir.path().join(name)
    }

    // #[test]
    // fn test_run_server_accept_client_parses_message_and_dispatches_event() {
    //     // This test is commented out because it requires significant refactoring
    //     // to work with the new server lifecycle management (signal handling, graceful shutdown, Poller).
    //     // The current `start_wayland_server` function now includes a signal handling thread
    //     // and `run_server` uses an event loop based on `Poller`.
    //     // Testing this would involve:
    //     // 1. Running the server in a thread.
    //     // 2. Connecting a client.
    //     // 3. Sending a message.
    //     // 4. Verifying the message was processed (potentially through side effects or mock objects).
    //     // 5. Signaling the server thread to shut down gracefully.
    //     // 6. Joining the server thread.
    //     // This is more of an integration test and is deferred for now.
    //     println!("Skipping test_run_server_accept_client_parses_message_and_dispatches_event due to Poller and signal handling changes.");
    // }
}
*/
