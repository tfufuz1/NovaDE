//! Filesystem Utilities.
//!
//! This module provides helper functions for common filesystem operations,
//! such as ensuring a directory exists and reading file contents to a string.
//! These functions are designed to integrate with the crate's error handling
//! by returning `CoreError`.

use crate::error::CoreError;
use std::fs;
use std::path::Path;

/// Ensures that a directory exists at the given path.
///
/// If the path does not exist, this function will attempt to create it, including
/// any necessary parent directories.
/// If the path already exists but is not a directory, an error is returned.
/// If the path exists and is a directory, the function succeeds.
///
/// # Arguments
///
/// * `path`: A reference to a `Path` representing the directory whose existence should be ensured.
///
/// # Returns
///
/// * `Ok(())` if the directory exists or was successfully created.
/// * `Err(CoreError)` if the path exists but is not a directory, or if directory creation fails.
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use novade_core::utils::fs::ensure_dir_exists;
/// # use tempfile::tempdir;
/// // Create a temporary directory for the example
/// let temp_dir = tempdir().unwrap();
/// let dir_path = temp_dir.path().join("my_app_data");
///
/// match ensure_dir_exists(&dir_path) {
///     Ok(_) => println!("Directory {:?} ensured.", dir_path),
///     Err(e) => eprintln!("Error ensuring directory: {}", e),
/// }
/// assert!(dir_path.exists());
/// assert!(dir_path.is_dir());
/// temp_dir.close().unwrap(); // Clean up
/// ```
pub fn ensure_dir_exists(path: &Path) -> Result<(), CoreError> {
    if path.exists() {
        if !path.is_dir() {
            Err(CoreError::Filesystem {
                message: "Path exists but is not a directory".to_string(),
                path: path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists, // Using AlreadyExists as path is occupied by a non-dir
                    "Path exists but is not a directory",
                ),
            })
        } else {
            Ok(())
        }
    } else {
        fs::create_dir_all(path).map_err(|e| CoreError::Filesystem {
            message: "Failed to create directory".to_string(),
            path: path.to_path_buf(),
            source: e,
        })
    }
}

/// Reads the entire contents of a file into a string.
///
/// This is a convenience wrapper around `std::fs::read_to_string` that maps
/// the `std::io::Error` to `CoreError::Filesystem`.
///
/// # Arguments
///
/// * `path`: A reference to a `Path` representing the file to read.
///
/// # Returns
///
/// * `Ok(String)` containing the file contents if successful.
/// * `Err(CoreError)` if the file cannot be read (e.g., does not exist, permissions error).
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use std::fs::File;
/// # use std::io::Write;
/// # use novade_core::utils::fs::read_to_string;
/// # use tempfile::NamedTempFile;
/// // Create a temporary file for the example
/// let mut temp_file = NamedTempFile::new().unwrap();
/// writeln!(temp_file, "Hello, NovaDE!").unwrap();
/// let file_path = temp_file.path();
///
/// match read_to_string(file_path) {
///     Ok(contents) => println!("File contents: {}", contents),
///     Err(e) => eprintln!("Error reading file: {}", e),
/// }
/// // temp_file is automatically deleted on drop
/// ```
pub fn read_to_string(path: &Path) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|e| CoreError::Filesystem {
        message: "Failed to read file to string".to_string(),
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_ensure_dir_exists_creates_new_directory() {
        let temp_root = tempdir().expect("Failed to create temp root dir for test");
        let new_dir_path = temp_root.path().join("new_dir");

        assert!(!new_dir_path.exists());
        let result = ensure_dir_exists(&new_dir_path);
        assert!(result.is_ok(), "ensure_dir_exists failed: {:?}", result.err());
        assert!(new_dir_path.exists(), "Directory was not created");
        assert!(new_dir_path.is_dir(), "Path created is not a directory");
    }

    #[test]
    fn test_ensure_dir_exists_creates_nested_directories() {
        let temp_root = tempdir().expect("Failed to create temp root dir for test");
        let nested_dir_path = temp_root.path().join("parent_dir/child_dir");

        assert!(!nested_dir_path.exists());
        let result = ensure_dir_exists(&nested_dir_path);
        assert!(result.is_ok(), "ensure_dir_exists failed for nested: {:?}", result.err());
        assert!(nested_dir_path.exists(), "Nested directory was not created");
        assert!(nested_dir_path.is_dir(), "Nested path created is not a directory");
    }

    #[test]
    fn test_ensure_dir_exists_succeeds_if_directory_already_exists() {
        let temp_root = tempdir().expect("Failed to create temp root dir for test");
        let existing_dir_path = temp_root.path().join("existing_dir");

        fs::create_dir(&existing_dir_path).expect("Failed to pre-create directory for test");
        assert!(existing_dir_path.exists() && existing_dir_path.is_dir());

        let result = ensure_dir_exists(&existing_dir_path);
        assert!(result.is_ok(), "ensure_dir_exists failed for existing dir: {:?}", result.err());
    }

    #[test]
    fn test_ensure_dir_exists_errors_if_path_is_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        writeln!(temp_file, "This is a file, not a directory.").unwrap();
        let file_path = temp_file.path().to_path_buf();

        assert!(file_path.exists() && file_path.is_file());

        let result = ensure_dir_exists(&file_path);
        assert!(result.is_err(), "ensure_dir_exists should have failed for a file path");

        match result.err().unwrap() {
            CoreError::Filesystem { message, path, source: _ } => {
                assert_eq!(message, "Path exists but is not a directory");
                assert_eq!(path, file_path);
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }

    // Note: Testing permissions errors for ensure_dir_exists is complex in a unit test environment
    // as it requires setting up specific non-writable paths.

    #[test]
    fn test_read_to_string_success() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file for test");
        let content = "Hello, NovaDE!\nThis is a test file.";
        writeln!(temp_file, "{}", content).unwrap();
        let file_path = temp_file.path();

        let result = read_to_string(file_path);
        assert!(result.is_ok(), "read_to_string failed: {:?}", result.err());
        // writeln! adds a newline, so the expected content should include it.
        let expected_content = format!("{}\n", content);
        assert_eq!(result.unwrap(), expected_content);
    }

    #[test]
    fn test_read_to_string_file_not_found() {
        let temp_root = tempdir().expect("Failed to create temp root dir for test");
        let non_existent_file_path = temp_root.path().join("does_not_exist.txt");

        let result = read_to_string(&non_existent_file_path);
        assert!(result.is_err(), "read_to_string should have failed for non-existent file");

        match result.err().unwrap() {
            CoreError::Filesystem { message, path, source: _ } => {
                assert_eq!(message, "Failed to read file to string");
                assert_eq!(path, non_existent_file_path);
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }

    #[test]
    fn test_read_to_string_on_directory_fails() {
        let temp_root = tempdir().expect("Failed to create temp root dir for test");
        let dir_path = temp_root.path(); // Path to the temp directory itself

        let result = read_to_string(dir_path);
        assert!(result.is_err(), "read_to_string should have failed for a directory");

        match result.err().unwrap() {
            CoreError::Filesystem { message, path, source: _ } => {
                assert_eq!(message, "Failed to read file to string"); // std::fs::read_to_string gives generic error for dirs
                assert_eq!(path, dir_path.to_path_buf());
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
    
    // Note: Testing permissions errors for read_to_string is also complex in unit tests.
    // Such tests are usually better as integration tests with specific filesystem setups.
}
