//! NovaDE Compositor Module
//!
//! This module encapsulates all the logic related to the Wayland compositor,
//! built using the Smithay library. It follows the modular design outlined in
//! ADR-0002.

// Allow dead code and unused imports for now, as the compositor is being built incrementally.
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod state;     // Manages the core compositor state (NovaCompositorState)
pub mod handlers;  // Contains handlers for various Wayland protocols
pub mod core;      // Core initialization, event loop, global management

pub mod render;    // Rendering pipeline and logic
pub mod backends;  // Hardware and display backends (Winit, DRM, etc.)

// TODO: Add other top-level compositor modules as they are developed:
// pub mod input; // More detailed input processing if needed beyond seat handlers
// pub mod window_management; // Advanced window management logic

// Example function to demonstrate usage (will be removed or refactored,
// actual initialization and run logic is in core.rs)
pub fn initial_setup_info() {
    println!("NovaDE Compositor modules declared.");
    println!("Run via core::run_nova_compositor() once implemented.");
}
