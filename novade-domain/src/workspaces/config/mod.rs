// Main module for workspace configuration.

pub mod types;
pub mod errors;
pub mod provider;

// Re-exports for easier access from parent modules (e.g., workspaces module)
pub use types::{WorkspaceSnapshot, WorkspaceSetSnapshot};
pub use errors::WorkspaceConfigError;
pub use provider::{WorkspaceConfigProvider, FilesystemConfigProvider};

#[cfg(test)]
mod tests {
    // Unit tests for types and provider are in their respective files.
    // This file could contain integration tests for the config module if needed,
    // or tests that span across types, errors, and provider logic.
    // For now, assuming individual file tests are sufficient.
    // Adding a dummy test to make sure this file compiles.
    #[test]
    fn config_mod_compiles() {
        assert!(true);
    }
}
