// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Renderers Module
//!
//! This module provides implementations of renderers for different backends.

mod drm_gbm;
mod winit;
pub mod egl_context;
pub mod egl_surface; // Added egl_surface module
pub mod shader; // Added shader module
pub mod geometry; // Added geometry module
pub mod texture; // Added texture module
pub mod framebuffer; // Added framebuffer module
pub mod client_buffer; // Added client_buffer module
pub mod gles2; // Added gles2 module
pub mod vulkan; // Added vulkan module

// Re-export public API
pub use drm_gbm::DrmGbmRenderer;
pub use winit::WinitRenderer;

use super::{CompositorError, CompositorResult};
use super::renderer_interface::FrameRenderer;
