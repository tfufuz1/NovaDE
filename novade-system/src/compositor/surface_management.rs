// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Surface Management Implementation
//!
//! This module implements the management of surfaces in the compositor,
//! including tracking surface state and buffer attachments.

use std::sync::{Arc, RwLock};
use smithay::reexports::wayland_server::protocol::{wl_surface::WlSurface, wl_buffer::WlBuffer};
use smithay::reexports::wayland_server::Weak;
use smithay::utils::{Logical, Point, Size, Rectangle, Transform, Region};

use super::{CompositorError, CompositorResult};

/// Data associated with a surface
#[derive(Debug)]
pub struct SurfaceData {
    /// Unique identifier for the surface (e.g., client ID)
    pub id: String,

    /// Current buffer information
    pub current_buffer_info: Option<AttachedBufferInfo>,

    /// Surface state
    pub state: Arc<RwLock<SurfaceState>>,

    /// Surface children
    pub children: Vec<Weak<WlSurface>>,

    /// Surface parent
    pub parent: Option<Weak<WlSurface>>,

    /// Damage regions in buffer coordinates
    pub damage_buffer_coords: Vec<Rectangle<i32, Logical>>,

    /// Opaque region in surface-local coordinates
    pub opaque_region_surface_local: Option<Region<Logical>>,

    /// Input region in surface-local coordinates
    pub input_region_surface_local: Option<Region<Logical>>,
    
    /// Handle to the renderer-specific texture for the current buffer.
    pub texture_handle: Option<Box<dyn crate::compositor::renderer_interface::abstraction::RenderableTexture>>,
}

/// Information about an attached buffer
#[derive(Debug, Clone)]
pub struct AttachedBufferInfo {
    /// The Wayland buffer resource.
    pub buffer: WlBuffer,

    /// Buffer dimensions (width, height) in pixels.
    pub dimensions: Size<i32, Logical>,
    
    /// Buffer scale factor.
    pub scale: i32,
    
    /// Buffer transform (e.g., rotation, flip).
    pub transform: Transform,
}

/// Surface state
#[derive(Debug, Clone)]
pub struct SurfaceState {
    /// Is the surface visible
    pub visible: bool,
    
    /// Surface position
    pub position: Point<i32, Logical>,
    
    /// Surface size
    pub size: Size<i32, Logical>,
    
    /// Surface opacity
    pub opacity: f64,
    
    /// Surface z-index
    pub z_index: i32,
    
    /// Surface workspace
    pub workspace: Option<u32>,
    
    /// Surface activation state
    pub activated: bool,
    
    /// Surface fullscreen state
    pub fullscreen: bool,
    
    /// Surface maximized state
    pub maximized: bool,
    
    /// Surface minimized state
    pub minimized: bool,
    
    /// Surface resizing state
    pub resizing: bool,
    
    /// Surface moving state
    pub moving: bool,
}

impl SurfaceData {
    /// Creates a new surface data
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_buffer_info: None,
            texture_handle: None,
            state: Arc::new(RwLock::new(SurfaceState {
                visible: true,
                position: Point::from((0, 0)),
                size: Size::from((0, 0)),
                opacity: 1.0,
                z_index: 0,
                workspace: Some(0),
                activated: false,
                fullscreen: false,
                maximized: false,
                minimized: false,
                resizing: false,
                moving: false,
            })),
            children: Vec::new(),
            parent: None,
            damage_buffer_coords: Vec::new(),
            opaque_region_surface_local: None,
            input_region_surface_local: None,
        }
    }

    /// Updates the buffer information and related state
    pub fn update_buffer(&mut self, buffer_info: AttachedBufferInfo) -> CompositorResult<()> {
        // Update surface size in state based on the new buffer's dimensions
        // Note: The scale factor from the buffer_info might need to be applied here
        // if state.size is expected to be in logical pixels.
        // For now, assuming dimensions are already logical or handled elsewhere.
        let new_size = buffer_info.dimensions;
        self.current_buffer_info = Some(buffer_info);

        let mut state = self.state.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surface state".to_string())
        })?;
        state.size = new_size;
        Ok(())
    }

    /// Sets the input region
    pub fn set_input_region(&mut self, region: Option<Region<Logical>>) -> CompositorResult<()> {
        self.input_region_surface_local = region;
        Ok(())
    }

    /// Sets the opaque region
    pub fn set_opaque_region(&mut self, region: Option<Region<Logical>>) -> CompositorResult<()> {
        self.opaque_region_surface_local = region;
        Ok(())
    }

    /// Updates the surface state
    pub fn update_state<F>(&self, update_fn: F) -> CompositorResult<()>
    where
        F: FnOnce(&mut SurfaceState),
    {
        let mut state = self.state.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surface state".to_string())
        })?;
        
        update_fn(&mut state);
        
        Ok(())
    }
    
    /// Gets the surface state
    pub fn get_state(&self) -> CompositorResult<SurfaceState> {
        let state = self.state.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surface state".to_string())
        })?;
        
        Ok(state.clone())
    }
    
    /// Gets the current buffer information
    pub fn get_buffer_info(&self) -> Option<AttachedBufferInfo> {
        self.current_buffer_info.clone()
    }

    /// Gets the children as a Vec of Weak<WlSurface>
    /// Note: Callers will need to upgrade the Weak pointers to use the WlSurface.
    pub fn get_children(&self) -> Vec<Weak<WlSurface>> {
        self.children.clone()
    }

    /// Gets the parent as an Option<Weak<WlSurface>>
    /// Note: Callers will need to upgrade the Weak pointer to use the WlSurface.
    pub fn get_parent(&self) -> Option<Weak<WlSurface>> {
        self.parent.clone()
    }
}
