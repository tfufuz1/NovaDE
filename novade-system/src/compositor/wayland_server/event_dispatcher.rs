use std::collections::{HashMap, VecDeque};
use crate::compositor::wayland_server::client::Client;
use crate::compositor::wayland_server::message::Message;
use crate::compositor::wayland_server::object_registry::{ObjectRegistry, WaylandObject}; // For later use

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
            println!("[EventDispatcher] Processing event: {:?}", event); // Log the event being processed

            match event {
                WaylandEvent::ClientMessage { client_id, message } => {
                    if let Some(_client) = clients.get_mut(&client_id) {
                        // Successfully found the client associated with the message
                        println!(
                            "[EventDispatcher] ClientMessage from client_id {}: object_id={}, opcode={}",
                            client_id, message.sender_id, message.opcode
                        );

                        // Attempt to find the target object in the registry
                        if let Some(_target_object) = registry.get_object_mut(message.sender_id) {
                            // TODO: Implement actual message dispatch to the object.
                            // This would involve calling a method on `_target_object` like:
                            // `_target_object.dispatch(message.opcode, message.args, client_context, server_context)?;`
                            // For now, just log that we found the object.
                            println!(
                                "[EventDispatcher] Target object_id {} found for message from client_id {}.",
                                message.sender_id, client_id
                            );
                            // Example: if it's wl_display.get_registry, the client::handle_readable already made the object.
                            // Here, we would dispatch the actual request to that object if it had methods.
                        } else {
                            eprintln!(
                                "[EventDispatcher] Error: Target object_id {} not found for message from client_id {}.",
                                message.sender_id, client_id
                            );
                            // TODO: Send a protocol error to the client (e.g., invalid object).
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
                        for (obj_id, entry) in registry.get_entries_iter() { // Assuming get_entries_iter() exists
                            if entry.client_id == client_id && *obj_id != 1 { // Don't remove wl_display
                                objects_to_remove.push(*obj_id);
                            }
                        }
                        for obj_id in objects_to_remove {
                            println!("[EventDispatcher] Destroying object {} for disconnected client {}.", obj_id, client_id);
                            if let Err(e) = registry.destroy_object(obj_id) {
                                eprintln!("[EventDispatcher] Error destroying object {}: {}", obj_id, e);
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
                    println!("[EventDispatcher] SendToClient event for client_id {}: {} bytes.", client_id, message_bytes.len());
                    if let Some(client) = clients.get_mut(&client_id) {
                        // TODO: Implement actual message sending logic on client.stream
                        // use std::io::Write;
                        // if let Err(e) = client.stream.write_all(&message_bytes) {
                        //     eprintln!("[EventDispatcher] Error sending message to client {}: {}", client_id, e);
                        //     // Potentially queue ClientDisconnect event for this client
                        // }
                        println!("[EventDispatcher] Placeholder: Would send {} bytes to client {}.", message_bytes.len(), client_id);
                    } else {
                        eprintln!("[EventDispatcher] Warning: SendToClient event for unknown client_id {}.", client_id);
                    }
                }
            }
        }
        if events_processed > 0 {
            println!("[EventDispatcher] Processed {} events.", events_processed);
        }
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
    use crate::compositor::wayland_server::message::{Argument, Message as WaylandMessage}; // Renamed for clarity
    use crate::compositor::wayland_server::object_registry::{ObjectRegistry, WlDisplay, WaylandObject};
    use crate::compositor::wayland_server::client::Client; // For test setup
    use std::os::unix::net::UnixStream;
    use nix::sys::socket::UnixCredentials;
    use nix::unistd::{Uid, Gid, Pid};


    // Mock WaylandObject for testing
    #[derive(Debug)]
    struct MockWaylandObject { pub name: String }
    impl WaylandObject for MockWaylandObject {}

    fn create_mock_client(id: u64) -> (Client, UnixStream) {
        // Create a dummy socket pair for the stream
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
