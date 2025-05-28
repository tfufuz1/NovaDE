pub mod common_types;
pub mod events;
pub mod manager;
pub mod core;
pub mod config;
pub mod assignment;

pub use common_types::*;
pub use events::*;
// Re-export items from manager (as it was before)
pub use manager::{StubWorkspaceManager, WorkspaceManager};

// For now, do not add `pub use` for core, config, and assignment.
// Let's first ensure they are correctly included in the module tree.
// Re-exports can be added later if compilation shows they are needed by other parts of the crate.
