// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Renderer Interface
//!
//! This module defines the interfaces for rendering in the compositor.

use std::sync::{Arc, Mutex};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::utils::Rectangle;

use super::{CompositorError, CompositorResult};
use super::surface_management::SurfaceData;

/// Interface for frame rendering
pub trait FrameRenderer: Send + Sync {
    /// Initializes the renderer
    fn initialize(&mut self) -> CompositorResult<()>;
    
    /// Begins a new frame
    fn begin_frame(&mut self) -> CompositorResult<()>;
    
    /// Renders a surface
    fn render_surface(&mut self, surface: &SurfaceData, position: (i32, i32)) -> CompositorResult<()>;
    
    /// Renders a texture
    fn render_texture(&mut self, texture: &dyn RenderableTexture, position: (i32, i32), size: (i32, i32)) -> CompositorResult<()>;
    
    /// Ends the current frame and presents it
    fn end_frame(&mut self) -> CompositorResult<()>;
    
    /// Cleans up resources
    fn cleanup(&mut self) -> CompositorResult<()>;
}

/// Interface for renderable textures
pub trait RenderableTexture: Send + Sync {
    /// Gets the texture size
    fn size(&self) -> (i32, i32);
    
    /// Gets the underlying GLES texture
    fn gles_texture(&self) -> Option<&GlesTexture>;
    
    /// Gets the texture damage regions
    fn damage(&self) -> Vec<Rectangle<i32>>;
}

/// Base implementation of a GLES renderer
pub struct GlesFrameRenderer {
    /// The GLES renderer
    renderer: Arc<Mutex<GlesRenderer>>,
    
    /// The output size
    output_size: (i32, i32),
    
    /// The output scale
    output_scale: f64,
}

impl GlesFrameRenderer {
    /// Creates a new GLES frame renderer
    pub fn new(renderer: Arc<Mutex<GlesRenderer>>, output_size: (i32, i32), output_scale: f64) -> Self {
        Self {
            renderer,
            output_size,
            output_scale,
        }
    }
}

impl FrameRenderer for GlesFrameRenderer {
    fn initialize(&mut self) -> CompositorResult<()> {
        // Implementation would initialize the renderer
        Ok(())
    }
    
    fn begin_frame(&mut self) -> CompositorResult<()> {
        let renderer = self.renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?;
        
        // Implementation would begin a new frame
        Ok(())
    }
    
    fn render_surface(&mut self, surface: &SurfaceData, position: (i32, i32)) -> CompositorResult<()> {
        let renderer = self.renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?;
        
        // Implementation would render the surface
        Ok(())
    }
    
    fn render_texture(&mut self, texture: &dyn RenderableTexture, position: (i32, i32), size: (i32, i32)) -> CompositorResult<()> {
        let renderer = self.renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?;
        
        // Implementation would render the texture
        Ok(())
    }
    
    fn end_frame(&mut self) -> CompositorResult<()> {
        let renderer = self.renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?;
        
        // Implementation would end the frame and present it
        Ok(())
    }
    
    fn cleanup(&mut self) -> CompositorResult<()> {
        // Implementation would clean up resources
        Ok(())
    }
}
