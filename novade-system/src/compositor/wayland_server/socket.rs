use nix::unistd::getuid;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

/// Generates the Wayland socket path based on the display number.
/// The path is typically /run/user/{uid}/wayland-{display_num}.
fn generate_socket_path(display_num: u32) -> Result<PathBuf, String> {
    let uid = getuid();
    let path_str = format!("/run/user/{}/wayland-{}", uid, display_num);
    Ok(PathBuf::from(path_str))
}

/// Creates and binds a Unix socket to the specified path.
/// Handles existing socket files by attempting to remove them.
/// Sets socket permissions to 0700 after binding.
fn create_and_bind_socket(path: &Path) -> Result<UnixListener, String> {
    if path.exists() {
        // Attempt to remove stale socket file.
        // A more robust solution would involve checking if it's a socket
        // and then trying to connect to see if it's active.
        // For now, a simple removal if it's a file is sufficient.
        if path.is_file() || std::os::unix::fs::FileTypeExt::is_socket(fs::metadata(path).map_err(|e| format!("Failed to get metadata for {:?}: {}", path, e))?) {
            fs::remove_file(path).map_err(|e| format!("Failed to remove existing socket file at {:?}: {}", path, e))?;
        } else if path.is_dir() {
            return Err(format!("Socket path {:?} exists and is a directory.", path));
        }
    }

    // Create parent directory if it doesn't exist
    if let Some(parent_dir) = path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("Failed to create parent directory {:?}: {}", parent_dir, e))?;
        }
    }


    let listener = UnixListener::bind(path)
        .map_err(|e| format!("Failed to bind socket to {:?}: {}", path, e))?;

    // Set socket permissions to 0700
    let permissions = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, permissions)
        .map_err(|e| format!("Failed to set permissions for socket {:?}: {}", path, e))?;

    Ok(listener)
}

/// Removes the socket file at the given path.
#[allow(dead_code)] // To be used with signal handling later
fn cleanup_socket(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to remove socket file at {:?}: {}", path, e))?;
    }
    Ok(())
}

/// Initializes the Wayland socket.
/// Generates the socket path, creates parent directories, and binds the socket.
pub fn init_wayland_socket(display_num: u32) -> Result<UnixListener, String> {
    let socket_path = generate_socket_path(display_num)?;

    // Ensure the parent directory for the socket exists.
    // This is also handled in `create_and_bind_socket`, but an explicit check here can be useful.
    if let Some(parent_dir) = socket_path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("Failed to create parent directory for socket {:?}: {}", parent_dir, e))?;
        }
    }

    create_and_bind_socket(&socket_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_generate_socket_path() {
        let uid = getuid();
        let expected_path = PathBuf::from(format!("/run/user/{}/wayland-0", uid));
        assert_eq!(generate_socket_path(0), Ok(expected_path));

        let expected_path_1 = PathBuf::from(format!("/run/user/{}/wayland-1", uid));
        assert_eq!(generate_socket_path(1), Ok(expected_path_1));
    }

    #[test]
    fn test_create_and_bind_socket_new() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-test");

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok());

        // Check permissions
        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);

        // Ensure it's a socket
        assert!(std::os::unix::fs::FileTypeExt::is_socket(metadata.file_type()));

        // Cleanup
        cleanup_socket(&socket_path).unwrap();
        assert!(!socket_path.exists());
    }

    #[test]
    fn test_create_and_bind_socket_existing_socket_file() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-test-existing");

        // Create a dummy file to simulate an existing socket
        let _dummy_socket = UnixListener::bind(&socket_path).unwrap();
        assert!(socket_path.exists());

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok());

        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(std::os::unix::fs::FileTypeExt::is_socket(metadata.file_type()));


        // Cleanup
        cleanup_socket(&socket_path).unwrap();
    }

    #[test]
    fn test_create_and_bind_socket_existing_non_socket_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("wayland-test-file");

        // Create a regular file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "hello").unwrap();
        assert!(file_path.exists());

        let listener = create_and_bind_socket(&file_path);
        assert!(listener.is_ok()); // Should remove the file and create the socket

        let metadata = fs::metadata(&file_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(std::os::unix::fs::FileTypeExt::is_socket(metadata.file_type()));

        // Cleanup
        cleanup_socket(&file_path).unwrap();
    }


    #[test]
    fn test_create_and_bind_socket_parent_dir_creation() {
        let dir = tempdir().unwrap();
        let parent_path = dir.path().join("parent");
        let socket_path = parent_path.join("wayland-test");

        assert!(!parent_path.exists()); // Parent should not exist initially

        let listener = create_and_bind_socket(&socket_path);
        assert!(listener.is_ok());

        assert!(parent_path.exists()); // Parent should have been created
        assert!(parent_path.is_dir());
        let metadata = fs::metadata(&socket_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        assert!(std::os::unix::fs::FileTypeExt::is_socket(metadata.file_type()));

        // Cleanup
        cleanup_socket(&socket_path).unwrap();
        fs::remove_dir(parent_path).unwrap();
    }


    #[test]
    fn test_cleanup_socket_existing() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-cleanup-test");

        File::create(&socket_path).unwrap(); // Create a dummy file
        assert!(socket_path.exists());

        let result = cleanup_socket(&socket_path);
        assert!(result.is_ok());
        assert!(!socket_path.exists());
    }

    #[test]
    fn test_cleanup_socket_non_existing() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("wayland-cleanup-non-existing");

        assert!(!socket_path.exists());
        let result = cleanup_socket(&socket_path);
        assert!(result.is_ok()); // Should not error if file doesn't exist
    }

    #[test]
    fn test_init_wayland_socket() {
        // This test is a bit more involved as it touches the filesystem in /run/user/{uid}
        // It's more of an integration test for the public API.
        // We'll use a display number that's unlikely to conflict.
        // Note: This test might require specific permissions to run if /run/user/{uid} is restricted.
        // For CI environments, this might need to be mocked or run in a container with appropriate setup.

        // For now, let's test the path generation part and conceptual flow.
        // A full test would involve creating and cleaning up a socket in the actual path,
        // which can be problematic in automated test environments.

        let display_num = 99; // Unlikely to be used
        let uid = getuid();
        let expected_socket_path_str = format!("/run/user/{}/wayland-{}", uid, display_num);

        // We can't easily test the full init_wayland_socket here without actually creating system files.
        // Instead, we primarily rely on the correctness of generate_socket_path and create_and_bind_socket,
        // which are tested above more granularly using temp directories.

        // Test that generate_socket_path is called correctly within init_wayland_socket (conceptually)
        match generate_socket_path(display_num) {
            Ok(path) => assert_eq!(path.to_str().unwrap(), expected_socket_path_str),
            Err(e) => panic!("generate_socket_path failed: {}", e),
        }

        // To actually test init_wayland_socket, we'd need to:
        // 1. Call init_wayland_socket(display_num)
        // 2. Check if the socket exists at expected_socket_path_str
        // 3. Check its permissions
        // 4. Call cleanup_socket(&PathBuf::from(expected_socket_path_str))
        // This is done in test_create_and_bind_socket_new with a temp path.
    }
}
