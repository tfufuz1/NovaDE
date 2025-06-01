// In novade-system/src/compositor/wayland_server/client.rs
use crate::compositor::wayland_server::error::WaylandServerError;
use nix::sys::socket::getpeerucred;
use nix::unistd::{Uid, Gid};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc; // For Arc<AtomicU32>
use std::time::SystemTime;
use tokio::net::UnixStream;
use tracing::{error, info, debug, warn};

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(u64);

impl ClientId {
    pub fn new() -> Self {
        ClientId(NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed))
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client-{}", self.0)
    }
}

#[derive(Debug, Clone)] // Clone needed if ClientCredentials is part of a clonable error or event
pub struct ClientCredentials {
    pub uid: Uid,
    pub gid: Gid,
    pub pid: Option<i32>,
}

#[derive(Debug)]
pub struct ClientContext {
    pub id: ClientId,
    // stream: UnixStream, // Stream is no longer directly stored here.
                          // It will be consumed by the task handling this client.
    pub credentials: ClientCredentials,
    pub connection_timestamp: SystemTime,
    pub object_count: Arc<AtomicU32>, // Basic resource tracking
                                      // In a real server, this would be part of a larger Client struct
                                      // that also holds its ObjectSpace, event queue sender for this client, etc.
}

impl ClientContext {
    // The actual UnixStream is passed here but should be consumed by the caller
    // to spawn a task for this client. ClientContext primarily stores metadata.
    pub fn new(stream_fd: std::os::unix::io::RawFd, allowed_uids: &[Uid]) -> Result<Self, WaylandServerError> {
        // let fd = stream.as_raw_fd(); // fd is now passed directly
        let ucred = getpeerucred(stream_fd).map_err(|errno| {
            error!("Failed to get peer credentials for fd {}: {}", stream_fd, errno);
            WaylandServerError::ClientConnection(format!(
                "Failed to get SO_PEERCRED for client: {}",
                errno
            ))
        })?;

        let credentials = ClientCredentials {
            uid: Uid::from_raw(ucred.uid),
            gid: Gid::from_raw(ucred.gid),
            pid: Some(ucred.pid),
        };

        // UID Validation
        if !allowed_uids.is_empty() && !allowed_uids.contains(&credentials.uid) {
            warn!(
                "Client connection rejected for UID {}. Allowed UIDs: {:?}.",
                credentials.uid, allowed_uids
            );
            return Err(WaylandServerError::ClientConnection(format!(
                "Client UID {} is not in the allowed list.",
                credentials.uid
            )));
        }

        let client_id = ClientId::new();
        info!(
            "New client {} validated. UID: {}, GID: {}, PID: {:?}",
            client_id, credentials.uid, credentials.gid, credentials.pid
        );

        Ok(ClientContext {
            id: client_id,
            credentials,
            connection_timestamp: SystemTime::now(),
            object_count: Arc::new(AtomicU32::new(0)),
        })
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn credentials(&self) -> &ClientCredentials {
        &self.credentials
    }

    pub fn connection_age_secs(&self) -> u64 {
        match self.connection_timestamp.elapsed() {
            Ok(duration) => duration.as_secs(),
            Err(_) => 0,
        }
    }

    pub fn increment_object_count(&self) {
        self.object_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_object_count(&self) {
        self.object_count.fetch_sub(1, Ordering::Relaxed);
        // Consider warning if count goes below zero, though AtomicU32 wraps on underflow.
        // let old_val = self.object_count.fetch_sub(1, Ordering::Relaxed);
        // if old_val == 0 {
        //     warn!("Client {} object_count decremented below zero.", self.id);
        // }
    }

    pub fn get_object_count(&self) -> u32 {
        self.object_count.load(Ordering::Relaxed)
    }
}

// Main server/compositor state would hold a list or map of active clients.
// e.g., clients: Arc<RwLock<HashMap<ClientId, Arc<ClientHandle>>>>
// where ClientHandle might contain the ClientContext, a way to send messages to it, etc.
//
// fn handle_new_connection(stream: UnixStream, server_state: Arc<ServerState>, event_queue: EventQueue) {
//     tokio::spawn(async move {
//         match ClientContext::new(stream.as_raw_fd(), &[Uid::current()] /* example allowed UIDs */) {
//             Ok(client_context) => {
//                 let client_id = client_context.id();
//                 // Store client_context or a handle to it in server_state
//                 // server_state.clients.write().await.insert(client_id, Arc::new(client_context)); // Simplified
//                 event_queue.send(WaylandEvent::NewClient { client_id }).await;
//
//                 // ... start client message processing loop using 'stream' ...
//                 // loop { read message from stream, dispatch to object, etc. }
//                 // On disconnect:
//                 // server_state.clients.write().await.remove(&client_id);
//                 // event_queue.send(WaylandEvent::ClientDisconnected { client_id, reason: "..."}).await;
//             }
//             Err(e) => {
//                 error!("Failed to create client context or client rejected: {}", e);
//                 // Stream is dropped here, connection closes.
//             }
//         }
//     });
// }


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tempfile::tempdir;
    use tokio::net::UnixStream as TokioUnixStream; // Renamed to avoid clash

    // Helper to create a connected UnixStream pair for testing ClientContext creation
    // Returns the server-side of the connection (as TokioUnixStream) and client-side (as std)
    fn create_stream_pair_for_test() -> (TokioUnixStream, std::os::unix::net::UnixStream) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_client_enh.sock");
        let listener = StdUnixListener::bind(&socket_path).unwrap();

        // Client connects
        let client_conn_std = std::os::unix::net::UnixStream::connect(&socket_path).unwrap();
        // Server accepts
        let (server_conn_std, _addr) = listener.accept().unwrap();

        // Convert server end to Tokio stream for ClientContext::new which expects a RawFd
        server_conn_std.set_nonblocking(true).unwrap();
        client_conn_std.set_nonblocking(true).unwrap(); // Also set client to non-blocking

        let server_conn_tokio = TokioUnixStream::from_std(server_conn_std).unwrap();

        (server_conn_tokio, client_conn_std)
    }

    #[tokio::test]
    async fn test_client_context_uid_validation_allowed() {
        let (server_stream, _client_stream_std) = create_stream_pair_for_test();
        let current_uid = nix::unistd::getuid();
        let allowed_uids = [current_uid, Uid::from_raw(current_uid.as_raw() + 1000)]; // Current UID is in the list

        let client_context_res = ClientContext::new(server_stream.as_raw_fd(), &allowed_uids);
        assert!(client_context_res.is_ok(), "ClientContext creation should succeed for allowed UID. Err: {:?}", client_context_res.err());
        let context = client_context_res.unwrap();
        assert_eq!(context.credentials.uid, current_uid);
        assert_eq!(context.get_object_count(), 0); // Initial object count
    }

    #[tokio::test]
    async fn test_client_context_uid_validation_denied() {
        let (server_stream, _client_stream_std) = create_stream_pair_for_test();
        let other_uid = Uid::from_raw(nix::unistd::getuid().as_raw() + 1000); // A UID not the current one
        if other_uid == nix::unistd::getuid() { // handle case where +1000 wraps or is not unique enough for test
             // This case is unlikely in typical OS but good for robustness of test logic
            warn!("Other UID for test is same as current UID, test may not be meaningful for denial.");
        }
        let allowed_uids = [other_uid]; // Current UID is NOT in this list

        let client_context_res = ClientContext::new(server_stream.as_raw_fd(), &allowed_uids);
        assert!(client_context_res.is_err(), "ClientContext creation should be denied for UID not in list.");

        if let Err(WaylandServerError::ClientConnection(msg)) = client_context_res {
            assert!(msg.contains("is not in the allowed list"));
        } else {
            panic!("Expected ClientConnection error for denied UID, got {:?}", client_context_res);
        }
    }

    #[tokio::test]
    async fn test_client_context_uid_validation_empty_list_allows_all() {
        // Current behavior: empty list means no restriction from this check.
        let (server_stream, _client_stream_std) = create_stream_pair_for_test();
        let allowed_uids: [Uid; 0] = []; // Empty list

        let client_context_res = ClientContext::new(server_stream.as_raw_fd(), &allowed_uids);
        assert!(client_context_res.is_ok(), "ClientContext creation should succeed if allowed_uids is empty. Err: {:?}", client_context_res.err());
    }


    #[tokio::test]
    async fn test_client_context_object_count() {
        let (server_stream, _client_stream_std) = create_stream_pair_for_test();
        let current_uid = nix::unistd::getuid();

        let context = ClientContext::new(server_stream.as_raw_fd(), &[current_uid]).unwrap();
        assert_eq!(context.get_object_count(), 0);
        context.increment_object_count();
        assert_eq!(context.get_object_count(), 1);
        context.increment_object_count();
        assert_eq!(context.get_object_count(), 2);
        context.decrement_object_count();
        assert_eq!(context.get_object_count(), 1);
        // Test decrementing to zero
        context.decrement_object_count();
        assert_eq!(context.get_object_count(), 0);
        // Test decrementing below zero (should wrap if not careful, but AtomicU32 handles this by wrapping)
        // For this test, we just ensure it decrements. A real system might check old_val.
        context.decrement_object_count();
        assert_eq!(context.get_object_count(), u32::MAX); // Wraps around
    }

    #[tokio::test]
    async fn test_original_credential_extraction_and_id() {
        let (server_stream, _client_stream_std) = create_stream_pair_for_test();
        let current_uid = nix::unistd::getuid();

        let client_context_res = ClientContext::new(server_stream.as_raw_fd(), &[current_uid]);
        assert!(client_context_res.is_ok());
        let context = client_context_res.unwrap();

        let current_gid = nix::unistd::getgid();
        assert_eq!(context.credentials.uid, current_uid);
        assert_eq!(context.credentials.gid, current_gid);
        assert!(context.credentials.pid.is_some());

        let first_id = context.id;
        let (server_stream_2, _client_stream_std_2) = create_stream_pair_for_test();
        let client_context_2 = ClientContext::new(server_stream_2.as_raw_fd(), &[current_uid]).unwrap();
        assert_ne!(first_id, client_context_2.id); // IDs should be unique
        assert_eq!(client_context_2.id.value(), first_id.value() + 1, "Client ID should increment");
    }
}
