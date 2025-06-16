use nix::unistd::getuid;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

// Ensure generate_socket_path uses XDG_RUNTIME_DIR if available, otherwise fallback.
fn generate_socket_path(display_num: u32) -> Result<PathBuf, String> {
    let uid = getuid();
    let runtime_dir_base = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", uid));

    // Ensure runtime_dir_base itself exists if we are using the fallback.
    // For XDG_RUNTIME_DIR, it's assumed to be managed by the system.
    if std::env::var("XDG_RUNTIME_DIR").is_err() {
        let base_path = PathBuf::from(&runtime_dir_base);
        if !base_path.exists() {
             fs::create_dir_all(&base_path)
                .map_err(|e| format!("Failed to create fallback runtime directory {:?}: {}", base_path, e))?;
             fs::set_permissions(&base_path, fs::Permissions::from_mode(0o700))
                .map_err(|e| format!("Failed to set permissions for fallback runtime directory {:?}: {}", base_path, e))?;
        }
    }
    Ok(PathBuf::from(runtime_dir_base).join(format!("wayland-{}", display_num)))
}

fn create_and_bind_socket(path: &Path) -> Result<UnixListener, String> {
    if path.exists() {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.file_type().is_socket() {
                    fs::remove_file(path).map_err(|e| format!("Failed to remove existing socket file at {:?}: {}", path, e))?;
                } else if metadata.is_file() { // It's a regular file, not a socket
                    fs::remove_file(path).map_err(|e| format!("Failed to remove existing file at {:?}: {}", path, e))?;
                } else if metadata.is_dir() {
                    return Err(format!("Socket path {:?} exists and is a directory. Please remove it manually.", path));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist, which is fine, we'll create it.
            }
            Err(e) => {
                return Err(format!("Failed to get metadata for path {:?}: {}", path, e));
            }
        }
    }

    if let Some(parent_dir) = path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("Failed to create parent directory {:?}: {}", parent_dir, e))?;
             // Set parent directory permissions to 0700 as well, if we created it.
            fs::set_permissions(parent_dir, fs::Permissions::from_mode(0o700))
                .map_err(|e| format!("Failed to set permissions for parent directory {:?}: {}", parent_dir, e))?;
        }
    }

    let listener = UnixListener::bind(path)
        .map_err(|e| format!("Failed to bind socket to {:?}: {}", path, e))?;

    let permissions = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, permissions)
        .map_err(|e| format!("Failed to set permissions for socket {:?}: {}", path, e))?;

    Ok(listener)
}

// Renamed and made public for server lifecycle
pub fn cleanup_socket_path_from_display_num(display_num: u32) -> Result<(), String> {
    let socket_path = generate_socket_path(display_num)?;
    if socket_path.exists() {
        match std::fs::metadata(&socket_path) {
            Ok(metadata) => {
                if metadata.file_type().is_socket() {
                    std::fs::remove_file(&socket_path).map_err(|e| {
                        format!(
                            "Failed to remove socket file at {:?}: {}",
                            socket_path, e
                        )
                    })?;
                    println!("[SignalHandler] Cleaned up socket {:?}", socket_path);
                } else {
                    eprintln!("[SignalHandler] Path {:?} exists but is not a socket. Skipping cleanup.", socket_path);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File/socket doesn't exist, so nothing to clean up.
            }
            Err(e) => {
                return Err(format!("Failed to get metadata for socket path {:?}: {}", socket_path, e));
            }
        }
    } else {
        println!("[SignalHandler] Socket path {:?} does not exist. No cleanup needed.", socket_path);
    }
    Ok(())
}

pub fn init_wayland_socket(display_num: u32) -> Result<UnixListener, String> {
    let socket_path = generate_socket_path(display_num)?;
    // Parent directory creation is handled in create_and_bind_socket
    create_and_bind_socket(&socket_path)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use std::os::unix::net::UnixListener as StdUnixListener;

    // Test-specific cleanup helper using a direct path
    // Renamed from cleanup_socket to cleanup_socket_at_path and marked #[cfg(test)]
    #[cfg(test)]
    fn cleanup_socket_at_path(path: &Path) -> Result<(), String> {
        if path.exists() {
            let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata for {:?}: {}", path, e))?;
            if metadata.file_type().is_socket() || metadata.is_file() { // Can clean up files too for test simplicity
                 fs::remove_file(path).map_err(|e| format!("Failed to remove socket/file at {:?}: {}", path, e))?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_generate_socket_path_xdg_runtime_dir_set() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let expected_path = dir.path().join("wayland-0");
        assert_eq!(generate_socket_path(0), Ok(expected_path));
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_generate_socket_path_fallback() {
        std::env::remove_var("XDG_RUNTIME_DIR"); // Ensure it's not set
        let uid = nix::unistd::getuid();
        // Create the fallback directory for the test to ensure it can be written to
        let fallback_dir_base = format!("/run/user/{}", uid);
        let fallback_dir = PathBuf::from(&fallback_dir_base);
        if !fallback_dir.exists() {
            fs::create_dir_all(&fallback_dir).expect("Test setup: Failed to create fallback base dir");
            fs::set_permissions(&fallback_dir, fs::Permissions::from_mode(0o700)).expect("Test setup: Failed to set perms on fallback base dir");
        }


        let expected_path = PathBuf::from(fallback_dir_base).join("wayland-1");
        assert_eq!(generate_socket_path(1), Ok(expected_path));
        // No cleanup of /run/user/{uid} here, assume test environment handles it or it's fine.
    }


    #[test]
    fn test_create_and_bind_socket_new() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());

        let socket_path = generate_socket_path(100).unwrap();

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok(), "create_and_bind_socket failed: {:?}", listener.err());
        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(metadata.file_type().is_socket());
        cleanup_socket_at_path(&socket_path).unwrap(); // Use new test-specific cleanup
        assert!(!socket_path.exists());
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_create_and_bind_socket_existing_socket_file() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let socket_path = generate_socket_path(101).unwrap();

        let _dummy_socket = StdUnixListener::bind(&socket_path).expect("Failed to create dummy test socket");
        assert!(socket_path.exists());

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok(), "create_and_bind_socket failed for existing socket: {:?}", listener.err());
        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(metadata.file_type().is_socket());
        cleanup_socket_at_path(&socket_path).unwrap(); // Use new test-specific cleanup
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_create_and_bind_socket_existing_non_socket_file() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let file_path = generate_socket_path(102).unwrap();

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "hello").unwrap();
        drop(file);
        assert!(file_path.exists());
        assert!(fs::metadata(&file_path).unwrap().is_file());


        let listener = create_and_bind_socket(&file_path);
        assert!(listener.is_ok(), "create_and_bind_socket failed for existing file: {:?}", listener.err());
        let metadata = fs::metadata(&file_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(metadata.file_type().is_socket(), "Path should now be a socket");
        cleanup_socket_at_path(&file_path).unwrap(); // Use new test-specific cleanup
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_create_and_bind_socket_parent_dir_creation() {
        let base_dir = tempdir().unwrap();
        let parent_name = "test_parent_dir_csbpdc";
        let parent_path = base_dir.path().join(parent_name);

        std::env::set_var("XDG_RUNTIME_DIR", &parent_path);

        let socket_path = generate_socket_path(103).unwrap();

        // generate_socket_path will try to create parent_path if XDG_RUNTIME_DIR is set to it
        // and it doesn't exist. create_and_bind_socket also has parent creation logic.
        // For this test, we want to ensure the socket's direct parent gets created if needed.
        // The socket_path itself will be parent_path/wayland-103.
        // So parent_path must exist.

        assert!(!parent_path.exists(), "Parent directory of XDG_RUNTIME_DIR should not exist for this test to be meaningful regarding its creation.");

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok(), "create_and_bind_socket failed with parent dir creation: {:?}", listener.err());

        // XDG_RUNTIME_DIR (parent_path) should have been created by generate_socket_path or create_and_bind_socket.
        assert!(parent_path.exists(), "XDG_RUNTIME_DIR (parent_path) directory should have been created.");
        assert!(parent_path.is_dir());
        let parent_meta = fs::metadata(&parent_path).unwrap();
        assert_eq!(parent_meta.permissions().mode() & 0o777, 0o700, "XDG_RUNTIME_DIR (parent_path) directory permissions should be 0700");

        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(metadata.file_type().is_socket());

        cleanup_socket_at_path(&socket_path).unwrap(); // Use new test-specific cleanup
        // No need to fs::remove_dir_all(base_dir.path()).unwrap(); tempdir handles it.
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_cleanup_socket_path_from_display_num_existing_socket() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let display_num_for_test = 9999u32;
        let socket_path = generate_socket_path(display_num_for_test).unwrap();

        let _listener = StdUnixListener::bind(&socket_path).expect("Failed to create test socket");
        assert!(socket_path.exists());

        let cleanup_result = cleanup_socket_path_from_display_num(display_num_for_test);
        assert!(cleanup_result.is_ok(), "cleanup_socket_path_from_display_num failed: {:?}", cleanup_result.err());
        assert!(!socket_path.exists(), "Socket should be cleaned up");
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_cleanup_socket_path_from_display_num_non_existing() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let display_num_for_test = 9998u32;

        let socket_path = generate_socket_path(display_num_for_test).unwrap();
        assert!(!socket_path.exists());

        let result = cleanup_socket_path_from_display_num(display_num_for_test);
        assert!(result.is_ok());
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

     #[test]
    fn test_cleanup_socket_path_from_display_num_path_is_file() {
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let display_num_for_test = 9997u32;
        let file_path = generate_socket_path(display_num_for_test).unwrap();

        File::create(&file_path).unwrap().write_all(b"not a socket").unwrap();
        assert!(file_path.exists());
        assert!(fs::metadata(&file_path).unwrap().is_file());

        let result = cleanup_socket_path_from_display_num(display_num_for_test);
        assert!(result.is_ok(), "cleanup_socket_path_from_display_num returned error for a non-socket file: {:?}", result.err());
        assert!(file_path.exists(), "Non-socket file should NOT be removed by cleanup_socket_path_from_display_num");

        cleanup_socket_at_path(&file_path).unwrap(); // Use test helper to remove the file
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    // Original tests using 'cleanup_socket' now need to use 'cleanup_socket_at_path'
    #[test]
    fn test_original_cleanup_socket_existing_now_at_path() { // Renamed test for clarity
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-cleanup-test-at-path");

        File::create(&socket_path).unwrap();
        assert!(socket_path.exists());

        let result = cleanup_socket_at_path(&socket_path); // Use new test-specific cleanup
        assert!(result.is_ok());
        assert!(!socket_path.exists());
    }

    #[test]
    fn test_original_cleanup_socket_non_existing_now_at_path() { // Renamed test for clarity
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-cleanup-non-existing-at-path");

        assert!(!socket_path.exists());
        let result = cleanup_socket_at_path(&socket_path); // Use new test-specific cleanup
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_wayland_socket_uses_xdg_or_fallback() {
        // Test with XDG_RUNTIME_DIR set
        let dir = tempdir().unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        let display_num_xdg = 200u32;
        let listener_xdg = init_wayland_socket(display_num_xdg);
        assert!(listener_xdg.is_ok(), "init_wayland_socket failed with XDG_RUNTIME_DIR: {:?}", listener_xdg.err());
        let expected_path_xdg = dir.path().join(format!("wayland-{}", display_num_xdg));
        assert!(expected_path_xdg.exists());
        cleanup_socket_at_path(&expected_path_xdg).unwrap();
        std::env::remove_var("XDG_RUNTIME_DIR");

        // Test with XDG_RUNTIME_DIR NOT set (fallback)
        // Ensure the fallback directory exists and is writable for the test
        let uid = nix::unistd::getuid();
        let fallback_dir_base_str = format!("/run/user/{}", uid);
        let fallback_dir_base_path = PathBuf::from(&fallback_dir_base_str);
        if !fallback_dir_base_path.exists() {
            fs::create_dir_all(&fallback_dir_base_path).expect("Test setup: Failed to create fallback base dir for init test");
            fs::set_permissions(&fallback_dir_base_path, fs::Permissions::from_mode(0o700)).expect("Test setup: Failed to set perms on fallback base dir for init test");
        }

        let display_num_fallback = 201u32;
        let listener_fallback = init_wayland_socket(display_num_fallback);
        assert!(listener_fallback.is_ok(), "init_wayland_socket failed with fallback: {:?}", listener_fallback.err());
        let expected_path_fallback = fallback_dir_base_path.join(format!("wayland-{}", display_num_fallback));
        assert!(expected_path_fallback.exists());
        cleanup_socket_at_path(&expected_path_fallback).unwrap();
        // No need to remove /run/user/{uid} itself
    }
}
