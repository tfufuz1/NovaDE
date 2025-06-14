// novade-system/src/filesystem_service.rs
//! Provides mediated access to the file system for specific, approved tasks.
use crate::error::SystemError;

/// Represents basic information about a file or directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileInfo {
    pub path: String,        // Full path to the file/directory
    pub name: String,        // Base name of the file/directory
    pub is_dir: bool,        // True if it's a directory, false if a file
    pub size_bytes: Option<u64>, // Size in bytes, if applicable (for files)
    // pub last_modified: Option<SystemTime>, // Or a string representation
    // pub mime_type: Option<String>,
}

/// Represents the user context for permission checking.
/// This needs to be defined based on how user identity and permissions are managed in NovaDE.
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String, // Example: "user123"
    // pub groups: Vec<String>, // Example: ["users", "audio"]
    // pub effective_permissions: u32, // Example bitmask or similar
}

pub trait FileSystemService: Send + Sync {
    /// Searches for files and directories based on criteria.
    ///
    /// This method should respect user permissions and privacy. It might interact with
    /// a file indexing service (like Tracker or Baloo) for efficiency and to avoid
    /// direct, broad filesystem crawls.
    ///
    /// # Arguments
    /// * `query` - A search query string. The exact syntax will depend on the
    ///           underlying search provider (e.g., "name:report type:pdf dir:~/Documents").
    /// * `user_context` - The context of the user initiating the search, for permission checks.
    ///
    /// # Returns
    /// A vector of `FileInfo` matching the query, or an error.
    fn search_files(&self, query: &str, user_context: &UserContext) -> Result<Vec<FileInfo>, SystemError>;

    // /// Reads a limited portion of a file's content, primarily for text files.
    // /// This should be used with extreme caution due to security and privacy implications.
    // /// Access control must be strictly enforced.
    // ///
    // /// # Arguments
    // /// * `path` - The full path to the file.
    // /// * `user_context` - The user context for permission checks.
    // /// * `max_bytes` - Maximum number of bytes to read (e.g., for a preview).
    // ///
    // /// # Returns
    // /// The file content as a String, or an error.
    // fn read_file_preview(&self, path: &str, user_context: &UserContext, max_bytes: u64) -> Result<String, SystemError>;

    // TODO: Consider methods for:
    // - Getting metadata for a specific path.
    // - Basic file operations if absolutely necessary and securely mediated (e.g., copy to a specific, safe location).
    //   However, direct file manipulation by the assistant is generally discouraged.
}

// TODO: Assistant Integration: Skills like "Find my presentation from last week" or
// "Search for 'budget.xlsx' in my financial documents" would use this service.
// TODO: This is a highly sensitive area. Permissions, security, and user privacy are paramount.
// The implementation must ensure that the assistant cannot access arbitrary files or
// perform unauthorized actions. Interaction with a dedicated, permission-aware file
// indexing and search service is strongly recommended over direct filesystem access.
// TODO: Define SystemError variants for filesystem specific errors (e.g., AccessDenied, SearchFailed, IndexerUnavailable).
