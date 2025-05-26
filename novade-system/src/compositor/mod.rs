//! Smithay-based Wayland Compositor module for NovaDE.
//!
//! This module contains the core logic for the Wayland compositor, including
//! state management, event handling, and interaction with Wayland protocols.

// Declare submodules within the compositor module
pub mod state;
pub mod main; // main.rs contains library functions like run_compositor()

// Re-export key items for easier access from outside the compositor module,
// but within the novade_system crate.
pub use main::run_compositor;
pub use state::NovadeCompositorState;
