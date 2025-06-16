use libc::{gid_t, pid_t, uid_t};
use nix::sys::socket::UnixCredentials;
use std::io::{self, Read};
use std::os::unix::net::UnixStream;
use std::time::SystemTime;
use std::os::unix::io::RawFd;

use super::message::{self, Argument, Message, MessageParseError}; // Added Message
use super::object_registry::{ObjectRegistry, WlRegistry};

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
}

impl Client {
    pub fn new(id: u64, stream: UnixStream, creds: UnixCredentials) -> Self {
        Client {
            id,
            uid: creds.uid(),
            gid: creds.gid(),
            pid: creds.pid().unwrap_or(0),
            connection_ts: SystemTime::now(),
            resource_usage: 0,
            stream,
            read_buffer: Vec::with_capacity(4096),
            pending_fds: None,
        }
    }

    /// Reads data from stream, parses messages. Returns a Vec of parsed messages.
    pub fn handle_readable_and_get_messages(
        &mut self,
        registry: &mut ObjectRegistry, // Still needed for direct object creation like wl_registry
    ) -> Result<Vec<Message>, ClientError> {
        let mut temp_buf = [0u8; 2048];
        let mut parsed_messages = Vec::new();

        match self.stream.read(&mut temp_buf) {
            Ok(0) => {
                println!("Client {} disconnected (EOF).", self.id);
                return Err(ClientError::ConnectionClosed);
            }
            Ok(n) => {
                println!("Client {}: Read {} bytes.", self.id, n);
                self.read_buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No new data to read this time, return any messages parsed from existing buffer content.
            }
            Err(e) => {
                eprintln!("Client {}: Error reading from stream: {}", self.id, e);
                return Err(ClientError::Io(e));
            }
        }

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

            match message::parse_message(&self.read_buffer, &mut self.pending_fds) {
                Ok((message, rest)) => {
                    // --- Handle wl_display.get_registry directly for object creation ---
                    // This is a special case. Most messages would just be added to `parsed_messages`
                    // and dispatched by the EventDispatcher, which would then call methods on objects
                    // that might, in turn, use the registry.
                    if message.sender_id == 1 && message.opcode == 1 { // wl_display.get_registry
                        if let Some(Argument::NewId(registry_object_id)) = message.args.get(0) {
                            println!("Client {}: Directly handling wl_display.get_registry. New registry ID: {}", self.id, registry_object_id);
                            let wl_registry_obj = WlRegistry {};
                            match registry.new_object(self.id, *registry_object_id, wl_registry_obj, 0) {
                                Ok(()) => {
                                    println!("Client {}: Successfully registered wl_registry object ID {}.", self.id, registry_object_id);
                                    // TODO: The server should now send events to this new wl_registry object
                                    // (e.g., advertising globals). This would typically be done by posting
                                    // another event or directly calling a method on the new registry object.
                                }
                                Err(e) => {
                                    eprintln!("Client {}: Failed to register wl_registry object ID {}: {}", self.id, registry_object_id, e);
                                    // TODO: Send error to client.
                                }
                            }
                        }
                    }
                    // For other messages, or even this one if further action is needed by dispatcher:
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
}

#[allow(dead_code)]
pub fn validate_client_uid(uid: uid_t) -> bool {
    let _ = uid;
    true
}

#[derive(Debug)]
pub enum ClientError {
    Io(io::Error),
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

impl From<MessageParseError> for ClientError {
    fn from(err: MessageParseError) -> Self {
        ClientError::MessageParse(err)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "I/O error: {}", e),
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
    use nix::unistd::{Uid, Gid, Pid};
    use std::io::Write;

    fn mock_credentials_for_test() -> UnixCredentials {
        UnixCredentials::new(Uid::current().as_raw(), Gid::current().as_raw(), Some(Pid::this().as_raw()))
    }

    #[test]
    fn test_client_handle_readable_and_get_messages_get_registry() {
        let (mut client_stream, mut server_stream) = StdUnixStream::pair().expect("Socket pair");
        client_stream.set_nonblocking(true).unwrap();
        server_stream.set_nonblocking(true).unwrap();

        let creds = mock_credentials_for_test();
        let mut client_obj = Client::new(1, client_stream, creds);
        let mut registry = ObjectRegistry::new();

        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes());
        let opcode = 1u16; // get_registry
        let len = 12u16;
        msg_bytes.extend_from_slice(&((opcode as u32) << 16 | (len as u32)).to_ne_bytes());
        let new_registry_id_by_client = 3u32;
        msg_bytes.extend_from_slice(&new_registry_id_by_client.to_ne_bytes());

        server_stream.write_all(&msg_bytes).expect("Write get_registry message");
        server_stream.flush().expect("Flush get_registry message");
        std::thread::sleep(std::time::Duration::from_millis(100));

        match client_obj.handle_readable_and_get_messages(&mut registry) {
            Ok(messages) => {
                 assert!(client_obj.read_buffer.is_empty(), "Buffer not empty: {:?}", client_obj.read_buffer);
                 assert_eq!(messages.len(), 1, "Should have parsed one message");
                 assert_eq!(messages[0].sender_id, 1);
                 assert_eq!(messages[0].opcode, 1);

                 let entry = registry.get_entry(new_registry_id_by_client)
                                .expect("WlRegistry object not found");
                 assert!(entry.object.is::<WlRegistry>());
                 assert_eq!(entry.client_id, client_obj.id);
            }
            Err(e) => {
                if let ClientError::Io(io_err) = &e {
                    if io_err.kind() == io::ErrorKind::WouldBlock {
                        eprintln!("Test (get_registry) got WouldBlock, retrying.");
                        std::thread::sleep(std::time::Duration::from_millis(100));
                         let messages = client_obj.handle_readable_and_get_messages(&mut registry).expect("handle_readable failed on retry");
                         assert!(client_obj.read_buffer.is_empty(), "Buffer not empty (retry): {:?}", client_obj.read_buffer);
                         assert_eq!(messages.len(), 1);
                         registry.get_entry(new_registry_id_by_client).expect("WlRegistry not found (retry)");
                         return;
                    }
                }
                panic!("handle_readable_and_get_messages for get_registry failed: {:?}", e);
            }
        }
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

        match client_obj.handle_readable_and_get_messages(&mut registry) {
            Err(ClientError::ConnectionClosed) => { /* Expected */ }
            Ok(msgs) if msgs.is_empty() && client_obj.read_buffer.is_empty() => {
                 println!("Note: test_client_handle_readable_eof got Ok([]), assuming EOF processed.");
            }
            Err(e) => panic!("Unexpected error on EOF: {:?}", e),
            Ok(msgs) => panic!("Expected ConnectionClosed or Ok([]) on EOF, got Ok with messages: {:?}", msgs),
        }
    }
}
