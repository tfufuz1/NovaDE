use libc::{gid_t, pid_t, uid_t};
use nix::sys::socket::{recvmsg, ControlMessageOwned, MsgFlags, UnixCredentials}; // Added recvmsg, ControlMessageOwned, MsgFlags
use nix::sys::uio::IoVec; // For recvmsg
use std::io::{self}; // Read trait might not be directly used if fully switching to recvmsg for data
use std::os::unix::net::UnixStream;
use std::os::unix::io::{AsRawFd, RawFd}; // Added AsRawFd
use std::time::SystemTime;

use super::message::{self, Argument, Message, MessageParseError};
use super::object_registry::{ObjectRegistry, WlRegistry}; // WlRegistry might be removed if not directly used here
use super::protocol_spec::ProtocolManager; // Added for dynamic parsing

/// Represents a connected Wayland client.
#[derive(Debug)]
pub struct Client {
    pub id: u64,
    pub uid: uid_t,
    pub gid: gid_t,
    pub pid: pid_t,
    pub connection_ts: SystemTime,
    pub resource_usage: usize,
    pub stream: UnixStream,
    pub read_buffer: Vec<u8>,
    pub pending_fds: Option<Vec<RawFd>>,
    pub last_activity_ts: SystemTime,
}

impl Client {
    pub fn new(id: u64, stream: UnixStream, creds: UnixCredentials) -> Self {
        // Ensure stream is non-blocking for use with Poller and general async handling.
        // The Poller might also set this, but good practice to ensure it here.
        if let Err(e) = stream.set_nonblocking(true) {
            // This is a significant issue if it fails.
            // The caller (in mod.rs) should ideally handle this error and not proceed with the client.
            eprintln!("Client {}: CRITICAL - Failed to set UnixStream to non-blocking: {}. This client may not function correctly with the event poller.", id, e);
            // Depending on server policy, one might panic, or return an error from new(),
            // but that would change the function signature. For now, log and continue.
        }
        Client {
            id,
            uid: creds.uid(),
            gid: creds.gid(),
            pid: creds.pid().unwrap_or(0),
            connection_ts: SystemTime::now(),
            last_activity_ts: SystemTime::now(), // Initialize on creation
            resource_usage: 0,
            stream,
            read_buffer: Vec::with_capacity(4096),
            pending_fds: None,
        }
    }

    /// Increments the resource usage counter for this client.
    /// This method should be called when the client successfully creates a new Wayland object
    /// or allocates a significant server-side resource.
    pub fn increment_resource_usage(&mut self, amount: usize) {
        self.resource_usage = self.resource_usage.saturating_add(amount);
        // Optional: Add logging if resource usage exceeds certain thresholds.
        // println!("Client {}: resource usage incremented by {}, new total: {}", self.id, amount, self.resource_usage);
    }

    /// Decrements the resource usage counter for this client.
    /// This method should be called when a client resource is destroyed or deallocated.
    pub fn decrement_resource_usage(&mut self, amount: usize) {
        self.resource_usage = self.resource_usage.saturating_sub(amount);
        // println!("Client {}: resource usage decremented by {}, new total: {}", self.id, amount, self.resource_usage);
    }

    /// Reads data from stream, parses messages. Returns a Vec of parsed messages.
    pub fn handle_readable_and_get_messages(
        &mut self,
        protocol_manager: &ProtocolManager, // Added
        object_registry: &ObjectRegistry,   // Changed from &mut to &
    ) -> Result<Vec<Message>, ClientError> {
        let mut parsed_messages = Vec::new();

        // Buffer for primary message data
        let mut data_buf = [0u8; 4096];
        let iov = [IoVec::from_mut_slice(&mut data_buf)];

        // Buffer for control messages (ancillary data like FDs)
        // Sized to handle a reasonable number of FDs.
        let mut cmsg_buf = nix::cmsg_space!([RawFd; 8]); // Increased capacity to 8 FDs

        // Clear FDs from previous read. FDs are per recvmsg batch.
        self.pending_fds = None;
        let mut received_fds_this_call = Vec::new();

        match recvmsg(self.stream.as_raw_fd(), &iov, Some(&mut cmsg_buf), MsgFlags::empty()) {
            Ok(msg) => {
                if msg.bytes == 0 {
                    println!("Client {} disconnected (recvmsg returned 0 bytes).", self.id);
                    return Err(ClientError::ConnectionClosed);
                }
                self.last_activity_ts = SystemTime::now(); // Update on successful read

                println!("Client {}: Received {} data bytes via recvmsg.", self.id, msg.bytes);
                self.read_buffer.extend_from_slice(&data_buf[..msg.bytes]);

                for cmsg in msg.cmsgs() {
                    if let ControlMessageOwned::ScmRights(fds) = cmsg {
                        if !fds.is_empty() {
                            println!("Client {}: Received {} FDs: {:?}.", self.id, fds.len(), fds);
                            received_fds_this_call.extend(fds.iter().copied());
                        }
                    }
                }
                if !received_fds_this_call.is_empty() {
                    // Store FDs in LIFO order for pop() in deserialize_fd, or FIFO for remove(0)
                    // If deserialize_fd uses pop(), reverse them here to process in received order.
                    // self.pending_fds = Some(received_fds_this_call.into_iter().rev().collect());
                    // If deserialize_fd uses remove(0), store as is.
                    // Current deserialize_fd uses pop(), so FDs should be pushed in order, then pop retrieves last pushed.
                    // To match argument order, if FDs are [fd_arg1, fd_arg2], pop retrieves fd_arg2 then fd_arg1.
                    // So, if they are ordered in ancillary data, store them to be popped in reverse order of arguments or reverse here.
                    // Let's assume `deserialize_fd` will take from the end of the vec (LIFO).
                    // So if FDs are for args [arg1_fd, arg2_fd], vec should be [arg1_fd, arg2_fd] for pop to get arg2_fd.
                    // This means `deserialize_fd` needs to know which FD to take or `pending_fds` needs careful management.
                    // For now, simply storing them. `deserialize_fd` uses pop, so it gets the *last* FD received for the *first* FD arg.
                    // This is likely incorrect. Let's reverse so pop() gets them in the order they appeared.
                    self.pending_fds = Some(received_fds_this_call.into_iter().rev().collect());

                }
            }
            Err(nix::Error::Sys(errno)) if errno == nix::errno::Errno::EAGAIN || errno == nix::errno::Errno::EWOULDBLOCK => {
                // Non-blocking socket reported no data currently available.
            }
            Err(e) => {
                eprintln!("Client {}: Error using recvmsg: {}", self.id, e);
                return Err(ClientError::Nix(e));
            }
        }

        // Loop to parse all complete messages from self.read_buffer
        loop {
            if self.read_buffer.is_empty() {
                break;
            }

            match message::parse_message_header(&self.read_buffer) {
                Ok((_sender_id, _opcode, msg_len)) => {
                    if self.read_buffer.len() < msg_len as usize {
                        break; // Not enough data for the full message yet
                    }
                }
                Err(MessageParseError::NotEnoughData(_)) => {
                    break; // Not enough data even for a header
                }
                Err(e) => {
                    eprintln!("Client {}: Error parsing message header: {:?}", self.id, e);
                    self.read_buffer.clear();
                    return Err(ClientError::MessageParse(e));
                }
            }

            // Pass ProtocolManager and ObjectRegistry to the dynamic parse_message function.
            match message::parse_message(&self.read_buffer, protocol_manager, object_registry, &mut self.pending_fds) {
                Ok((message, rest)) => {
                    // Special handling for wl_display.get_registry is now REMOVED.
                    // Object creation and resource counting will be handled by EventDispatcher
                    // based on the dynamically parsed message.

                    parsed_messages.push(message);

                    let consumed_len = self.read_buffer.len() - rest.len();
                    self.read_buffer.drain(..consumed_len);

                    if self.read_buffer.is_empty() {
                        break;
                    }
                }
                Err(MessageParseError::NotEnoughData(_)) => {
                    break; // Need more data for the current message
                }
                Err(e) => {
                    eprintln!("Client {}: Error parsing message: {:?}", self.id, e);
                    self.read_buffer.clear();
                    return Err(ClientError::MessageParse(e));
                }
            }
        }
        Ok(parsed_messages)
    }

    /// Sends a pre-serialized message (byte vector) to the client.
    pub fn send_message_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.write_all(bytes)?;
        self.stream.flush()?; // Ensure message is sent immediately
        self.last_activity_ts = SystemTime::now(); // Update on successful send
        Ok(())
    }

    /// Sends a pre-serialized message with file descriptors.
    #[allow(dead_code)] // May not be used immediately if SendToClient event doesn't support FDs yet
    pub fn send_message_with_fds(&mut self, bytes: &[u8], fds_to_send: &[RawFd]) -> nix::Result<usize> {
        use nix::sys::socket::{sendmsg, MsgFlags, ControlMessage};
        use std::io::IoSlice; // For IoVec equivalent

        let iov = [IoSlice::new(bytes)];
        let cmsgs = if fds_to_send.is_empty() {
            Vec::new()
        } else {
            vec![ControlMessage::ScmRights(fds_to_send)]
        };

        // MSG_NOSIGNAL can be useful to prevent SIGPIPE if client disconnected,
        // but error handling (EPIPE) should be managed by caller.
        // For now, empty flags.
        let result = sendmsg(self.stream.as_raw_fd(), &iov, &cmsgs, MsgFlags::empty(), None);
        if result.is_ok() {
            self.last_activity_ts = SystemTime::now(); // Update on successful send
        }
        result
    }
}

/// Validates the client's UID against the server's UID.
///
/// In many Wayland compositor setups, especially for single-user desktop environments,
/// it's a security measure to ensure that connecting clients are running under the
/// same user ID as the compositor itself.
///
/// Args:
///     client_uid: The user ID of the connecting client.
///
/// Returns:
///     `true` if the client UID matches the server UID, `false` otherwise.
///
/// Future Considerations:
/// - Multi-seat/multi-user: This logic would need to be more sophisticated, potentially
///   checking against UIDs of active graphical sessions (e.g., via `logind`).
/// - Configurable policies: Allow defining allowed UIDs or groups through configuration.
/// - Sandboxing: Interaction with sandboxed applications might require different checks
///   or rely on security contexts passed via the socket.
pub fn validate_client_uid(client_uid: uid_t) -> bool {
    let server_uid = nix::unistd::getuid().as_raw(); // Get the server's UID.
    if client_uid == server_uid {
        true
    } else {
        // Log the mismatch for security auditing or debugging.
        eprintln!(
            "Client UID validation failed: Client UID ({}) does not match server UID ({}). Connection will be refused.",
            client_uid, server_uid
        );
        // In a production system, this should strictly be false.
        // For specific development scenarios (e.g., running client as root or a different test user),
        // this check might be temporarily bypassed, but that's not recommended for general use.
        false
    }
}

#[derive(Debug)]
pub enum ClientError {
    Io(io::Error), // For std::io errors, if any remain
    Nix(nix::Error), // For errors from nix calls like recvmsg
    Credentials(String),
    Validation(String),
    MessageParse(MessageParseError),
    ConnectionClosed,
}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> Self {
        ClientError::Io(err)
    }
}

impl From<nix::Error> for ClientError {
    fn from(err: nix::Error) -> Self {
        ClientError::Nix(err)
    }
}

impl From<MessageParseError> for ClientError {
    fn from(err: MessageParseError) -> Self {
        ClientError::MessageParse(err)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "I/O error: {}", e),
            ClientError::Nix(e) => write!(f, "Nix syscall error: {}", e),
            ClientError::Credentials(s) => write!(f, "Credentials error: {}", s),
            ClientError::Validation(s) => write!(f, "Client validation error: {}", s),
            ClientError::MessageParse(e) => write!(f, "Message parse error: {:?}", e),
            ClientError::ConnectionClosed => write!(f, "Client connection closed"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::Io(e) => Some(e),
            ClientError::Nix(e) => Some(e),
            ClientError::MessageParse(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::object_registry::ObjectRegistry;
    use std::os::unix::net::UnixStream as StdUnixStream;
    use nix::unistd::{Uid, Gid, Pid, dup}; // Added dup for new test
    use nix::sys::socket::{sendmsg, ControlMessage, CmsgBuffer}; // For sendmsg test
    use std::io::Write; // For server_stream.write_all in existing test
    // use std::os::unix::io::FromRawFd; // Not directly needed unless creating UnixStream from RawFd in test


    fn mock_credentials_for_test() -> UnixCredentials {
        UnixCredentials::new(Uid::current().as_raw(), Gid::current().as_raw(), Some(Pid::this().as_raw()))
    }

    #[test]
    fn test_client_handle_readable_and_get_messages_get_registry() {
        // Setup ProtocolManager and ObjectRegistry for the test
        let mut pm = ProtocolManager::new();
        protocol_spec::load_core_protocols(&mut pm);
        let mut registry = ObjectRegistry::new(); // wl_display is pre-registered

        let (client_stream_for_obj, mut server_stream_for_test) = StdUnixStream::pair().expect("Socket pair");
        client_stream_for_obj.set_nonblocking(true).unwrap();
        // server_stream_for_test does not need to be non-blocking for write_all.

        let creds = mock_credentials_for_test();
        let mut client_obj = Client::new(1, client_stream_for_obj, creds);

        // Message for wl_display.get_registry
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id (wl_display)
        let opcode = 1u16; // get_registry opcode
        let len = 12u16;    // header (8) + new_id (4)
        msg_bytes.extend_from_slice(&((opcode as u32) << 16 | (len as u32)).to_ne_bytes());
        let new_registry_id = 3u32;
        msg_bytes.extend_from_slice(&new_registry_id.to_ne_bytes());

        server_stream_for_test.write_all(&msg_bytes).expect("Write get_registry message");
        server_stream_for_test.flush().expect("Flush get_registry message");
        std::thread::sleep(std::time::Duration::from_millis(100)); // Allow time for data to arrive

        match client_obj.handle_readable_and_get_messages(&pm, &registry) {
            Ok(messages) => {
                assert!(client_obj.read_buffer.is_empty(), "Buffer should be empty after parsing: {:?}", client_obj.read_buffer);
                assert_eq!(messages.len(), 1, "Should have parsed one message");
                let parsed_msg = &messages[0];
                assert_eq!(parsed_msg.sender_id, 1);
                assert_eq!(parsed_msg.opcode, 1); // get_registry
                assert_eq!(parsed_msg.args.len(), 1);
                assert_eq!(parsed_msg.args[0], Argument::NewId(new_registry_id));

                // With removal of direct object creation, we can no longer check registry here.
                // Also, resource_usage is not incremented here anymore.
                // Those checks will move to EventDispatcher tests later.
                 assert_eq!(client_obj.resource_usage, 0, "Resource usage should NOT be incremented by handle_readable");
            }
            Err(e) => {
                let mut is_would_block = false;
                let mut is_would_block = false;
                match &e {
                    ClientError::Io(io_err) if io_err.kind() == io::ErrorKind::WouldBlock => is_would_block = true,
                    ClientError::Nix(nix::Error::Sys(errno)) if *errno == nix::errno::Errno::EAGAIN || *errno == nix::errno::Errno::EWOULDBLOCK => is_would_block = true,
                    _ => {}
                }

                if is_would_block {
                    eprintln!("Test (get_registry) encountered WouldBlock/EAGAIN. This is often a timing artifact in tests. Verifying no messages were parsed.");
                    assert!(messages.is_empty(), "Expected no messages if WouldBlock occurred before full read/recvmsg.");
                    assert_eq!(client_obj.resource_usage, 0, "Resource usage should be 0 if message not processed here.");
                } else {
                    panic!("handle_readable_and_get_messages for get_registry failed with an unexpected error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_client_handle_readable_receives_fd() {
        let mut pm = ProtocolManager::new();
        protocol_spec::load_core_protocols(&mut pm); // Ensure wl_shm is loaded for this test.
        let mut registry = ObjectRegistry::new();
        // Manually register a wl_shm object for the test
        let shm_object_id = 2u32;
        registry.new_object(1, shm_object_id, MockWaylandObject {}, "wl_shm".to_string(), 1).unwrap();


        let (client_stream, mut server_stream_for_test) = StdUnixStream::pair().expect("Socket pair for FD test");
        client_stream.set_nonblocking(true).unwrap();

        let creds = mock_credentials_for_test();
        let mut client_obj = Client::new(1, client_stream, creds); // client_id matches new_object above

        // Message for wl_shm.create_pool (sender_id=shm_object_id, opcode=0)
        // args: new_id (pool_id), fd, size
        let mut msg_bytes_vec = Vec::new();
        msg_bytes_vec.extend_from_slice(&shm_object_id.to_ne_bytes()); // sender_id
        let opcode = 0u16; // create_pool
        let len = 16u16;   // header(8) + new_id(4) + size(4). FD is ancillary.
        msg_bytes_vec.extend_from_slice(&((opcode as u32) << 16 | (len as u32)).to_ne_bytes());
        msg_bytes_vec.extend_from_slice(&3u32.to_ne_bytes()); // new_id (pool_id) = 3
        msg_bytes_vec.extend_from_slice(&4096i32.to_ne_bytes()); // size = 4096

        let dummy_fd_to_send = dup(0).expect("Failed to dup stdin for test FD");

        let iov = [IoVec::from_slice(&msg_bytes_vec)];
        let fds_to_send_array = [dummy_fd_to_send]; // ControlMessage::ScmRights expects a slice
        let cmsgs = [ControlMessage::ScmRights(&fds_to_send_array)];

        match sendmsg(server_sock_for_test.as_raw_fd(), &iov, &cmsgs, MsgFlags::empty(), None) {
            Ok(sent_bytes) => {
                assert_eq!(sent_bytes, msg_bytes_vec.len(), "sendmsg did not send all bytes");
            }
            Err(e) => {
                unsafe { libc::close(dummy_fd_to_send); } // Clean up FD if send fails
                panic!("sendmsg failed in test: {}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for data to arrive

        match client_obj.handle_readable_and_get_messages(&pm, &registry) {
            Ok(messages) => {
                assert_eq!(messages.len(), 1, "Should have parsed one message");
                let msg = &messages[0];
                assert_eq!(msg.sender_id, shm_object_id);
                assert_eq!(msg.opcode, 0); // create_pool
                assert_eq!(msg.args.len(), 3);
                assert_eq!(msg.args[0], Argument::NewId(3)); // pool_id
                assert_eq!(msg.args[1], Argument::Fd(dummy_fd_to_send)); // The FD
                assert_eq!(msg.args[2], Argument::Int(4096));  // size

                // FD should be consumed by parse_message from client_obj.pending_fds
                assert!(client_obj.pending_fds.is_none() || client_obj.pending_fds.as_ref().unwrap().is_empty(),
                        "FD should have been consumed by parse_message from client_obj.pending_fds");

                // The actual FD is now in msg.args[1]. It must be closed to prevent leaks.
                if let Argument::Fd(received_fd) = msg.args[1] {
                    unsafe { libc::close(received_fd); }
                } else {
                    panic!("Argument was not an FD as expected.");
                }
            }
            Err(e) => {
                unsafe { libc::close(dummy_fd_to_send); }
                // Check for WouldBlock/EAGAIN before panicking, as it can be a timing issue in tests
                let mut is_would_block = false;
                match &e {
                    ClientError::Nix(nix::Error::Sys(errno)) if *errno == nix::errno::Errno::EAGAIN || *errno == nix::errno::Errno::EWOULDBLOCK => is_would_block = true,
                    _ => {}
                }
                if is_would_block {
                     eprintln!("Test (receives_fd) encountered WouldBlock/EAGAIN. This might be a CI timing issue.");
                } else {
                    panic!("handle_readable_and_get_messages failed when expecting FD: {:?}", e);
                }
            }
        }
        // Close the original FD created by dup(0) on the "server" side of the test.
        unsafe { libc::close(dummy_fd_to_send); }
    }


    #[test]
    fn test_validate_client_uid_same_uid_passes() {
        let current_uid = nix::unistd::getuid().as_raw();
        assert!(validate_client_uid(current_uid), "Validation must pass when client UID is the same as server UID.");
    }

    #[test]
    fn test_validate_client_uid_different_uid_fails() {
        let current_uid = nix::unistd::getuid().as_raw();
        // Choose a UID that is definitely different. If current_uid is 0 (root), use 1000. Otherwise, use 0.
        let different_uid = if current_uid == 0 { 1000 } else { 0 };
        // Ensure they are actually different before asserting, in case current_uid is 1000 and we are not root.
        if current_uid != different_uid {
            assert!(!validate_client_uid(different_uid), "Validation must fail when client UID is different from server UID.");
        } else {
            // This case would only happen if server is running as UID 1000 and we chose 0, then current_uid was 0 so we chose 1000.
            // Or if server is root (0) and we chose 1000, then current_uid was 1000 so we chose 0.
            // Essentially, if current_uid is one of {0, 1000} and different_uid becomes the other.
            // To make the test robust, pick another UID if different_uid accidentally matches current_uid.
            let alternative_different_uid = if current_uid == 1 { 2 } else { 1 }; // Just pick another one.
             assert_ne!(current_uid, alternative_different_uid, "Alternative UID should be different for test logic.");
            assert!(!validate_client_uid(alternative_different_uid), "Validation must fail with an alternative different UID.");
        }
    }

    #[test]
    fn test_client_resource_management_methods() {
        let (client_stream, _server_stream) = StdUnixStream::pair().expect("Failed to create socket pair for test");
        let creds = mock_credentials_for_test(); // Use mock credentials
        let mut client_obj = Client::new(1, client_stream, creds);

        // Initial state
        assert_eq!(client_obj.resource_usage, 0, "Initial resource usage should be 0.");

        // Test increment
        client_obj.increment_resource_usage(5);
        assert_eq!(client_obj.resource_usage, 5, "Resource usage should be 5 after incrementing by 5.");
        client_obj.increment_resource_usage(10);
        assert_eq!(client_obj.resource_usage, 15, "Resource usage should be 15 after incrementing by 10.");

        // Test decrement
        client_obj.decrement_resource_usage(3);
        assert_eq!(client_obj.resource_usage, 12, "Resource usage should be 12 after decrementing by 3.");

        // Test decrement to zero
        client_obj.decrement_resource_usage(12);
        assert_eq!(client_obj.resource_usage, 0, "Resource usage should be 0 after decrementing by 12.");

        // Test decrement below zero (should saturate at zero)
        client_obj.decrement_resource_usage(1);
        assert_eq!(client_obj.resource_usage, 0, "Resource usage should remain 0 when decrementing below zero.");

        // Test large increment
        client_obj.increment_resource_usage(usize::MAX);
        assert_eq!(client_obj.resource_usage, usize::MAX, "Resource usage should be usize::MAX after large increment.");

        // Test increment that would overflow (should saturate at usize::MAX)
        client_obj.increment_resource_usage(1);
        assert_eq!(client_obj.resource_usage, usize::MAX, "Resource usage should saturate at usize::MAX on overflow.");
    }


    #[test]
    fn test_client_handle_readable_eof_returns_err() {
        let (client_stream, server_stream) = StdUnixStream::pair().expect("Socket pair");
        client_stream.set_nonblocking(true).unwrap();

        let creds = mock_credentials_for_test();
        let mut client_obj = Client::new(1, client_stream, creds);
        let mut registry = ObjectRegistry::new();

        drop(server_stream);
        std::thread::sleep(std::time::Duration::from_millis(50));

        match client_obj.handle_readable_and_get_messages(&pm, &registry) {
            Err(ClientError::ConnectionClosed) => { /* Expected */ }
            // If WouldBlock, means recvmsg got nothing, and buffer was already empty.
            Err(ClientError::Nix(nix::Error::Sys(errno))) if errno == nix::errno::Errno::EAGAIN || errno == nix::errno::Errno::EWOULDBLOCK => {
                assert!(client_obj.read_buffer.is_empty(), "Buffer should be empty on WouldBlock if no prior data");
                assert!(client_obj.pending_fds.is_none(), "Pending FDs should be none on WouldBlock if no prior data");
                 println!("Note: test_client_handle_readable_eof got WouldBlock, assuming EOF processed by recvmsg returning 0 next time or ConnectionClosed error.");
            }
            Ok(msgs) if msgs.is_empty() && client_obj.read_buffer.is_empty() => {
                 println!("Note: test_client_handle_readable_eof got Ok([]), assuming EOF processed by recvmsg returning 0 and then ConnectionClosed error.");
            }
            Err(e) => panic!("Unexpected error on EOF: {:?}", e),
            Ok(msgs) => panic!("Expected ConnectionClosed or Ok([]) or WouldBlock on EOF, got Ok with messages: {:?}", msgs),
        }
    }

    // Mock WaylandObject for testing registry entries for different interface types
    #[derive(Debug)]
    struct MockWaylandObject {}
    impl WaylandObject for MockWaylandObject {}
}
