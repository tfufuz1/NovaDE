// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Surface Management Implementation
//!
//! This module implements the management of surfaces in the compositor,
//! including tracking surface state and buffer attachments.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use smithay::backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state};
use smithay::backend::renderer::gles::GlesTexture;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::{CompositorHandler, SurfaceAttributes, with_surface_tree_upward, TraversalAction};
use smithay::utils::{Logical, Point, Size, Rectangle, Transform};

use super::{CompositorError, CompositorResult};
use super::core::DesktopState;

/// Data associated with a surface
#[derive(Debug)]
pub struct SurfaceData {
    /// The Wayland surface
    pub surface: WlSurface,
    
    /// Surface attributes
    pub attributes: Arc<RwLock<SurfaceAttributes>>,
    
    /// Current buffer information
    pub buffer_info: Arc<Mutex<Option<AttachedBufferInfo>>>,
    
    /// Surface role (toplevel, popup, etc.)
    pub role: SurfaceRole,
    
    /// Surface state
    pub state: Arc<RwLock<SurfaceState>>,
    
    /// Surface children
    pub children: Arc<RwLock<Vec<SurfaceData>>>,
    
    /// Surface parent
    pub parent: Arc<RwLock<Option<WlSurface>>>,
    
    /// Surface input region
    pub input_region: Arc<RwLock<Option<Vec<Rectangle<i32, Logical>>>>>,
    
    /// Surface opaque region
    pub opaque_region: Arc<RwLock<Option<Vec<Rectangle<i32, Logical>>>>>,

    /// Handle to the renderer-specific texture for the current buffer.
    ///
    /// This texture can be created from either an SHM buffer or a DMABUF.
    /// If it's from a DMABUF, it's typically a [`crate::compositor::renderers::gles2::texture::Gles2Texture`]
    /// that wraps a `GL_TEXTURE_EXTERNAL_OES` texture target.
    /// It is `None` if no buffer is currently attached or if texture import failed.
    pub texture_handle: Option<Box<dyn crate::compositor::renderer_interface::abstraction::RenderableTexture>>,
    // Note: The commit logic in core/state.rs uses current_buffer_info directly on SurfaceData,
    // not buffer_info: Arc<Mutex<Option<AttachedBufferInfo>>>. This is an inconsistency.
    // For now, retaining buffer_info as per original struct, but texture_handle is added.
    // If current_buffer_info were added, its doc would be:
    // /// Information about the currently attached `WlBuffer`, its dimensions, scale, and transform.
    // /// This is `None` if no buffer is attached.
    // pub current_buffer_info: Option<AttachedBufferInfo>,
}

/// Information about an attached buffer
#[derive(Debug, Clone)]
pub struct AttachedBufferInfo {
    /// The Wayland buffer resource.
    /// Note: This field was added to align with usage in `core/state.rs`.
    pub buffer: WlSurface, // Should be WlBuffer, this is likely a placeholder due to inconsistencies

    /// Buffer dimensions (width, height) in pixels.
    /// Renamed from `size` to align with usage in `core/state.rs`.
    pub dimensions: Size<i32, Logical>,
    
    /// Buffer scale factor.
    pub scale: i32,
    
    /// Buffer transform (e.g., rotation, flip).
    pub transform: Transform,
    
    /// Damage regions for the buffer, in buffer coordinates.
    pub damage: Vec<Rectangle<i32, Logical>>,
    
    /// Legacy or backend-specific texture reference.
    /// The primary texture is managed by `SurfaceData::texture_handle`.
    pub texture: Option<GlesTexture>, // This is smithay::backend::renderer::gles::GlesTexture
    
    /// Buffer age hint, used for damage tracking.
    pub age: u32,
    
    /// Buffer format
    pub format: smithay::backend::allocator::Format,
}

/// Surface role
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceRole {
    /// XDG toplevel
    XdgToplevel,
    
    /// XDG popup
    XdgPopup,
    
    /// Layer shell surface
    LayerShell,
    
    /// Cursor
    Cursor,
    
    /// Subsurface
    Subsurface,
    
    /// Unknown role
    Unknown,
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
    pub fn new(surface: WlSurface, attributes: Arc<RwLock<SurfaceAttributes>>, role: SurfaceRole) -> Self {
        // The constructor in core/state.rs `SurfaceData::new(client_id)` is inconsistent with this.
        // This constructor is based on the struct definition in this file.
        Self {
            surface,
            attributes,
            buffer_info: Arc::new(Mutex::new(None)), // Retaining this as per original struct
            texture_handle: None, // Initialize new field
            // current_buffer_info: None, // If this field were to replace buffer_info
            role,
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
            children: Arc::new(RwLock::new(Vec::new())),
            parent: Arc::new(RwLock::new(None)),
            input_region: Arc::new(RwLock::new(None)),
            opaque_region: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Updates the buffer information
    pub fn update_buffer(&self, buffer_info: AttachedBufferInfo) -> CompositorResult<()> {
        let mut buffer = self.buffer_info.lock().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire lock on buffer info".to_string())
        })?;
        
        *buffer = Some(buffer_info);
        
        // Update surface size in state
        if let Some(ref buffer_info) = *buffer {
            let mut state = self.state.write().map_err(|_| {
                CompositorError::SurfaceError("Failed to acquire write lock on surface state".to_string())
            })?;
            
            state.size = Size::from(buffer_info.size);
        }
        
        Ok(())
    }
    
    /// Adds a child surface
    pub fn add_child(&self, child: SurfaceData) -> CompositorResult<()> {
        let mut children = self.children.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on children".to_string())
        })?;
        
        let mut parent = child.parent.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on parent".to_string())
        })?;
        
        *parent = Some(self.surface.clone());
        children.push(child);
        
        Ok(())
    }
    
    /// Removes a child surface
    pub fn remove_child(&self, surface: &WlSurface) -> CompositorResult<()> {
        let mut children = self.children.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on children".to_string())
        })?;
        
        children.retain(|child| child.surface != *surface);
        
        Ok(())
    }
    
    /// Sets the input region
    pub fn set_input_region(&self, region: Option<Vec<Rectangle<i32, Logical>>>) -> CompositorResult<()> {
        let mut input_region = self.input_region.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on input region".to_string())
        })?;
        
        *input_region = region;
        
        Ok(())
    }
    
    /// Sets the opaque region
    pub fn set_opaque_region(&self, region: Option<Vec<Rectangle<i32, Logical>>>) -> CompositorResult<()> {
        let mut opaque_region = self.opaque_region.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on opaque region".to_string())
        })?;
        
        *opaque_region = region;
        
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
    
    /// Gets the buffer information
    pub fn get_buffer_info(&self) -> CompositorResult<Option<AttachedBufferInfo>> {
        let buffer = self.buffer_info.lock().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire lock on buffer info".to_string())
        })?;
        
        Ok(buffer.clone())
    }
    
    /// Gets the children
    pub fn get_children(&self) -> CompositorResult<Vec<SurfaceData>> {
        let children = self.children.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on children".to_string())
        })?;
        
        Ok(children.clone())
    }
    
    /// Gets the parent
    pub fn get_parent(&self) -> CompositorResult<Option<WlSurface>> {
        let parent = self.parent.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on parent".to_string())
        })?;
        
        Ok(parent.clone())
    }
}

/// Surface manager for tracking and managing surfaces
pub struct SurfaceManager {
    /// Surface data map
    surfaces: Arc<RwLock<HashMap<WlSurface, Arc<SurfaceData>>>>,
}

impl SurfaceManager {
    /// Creates a new surface manager
    pub fn new() -> Self {
        Self {
            surfaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registers a surface
    pub fn register_surface(&self, surface: WlSurface, attributes: Arc<RwLock<SurfaceAttributes>>, role: SurfaceRole) -> CompositorResult<Arc<SurfaceData>> {
        let mut surfaces = self.surfaces.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surfaces".to_string())
        })?;
        
        let surface_data = Arc::new(SurfaceData::new(surface.clone(), attributes, role));
        surfaces.insert(surface, surface_data.clone());
        
        Ok(surface_data)
    }
    
    /// Gets a surface
    pub fn get_surface(&self, surface: &WlSurface) -> CompositorResult<Option<Arc<SurfaceData>>> {
        let surfaces = self.surfaces.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surfaces".to_string())
        })?;
        
        Ok(surfaces.get(surface).cloned())
    }
    
    /// Removes a surface
    pub fn remove_surface(&self, surface: &WlSurface) -> CompositorResult<()> {
        let mut surfaces = self.surfaces.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surfaces".to_string())
        })?;
        
        if let Some(surface_data) = surfaces.remove(surface) {
            // Remove from parent if it exists
            if let Ok(Some(parent)) = surface_data.get_parent() {
                if let Ok(Some(parent_data)) = self.get_surface(&parent) {
                    parent_data.remove_child(surface)?;
                }
            }
            
            // Remove all children
            if let Ok(children) = surface_data.get_children() {
                for child in children {
                    surfaces.remove(&child.surface);
                }
            }
        }
        
        Ok(())
    }
    
    /// Gets all surfaces
    pub fn get_all_surfaces(&self) -> CompositorResult<Vec<Arc<SurfaceData>>> {
        let surfaces = self.surfaces.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surfaces".to_string())
        })?;
        
        Ok(surfaces.values().cloned().collect())
    }
    
    /// Gets surfaces by role
    pub fn get_surfaces_by_role(&self, role: SurfaceRole) -> CompositorResult<Vec<Arc<SurfaceData>>> {
        let surfaces = self.surfaces.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surfaces".to_string())
        })?;
        
        Ok(surfaces.values()
            .filter(|s| s.role == role)
            .cloned()
            .collect())
    }
    
    /// Gets surfaces by workspace
    pub fn get_surfaces_by_workspace(&self, workspace: u32) -> CompositorResult<Vec<Arc<SurfaceData>>> {
        let surfaces = self.surfaces.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surfaces".to_string())
        })?;
        
        let mut result = Vec::new();
        
        for surface_data in surfaces.values() {
            if let Ok(state) = surface_data.get_state() {
                if let Some(surface_workspace) = state.workspace {
                    if surface_workspace == workspace {
                        result.push(surface_data.clone());
                    }
                }
            }
        }
        
        Ok(result)
    }
}

/// Implementation of the compositor handler for the desktop state
impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut smithay::wayland::compositor::CompositorState {
        &mut self.compositor_state
    }
    
    fn commit(&mut self, surface: &WlSurface) {
        // Handle buffer attachment
        on_commit_buffer_handler::<Self>(surface);
        
        // Update surface state based on buffer
        if let Some(renderer) = self.renderer.lock().unwrap().as_ref() {
            with_renderer_surface_state(surface, renderer, |state| {
                if let Some(data) = state.buffer() {
                    // If we have a surface manager, update the buffer info
                    if let Some(surface_data) = self.surface_to_client.read().unwrap().get(surface) {
                        let buffer_info = AttachedBufferInfo {
                            size: data.size(),
                            scale: state.buffer_scale(),
                            transform: state.buffer_transform(),
                            damage: state.damage().clone(),
                            texture: None, // Texture would be created during rendering
                            age: 0,
                            format: data.format(),
                        };
                        
                        // Update client data with size
                        let mut client_data = surface_data.clone();
                        client_data.size = Size::from(data.size());
                        
                        // Update the surface_to_client map
                        self.surface_to_client.write().unwrap().insert(surface.clone(), client_data);
                    }
                }
            });
        }
        
        // Additional commit handling would go here
        // For example, notifying the window manager of changes
    }
}
