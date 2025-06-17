use nix::sys::socket::UCred;
use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use super::object::ObjectId; // Changed from crate::wayland::object

pub type ClientId = u32;

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    Disconnected, // Or more specific error like ConnectionReset, BrokenPipe
    ObjectDispatchError, // Placeholder for when dispatching to Wayland objects fails
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::IoError(err)
    }
}

pub struct Client {
    id: ClientId,
    stream: UnixStream, // Make this mutable if send/receive take &mut self on stream
    credentials: UCred,
    object_ids: Vec<ObjectId>, // Placeholder for client-specific resources
                               // In a more complex setup, this might be an Arc<Mutex<ClientObjectManager>>
}

impl Client {
    pub fn new(id: ClientId, stream: UnixStream, credentials: UCred) -> Self {
        // It's often useful to set the stream to non-blocking mode early on,
        // especially when integrating with an event loop.
        // stream.set_nonblocking(true).expect("Failed to set non-blocking");
        Client {
            id,
            stream,
            credentials,
            object_ids: Vec::new(),
        }
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn credentials(&self) -> &UCred {
        &self.credentials
    }

    /// Sends raw byte data to the client.
    /// The stream is mutable here because write operations modify its internal state.
    pub fn send_data(&mut self, data: &[u8]) -> Result<(), ClientError> {
        match self.stream.write_all(data) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // This should not happen if stream is blocking (default)
                // If non-blocking, this means try again later.
                // For now, consider it an error or handle as per event loop strategy.
                Err(ClientError::IoError(e))
            }
            Err(e) => Err(ClientError::IoError(e)),
        }
    }

    /// Receives raw byte data from the client, reading up to `buffer.capacity()` bytes.
    /// Returns the number of bytes read.
    /// The stream is mutable here because read operations modify its internal state.
    pub fn receive_data(&mut self, buffer: &mut Vec<u8>) -> Result<usize, ClientError> {
        // Ensure buffer has some capacity to read into.
        // If Vec has len 0 but capacity > 0, read will use that capacity.
        // If Vec has len > 0, read will append. Usually, we want to clear and reuse.
        // For this example, let's assume buffer is cleared or its len is managed by caller.
        // A common pattern is to use a fixed-size array buffer first, then copy to Vec.

        // Make sure we don't read into already existing data if buffer is not empty.
        // This depends on how the caller uses the buffer.
        // A simple approach: use a temporary fixed-size buffer.
        let mut temp_buf = [0u8; 4096]; // Max read size per call

        match self.stream.read(&mut temp_buf) {
            Ok(0) => Err(ClientError::Disconnected), // Read 0 bytes means EOF
            Ok(n) => {
                buffer.extend_from_slice(&temp_buf[..n]);
                Ok(n)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // This should not happen if stream is blocking (default)
                // If non-blocking, this means try again later. Return 0 bytes read.
                Ok(0) // Or a specific WouldBlock error
            }
            Err(e) => Err(ClientError::IoError(e)),
        }
    }

    // Placeholder for methods to manage this client's Wayland objects
    pub fn add_object_id(&mut self, object_id: ObjectId) {
        self.object_ids.push(object_id);
    }

    pub fn list_object_ids(&self) -> &[ObjectId] {
        &self.object_ids
    }
}

pub struct ClientManager {
    clients: HashMap<ClientId, Arc<Mutex<Client>>>,
    next_client_id: ClientId,
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            clients: HashMap::new(),
            next_client_id: 1, // Start client IDs from 1
        }
    }

    fn generate_client_id(&mut self) -> ClientId {
        let id = self.next_client_id;
        self.next_client_id += 1;
        id
    }

    pub fn add_client(&mut self, stream: UnixStream, credentials: UCred) -> ClientId {
        let client_id = self.generate_client_id();
        let client = Client::new(client_id, stream, credentials);
        self.clients.insert(client_id, Arc::new(Mutex::new(client)));
        client_id
    }

    pub fn get_client(&self, id: ClientId) -> Option<Arc<Mutex<Client>>> {
        self.clients.get(&id).cloned()
    }

    pub fn remove_client(&mut self, id: ClientId) -> Option<Arc<Mutex<Client>>> {
        self.clients.remove(&id)
    }

    pub fn list_clients(&self) -> Vec<ClientId> {
        self.clients.keys().cloned().collect()
    }
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream as StdUnixStream; // For creating a pair

    // Helper to create a UCred for testing (requires nix feature and may be platform specific)
    // For simplicity, we might not be able to create a real UCred easily in unit tests
    // without actual socket communication or mocking.
    // Let's assume UCred can be defaulted or created with placeholder values if needed by Client::new.
    // For these tests, we'll focus on manager logic, mocking the stream and creds.

    fn mock_ucred() -> UCred {
        // This is not a valid UCred, but serves as a placeholder for tests
        // if the fields of UCred are not directly constructible or easily mockable.
        // On Linux, UCred { pid, uid, gid }
        // Depending on `nix` version and features, direct construction might be possible.
        // For this example, let's assume Client::new just takes it.
        // If UCred is not Default-constructible or easily made:
        // We might need to wrap Client::new or test ClientManager without constructing Client directly.

        // On Linux, UCred has pid, uid, gid. Let's try to construct one.
        // This will only work on Linux.
        #[cfg(target_os = "linux")]
        return UCred { pid: 0, uid: 0, gid: 0 };

        // Fallback for other OS where UCred might be different or not easily mocked
        #[cfg(not(target_os = "linux"))]
        {
            // This is a HACK. We can't easily create a UCred on non-Linux without involving OS calls.
            // For unit tests that don't rely on UCred content, this might be skipped by testing
            // ClientManager logic at a higher level or by conditional compilation of tests.
            // For now, let's make tests that need UCred Linux-only or use a feature flag.
            panic!("UCred mocking is tricky on non-Linux for this test setup");
        }
    }


    #[test]
    #[cfg(target_os = "linux")] // UCred construction is Linux-specific here
    fn test_client_manager_add_remove_client() {
        let mut manager = ClientManager::new();

        // Create a dummy UnixStream pair for testing
        let (stream1, _stream2) = StdUnixStream::pair().expect("Failed to create socket pair");

        let creds = mock_ucred(); // Placeholder

        let client_id1 = manager.add_client(stream1, creds);
        assert!(manager.get_client(client_id1).is_some(), "Client 1 should exist");

        let client_arc = manager.get_client(client_id1).unwrap();
        let client_locked = client_arc.lock().unwrap();
        assert_eq!(client_locked.id(), client_id1);
        // We can't easily check stream or creds without more complex mocking or real FDs.

        let (stream3, _stream4) = StdUnixStream::pair().expect("Failed to create socket pair");
        let client_id2 = manager.add_client(stream3, creds);
        assert!(manager.get_client(client_id2).is_some(), "Client 2 should exist");
        assert_ne!(client_id1, client_id2, "Client IDs should be unique");

        assert_eq!(manager.list_clients().len(), 2);

        manager.remove_client(client_id1);
        assert!(manager.get_client(client_id1).is_none(), "Client 1 should be removed");
        assert_eq!(manager.list_clients().len(), 1);

        manager.remove_client(client_id2);
        assert!(manager.get_client(client_id2).is_none(), "Client 2 should be removed");
        assert!(manager.list_clients().is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_client_new() {
        let (stream, _) = StdUnixStream::pair().unwrap();
        let creds = mock_ucred();
        let client = Client::new(1, stream, creds);
        assert_eq!(client.id(), 1);
        assert_eq!(client.credentials().uid, creds.uid); // Example check
        assert!(client.list_object_ids().is_empty());
    }

    // Test client send/receive would require more elaborate setup, possibly with threads
    // or a mock stream that can be controlled. For now, focus on manager logic.
    // Example of what a send/receive test might look like (conceptual):
    /*
    #[test]
    #[cfg(target_os = "linux")]
    fn test_client_send_receive_data() {
        let (mut stream_server_end, stream_client_end) = StdUnixStream::pair().unwrap();
        let creds = mock_ucred();
        let mut client = Client::new(1, stream_client_end, creds);

        let send_data = b"hello wayland";
        client.send_data(send_data).expect("Send failed");

        let mut receive_buffer_at_server = [0u8; 1024];
        let bytes_read_at_server = stream_server_end.read(&mut receive_buffer_at_server).unwrap();

        assert_eq!(bytes_read_at_server, send_data.len());
        assert_eq!(&receive_buffer_at_server[..bytes_read_at_server], send_data);

        // Now test receive on client side
        let send_data_from_server = b"hello from server";
        stream_server_end.write_all(send_data_from_server).unwrap();

        let mut client_receive_vec = Vec::new();
        let bytes_read_by_client = client.receive_data(&mut client_receive_vec).unwrap();

        assert_eq!(bytes_read_by_client, send_data_from_server.len());
        assert_eq!(client_receive_vec, send_data_from_server);
    }
    */
}
