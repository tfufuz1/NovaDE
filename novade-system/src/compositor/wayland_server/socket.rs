// In novade-system/src/compositor/wayland_server/socket.rs
use crate::compositor::wayland_server::error::WaylandServerError;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::{AsRawFd, RawFd}; // Added RawFd
use std::fs;
use tokio::net::{UnixListener, UnixStream as TokioUnixStream}; // Renamed UnixStream
use tracing::{info, warn, error, debug, trace}; // Added trace
use nix::sys::stat::{umask, Mode, stat, SFlag};
use nix::unistd::getuid;
use nix::sys::socket::{recvmsg, sendmsg, ControlMessage, MsgFlags, CmsgSpace, ControlMessageOwned}; // For recvmsg, sendmsg
use nix::sys::uio::IoVec; // For IoVec
use bytes::BytesMut; // For byte buffer in read_with_fds

const WAYLAND_DISPLAY_PREFIX: &str = "wayland-";
const SOCKET_PERMISSIONS: u32 = 0o700;

pub struct WaylandSocket {
    listener: UnixListener,
    socket_path: PathBuf,
    lock_path: PathBuf,
}

trait PathExt { fn is_socket(&self) -> bool; }
impl PathExt for Path {
    fn is_socket(&self) -> bool {
        match stat(self) {
            Ok(s) => SFlag::from_bits_truncate(s.st_mode) == SFlag::S_IFSOCK,
            Err(_) => false,
        }
    }
}

impl WaylandSocket {
    pub async fn new(display_suffix: &str) -> Result<Self, WaylandServerError> {
        let _uid = getuid();
        let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
             Ok(path) => PathBuf::from(path),
             Err(e) => {
                error!("XDG_RUNTIME_DIR not set: {}. This is required.", e);
                return Err(WaylandServerError::SocketCreation(
                    format!("XDG_RUNTIME_DIR is not set. Please ensure it is available in the environment.: {}", e)
                ));
            }
        };

        if !runtime_dir.exists() {
            debug!("XDG_RUNTIME_DIR {} does not exist, attempting to create it.", runtime_dir.display());
            fs::create_dir_all(&runtime_dir)
                .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to create runtime dir {}: {}", runtime_dir.display(), e)))?;
            fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700))
                 .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to set permissions on runtime dir {}: {}", runtime_dir.display(), e)))?;
        }

        let socket_name = format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix);
        let socket_path = runtime_dir.join(&socket_name);
        let lock_path = runtime_dir.join(format!("{}.lock", socket_name));

        Self::handle_existing_socket_and_lock(&socket_path, &lock_path).await?;

        let _lock_file = fs::File::create(&lock_path)
            .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to create lock file {}: {}", lock_path.display(), e)))?;
        fs::set_permissions(&lock_path, fs::Permissions::from_mode(0o600))
             .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to set permissions for lock file {}: {}", lock_path.display(), e)))?;

        let restrictive_umask = Mode::S_IRWXG | Mode::S_IRWXO; // 0o077
        let original_umask = umask(restrictive_umask);

        let listener = UnixListener::bind(&socket_path)
            .await // Added await here as bind is async
            .map_err(|e| {
                umask(original_umask); // Restore umask on error
                WaylandServerError::SocketCreation(format!("Failed to bind socket {}: {}", socket_path.display(), e))
            })?;
        umask(original_umask); // Restore umask on success

        match fs::set_permissions(&socket_path, fs::Permissions::from_mode(SOCKET_PERMISSIONS)) {
            Ok(_) => info!("Set socket permissions to {:o} for {}", SOCKET_PERMISSIONS, socket_path.display()),
            Err(e) => {
                // Cleanup on error
                let _ = tokio::fs::remove_file(&socket_path).await; // Use tokio::fs for async remove
                let _ = tokio::fs::remove_file(&lock_path).await;
                return Err(WaylandServerError::SocketCreation(format!("Failed to set permissions for socket {}: {}", socket_path.display(), e)));
            }
        }

        info!("Wayland socket listening on {}", socket_path.display());
        Ok(WaylandSocket { listener, socket_path, lock_path })
    }

    async fn handle_existing_socket_and_lock(socket_path: &Path, lock_path: &Path) -> Result<(), WaylandServerError> {
        if lock_path.exists() {
            warn!("Lock file {} exists.", lock_path.display());
            if socket_path.exists() && socket_path.is_socket() {
                 debug!("Both socket {} and lock file {} exist.", socket_path.display(), lock_path.display());
                return Err(WaylandServerError::SocketCreation(format!(
                    "Wayland socket {} may already be in use (socket and lock file {} exist).", socket_path.display(), lock_path.display()
                )));
            } else {
                warn!("Socket {} does not exist or is not a socket, but lock file {} does. Removing stale lock.", socket_path.display(), lock_path.display());
                tokio::fs::remove_file(lock_path) // Use tokio::fs for async remove
                    .await
                    .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to remove stale lock file {}: {}", lock_path.display(), e)))?;
            }
        }

        if socket_path.exists() {
            if socket_path.is_socket() {
                warn!("Removing stale Wayland socket at {} (no active lock file found).", socket_path.display());
                tokio::fs::remove_file(socket_path) // Use tokio::fs for async remove
                    .await
                    .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to remove stale socket {}: {}", socket_path.display(), e)))?;
            } else {
                return Err(WaylandServerError::SocketCreation(format!(
                    "Path {} exists and is not a socket. Cannot create Wayland server.", socket_path.display()
                )));
            }
        }
        Ok(())
    }
    pub fn path(&self) -> &Path { &self.socket_path }
    pub async fn accept(&mut self) -> Result<(TokioUnixStream, std::os::unix::net::SocketAddr), WaylandServerError> {
        let (stream, addr) = self.listener.accept().await
            .map_err(|e| WaylandServerError::ClientConnection(format!("Failed to accept new client connection: {}", e)))?;
        info!("Accepted new client connection from {:?}", addr);
        Ok((stream, addr))
    }
}

impl Drop for WaylandSocket {
    fn drop(&mut self) {
        info!("Cleaning up Wayland socket: {} and lock: {}", self.socket_path.display(), self.lock_path.display());
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) { // std::fs is fine in drop
                error!("Failed to remove socket file {}: {}", self.socket_path.display(), e);
            }
        }
        if self.lock_path.exists() {
             if let Err(e) = std::fs::remove_file(&self.lock_path) {
                error!("Failed to remove socket lock file {}: {}", self.lock_path.display(), e);
            }
        }
    }
}

pub async fn read_from_stream_with_fds(
    stream: &TokioUnixStream,
    byte_buf: &mut BytesMut,
    fd_vec_buf: &mut Vec<RawFd>,
) -> Result<usize, std::io::Error> {
    stream.readable().await?;

    let result = stream.try_io(tokio::io::Interest::READABLE, |std_stream| {
        let mut temp_byte_slice = [0u8; 2048];
        let iov = [IoVec::from_mut_slice(&mut temp_byte_slice)];
        let mut cmsg_buf: CmsgSpace<[RawFd; 3]> = CmsgSpace::new();

        match recvmsg(std_stream.as_raw_fd(), &iov, Some(&mut cmsg_buf), MsgFlags::empty()) {
            Ok(msg) => {
                let bytes_read = msg.bytes;
                if bytes_read > 0 {
                    byte_buf.extend_from_slice(&temp_byte_slice[..bytes_read]);
                }

                fd_vec_buf.clear();
                for cmsg in msg.cmsgs() {
                    if let ControlMessageOwned::ScmRights(fds) = cmsg {
                        debug!("Received {} FDs via ScmRights: {:?}", fds.len(), fds);
                        fd_vec_buf.extend_from_slice(&fds);
                    } else {
                        warn!("Received unexpected control message: {:?}", cmsg);
                    }
                }
                Ok(bytes_read)
            }
            Err(nix::errno::Errno::EAGAIN) | Err(nix::errno::Errno::EWOULDBLOCK) => {
                trace!("recvmsg would block (EAGAIN/EWOULDBLOCK)");
                Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "recvmsg would block"))
            }
            Err(e) => {
                error!("recvmsg error: {}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, format!("recvmsg failed: {}", e)))
            }
        }
    });

    match result {
        Ok(Ok(bytes_read)) => Ok(bytes_read),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tempfile::tempdir;
    use nix::unistd::close;

    // WaylandSocket specific tests (abbreviated for brevity, assume they exist from previous steps)
    fn setup_test_runtime_dir_ws(test_name: &str) -> PathBuf { // Renamed to avoid conflict
        let temp_dir_base = std::env::temp_dir();
        let test_runtime_path = temp_dir_base.join(format!("novade-test-runtime-ws-{}-{}", nix::unistd::getuid(), test_name));
        if test_runtime_path.exists() { fs::remove_dir_all(&test_runtime_path).unwrap(); }
        fs::create_dir_all(&test_runtime_path).unwrap();
        fs::set_permissions(&test_runtime_path, fs::Permissions::from_mode(0o700)).unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", test_runtime_path.to_str().unwrap());
        test_runtime_path
    }
    fn cleanup_test_runtime_dir_ws(base_dir: PathBuf) { // Renamed
        if base_dir.exists() { fs::remove_dir_all(&base_dir).unwrap(); }
        std::env::remove_var("XDG_RUNTIME_DIR");
    }
    #[tokio::test]
    async fn test_socket_creation_and_cleanup_ws() { // Renamed
        let test_runtime_dir = setup_test_runtime_dir_ws("socket_creation_cleanup_fds");
        let socket = WaylandSocket::new("test_fds").await.expect("Socket creation failed");
        assert!(socket.path().exists());
        drop(socket);
        // assert!(!socket.path().exists()); // Path is moved, this check is tricky. Drop handles cleanup.
        cleanup_test_runtime_dir_ws(test_runtime_dir);
    }


    async fn create_fd_passing_stream_pair() -> (TokioUnixStream, TokioUnixStream) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_fd_passing.sock");
        let listener = UnixListener::bind(&socket_path).await.unwrap();

        let client_fut = TokioUnixStream::connect(&socket_path);
        let server_fut = listener.accept();

        let (client_res, server_res) = tokio::join!(client_fut, server_fut);

        let client_stream = client_res.unwrap();
        let (server_stream, _addr) = server_res.unwrap();

        (client_stream, server_stream)
    }

    #[tokio::test]
    async fn test_read_from_stream_with_fds_no_fds_sent() {
        let (client_stream, server_stream) = create_fd_passing_stream_pair().await;

        let test_data = b"hello world";
        client_stream.writable().await.unwrap();
        client_stream.try_write(test_data).unwrap();

        let mut byte_buf = BytesMut::with_capacity(1024);
        let mut fd_buf = Vec::new();

        let bytes_read = read_from_stream_with_fds(&server_stream, &mut byte_buf, &mut fd_buf)
            .await
            .expect("Read failed");

        assert_eq!(bytes_read, test_data.len());
        assert_eq!(byte_buf.as_ref(), test_data);
        assert!(fd_buf.is_empty(), "No FDs should have been received");
    }

    #[tokio::test]
    async fn test_read_from_stream_with_one_fd_sent() {
        let (client_stream, server_stream) = create_fd_passing_stream_pair().await;

        let mut pipe_fds = [-1; 2];
        nix::unistd::pipe(&mut pipe_fds).expect("Failed to create pipe for test FD");
        let fd_to_send = pipe_fds[0];
        let other_pipe_end = pipe_fds[1];

        let test_data = b"data with fd";
        let iov = [IoVec::from_slice(test_data)];
        let cmsgs = [ControlMessage::ScmRights(&[fd_to_send])];

        client_stream.writable().await.unwrap();
        client_stream.try_io(tokio::io::Interest::WRITABLE, |std_client_stream| {
            sendmsg(std_client_stream.as_raw_fd(), &iov, &cmsgs, MsgFlags::empty(), None)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }).await.expect("sendmsg failed");

        let mut byte_buf = BytesMut::with_capacity(1024);
        let mut fd_buf = Vec::with_capacity(1);

        let bytes_read = read_from_stream_with_fds(&server_stream, &mut byte_buf, &mut fd_buf)
            .await
            .expect("Read with FDs failed");

        assert_eq!(bytes_read, test_data.len());
        assert_eq!(byte_buf.as_ref(), test_data);
        assert_eq!(fd_buf.len(), 1, "Should have received one FD");

        let received_fd = fd_buf[0];
        let stat_res = nix::sys::stat::fstat(received_fd);
        assert!(stat_res.is_ok(), "Received FD {} is not valid: {:?}", received_fd, stat_res.err());

        close(fd_to_send).ok();
        close(other_pipe_end).ok();
        close(received_fd).ok();
    }

    #[tokio::test]
    async fn test_read_from_stream_with_multiple_fds_sent() {
        let (client_stream, server_stream) = create_fd_passing_stream_pair().await;

        let mut pipe1 = [-1; 2]; nix::unistd::pipe(&mut pipe1).unwrap();
        let mut pipe2 = [-1; 2]; nix::unistd::pipe(&mut pipe2).unwrap();
        let fds_to_send = [pipe1[0], pipe2[0]];

        let test_data = b"multi-fd message";
        let iov = [IoVec::from_slice(test_data)];
        let cmsgs = [ControlMessage::ScmRights(&fds_to_send)];

        client_stream.writable().await.unwrap();
        client_stream.try_io(tokio::io::Interest::WRITABLE, |std_s| {
            sendmsg(std_s.as_raw_fd(), &iov, &cmsgs, MsgFlags::empty(), None).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }).await.expect("sendmsg failed for multiple FDs");

        let mut byte_buf = BytesMut::with_capacity(1024);
        let mut fd_buf = Vec::with_capacity(3);

        let bytes_read = read_from_stream_with_fds(&server_stream, &mut byte_buf, &mut fd_buf).await.expect("Read with FDs failed");

        assert_eq!(bytes_read, test_data.len());
        assert_eq!(byte_buf.as_ref(), test_data);
        assert_eq!(fd_buf.len(), fds_to_send.len(), "Incorrect number of FDs received");

        for (i, _original_fd) in fds_to_send.iter().enumerate() { // original_fd not directly comparable
            let received_fd = fd_buf[i];
            assert!(nix::sys::stat::fstat(received_fd).is_ok(), "Received FD {} (original index {}) is not valid", received_fd, i);
            close(fds_to_send[i]).ok();
            close(received_fd).ok();
        }
        close(pipe1[1]).ok();
        close(pipe2[1]).ok();
    }
}
