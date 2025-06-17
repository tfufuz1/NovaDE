use nix::sys::socket::{accept, bind, listen, socket, sockopt, AddressFamily, SockFlag, SockType, UnixAddr, UCred};
use nix::unistd::close;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::path::Path;
use std::sync::Arc;
use std::io;

// Re-export UnixStream for convenience if Server is the main entry point for connections
pub use std::os::unix::net::UnixStream;


#[derive(Debug)]
pub enum ServerError {
    SocketCreateFailed(nix::Error),
    SocketBindFailed(nix::Error),
    SocketListenFailed(nix::Error),
    AcceptFailed(nix::Error),
    CredentialError(nix::Error),
    PathConversionError, // For &Path to CStr like conversions if needed by nix
    IoError(io::Error),
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> ServerError {
        ServerError::IoError(err)
    }
}

pub struct Server {
    fd: RawFd,
    socket_path: Arc<Path>, // Keep track of the path for cleanup
}

impl Server {
    pub fn new(socket_path: &Path) -> Result<Self, ServerError> {
        // Ensure the socket path directory exists (optional, depends on desired behavior)
        // if let Some(parent) = socket_path.parent() {
        //     if !parent.exists() {
        //         std::fs::create_dir_all(parent)?;
        //     }
        // }

        // Remove old socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        let socket_fd = socket(
            AddressFamily::Unix,
            SockType::Stream,
            SockFlag::SOCK_CLOEXEC, // Close on exec
            None,
        )
        .map_err(ServerError::SocketCreateFailed)?;

        // Set SO_REUSEADDR, useful for quickly restarting the server
        sockopt::set_reuseaddr(socket_fd, true).map_err(|e| {
            close(socket_fd).ok(); // best effort close
            ServerError::SocketCreateFailed(e) // Or a more specific error
        })?;


        let unix_addr = UnixAddr::new(socket_path).map_err(|_| ServerError::PathConversionError)?;
        bind(socket_fd, &unix_addr).map_err(|e| {
            close(socket_fd).ok();
            ServerError::SocketBindFailed(e)
        })?;

        listen(socket_fd, 128).map_err(|e| { // 128 is a common backlog size
            close(socket_fd).ok();
            ServerError::SocketListenFailed(e)
        })?;

        Ok(Server {
            fd: socket_fd,
            socket_path: Arc::from(socket_path.to_path_buf()),
        })
    }

    pub fn accept_connection(&self) -> Result<(UnixStream, UCred), ServerError> {
        let client_fd = accept(self.fd).map_err(ServerError::AcceptFailed)?;

        // Get client credentials
        let ucred = sockopt::get_peer_creds(client_fd).map_err(ServerError::CredentialError)?;

        // Convert RawFd to UnixStream
        // UnixStream::from_raw_fd is unsafe because it takes ownership and Rust can't know if fd is valid
        let stream = unsafe { UnixStream::from_raw_fd(client_fd) };

        Ok((stream, ucred))
    }

    pub fn path(&self) -> Arc<Path> {
        self.socket_path.clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        close(self.fd).ok(); // Ignore error on close
        // Attempt to remove the socket file.
        // This might fail if the server didn't shut down cleanly or due to permissions.
        if self.socket_path.exists() {
             std::fs::remove_file(&*self.socket_path).ok();
        }
        // println!("Server socket {} closed and file {:?} removed.", self.fd, self.socket_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)] // Example: skip if not integration_tests feature
    fn test_server_new_and_drop_cleanup() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_wayland.socket");

        {
            let server = Server::new(&socket_path).expect("Failed to create server");
            assert!(socket_path.exists(), "Socket file should exist after server creation");
            assert_eq!(server.path().as_ref(), socket_path.as_path());
        } // server goes out of scope, Drop is called

        assert!(!socket_path.exists(), "Socket file should be removed after server drop");
    }

    #[test]
    fn test_server_new_removes_old_socket() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_wayland_stale.socket");

        // Create a stale file at the socket path
        fs::write(&socket_path, "stale_content").unwrap();
        assert!(socket_path.exists());

        {
            let _server = Server::new(&socket_path).expect("Failed to create server with stale socket");
            assert!(socket_path.exists(), "Socket file should exist after server creation");
            // Check it's a socket, not the stale file (hard to check type directly without connecting)
        }
        assert!(!socket_path.exists(), "Socket file should be removed after server drop");
    }


    // Note: Testing `accept_connection` typically requires a client to connect.
    // This would be more of an integration test.
    // For unit tests, one might need to use `socketpair` or mock `nix::sys::socket::accept`.
    // For now, we'll rely on the `new` and `drop` tests for basic validation.
}
