// In novade-system/src/compositor/wayland_server/client.rs
use crate::compositor::wayland_server::error::WaylandServerError;
use nix::sys::socket::getpeerucred;
use nix::unistd::{Uid, Gid};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UnixStream;
use tracing::{error, info, debug};

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(u64);

impl ClientId {
    fn new() -> Self {
        ClientId(NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client-{}", self.0)
    }
}

#[derive(Debug)]
pub struct ClientCredentials {
    pub uid: Uid,
    pub gid: Gid,
    pub pid: Option<i32>, // pid is part of ucred but might not always be available or relevant
}

#[derive(Debug)]
pub struct ClientContext {
    pub id: ClientId,
    pub stream: UnixStream, // The actual stream will be owned by a per-client task
    pub credentials: ClientCredentials,
    pub connection_timestamp: SystemTime,
    // TODO: Add resource usage tracking, e.g., Arc<AtomicUsize> for memory, object count, etc.
}

impl ClientContext {
    pub fn new(stream: UnixStream) -> Result<Self, WaylandServerError> {
        let fd = stream.as_raw_fd();
        let ucred = getpeerucred(fd).map_err(|errno| {
            error!("Failed to get peer credentials for fd {}: {}", fd, errno);
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

        let client_id = ClientId::new();
        info!(
            "New client {} connected. UID: {}, GID: {}, PID: {:?}",
            client_id, credentials.uid, credentials.gid, credentials.pid
        );

        // TODO: Validate client UID against an allowed user list
        // For now, we accept any client that successfully connects and provides credentials.
        // Example: if credentials.uid != Uid::current() { /* reject */ }

        Ok(ClientContext {
            id: client_id,
            stream, // Stream is passed in, will be taken by the client's task
            credentials,
            connection_timestamp: SystemTime::now(),
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
            Err(_) => 0, // Time went backwards, should not happen
        }
    }
}

// The main accept loop itself will live in a higher-level server module
// that uses WaylandSocket::accept() and then creates a ClientContext.
// For now, this file defines what a client *is*.

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UnixStream;
    use std::os::unix::net::UnixListener as StdUnixListener; // For creating a pair for tests
    use tempfile::tempdir;

    // Helper to create a connected UnixStream pair for testing ClientContext creation
    fn create_stream_pair() -> (UnixStream, std::os::unix::net::UnixStream) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_client.sock"); // Corrected let-socket_path
        let listener = StdUnixListener::bind(&socket_path).unwrap();

        let server_end_std = std::os::unix::net::UnixStream::connect(&socket_path).unwrap();
        let (client_end_std, _addr) = listener.accept().unwrap();

        // Convert to Tokio streams
        server_end_std.set_nonblocking(true).unwrap();
        client_end_std.set_nonblocking(true).unwrap();

        let server_end_tokio = UnixStream::from_std(server_end_std).unwrap();
        // client_end_tokio is not directly used here, but its existence proves connection
        (server_end_tokio, client_end_std)
    }

    #[tokio::test]
    async fn test_client_context_creation_and_credential_extraction() {
        // This test relies on the OS correctly returning credentials for a unix socket pair.
        // It effectively tests `getpeerucred` in a controlled environment.
        let (server_stream, _client_stream_std) = create_stream_pair();

        let client_context = ClientContext::new(server_stream);

        assert!(client_context.is_ok(), "ClientContext creation failed: {:?}", client_context.err());
        let context = client_context.unwrap();

        // Check credentials
        let current_uid = nix::unistd::getuid();
        let current_gid = nix::unistd::getgid();
        // PID can be tricky as it might be the PID of the test runner itself.
        // We mainly care that UID and GID are correct.
        assert_eq!(context.credentials.uid, current_uid, "Client UID does not match current UID");
        assert_eq!(context.credentials.gid, current_gid, "Client GID does not match current GID");
        assert!(context.credentials.pid.is_some(), "Client PID should be available");

        info!("ClientContext created successfully: Client ID {}, UID {}, GID {}, PID {:?}",
            context.id, context.credentials.uid, context.credentials.gid, context.credentials.pid);

        let first_id = context.id;
        // Create another client to check ID increment
        let (server_stream_2, _client_stream_std_2) = create_stream_pair();
        let client_context_2 = ClientContext::new(server_stream_2).unwrap();
        assert_ne!(first_id, client_context_2.id, "Client IDs should be unique");
        assert_eq!(client_context_2.id.0, first_id.0 + 1, "Client ID should increment");
    }

    #[test]
    fn test_client_id_display() {
        let id = ClientId(123);
        assert_eq!(format!("{}", id), "client-123");
    }
}
