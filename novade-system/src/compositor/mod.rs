//! Smithay-based Wayland Compositor module for NovaDE.
//!
//! This module contains the core logic for the Wayland compositor, including
//! state management, event handling, and interaction with Wayland protocols.

// Declare submodules within the compositor module
pub mod core;
pub mod surface_management;
pub mod shm;
pub mod xdg_shell;
pub mod decoration; // Added this line
// pub mod rendering; // Add when created
// pub mod input_handling; // Add when created
// pub mod window_management; // Add when created
// pub mod xwayland; // Add when created

// Re-export key items if needed, or handle them at a higher level.
// For example, the main DesktopState might be re-exported from core.
pub use self::core::state::DesktopState; // Assuming DesktopState is the main state struct.
                                         // Adjust if NovadeCompositorState is a different, higher-level wrapper.

// If there was a main::run_compositor, its new location or definition needs to be considered.
// For now, focusing on module structure.
// pub use main::run_compositor; // This 'main' submodule seems to be from an older structure.
                               // A proper entry point will be part of a later task.
