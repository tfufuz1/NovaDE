// In novade-system/src/compositor/wayland_server/events.rs

use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::protocol::{RawMessage, ObjectId};
use tokio::sync::mpsc::{self, Sender, Receiver, error::TryRecvError};
use tokio::sync::broadcast; // For events that might have multiple listeners
use tracing::{debug, error, warn};
use std::fmt;

// Enum defining the different types of events that can occur within the Wayland server module.
// These are high-level internal events, not Wayland protocol events directly,
// though some might wrap Wayland protocol messages.
#[derive(Clone)] // Clone is needed if events are sent to multiple places or re-queued.
pub enum WaylandEvent {
    NewClient {
        client_id: ClientId,
        // stream: tokio::net::UnixStream, // Stream is managed by ClientContext / client task
    },
    ClientDisconnected {
        client_id: ClientId,
        reason: String,
    },
    ClientMessage {
        client_id: ClientId,
        message: RawMessage, // The raw message received from the client
    },
    // Example of a more specific event that could be derived after initial parsing:
    // ProcessRequest {
    //     client_id: ClientId,
    //     object_id: ObjectId,
    //     opcode: u16,
    //     args: Vec<WaylandArgumentValue>, // Parsed arguments
    // },
    ServerError {
        description: String,
    },
    // Add more event types as needed, e.g., for specific protocol actions, timers, etc.
}

// Implement Debug manually for more control if RawMessage or other fields become complex
impl fmt::Debug for WaylandEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaylandEvent::NewClient { client_id } => f.debug_struct("NewClient").field("client_id", client_id).finish(),
            WaylandEvent::ClientDisconnected { client_id, reason } => f.debug_struct("ClientDisconnected")
                .field("client_id", client_id)
                .field("reason", reason)
                .finish(),
            WaylandEvent::ClientMessage { client_id, message } => f.debug_struct("ClientMessage")
                .field("client_id", client_id)
                // Avoid printing full message content if it's too verbose or contains sensitive data
                .field("object_id", &message.header.object_id)
                .field("opcode", &message.header.opcode)
                .field("size", &message.header.size)
                .finish(),
            WaylandEvent::ServerError { description } => f.debug_struct("ServerError").field("description", description).finish(),
        }
    }
}


// Central event queue for Wayland server events.
// Using tokio's mpsc channel for a multi-producer, single-consumer queue.
// This queue would typically be processed by a main event loop in the compositor.
#[derive(Debug)]
pub struct EventQueue {
    sender: Sender<WaylandEvent>,
    // Receiver is not stored here; it's taken by the event processing loop.
}

impl EventQueue {
    // Creates a new event queue with a specified buffer size for the channel.
    pub fn new(buffer_size: usize) -> (Self, Receiver<WaylandEvent>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (EventQueue { sender }, receiver)
    }

    // Sends an event into the queue.
    // This can be called from various parts of the system (e.g., socket listener, client handlers).
    pub async fn send(&self, event: WaylandEvent) {
        if let Err(e) = self.sender.send(event.clone()).await { // Clone event before sending
            error!("Failed to send Wayland event: {}. Receiver likely dropped.", e);
            // Depending on the event, might attempt a retry or handle error.
        } else {
            debug!("Successfully sent Wayland event: {:?}", event);
        }
    }

    // Tries to send an event without blocking. Useful for contexts where .await is not desired/possible.
    // Note: this can fail if the channel buffer is full.
    pub fn try_send(&self, event: WaylandEvent) -> Result<(), mpsc::error::TrySendError<WaylandEvent>> {
        self.sender.try_send(event.clone()).map_err(|e| { // Clone event
            warn!("Failed to try_send Wayland event (channel full or closed): {:?}", e.event());
            e
        })
    }
}

impl Clone for EventQueue {
    fn clone(&self) -> Self {
        EventQueue {
            sender: self.sender.clone(),
        }
    }
}


// For events that need to be broadcast to multiple listeners (e.g., global state changes,
// notifications that multiple parts of the compositor might care about).
// This is distinct from the main MPSC queue which is usually for a central processing loop.
#[derive(Debug)]
pub struct EventBroadcaster<T: Clone + Send + fmt::Debug + 'static> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone + Send + fmt::Debug + 'static> EventBroadcaster<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(capacity); // receiver is not stored here
        EventBroadcaster { sender }
    }

    pub fn send(&self, event: T) -> Result<usize, broadcast::error::SendError<T>> {
        match self.sender.send(event.clone()) { // Clone event before sending
            Ok(receivers) => {
                debug!("Successfully broadcast event {:?} to {} active receivers.", event, receivers);
                Ok(receivers)
            }
            Err(e) => {
                error!("Failed to broadcast event {:?}: {}", event, e);
                Err(e)
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

impl<T: Clone + Send + fmt::Debug + 'static> Clone for EventBroadcaster<T> {
    fn clone(&self) -> Self {
        EventBroadcaster {
            sender: self.sender.clone(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::protocol::{MessageHeader, ObjectId};
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_event_queue_send_and_receive() {
        let (event_queue, mut receiver) = EventQueue::new(10);
        let client_id = ClientId::new(); // Assuming ClientId::new() is available and works

        let event = WaylandEvent::NewClient { client_id };
        event_queue.send(event.clone()).await;

        match timeout(Duration::from_millis(100), receiver.recv()).await {
            Ok(Some(received_event)) => {
                match (received_event, event) { // Compare content, not just type
                    (WaylandEvent::NewClient{ client_id: cid_recv }, WaylandEvent::NewClient{ client_id: cid_sent }) => {
                        assert_eq!(cid_recv, cid_sent);
                    }
                    _ => panic!("Received event does not match sent event or has wrong type"),
                }
            }
            Ok(None) => panic!("Event queue receiver closed prematurely."),
            Err(_) => panic!("Timed out waiting for event."),
        }
    }

    #[tokio::test]
    async fn test_event_queue_try_send_full() {
        let (event_queue, mut receiver) = EventQueue::new(1); // Buffer size 1
        let client_id = ClientId::new();
        let event1 = WaylandEvent::NewClient { client_id };
        let event2 = WaylandEvent::ClientDisconnected { client_id, reason: "test".to_string() };
        let event3 = WaylandEvent::ServerError { description: "full".to_string() };

        assert!(event_queue.try_send(event1.clone()).is_ok(), "First try_send should succeed");
        assert!(event_queue.try_send(event2.clone()).is_err(), "Second try_send should fail (queue full)");

        // Receive the first event
        assert!(receiver.recv().await.is_some());

        // Now try_send should succeed again
        assert!(event_queue.try_send(event3.clone()).is_ok(), "try_send should succeed after making space");
        assert!(receiver.recv().await.is_some());
    }

    #[tokio::test]
    async fn test_event_broadcaster_send_and_subscribe() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct TestBroadcastEvent(String);

        let broadcaster = EventBroadcaster::<TestBroadcastEvent>::new(5);
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        let event_data = TestBroadcastEvent("hello broadcast".to_string());
        let send_result = broadcaster.send(event_data.clone());
        assert!(send_result.is_ok());
        assert_eq!(send_result.unwrap(), 2, "Should have 2 active subscribers");

        let received1 = timeout(Duration::from_millis(50), rx1.recv()).await.unwrap().unwrap();
        let received2 = timeout(Duration::from_millis(50), rx2.recv()).await.unwrap().unwrap();

        assert_eq!(received1, event_data);
        assert_eq!(received2, event_data);

        // Test slow receiver
        let mut rx3 = broadcaster.subscribe();
        broadcaster.send(TestBroadcastEvent("msg1".to_string())).unwrap();
        broadcaster.send(TestBroadcastEvent("msg2".to_string())).unwrap();
        // rx3 hasn't received yet. If channel capacity is small, it might miss events.
        // Default capacity for broadcast::channel is 32 if not specified at creation,
        // but we set it to 5.

        // Drop one receiver
        drop(rx1);
        let send_result_after_drop = broadcaster.send(TestBroadcastEvent("msg3".to_string()));
        assert!(send_result_after_drop.is_ok());
        // Now should be 2 receivers (rx2, rx3) that got msg3.
        // The count returned by send() is the number of *active* receivers *at the time of send*.
        // If rx3 was slow and lagged, it might miss some, but send() still reports based on current subscriptions.
        // For this test, we expect rx2 and rx3 to get msg3.
        assert_eq!(send_result_after_drop.unwrap(), 2);

        assert_eq!(timeout(Duration::from_millis(50), rx2.recv()).await.unwrap().unwrap().0, "msg3");
        assert_eq!(timeout(Duration::from_millis(50), rx3.recv()).await.unwrap().unwrap().0, "msg1"); // rx3 gets first msg it missed
        assert_eq!(timeout(Duration::from_millis(50), rx3.recv()).await.unwrap().unwrap().0, "msg2");
        assert_eq!(timeout(Duration::from_millis(50), rx3.recv()).await.unwrap().unwrap().0, "msg3");

    }

    #[test]
    fn test_wayland_event_debug_format() {
        let client_id = ClientId::new(); // Create a real ClientId
        let new_client_event = WaylandEvent::NewClient { client_id };
        // Format will be like "NewClient { client_id: client-XYZ }"
        let debug_new_client = format!("{:?}", new_client_event);
        assert!(debug_new_client.starts_with("NewClient"));
        assert!(debug_new_client.contains(&format!("{:?}", client_id))); // Compare with ClientId's Debug output


        let header = MessageHeader { object_id: ObjectId(1), size: 8, opcode: 0 };
        let raw_message = RawMessage { header, content: bytes::Bytes::new() };
        let client_message_event = WaylandEvent::ClientMessage { client_id, message: raw_message };
        // Format will be like "ClientMessage { client_id: client-XYZ, object_id: ObjectId(1), opcode: 0, size: 8 }"
        let debug_client_msg = format!("{:?}", client_message_event);
        assert!(debug_client_msg.starts_with("ClientMessage"));
        assert!(debug_client_msg.contains(&format!("{:?}", client_id))); // Compare with ClientId's Debug output
        assert!(debug_client_msg.contains("object_id: ObjectId(1)"));
        assert!(debug_client_msg.contains("opcode: 0"));
        assert!(debug_client_msg.contains("size: 8"));
    }
}
