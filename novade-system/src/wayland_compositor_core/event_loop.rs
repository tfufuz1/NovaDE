use super::client::ClientId; // Changed
use super::wire::{MessageHeader, WlArgument}; // Changed
use mio::{Events, Interest, Poll, Token};
use std::collections::{HashMap, VecDeque};
use std::os::unix::io::RawFd;
use std::sync::{Arc, Mutex}; // For context items
use std::time::Duration;

// --- Event Types ---
#[derive(Debug, Clone)] // Clone might be too restrictive if stream/args are large. Consider Arc for some fields.
pub enum Event {
    // ClientConnected { client_id: ClientId, stream: Arc<UnixStream> }, // UnixStream is not Clone, wrap in Arc or pass RawFd
    ClientConnected { client_id: ClientId, fd: RawFd }, // Server accepts, gets fd, then informs event loop
    ClientDisconnected { client_id: ClientId },
    MessageReceived {
        client_id: ClientId,
        header: MessageHeader,
        args: Vec<WlArgument>, // This could be large, consider Arc<Vec<WlArgument>> or similar
    },
    FdReadable { client_id: ClientId, fd: RawFd }, // Specific to a client's stream
    FdHangup { client_id: ClientId, fd: RawFd },
    SignalReceived(i32), // Example: Unix signal
    InternalError(String), // For event loop's own errors
}

// For handler registration, we might want to distinguish events by type
// This can be done using std::mem::discriminant or a custom enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EventTypeDiscriminant {
    ClientConnected,
    ClientDisconnected,
    MessageReceived,
    FdReadable,
    FdHangup,
    SignalReceived,
    InternalError,
    // Add more as Event enum grows
}

impl From<&Event> for EventTypeDiscriminant {
    fn from(event: &Event) -> Self {
        match event {
            Event::ClientConnected { .. } => EventTypeDiscriminant::ClientConnected,
            Event::ClientDisconnected { .. } => EventTypeDiscriminant::ClientDisconnected,
            Event::MessageReceived { .. } => EventTypeDiscriminant::MessageReceived,
            Event::FdReadable { .. } => EventTypeDiscriminant::FdReadable,
            Event::FdHangup { .. } => EventTypeDiscriminant::FdHangup,
            Event::SignalReceived { .. } => EventTypeDiscriminant::SignalReceived,
            Event::InternalError { .. } => EventTypeDiscriminant::InternalError,
        }
    }
}


// --- Event Loop Context ---
// This context provides handlers with access to shared compositor resources.
// For now, it's a placeholder. In a real scenario, it would hold Arcs/Mutexes
// to ClientManager, ObjectManager, etc.
#[derive(Clone)] // If callbacks need to own a piece of it or if it's passed by value often.
pub struct EventLoopContext {
    // pub client_manager: Arc<Mutex<ClientManager>>,
    // pub object_manager: Arc<Mutex<ObjectManager>>,
    // For this subtask, keep it simple.
    _placeholder: (), // To avoid empty struct warnings if no fields yet
}

impl EventLoopContext {
    pub fn new(/* ... shared resources ... */) -> Self {
        EventLoopContext { _placeholder: () }
    }
}

// --- Callback Mechanism ---
#[derive(Debug)] // For EventLoopError
pub enum EventLoopError {
    HandlerFailed(String), // Error propagated from a handler
    IoError(std::io::Error),
    RegistrationFailed(String),
    // Add more specific errors
}

impl From<std::io::Error> for EventLoopError {
    fn from(err: std::io::Error) -> Self {
        EventLoopError::IoError(err)
    }
}

// Callbacks are FnMut to allow them to modify their captured state (if any)
// and the EventLoopContext.
pub type Callback = Box<dyn FnMut(Event, &mut EventLoopContext) -> Result<(), EventLoopError> + Send + Sync>;


// --- Event Loop ---
pub struct EventLoop {
    poll: Poll,
    events: Events, // Storage for events from mio::Poll
    pending_events: VecDeque<Event>, // For events not directly from mio (e.g., internal, signals)
    handlers: HashMap<EventTypeDiscriminant, Vec<Callback>>,
    next_token: usize, // For generating unique mio Tokens
    // We need a way to map Token back to Fd or client_id if events come from mio::Poll
    // For now, we are not fully integrating mio's event dispatching.
}

impl EventLoop {
    pub fn new(event_capacity: usize) -> Result<Self, EventLoopError> {
        Ok(EventLoop {
            poll: Poll::new()?,
            events: Events::with_capacity(event_capacity),
            pending_events: VecDeque::new(),
            handlers: HashMap::new(),
            next_token: 0,
        })
    }

    fn generate_token(&mut self) -> Token {
        let token = Token(self.next_token);
        self.next_token += 1;
        token
    }

    /// Registers an FD with mio for monitoring.
    /// This is a placeholder for full mio integration.
    pub fn register_fd_source(&mut self, fd: RawFd, interest: Interest) -> Result<Token, EventLoopError> {
        let token = self.generate_token();
        // mio::Source trait is needed for registry. For RawFd, use SourceFd.
        // On Unix, RawFd is AsRawFd. Mio needs &Source.
        // For UnixStream, it implements Evented.
        // For now, this is highly conceptual as we are not using a real stream here.
        // self.poll.registry().register(&mut SourceFd(&fd), token, interest)?;
        // This line above would require SourceFd to be mutable if it stores state for registration.
        // Or, if it's just a wrapper, it might be fine.
        // The actual registration depends on the type that owns fd.
        // Let's assume for now this is handled externally or by a wrapper type.
        println!("Placeholder: Registering fd {} with token {:?} for interest {:?}", fd, token, interest);
        // Actual registration would be:
        // let mut source = mio::unix::SourceFd(&fd); // fd must be kept open
        // self.poll.registry().register(&mut source, token, interest)?;
        Ok(token)
    }


    pub fn register_handler(
        &mut self,
        event_type: EventTypeDiscriminant,
        callback: Callback,
    ) {
        self.handlers.entry(event_type).or_default().push(callback);
    }

    pub fn push_event(&mut self, event: Event) {
        self.pending_events.push_back(event);
    }

    /// Processes internally queued events.
    pub fn dispatch_pending_manual_events(&mut self, context: &mut EventLoopContext) -> Vec<Result<(), EventLoopError>> {
        let mut results = Vec::new();
        for _ in 0..self.pending_events.len() { // Process all events currently in queue
            if let Some(event) = self.pending_events.pop_front() {
                let event_type_disc = EventTypeDiscriminant::from(&event);
                if let Some(handler_callbacks) = self.handlers.get_mut(&event_type_disc) {
                    for cb in handler_callbacks.iter_mut() {
                        // Clone event if multiple handlers might need ownership,
                        // or if handlers don't consume it.
                        // For FnMut, they get a mutable reference.
                        // Current signature takes ownership of Event.
                        // If multiple handlers, event must be cloned.
                        results.push(cb(event.clone(), context));
                    }
                } else {
                    // No handler registered for this event type
                    println!("No handler for event type: {:?}", event_type_disc);
                }
            }
        }
        results
    }

    /// Placeholder for a single poll iteration using mio.
    /// This does not yet correctly map mio::Events back to application Events.
    pub fn poll_and_dispatch(&mut self, context: &mut EventLoopContext, timeout: Option<Duration>) -> Result<(), EventLoopError> {
        self.poll.poll(&mut self.events, timeout)?;

        for mio_event in self.events.iter() {
            // Here, we would need to map mio_event.token() back to an fd or client_id,
            // determine the actual event (e.g., FdReadable, FdHangup),
            // and then construct and push our application Event.
            println!("Mio event received: {:?}", mio_event);

            // Example conceptual mapping:
            // let app_event = map_mio_event_to_app_event(mio_event);
            // self.push_event(app_event);
        }

        // After polling and potentially queueing new app events from mio sources,
        // dispatch any pending manual/internal events (including those just added from mio).
        self.dispatch_pending_manual_events(context); // Ignoring results for now in this combined func

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_event_loop_new() {
        let event_loop = EventLoop::new(1024);
        assert!(event_loop.is_ok());
    }

    #[test]
    fn test_push_and_dispatch_event() {
        let mut event_loop = EventLoop::new(1024).unwrap();
        let mut context = EventLoopContext::new();

        let test_event = Event::ClientDisconnected { client_id: 1 };
        let event_type_disc = EventTypeDiscriminant::from(&test_event);

        let handler_called = Arc::new(AtomicBool::new(false));
        let handler_called_clone = handler_called.clone();

        let callback: Callback = Box::new(move |event, _ctx| {
            if let Event::ClientDisconnected { client_id } = event {
                if client_id == 1 {
                    handler_called_clone.store(true, Ordering::SeqCst);
                }
            }
            Ok(())
        });

        event_loop.register_handler(event_type_disc, callback);
        event_loop.push_event(test_event);

        let results = event_loop.dispatch_pending_manual_events(&mut context);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        assert!(handler_called.load(Ordering::SeqCst), "Handler was not called or event not matched");
    }

    #[test]
    fn test_multiple_handlers_for_event_type() {
        let mut event_loop = EventLoop::new(1024).unwrap();
        let mut context = EventLoopContext::new();
        let test_event = Event::SignalReceived(15); // SIGTERM
        let event_type_disc = EventTypeDiscriminant::from(&test_event);

        let handler1_called = Arc::new(AtomicBool::new(false));
        let handler1_clone = handler1_called.clone();
        let cb1: Callback = Box::new(move |event, _ctx| {
            if let Event::SignalReceived(s) = event { if s == 15 { handler1_clone.store(true, Ordering::SeqCst); }}
            Ok(())
        });

        let handler2_called = Arc::new(AtomicBool::new(false));
        let handler2_clone = handler2_called.clone();
        let cb2: Callback = Box::new(move |event, _ctx| {
            if let Event::SignalReceived(s) = event { if s == 15 { handler2_clone.store(true, Ordering::SeqCst); }}
            Ok(())
        });

        event_loop.register_handler(event_type_disc, cb1);
        event_loop.register_handler(event_type_disc, cb2);
        event_loop.push_event(test_event);

        let results = event_loop.dispatch_pending_manual_events(&mut context);
        assert_eq!(results.len(), 2); // Two handlers should have been called
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(handler1_called.load(Ordering::SeqCst), "Handler 1 was not called");
        assert!(handler2_called.load(Ordering::SeqCst), "Handler 2 was not called");
    }

    #[test]
    fn test_dispatch_no_handler() {
        let mut event_loop = EventLoop::new(1024).unwrap();
        let mut context = EventLoopContext::new();
        let test_event = Event::InternalError("test error".to_string());
        // No handler registered for InternalError

        event_loop.push_event(test_event);
        let results = event_loop.dispatch_pending_manual_events(&mut context);
        assert!(results.is_empty(), "No results should be returned if no handler is registered");
        // Check logs/stdout for "No handler for event type" if that's the behavior
    }

    #[test]
    fn test_handler_returns_error() {
        let mut event_loop = EventLoop::new(1024).unwrap();
        let mut context = EventLoopContext::new();
        let test_event = Event::ClientDisconnected{ client_id: 2 };
        let event_type_disc = EventTypeDiscriminant::from(&test_event);

        let cb: Callback = Box::new(|_event, _ctx| {
            Err(EventLoopError::HandlerFailed("Simulated error".to_string()))
        });

        event_loop.register_handler(event_type_disc, cb);
        event_loop.push_event(test_event);

        let results = event_loop.dispatch_pending_manual_events(&mut context);
        assert_eq!(results.len(), 1);
        match &results[0] {
            Err(EventLoopError::HandlerFailed(msg)) => assert_eq!(msg, "Simulated error"),
            _ => panic!("Expected HandlerFailed error"),
        }
    }

    // Placeholder for mio registration test - this is very conceptual without real FDs.
    #[test]
    fn test_register_fd_source_placeholder() {
        let mut event_loop = EventLoop::new(1024).unwrap();
        // This test doesn't actually use mio::Poll::registry()->register() because
        // it requires a valid source (like a real, open FD).
        // It just tests that our placeholder function can be called and returns a token.
        let dummy_fd: RawFd = 0; // Use a dummy FD. Note: 0, 1, 2 are stdin, stdout, stderr.
        let result = event_loop.register_fd_source(dummy_fd, Interest::READABLE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Token(0));

        let result2 = event_loop.register_fd_source(dummy_fd, Interest::WRITABLE);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Token(1)); // Token should increment
    }
}
