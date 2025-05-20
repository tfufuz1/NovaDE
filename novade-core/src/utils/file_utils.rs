//! File utilities for the NovaDE core layer.
//!
//! This module provides file-related utilities used throughout the
//! NovaDE desktop environment.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Ensures that a directory exists, creating it if necessary.
///
/// # Arguments
///
/// * `path` - The directory path
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Path exists but is not a directory: {}", path.display())
        ));
    }
    
    Ok(())
}

/// Reads a file to a string.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// A `Result` containing the file content if successful,
/// or an `io::Error` if reading failed.
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref();
    
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    Ok(content)
}

/// Writes a string to a file.
///
/// # Arguments
///
/// * `path` - The file path
/// * `content` - The content to write
/// * `create_dirs` - Whether to create parent directories if they don't exist
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str, create_dirs: bool) -> io::Result<()> {
    let path = path.as_ref();
    
    if create_dirs {
        if let Some(parent) = path.parent() {
            ensure_directory_exists(parent)?;
        }
    }
    
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}

/// Copies a file from source to destination.
///
/// # Arguments
///
/// * `src` - The source file path
/// * `dst` - The destination file path
/// * `create_dirs` - Whether to create parent directories if they don't exist
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q, create_dirs: bool) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if create_dirs {
        if let Some(parent) = dst.parent() {
            ensure_directory_exists(parent)?;
        }
    }
    
    fs::copy(src, dst)?;
    
    Ok(())
}

/// Gets all files in a directory recursively.
///
/// # Arguments
///
/// * `dir` - The directory path
///
/// # Returns
///
/// A `Result` containing a vector of file paths if successful,
/// or an `io::Error` if reading failed.
pub fn get_all_files<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut files = Vec::new();
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let mut subdir_files = get_all_files(&path)?;
                files.append(&mut subdir_files);
            } else {
                files.push(path);
            }
        }
    }
    
    Ok(files)
}

/// Gets the file extension as a string.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// An `Option` containing the file extension if it exists,
/// or `None` if it doesn't.
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string())
}

/// Gets the file name without extension.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// An `Option` containing the file name without extension if it exists,
/// or `None` if it doesn't.
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
    
    #[test]
    fn test_ensure_directory_exists_new() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        
        assert!(!test_dir.exists());
        
        let result = ensure_directory_exists(&test_dir);
        assert!(result.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }
    
    #[test]
    fn test_ensure_directory_exists_existing() {
        let temp_dir = TempDir::new().unwrap();
        
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
        
        let result = ensure_directory_exists(temp_dir.path());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_ensure_directory_exists_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file");
        
        fs::write(&test_file, "test").unwrap();
        
        assert!(test_file.exists());
        assert!(!test_file.is_dir());
        
        let result = ensure_directory_exists(&test_file);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_read_file_to_string() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file");
        
        let content = "Hello, world!";
        fs::write(&test_file, content).unwrap();
        
        let result = read_file_to_string(&test_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }
    
    #[test]
    fn test_read_file_to_string_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("nonexistent");
        
        let result = read_file_to_string(&test_file);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_write_string_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file");
        
        let content = "Hello, world!";
        let result = write_string_to_file(&test_file, content, false);
        
        assert!(result.is_ok());
        assert!(test_file.exists());
        
        let read_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_write_string_to_file_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("subdir").join("test_file");
        
        assert!(!test_file.parent().unwrap().exists());
        
        let content = "Hello, world!";
        let result = write_string_to_file(&test_file, content, true);
        
        assert!(result.is_ok());
        assert!(test_file.parent().unwrap().exists());
        assert!(test_file.exists());
        
        let read_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("src_file");
        let dst_file = temp_dir.path().join("dst_file");
        
        let content = "Hello, world!";
        fs::write(&src_file, content).unwrap();
        
        let result = copy_file(&src_file, &dst_file, false);
        
        assert!(result.is_ok());
        assert!(dst_file.exists());
        
        let read_content = fs::read_to_string(&dst_file).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_copy_file_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("src_file");
        let dst_file = temp_dir.path().join("subdir").join("dst_file");
        
        let content = "Hello, world!";
        fs::write(&src_file, content).unwrap();
        
        assert!(!dst_file.parent().unwrap().exists());
        
        let result = copy_file(&src_file, &dst_file, true);
        
        assert!(result.is_ok());
        assert!(dst_file.parent().unwrap().exists());
        assert!(dst_file.exists());
        
        let read_content = fs::read_to_string(&dst_file).unwrap();
        assert_eq!(read_content, content);
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
