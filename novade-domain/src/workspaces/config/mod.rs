// Publicly declare submodules for external use
pub mod types;
pub mod errors;
pub mod provider;

// Re-export key public types from this module
pub use self::types::{WorkspaceSnapshot, WorkspaceSetSnapshot};
pub use self::errors::WorkspaceConfigError;
pub use self::provider::{WorkspaceConfigProvider, FilesystemConfigProvider};
