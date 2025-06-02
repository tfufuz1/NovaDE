// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Renderer Interface
//!
//! This module defines the primary interfaces for rendering within the compositor.
//! It re-exports the core traits and types from the `abstraction` submodule.

// Declare the abstraction submodule
pub mod abstraction;

// Re-export key rendering abstractions from the abstraction module.
pub use abstraction::{
    FrameRenderer, RenderableTexture, RenderElement, RendererError,
};

// Note: The original, simpler FrameRenderer and RenderableTexture traits,
// and the GlesFrameRenderer struct previously defined in this file (when it was renderer_interface.rs),
// have been removed in favor of the more comprehensive versions
// in abstraction.rs and the Gles2Renderer in crate::compositor::renderers::gles2.
