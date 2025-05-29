//! XDG Shell protocol integration for the NovaDE compositor.
//!
//! This module provides the necessary structures and handlers for supporting
//! XDG toplevel and popup surfaces, making them manageable as windows within
//! the compositor's desktop space.

pub mod errors;
pub mod types;
pub mod handlers; // Added for XdgShellHandler implementations
// pub mod state;    // To be added in a future task for XdgShellState related data (like NovaXdgShellState or surface-specific user data)

// Re-export key types for convenience
pub use errors::XdgShellError;
pub use types::ManagedWindow;
// Example for future re-exports if custom surface data types are defined:
// pub use state::{NovaXdgToplevelSurfaceData, NovaXdgPopupSurfaceData};
