// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Renderers Module
//!
//! This module provides implementations of renderers for different backends.

mod drm_gbm;
mod winit;

// Re-export public API
pub use drm_gbm::DrmGbmRenderer;
pub use winit::WinitRenderer;

use super::{CompositorError, CompositorResult};
use super::renderer_interface::FrameRenderer;
