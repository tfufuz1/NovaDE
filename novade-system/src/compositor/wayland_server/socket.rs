// In novade-system/src/compositor/wayland_server/socket.rs
use crate::compositor::wayland_server::error::WaylandServerError;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::fs;
use tokio::net::UnixListener;
use tracing::{info, warn, error, debug};
use nix::sys::stat::{umask, Mode, stat, SFlag};
use nix::unistd::getuid;

const WAYLAND_DISPLAY_PREFIX: &str = "wayland-";
const SOCKET_PERMISSIONS: u32 = 0o700;

pub struct WaylandSocket {
    listener: UnixListener,
    socket_path: PathBuf,
    lock_path: PathBuf,
}

trait PathExt {
    fn is_socket(&self) -> bool;
}

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
                    "XDG_RUNTIME_DIR is not set. Please ensure it is available in the environment.".to_string()
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
            .map_err(|e| {
                umask(original_umask);
                WaylandServerError::SocketCreation(format!("Failed to bind socket {}: {}", socket_path.display(), e))
            })?;
        umask(original_umask);

        match fs::set_permissions(&socket_path, fs::Permissions::from_mode(SOCKET_PERMISSIONS)) {
            Ok(_) => info!("Set socket permissions to {:o} for {}", SOCKET_PERMISSIONS, socket_path.display()),
            Err(e) => {
                let _ = tokio::fs::remove_file(&socket_path).await;
                let _ = tokio::fs::remove_file(&lock_path).await;
                return Err(WaylandServerError::SocketCreation(format!("Failed to set permissions for socket {}: {}", socket_path.display(), e)));
            }
        }

        info!("Wayland socket listening on {}", socket_path.display());

        Ok(WaylandSocket {
            listener,
            socket_path,
            lock_path,
        })
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
                tokio::fs::remove_file(lock_path)
                    .await
                    .map_err(|e| WaylandServerError::SocketCreation(format!("Failed to remove stale lock file {}: {}", lock_path.display(), e)))?;
            }
        }

        if socket_path.exists() {
            if socket_path.is_socket() {
                warn!("Removing stale Wayland socket at {} (no active lock file found).", socket_path.display());
                tokio::fs::remove_file(socket_path)
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

    pub fn path(&self) -> &Path {
        &self.socket_path
    }

    pub async fn accept(&mut self) -> Result<(tokio::net::UnixStream, std::os::unix::net::SocketAddr), WaylandServerError> {
        let (stream, addr) = self.listener.accept()
            .await
            .map_err(|e| WaylandServerError::ClientConnection(format!("Failed to accept new client connection: {}", e)))?;
        info!("Accepted new client connection from {:?}", addr);
        Ok((stream, addr))
    }
}

impl Drop for WaylandSocket {
    fn drop(&mut self) {
        info!("Cleaning up Wayland socket: {} and lock: {}", self.socket_path.display(), self.lock_path.display());
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    // Removed use tokio::runtime::Runtime; as #[tokio::test] provides its own runtime.
    use std::fs;

    fn setup_test_runtime_dir(test_name: &str) -> PathBuf {
        let uid = nix::unistd::getuid();
        let temp_dir_base = std::env::temp_dir();
        let test_runtime_path = temp_dir_base.join(format!("novade-test-runtime-{}-{}", uid, test_name));

        if test_runtime_path.exists() {
            fs::remove_dir_all(&test_runtime_path).expect("Failed to clean up old test temp dir before setup");
        }
        fs::create_dir_all(&test_runtime_path).expect("Failed to create test temp dir");
        fs::set_permissions(&test_runtime_path, fs::Permissions::from_mode(0o700)).expect("Failed to set perms on test temp dir");

        std::env::set_var("XDG_RUNTIME_DIR", test_runtime_path.to_str().unwrap());
        test_runtime_path
    }

    fn cleanup_test_runtime_dir(base_dir: PathBuf) {
        if base_dir.exists() {
            fs::remove_dir_all(&base_dir).expect("Failed to remove test temp dir after test");
        }
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[tokio::test]
    async fn test_socket_creation_and_cleanup() {
        let test_runtime_dir = setup_test_runtime_dir("socket_creation_cleanup_v2"); // Changed name slightly for uniqueness
        let display_suffix = "test0_create_cleanup_v2";
        let expected_socket_path = test_runtime_dir.join(format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix));
        let expected_lock_path = test_runtime_dir.join(format!("{}{}.lock", WAYLAND_DISPLAY_PREFIX, display_suffix));

        let socket = WaylandSocket::new(display_suffix).await.expect("Socket creation failed");

        assert_eq!(socket.path(), expected_socket_path);
        assert!(expected_socket_path.exists(), "Socket file should exist. Path: {}", expected_socket_path.display());
        assert!(expected_socket_path.is_socket(), "Path should be a socket. Path: {}", expected_socket_path.display());
        assert!(expected_lock_path.exists(), "Lock file should exist. Path: {}", expected_lock_path.display());

        let socket_perms = fs::metadata(&expected_socket_path).unwrap().permissions();
        assert_eq!(socket_perms.mode() & 0o777, SOCKET_PERMISSIONS, "Socket permissions incorrect");

        let lock_perms = fs::metadata(&expected_lock_path).unwrap().permissions();
        assert_eq!(lock_perms.mode() & 0o777, 0o600, "Lock file permissions incorrect");

        drop(socket);
        assert!(!expected_socket_path.exists(), "Socket file should be cleaned up on drop");
        assert!(!expected_lock_path.exists(), "Lock file should be cleaned up on drop");

        cleanup_test_runtime_dir(test_runtime_dir);
    }

    #[tokio::test]
    async fn test_stale_socket_removal_no_lock() {
        let test_runtime_dir = setup_test_runtime_dir("stale_socket_no_lock_v2");
        let display_suffix = "stale1_v2";
        let stale_socket_path = test_runtime_dir.join(format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix));

        let _stale_listener = UnixListener::bind(&stale_socket_path).await.expect("Failed to create stale listener");
        assert!(stale_socket_path.exists());
        assert!(stale_socket_path.is_socket());

        let socket = WaylandSocket::new(display_suffix).await.expect("Socket creation failed, possibly stale socket not removed");
        assert!(stale_socket_path.exists());
        assert!(stale_socket_path.is_socket());

        drop(socket);
        cleanup_test_runtime_dir(test_runtime_dir);
    }

    #[tokio::test]
    async fn test_stale_lock_removal_if_socket_missing() {
        let test_runtime_dir = setup_test_runtime_dir("stale_lock_socket_missing_v2");
        let display_suffix = "stale2_v2";
        let socket_path = test_runtime_dir.join(format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix));
        let stale_lock_path = test_runtime_dir.join(format!("{}{}.lock", WAYLAND_DISPLAY_PREFIX, display_suffix));

        fs::File::create(&stale_lock_path).expect("Failed to create stale lock file");
        assert!(stale_lock_path.exists());
        assert!(!socket_path.exists());

        let socket = WaylandSocket::new(display_suffix).await.expect("Socket creation failed, possibly stale lock not removed");
        assert!(socket.lock_path.exists(), "New lock file should exist");

        drop(socket);
        cleanup_test_runtime_dir(test_runtime_dir);
    }

    #[tokio::test]
    async fn test_error_if_socket_and_lock_exist() {
        let test_runtime_dir = setup_test_runtime_dir("socket_and_lock_exist_v2");
        let display_suffix = "busy0_v2";
        let socket_path = test_runtime_dir.join(format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix));
        let lock_path = test_runtime_dir.join(format!("{}{}.lock", WAYLAND_DISPLAY_PREFIX, display_suffix));

        let _listener = UnixListener::bind(&socket_path).await.expect("Failed to bind for test");
        fs::File::create(&lock_path).expect("Failed to create lock for test");

        let result = WaylandSocket::new(display_suffix).await;
        assert!(result.is_err(), "Expected error when socket and lock exist");
        if let Err(WaylandServerError::SocketCreation(msg)) = result {
            assert!(msg.contains("may already be in use"));
        } else {
            panic!("Expected SocketCreation error, got {:?}", result);
        }

        std::fs::remove_file(&socket_path).ok();
        std::fs::remove_file(&lock_path).ok();
        cleanup_test_runtime_dir(test_runtime_dir);
    }

    #[tokio::test]
    async fn test_error_if_path_exists_and_is_not_socket() {
        let test_runtime_dir = setup_test_runtime_dir("path_not_socket_v2");
        let display_suffix = "invalid_path_v2";
        let non_socket_path = test_runtime_dir.join(format!("{}{}", WAYLAND_DISPLAY_PREFIX, display_suffix));

        fs::File::create(&non_socket_path).expect("Failed to create dummy file");

        let result = WaylandSocket::new(display_suffix).await;
        assert!(result.is_err());
        if let Err(WaylandServerError::SocketCreation(msg)) = result {
            assert!(msg.contains("exists and is not a socket"));
        } else {
            panic!("Expected SocketCreation error for non-socket file, got {:?}", result);
        }

        fs::remove_file(&non_socket_path).ok();
        cleanup_test_runtime_dir(test_runtime_dir);
    }

    #[tokio::test]
    async fn test_xdg_runtime_dir_must_be_set() {
        let original_xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok();
        std::env::remove_var("XDG_RUNTIME_DIR");

        let result = WaylandSocket::new("test_no_xdg_v2").await;
        assert!(result.is_err(), "Should fail if XDG_RUNTIME_DIR is not set");
        if let Err(WaylandServerError::SocketCreation(msg)) = result {
            assert!(msg.contains("XDG_RUNTIME_DIR is not set"));
        } else {
            panic!("Expected SocketCreation error due to missing XDG_RUNTIME_DIR, got {:?}", result);
        }

        if let Some(val) = original_xdg_runtime_dir {
            std::env::set_var("XDG_RUNTIME_DIR", val);
        }
    }
}
