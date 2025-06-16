use std::collections::{HashMap, VecDeque};
use crate::compositor::wayland_server::client::Client;
use crate::compositor::wayland_server::message::Message;
use crate::compositor::wayland_server::object_registry::{ObjectRegistry, WaylandObject};
use super::event_sender::EventSender; // Added import

// Protocol error codes (example, replace with actual Wayland codes or an enum)
const WL_DISPLAY_ERROR_INVALID_OBJECT: u32 = 0; // Example code
const WL_DISPLAY_ERROR_INVALID_METHOD: u32 = 1; // Example code
const WL_DISPLAY_ERROR_NO_MEMORY: u32 = 2;    // Example code
const WL_DISPLAY_ERROR_IMPLEMENTATION: u32 = 3; // Example for dispatch_request failure


/// Represents events within the Wayland server.
#[derive(Debug, Clone)]
pub enum WaylandEvent {
    ClientMessage {
        client_id: u64,
        message: Message,
    },
    ClientDisconnect {
        client_id: u64,
        // reason: Option<String>, // Future enhancement
    },
    Signal {
        signal_name: String, // e.g., "shutdown"
    },
    SendToClient { // Placeholder for outgoing message handling
        client_id: u64,
        message_bytes: Vec<u8>,
    },
    // Could add TimerElapsed, UserInput, etc. later
}

/// A simple FIFO event queue.
#[derive(Debug)]
pub struct EventQueue {
    queue: VecDeque<WaylandEvent>,
}

impl EventQueue {
    /// Creates a new, empty event queue.
    pub fn new() -> Self {
        EventQueue {
            queue: VecDeque::new(),
        }
    }

    /// Adds an event to the back of the queue.
    pub fn enqueue(&mut self, event: WaylandEvent) {
        self.queue.push_back(event);
    }

    /// Removes and returns an event from the front of the queue.
    /// Returns `None` if the queue is empty.
    pub fn dequeue(&mut self) -> Option<WaylandEvent> {
        self.queue.pop_front()
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of events in the queue.
    #[allow(dead_code)] // May be useful later
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Manages and processes events for the Wayland server.
#[derive(Debug)]
pub struct EventDispatcher {
    event_queue: EventQueue,
}

impl EventDispatcher {
    /// Creates a new event dispatcher.
    pub fn new() -> Self {
        EventDispatcher {
            event_queue: EventQueue::new(),
        }
    }

    /// Posts an event to the event queue.
    pub fn post_event(&mut self, event: WaylandEvent) {
        self.event_queue.enqueue(event);
        // In a multi-threaded scenario, this might involve waking up a processing thread.
    }

    /// Processes all currently pending events in the queue.
    ///
    /// # Arguments
    /// * `registry`: A mutable reference to the `ObjectRegistry` to interact with Wayland objects.
    /// * `clients`: A mutable reference to the `HashMap` of connected clients.
    pub fn process_pending_events(
        &mut self,
        registry: &mut ObjectRegistry,
        clients: &mut HashMap<u64, Client>,
    ) {
        let mut events_processed = 0;
        while let Some(event) = self.event_queue.dequeue() {
            events_processed += 1;
            // println!("[EventDispatcher] Processing event: {:?}", event); // Log the event being processed

            match event {
                WaylandEvent::ClientMessage { client_id, message } => {
                    if let Some(client_info) = clients.get(&client_id) { // Get immutable ref first
                        println!(
                            "[EventDispatcher] ClientMessage from client_id {}: object_id={}, opcode={}",
                            client_id, message.sender_id, message.opcode
                        );

                        if let Some(target_object) = registry.get_object_mut(message.sender_id) {
                            // Create an EventSender for this dispatch operation.
                            // This borrows self.event_queue.queue mutably for the duration of dispatch.
                            let mut event_sender = EventSender::new(&mut self.event_queue.queue);

                            match target_object.dispatch_request(
                                message.opcode,
                                message.args, // Consumed here
                                client_info,  // Pass client info
                                message.sender_id, // ID of the object receiving the call
                                &mut event_sender,
                                registry, // Pass mutable registry
                            ) {
                                Ok(()) => {
                                    // Request dispatched successfully
                                    println!("[EventDispatcher] Request (obj: {}, opcode: {}) dispatched successfully for client {}.", message.sender_id, message.opcode, client_id);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[EventDispatcher] Error dispatching request (obj: {}, opcode: {}) for client {}: {}",
                                        message.sender_id, message.opcode, client_id, e
                                    );
                                    // Send a wl_display.error event to the client.
                                    // Use message.sender_id as the error_object_id, as it's the object that failed.
                                    // Error code can be generic for now.
                                    if let Err(send_err) = event_sender.send_protocol_error(
                                        client_id,
                                        message.sender_id, // Object where error occurred
                                        WL_DISPLAY_ERROR_IMPLEMENTATION, // Generic "implementation" error
                                        format!("Request failed: {}", e),
                                    ) {
                                        eprintln!("[EventDispatcher] CRITICAL: Failed to send protocol error to client {}: {}", client_id, send_err);
                                        // If sending the error fails, we might need to disconnect the client.
                                        // self.post_event(WaylandEvent::ClientDisconnect { client_id }); // Avoid direct call in loop
                                    }
                                }
                            }
                        } else {
                            eprintln!(
                                "[EventDispatcher] Error: Target object_id {} not found for message from client_id {}.",
                                message.sender_id, client_id
                            );
                            // Create an EventSender to send the error
                            let mut event_sender = EventSender::new(&mut self.event_queue.queue);
                            if let Err(send_err) = event_sender.send_protocol_error(
                                client_id,
                                message.sender_id, // The object that was not found
                                WL_DISPLAY_ERROR_INVALID_OBJECT, // "invalid object" error
                                format!("Object ID {} not found.", message.sender_id),
                            ) {
                                 eprintln!("[EventDispatcher] CRITICAL: Failed to send 'invalid object' protocol error to client {}: {}", client_id, send_err);
                            }
                        }
                    } else {
                        eprintln!(
                            "[EventDispatcher] Error: Client with ID {} not found for ClientMessage.",
                            client_id
                        );
                    }
                }
                WaylandEvent::ClientDisconnect { client_id } => {
                    println!("[EventDispatcher] ClientDisconnect event for client_id {}.", client_id);
                    if clients.remove(&client_id).is_some() {
                        println!("[EventDispatcher] Client {} removed.", client_id);
                        // TODO: Clean up resources owned by this client.
                        // This would involve iterating through the object_registry and removing objects
                        // associated with this client_id. This needs careful design, as some objects
                        // might be shared or have different ownership semantics.
                        // For now, we'll implement a simple cleanup.
                        let mut objects_to_remove: Vec<u32> = Vec::new();
                        // Collect all objects associated with the disconnected client.
                        // We don't need to worry about parent/child relationships here specifically,
                        // as destroy_object() will handle cascading.
                        // We also ensure not to try and remove wl_display (ID 1).
                        for (obj_id, entry) in registry.get_entries_iter() {
                            if entry.client_id == client_id && *obj_id != 1 {
                                objects_to_remove.push(*obj_id);
                            }
                        }

                        println!("[EventDispatcher] Client {} disconnected. Found {} objects to clean up: {:?}.", client_id, objects_to_remove.len(), objects_to_remove);

                        for obj_id in objects_to_remove {
                            // Check if the object still exists, as it might have been removed by a cascading delete
                            // from a parent object also owned by the same client.
                            if registry.get_object(obj_id).is_some() {
                                println!("[EventDispatcher] Destroying object {} for disconnected client {}.", obj_id, client_id);
                                if let Err(e) = registry.destroy_object(obj_id) {
                                    // Log error, but continue cleanup. This might happen if an object was already
                                    // destroyed due to cascading from another object in this list.
                                    eprintln!("[EventDispatcher] Error destroying object {}: {} (it might have been already removed by cascade).", obj_id, e);
                                }
                            } else {
                                println!("[EventDispatcher] Object {} for client {} was already removed (likely by cascade). Skipping.", obj_id, client_id);
                            }
                        }
                    } else {
                        eprintln!(
                            "[EventDispatcher] Warning: ClientDisconnect event for unknown client_id {}.",
                            client_id
                        );
                    }
                }
                WaylandEvent::Signal { signal_name } => {
                    println!("[EventDispatcher] Received signal: {}", signal_name);
                    // TODO: Handle signals like shutdown, reconfigure, etc.
                }
                WaylandEvent::SendToClient { client_id, message_bytes } => {
                    if let Some(client) = clients.get_mut(&client_id) {
                        if let Err(e) = client.send_message_bytes(&message_bytes) {
                            eprintln!("[EventDispatcher] Error sending message to client {}: {}. Marking for disconnect.", client_id, e);
                            // Queue a disconnect event for this client.
                            // Avoid direct manipulation of `clients` or `registry` here if it can be handled
                            // by posting another event.
                            self.event_queue.enqueue(WaylandEvent::ClientDisconnect { client_id });
                        } else {
                             println!("[EventDispatcher] Sent {} bytes to client {}.", message_bytes.len(), client_id);
                        }
                    } else {
                        eprintln!("[EventDispatcher] Warning: SendToClient event for unknown client_id {}.", client_id);
                    }
                }
            }
        }
        // if events_processed > 0 {
        //     println!("[EventDispatcher] Processed {} events in this cycle.", events_processed);
        // }
    }
}

// Helper method for ObjectRegistry to iterate for cleanup (temporary location)
// This should ideally be part of ObjectRegistry impl itself.
impl ObjectRegistry {
    pub fn get_entries_iter(&self) -> impl Iterator<Item = (&u32, &super::object_registry::RegistryEntry)> {
        self.entries.iter()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::message::{Argument, Message as WaylandMessage};
    use crate::compositor::wayland_server::object_registry::{ObjectRegistry, WlDisplay, WaylandObject, RegistryEntry}; // Added RegistryEntry
    use crate::compositor::wayland_server::client::Client;
    use std::os::unix::net::UnixStream;
    use nix::sys::socket::UnixCredentials;
    use nix::unistd::{Uid, Gid, Pid};
    use std::io::Read; // For test client to read what server sends


    // Mock WaylandObject for testing dispatch_request
    #[derive(Debug)]
    struct MockDispatchObject {
        name: String,
        last_opcode: Option<u16>,
        should_error: bool,
        events_to_send_on_dispatch: Vec<(u32, u16, Vec<Argument>)>, // (sender_id, opcode, args)
    }
    impl WaylandObject for MockDispatchObject {
        fn dispatch_request(
            &mut self,
            request_opcode: u16,
            _request_args: Vec<Argument>,
            _client_info: &Client,
            object_id: u32,
            event_sender: &mut EventSender,
            _object_registry: &mut ObjectRegistry,
        ) -> Result<(), String> {
            self.last_opcode = Some(request_opcode);
            println!("[MockDispatchObject:{}] Received request opcode {}", self.name, request_opcode);
            if self.should_error {
                Err(format!("Mock error from {}", self.name))
            } else {
                for (sender_obj_id, opcode, args) in self.events_to_send_on_dispatch.drain(..) {
                    event_sender.send_event(_client_info.id, sender_obj_id, opcode, args).unwrap();
                }
                Ok(())
            }
        }
    }


    fn create_mock_client(id: u64) -> (Client, UnixStream) {
        let (stream1, stream2) = UnixStream::pair().unwrap();
        let creds = UnixCredentials::new(Uid::current().as_raw(), Gid::current().as_raw(), Some(Pid::this().as_raw()));
        (Client::new(id, stream1, creds), stream2)
    }


    #[test]
    fn test_event_queue_simple_operations() {
        let mut queue = EventQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        let event1 = WaylandEvent::Signal { signal_name: "test1".to_string() };
        queue.enqueue(event1.clone());
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);

        let event2 = WaylandEvent::Signal { signal_name: "test2".to_string() };
        queue.enqueue(event2.clone());
        assert_eq!(queue.len(), 2);

        let dequeued1 = queue.dequeue().unwrap();
        // Cannot directly compare WaylandEvent due to Message not deriving PartialEq fully yet.
        // Let's check by structure for now.
        if let WaylandEvent::Signal{signal_name} = dequeued1 {
            assert_eq!(signal_name, "test1");
        } else { panic!("Wrong event type"); }


        let dequeued2 = queue.dequeue().unwrap();
         if let WaylandEvent::Signal{signal_name} = dequeued2 {
            assert_eq!(signal_name, "test2");
        } else { panic!("Wrong event type"); }

        assert!(queue.is_empty());
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_event_dispatcher_post_and_process_signal() {
        let mut dispatcher = EventDispatcher::new();
        let mut registry = ObjectRegistry::new();
        let mut clients = HashMap::new();

        let signal_event = WaylandEvent::Signal { signal_name: "shutdown".to_string() };
        dispatcher.post_event(signal_event);

        // process_pending_events should consume and log this event
        // For now, we check if the queue becomes empty. Log output would confirm processing.
        assert!(!dispatcher.event_queue.is_empty());
        dispatcher.process_pending_events(&mut registry, &mut clients);
        assert!(dispatcher.event_queue.is_empty());
    }

    #[test]
    fn test_event_dispatcher_process_client_message() {
        let mut dispatcher = EventDispatcher::new();
        let mut registry = ObjectRegistry::new();
        let mut clients = HashMap::new();

        // Setup a client and an object
        let (client1, _client1_server_stream) = create_mock_client(1);
        let client1_id = client1.id;
        clients.insert(client1_id, client1);

        let obj_id = 100; // Assume this object exists
        registry.new_object(client1_id, obj_id, MockWaylandObject { name: "obj100".into() }, 0).unwrap();

        let test_message = WaylandMessage {
            sender_id: obj_id,
            opcode: 0,
            len: 8, // Header only
            args: vec![],
        };
        let msg_event = WaylandEvent::ClientMessage { client_id: client1_id, message: test_message.clone() };
        dispatcher.post_event(msg_event);

        dispatcher.process_pending_events(&mut registry, &mut clients);
        assert!(dispatcher.event_queue.is_empty());
        // TODO: Verify logging output or mock object interaction if dispatch was implemented.
    }

    #[test]
    fn test_event_dispatcher_process_client_message_object_not_found() {
        let mut dispatcher = EventDispatcher::new();
        let mut registry = ObjectRegistry::new();
        let mut clients = HashMap::new();

        let (client1, _client1_server_stream) = create_mock_client(1);
        let client1_id = client1.id;
        clients.insert(client1_id, client1);

        let non_existent_obj_id = 999;
        let test_message = WaylandMessage {
            sender_id: non_existent_obj_id,
            opcode: 0,
            len: 8,
            args: vec![],
        };
        let msg_event = WaylandEvent::ClientMessage { client_id: client1_id, message: test_message.clone() };
        dispatcher.post_event(msg_event);

        dispatcher.process_pending_events(&mut registry, &mut clients);
        // Check logs for "Target object_id 999 not found"
        assert!(dispatcher.event_queue.is_empty());
    }


    #[test]
    fn test_event_dispatcher_process_client_disconnect() {
        let mut dispatcher = EventDispatcher::new();
        let mut registry = ObjectRegistry::new();
        let mut clients = HashMap::new();

        // Setup a client and an object owned by it
        let (client1, _client1_server_stream) = create_mock_client(1);
        let client1_id = client1.id;
        clients.insert(client1_id, client1);

        let client_obj_id = 101;
        registry.new_object(client1_id, client_obj_id, MockWaylandObject { name: "client1_obj".into() }, 0).unwrap();
        // Server object associated with client (but not owned in same way for cleanup test)
        let server_obj_for_client = registry.new_server_object(client1_id, MockWaylandObject { name: "server_obj_for_client1".into() }, 0).unwrap();


        assert!(clients.contains_key(&client1_id));
        assert!(registry.get_object(client_obj_id).is_some());
        assert!(registry.get_object(server_obj_for_client).is_some());


        let disconnect_event = WaylandEvent::ClientDisconnect { client_id: client1_id };
        dispatcher.post_event(disconnect_event);

        dispatcher.process_pending_events(&mut registry, &mut clients);

        assert!(!clients.contains_key(&client1_id), "Client should be removed from clients map");
        assert!(registry.get_object(client_obj_id).is_none(), "Client's object should be removed from registry");
        // The current cleanup logic in dispatcher also removes server objects associated via client_id.
        assert!(registry.get_object(server_obj_for_client).is_none(), "Server object associated with client should be removed");
        assert!(registry.get_object(1).is_some(), "wl_display should not be removed");
        assert!(dispatcher.event_queue.is_empty());
    }
}
