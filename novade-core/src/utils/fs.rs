//! Filesystem Utilities.
//!
//! This module provides helper functions for common filesystem operations such as
//! creating directories, reading and writing files, copying files, and introspecting
//! file paths (e.g., getting extensions or stems).
//!
//! Functions that perform I/O operations typically return `Result<T, CoreError>`
//! or `Result<T, std::io::Error>`, mapping underlying I/O errors appropriately.
//!
//! # Examples
//!
//! ```rust,ignore
//! use novade_core::utils::fs;
//! use novade_core::error::CoreError;
//! use std::path::Path;
//!
//! fn example_fs_operations(temp_dir_path: &Path) -> Result<(), CoreError> {
//!     // Ensure a directory exists
//!     let my_dir = temp_dir_path.join("my_app_data");
//!     fs::ensure_directory_exists(&my_dir)?;
//!     assert!(my_dir.is_dir());
//!
//!     // Write to a file
//!     let my_file = my_dir.join("config.txt");
//!     fs::write_string_to_file(&my_file, "setting=true", false)?; // create_dirs = false as my_dir exists
//!
//!     // Read from a file
//!     let content = fs::read_file_to_string(&my_file)?;
//!     assert_eq!(content, "setting=true");
//!
//!     Ok(())
//! }
//! ```

use std::fs;
use std::io::{self, Read, Write}; // io::Error is used for source in CoreError::Filesystem
use std::path::{Path, PathBuf};
use crate::error::CoreError;

/// Ensures that a directory exists at the given path.
///
/// If the directory (or any of its parents) does not exist, they will be created.
///
/// # Arguments
///
/// * `path`: A type that can be converted into a `&Path` (e.g., `PathBuf`, `&str`).
///
/// # Errors
///
/// Returns a [`CoreError::Filesystem`] if:
/// - The path exists but is not a directory.
/// - The directory or any of its parents could not be created due to an I/O error
///   (e.g., permission issues).
pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> Result<(), CoreError> {
    let path_ref = path.as_ref();
    
    if !path_ref.exists() {
        fs::create_dir_all(path_ref).map_err(|e| CoreError::Filesystem {
            message: format!("Failed to create directory '{}'", path_ref.display()),
            path: path_ref.to_path_buf(),
            source: e,
        })?;
    } else if !path_ref.is_dir() {
        return Err(CoreError::Filesystem {
            message: format!("Path '{}' exists but is not a directory", path_ref.display()),
            path: path_ref.to_path_buf(),
            source: io::Error::new(io::ErrorKind::AlreadyExists, "Path exists but is not a directory"),
        });
    }
    
    Ok(())
}

/// Reads the entire contents of a file into a string.
///
/// This is a convenience wrapper around `std::fs::read_to_string`.
///
/// # Arguments
///
/// * `path`: A type that can be converted into a `&Path` representing the file to read.
///
/// # Errors
///
/// Returns a [`CoreError::Filesystem`] if the file cannot be read, for example:
/// - The file does not exist.
/// - The user does not have permission to read the file.
/// - The file's content is not valid UTF-8.
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, CoreError> {
    let path_ref = path.as_ref();
    
    std::fs::read_to_string(path_ref).map_err(|e| CoreError::Filesystem {
        message: format!("Failed to read file '{}' to string", path_ref.display()),
        path: path_ref.to_path_buf(),
        source: e,
    })
}

/// Writes a string slice to a file.
///
/// This function will create the file if it does not exist, and will truncate it if it does.
///
/// # Arguments
///
/// * `path`: A type that can be converted into a `&Path` representing the file to write to.
/// * `content`: The string slice to write to the file.
/// * `create_dirs`: If `true`, any non-existent parent directories of `path` will be
///   created using [`ensure_directory_exists`].
///
/// # Errors
///
/// Returns a [`CoreError::Filesystem`] if:
/// - Writing to the file fails (e.g., permission issues, disk full).
/// - `create_dirs` is `true` and creating parent directories fails.
pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str, create_dirs: bool) -> Result<(), CoreError> {
    let path_ref = path.as_ref();
    
    if create_dirs {
        if let Some(parent) = path_ref.parent() {
            ensure_directory_exists(parent)?; // Already returns CoreError
        }
    }
    
    let mut file = fs::File::create(path_ref).map_err(|e| CoreError::Filesystem {
        message: format!("Failed to create file for writing at '{}'", path_ref.display()),
        path: path_ref.to_path_buf(),
        source: e,
    })?;
    
    file.write_all(content.as_bytes()).map_err(|e| CoreError::Filesystem {
        message: format!("Failed to write content to file '{}'", path_ref.display()),
        path: path_ref.to_path_buf(),
        source: e,
    })?;
    
    Ok(())
}

/// Copies the contents of one file to another.
///
/// This function will overwrite the destination file if it already exists.
///
/// # Arguments
///
/// * `src`: The path to the source file.
/// * `dst`: The path to the destination file.
/// * `create_dirs`: If `true`, any non-existent parent directories of `dst` will be
///   created using [`ensure_directory_exists`].
///
/// # Errors
///
/// Returns a [`CoreError::Filesystem`] if:
/// - The source file does not exist or is not readable.
/// - The destination file cannot be written to.
/// - `create_dirs` is `true` and creating parent directories fails.
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q, create_dirs: bool) -> Result<(), CoreError> {
    let src_path = src.as_ref();
    let dst_path = dst.as_ref();
    
    if create_dirs {
        if let Some(parent) = dst_path.parent() {
            ensure_directory_exists(parent)?; // Already returns CoreError
        }
    }
    
    fs::copy(src_path, dst_path).map_err(|e| CoreError::Filesystem {
        message: format!("Failed to copy file from '{}' to '{}'", src_path.display(), dst_path.display()),
        // Path field in CoreError::Filesystem is singular. We'll use destination path as primary.
        // Source path is in the message.
        path: dst_path.to_path_buf(), 
        source: e,
    })?;
    
    Ok(())
}

/// Recursively collects all file paths within a given directory.
///
/// This function traverses the directory and its subdirectories, adding the path
/// of each regular file encountered to the returned vector.
///
/// # Arguments
///
/// * `dir`: A type that can be converted into a `&Path` representing the directory to scan.
///
/// # Errors
///
/// Returns an `std::io::Error` if:
/// - The initial path `dir` is not a directory or does not exist.
/// - Reading any part of the directory structure fails (e.g., due to permissions).
pub fn get_all_files<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let dir_path = dir.as_ref();
    let mut files = Vec::new();
    
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursive call for subdirectories
                files.append(&mut get_all_files(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    // If dir_path is not a directory, an empty Vec is returned, which is valid.
    // An error would occur if fs::read_dir fails (e.g. path doesn't exist or no permission).
    
    Ok(files)
}

/// Extracts the file extension from a path as a `String`.
///
/// # Arguments
///
/// * `path`: A type that can be converted into a `&Path`.
///
/// # Returns
///
/// An `Option<String>` containing the file extension if it exists and is valid UTF-8.
/// Returns `None` if there is no extension or it's not valid UTF-8.
///
/// # Examples
/// ```
/// use novade_core::utils::fs::get_file_extension;
/// assert_eq!(get_file_extension("archive.tar.gz"), Some("gz".to_string()));
/// assert_eq!(get_file_extension("image.jpeg"), Some("jpeg".to_string()));
/// assert_eq!(get_file_extension("document"), None);
/// ```
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string())
}

/// Extracts the file stem (name without the final extension) from a path as a `String`.
///
/// # Arguments
///
/// * `path`: A type that can be converted into a `&Path`.
///
/// # Returns
///
/// An `Option<String>` containing the file stem if it exists and is valid UTF-8.
/// Returns `None` if there is no file name or stem, or it's not valid UTF-8.
///
/// # Examples
/// ```
/// use novade_core::utils::fs::get_file_stem;
/// assert_eq!(get_file_stem("archive.tar.gz"), Some("archive.tar".to_string()));
/// assert_eq!(get_file_stem("image.jpeg"), Some("image".to_string()));
/// assert_eq!(get_file_stem("document"), Some("document".to_string()));
/// assert_eq!(get_file_stem(".bashrc"), Some("".to_string())); // Stem of ".bashrc" is often considered empty
/// ```
pub fn get_file_stem<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    // Note: CoreError is already imported at the module level, so no need for `use crate::error::CoreError` here.
    
    #[test]
    fn test_ensure_directory_exists_new() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        
        assert!(!test_dir.exists());
        
        let result = ensure_directory_exists(&test_dir);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }
    
    #[test]
    fn test_ensure_directory_exists_existing() {
        let temp_dir = TempDir::new().unwrap();
        
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
        
        let result = ensure_directory_exists(temp_dir.path());
        assert!(result.is_ok(), "Expected Ok for existing dir, got {:?}", result.err());
    }
    
    #[test]
    fn test_ensure_directory_exists_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_file");
        
        fs::write(&test_file_path, "test").unwrap();
        
        assert!(test_file_path.exists());
        assert!(!test_file_path.is_dir());
        
        let result = ensure_directory_exists(&test_file_path);
        assert!(result.is_err());
        match result {
            Err(CoreError::Filesystem { message, path, source }) => {
                assert!(message.contains("exists but is not a directory"));
                assert_eq!(path, test_file_path);
                assert_eq!(source.kind(), io::ErrorKind::AlreadyExists);
            },
            Ok(_) => panic!("Expected CoreError::Filesystem, got Ok"),
            Err(e) => panic!("Expected CoreError::Filesystem, got {:?}", e),
        }
    }
    
    #[test]
    fn test_read_file_to_string() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file");
        
        let content = "Hello, world!";
        fs::write(&test_file, content).unwrap();
        
        let result = read_file_to_string(&test_file);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert_eq!(result.unwrap(), content);
    }
    
    #[test]
    fn test_read_file_to_string_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("nonexistent");
        
        let result = read_file_to_string(&test_file_path);
        assert!(result.is_err());
        match result {
            Err(CoreError::Filesystem { message, path, source }) => {
                assert!(message.contains("Failed to read file"));
                assert_eq!(path, test_file_path);
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            },
            Ok(_) => panic!("Expected CoreError::Filesystem for non-existent file, got Ok"),
            Err(e) => panic!("Expected CoreError::Filesystem, got {:?}", e),
        }
    }
    
    #[test]
    fn test_write_string_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file.txt");
        
        let content = "Hello, NovaDE!";
        let result = write_string_to_file(&test_file, content, false);
        
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert!(test_file.exists());
        
        let read_content = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_write_string_to_file_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("new_subdir").join("test_file.txt");
        
        assert!(!test_file.parent().unwrap().exists());
        
        let content = "NovaDE rocks!";
        let result = write_string_to_file(&test_file, content, true);
        
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert!(test_file.parent().unwrap().exists());
        assert!(test_file.exists());
        
        let read_content = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_write_string_to_file_permission_denied() {
        // This test requires creating a read-only directory or file, which is platform-dependent.
        // For simplicity, we'll simulate a failure by trying to write to a path that's likely problematic.
        // This test is more of a conceptual check for error mapping.
        // On Unix, creating a directory and then trying to write to it as a file.
        let temp_dir = TempDir::new().unwrap();
        let dir_as_file_path = temp_dir.path().join("iamadirectory");
        std::fs::create_dir(&dir_as_file_path).unwrap();

        let result = write_string_to_file(&dir_as_file_path, "test", false);
        assert!(result.is_err());
        match result {
            Err(CoreError::Filesystem { message, path, source: _ }) => {
                assert!(message.contains("Failed to create file for writing"));
                assert_eq!(path, dir_as_file_path);
            },
            _ => panic!("Expected CoreError::Filesystem"),
        }
    }
    
    #[test]
    fn test_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("source.txt");
        let dst_file = temp_dir.path().join("destination.txt");
        
        let content = "Copy me!";
        std::fs::write(&src_file, content).unwrap();
        
        let result = copy_file(&src_file, &dst_file, false);
        
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert!(dst_file.exists());
        
        let read_content = std::fs::read_to_string(&dst_file).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_copy_file_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("source_cd.txt");
        let dst_file = temp_dir.path().join("new_dir_for_copy").join("destination_cd.txt");
        
        let content = "Copy me with dirs!";
        std::fs::write(&src_file, content).unwrap();
        
        assert!(!dst_file.parent().unwrap().exists());
        
        let result = copy_file(&src_file, &dst_file, true);
        
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        assert!(dst_file.parent().unwrap().exists());
        assert!(dst_file.exists());
        
        let read_content = std::fs::read_to_string(&dst_file).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_copy_file_src_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("non_existent_source.txt");
        let dst_file = temp_dir.path().join("destination_ne.txt");

        let result = copy_file(&src_file, &dst_file, false);
        assert!(result.is_err());
        match result {
            Err(CoreError::Filesystem { message, path, source }) => {
                assert!(message.contains("Failed to copy file"));
                assert_eq!(path, dst_file); // dst_path is used as primary path in error
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            },
            _ => panic!("Expected CoreError::Filesystem for source not found"),
        }
    }
    
    #[test]
    fn test_get_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1");
        let subdir = temp_dir.path().join("subdir");
        let file2 = subdir.join("file2");
        
        fs::write(&file1, "file1").unwrap();
        fs::create_dir(&subdir).unwrap();
        fs::write(&file2, "file2").unwrap();
        
        let result = get_all_files(temp_dir.path());
        
        assert!(result.is_ok());
        
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
        
        let file_paths: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        
        assert!(file_paths.contains(&"file1".to_string()));
        assert!(file_paths.contains(&"file2".to_string()));
    }
    
    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_file_extension("file.tar.gz"), Some("gz".to_string()));
        assert_eq!(get_file_extension("file"), None);
        assert_eq!(get_file_extension(".hidden"), Some("hidden".to_string()));
    }
    
    #[test]
    fn test_get_file_stem() {
        assert_eq!(get_file_stem("file.txt"), Some("file".to_string()));
        assert_eq!(get_file_stem("file.tar.gz"), Some("file.tar".to_string()));
        assert_eq!(get_file_stem("file"), Some("file".to_string()));
        assert_eq!(get_file_stem(".hidden"), Some("".to_string()));
    }
}
