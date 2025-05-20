// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Compositor Module
//!
//! This module implements the compositor functionality for the NovaDE desktop environment.
//! It handles window management, surface composition, and rendering through Wayland protocols.
//!
//! ## Architecture
//!
//! The compositor is structured into several submodules:
//! - `core`: Core compositor functionality and state management
//! - `xdg_shell`: XDG shell protocol implementation
//! - `layer_shell`: Layer shell protocol implementation
//! - `renderers`: Rendering backends (DRM/GBM, Winit)
//!
//! ## Thread Safety
//!
//! All compositor components are designed to be thread-safe, using appropriate
//! synchronization primitives like `Arc`, `Mutex`, and `RwLock` where needed.

mod core;
mod xdg_shell;
mod layer_shell;
mod renderers;
mod surface_management;
mod renderer_interface;
mod init;
mod thread_safety;

// Re-export public API
pub use core::{DesktopState, ClientCompositorData};
pub use surface_management::{SurfaceData, AttachedBufferInfo};
pub use renderer_interface::{FrameRenderer, RenderableTexture};
pub use init::initialize_compositor;
pub use thread_safety::run_all_validations;

// Error types
use crate::error::SystemError;

/// Errors that can occur in the compositor module
#[derive(Debug, thiserror::Error)]
pub enum CompositorError {
    /// Error initializing the compositor
    #[error("Failed to initialize compositor: {0}")]
    InitializationError(String),
    
    /// Error in surface management
    #[error("Surface management error: {0}")]
    SurfaceError(String),
    
    /// Error in rendering
    #[error("Rendering error: {0}")]
    RenderError(String),
    
    /// Error in XDG shell protocol
    #[error("XDG shell error: {0}")]
    XdgShellError(String),
    
    /// Error in layer shell protocol
    #[error("Layer shell error: {0}")]
    LayerShellError(String),
    
    /// Thread safety error
    #[error("Thread safety error: {0}")]
    ThreadSafetyError(String),
    
    /// System error
    #[error("System error: {0}")]
    SystemError(#[from] SystemError),
}

/// Result type for compositor operations
pub type CompositorResult<T> = Result<T, CompositorError>;
