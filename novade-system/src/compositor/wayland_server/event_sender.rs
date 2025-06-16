use crate::compositor::wayland_server::event_dispatcher::WaylandEvent;
use crate::compositor::wayland_server::message::{self, Argument}; // For serialize_message
use std::collections::VecDeque;

/// `EventSender` provides a way for Wayland object implementations to send events
/// or protocol errors back to a client. It does this by queuing a `WaylandEvent::SendToClient`
/// or a `wl_display.error` event onto the main server event queue.
#[derive(Debug)]
pub struct EventSender<'a> {
    // A mutable reference to the dispatcher's event queue's internal VecDeque.
    event_queue_handle: &'a mut VecDeque<WaylandEvent>,
}

impl<'a> EventSender<'a> {
    /// Creates a new `EventSender`.
    ///
    /// # Arguments
    /// * `queue_handle`: A mutable reference to the `VecDeque` of `WaylandEvent`s
    ///   from the `EventDispatcher`'s `EventQueue`.
    pub fn new(queue_handle: &'a mut VecDeque<WaylandEvent>) -> Self {
        EventSender { event_queue_handle }
    }

    /// Serializes a Wayland event (which is just a message from server to client)
    /// and queues it to be sent to the specified client.
    ///
    /// # Arguments
    /// * `target_client_id`: The ID of the client to send the event to.
    /// * `sender_object_id`: The object ID on the server that is emitting the event.
    /// * `event_opcode`: The opcode of the event specific to the sender_object_id's interface.
    /// * `args`: A vector of `Argument`s for the event.
    pub fn send_event(
        &mut self,
        target_client_id: u64,
        sender_object_id: u32,
        event_opcode: u16,
        args: Vec<Argument>,
    ) -> Result<(), String> {
        println!(
            "[EventSender] Queuing event: client_id={}, sender_obj={}, opcode={}",
            target_client_id, sender_object_id, event_opcode
        );

        match message::serialize_message(sender_object_id, event_opcode, &args) {
            Ok(message_bytes) => {
                self.event_queue_handle.push_back(WaylandEvent::SendToClient {
                    client_id: target_client_id,
                    message_bytes,
                });
                Ok(())
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to serialize event (obj: {}, opcode: {}): {}",
                    sender_object_id, event_opcode, e
                );
                eprintln!("[EventSender] {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Queues a `wl_display.error` event to be sent to the client.
    /// This is a specific type of event used for protocol errors.
    ///
    /// # Arguments
    /// * `target_client_id`: The ID of the client to send the error to.
    /// * `error_object_id`: The ID of the object where the error occurred.
    /// * `error_code`: A protocol-specific error code.
    /// * `error_text`: A human-readable error message.
    pub fn send_protocol_error(
        &mut self,
        target_client_id: u64,
        error_object_id: u32,
        error_code: u32,
        error_text: String,
    ) -> Result<(), String> {
        println!(
            "[EventSender] Queuing protocol error: client_id={}, object_id={}, code={}, text='{}'",
            target_client_id, error_object_id, error_code, error_text
        );

        // wl_display.error event has opcode 0.
        // Arguments are: object_id (the object where error occurred), code (u32), message (string).
        // The sender of wl_display.error is always wl_display (object ID 1).
        let args = vec![
            Argument::Object(error_object_id),
            Argument::Uint(error_code),
            Argument::String(error_text),
        ];

        // Opcode 0 for wl_display.error. Sender is wl_display (ID 1).
        self.send_event(target_client_id, 1, 0, args)
    }
}
