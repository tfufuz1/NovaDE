// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # DRM/GBM Renderer Implementation
//!
//! This module implements a renderer for DRM/GBM backends.

use std::sync::{Arc, Mutex, RwLock};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::drm::{DrmDevice, DrmNode, DrmEvent, DrmEventTime};
use smithay::backend::egl::{EGLDisplay, EGLContext, EGLSurface};
use smithay::backend::gbm::{GbmDevice, GbmBufferObject};
use smithay::backend::allocator::{dmabuf::Dmabuf, Format};
use smithay::backend::drm::compositor::DrmCompositor;
use smithay::utils::{Logical, Physical, Point, Size, Rectangle, Transform};

use super::super::{CompositorError, CompositorResult};
use super::super::renderer_interface::{FrameRenderer, RenderableTexture};
use super::super::surface_management::{SurfaceData, AttachedBufferInfo};

/// DRM/GBM renderer implementation
pub struct DrmGbmRenderer {
    /// The DRM device
    drm: Arc<Mutex<DrmDevice>>,
    
    /// The GBM device
    gbm: Arc<Mutex<GbmDevice<DrmDevice>>>,
    
    /// The EGL display
    egl_display: Arc<Mutex<EGLDisplay>>,
    
    /// The EGL context
    egl_context: Arc<Mutex<EGLContext>>,
    
    /// The GLES renderer
    renderer: Arc<Mutex<GlesRenderer>>,
    
    /// The DRM compositor
    compositor: Arc<Mutex<DrmCompositor>>,
    
    /// The output size
    output_size: Size<i32, Physical>,
    
    /// The output scale
    output_scale: f64,
    
    /// The current frame
    current_frame: Arc<RwLock<Option<FrameData>>>,
    
    /// The pending damage
    pending_damage: Arc<RwLock<Vec<Rectangle<i32, Physical>>>>,
}

/// Frame data
struct FrameData {
    /// The GBM buffer object
    buffer: GbmBufferObject<DrmDevice>,
    
    /// The EGL surface
    surface: EGLSurface,
    
    /// The frame time
    time: DrmEventTime,
}

impl DrmGbmRenderer {
    /// Creates a new DRM/GBM renderer
    pub fn new(
        drm_node: DrmNode,
        output_size: Size<i32, Physical>,
        output_scale: f64,
    ) -> CompositorResult<Self> {
        // Open the DRM device
        let drm = DrmDevice::new(drm_node, true).map_err(|e| {
            CompositorError::RenderError(format!("Failed to open DRM device: {}", e))
        })?;
        
        // Create the GBM device
        let gbm = GbmDevice::new(drm.clone()).map_err(|e| {
            CompositorError::RenderError(format!("Failed to create GBM device: {}", e))
        })?;
        
        // Create the EGL display
        let egl_display = EGLDisplay::new(gbm.clone(), None).map_err(|e| {
            CompositorError::RenderError(format!("Failed to create EGL display: {}", e))
        })?;
        
        // Create the EGL context
        let egl_context = EGLContext::new(&egl_display, None).map_err(|e| {
            CompositorError::RenderError(format!("Failed to create EGL context: {}", e))
        })?;
        
        // Create the GLES renderer
        let renderer = unsafe { GlesRenderer::new(egl_context.clone(), None) }.map_err(|e| {
            CompositorError::RenderError(format!("Failed to create GLES renderer: {}", e))
        })?;
        
        // Create the DRM compositor
        let compositor = DrmCompositor::new(
            &drm,
            &gbm,
            drm.crtc().unwrap(),
            drm.primary_plane().unwrap(),
            None,
            Some(drm.cursor_plane().unwrap()),
            None,
            renderer.clone(),
            Some(output_size),
            None,
        ).map_err(|e| {
            CompositorError::RenderError(format!("Failed to create DRM compositor: {}", e))
        })?;
        
        Ok(Self {
            drm: Arc::new(Mutex::new(drm)),
            gbm: Arc::new(Mutex::new(gbm)),
            egl_display: Arc::new(Mutex::new(egl_display)),
            egl_context: Arc::new(Mutex::new(egl_context)),
            renderer: Arc::new(Mutex::new(renderer)),
            compositor: Arc::new(Mutex::new(compositor)),
            output_size,
            output_scale,
            current_frame: Arc::new(RwLock::new(None)),
            pending_damage: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Handles a DRM event
    pub fn handle_drm_event(&self, event: DrmEvent) -> CompositorResult<()> {
        match event {
            DrmEvent::VBlank(crtc) => {
                // Handle VBlank event
                let mut compositor = self.compositor.lock().map_err(|_| {
                    CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
                })?;
                
                compositor.frame_submitted().map_err(|e| {
                    CompositorError::RenderError(format!("Failed to handle frame submission: {}", e))
                })?;
            }
            DrmEvent::Error(error) => {
                return Err(CompositorError::RenderError(format!("DRM error: {}", error)));
            }
        }
        
        Ok(())
    }
    
    /// Adds damage to the pending damage list
    pub fn add_damage(&self, damage: Rectangle<i32, Physical>) -> CompositorResult<()> {
        let mut pending_damage = self.pending_damage.write().map_err(|_| {
            CompositorError::RenderError("Failed to acquire write lock on pending damage".to_string())
        })?;
        
        pending_damage.push(damage);
        
        Ok(())
    }
    
    /// Clears the pending damage list
    pub fn clear_damage(&self) -> CompositorResult<()> {
        let mut pending_damage = self.pending_damage.write().map_err(|_| {
            CompositorError::RenderError("Failed to acquire write lock on pending damage".to_string())
        })?;
        
        pending_damage.clear();
        
        Ok(())
    }
    
    /// Gets the pending damage
    pub fn get_damage(&self) -> CompositorResult<Vec<Rectangle<i32, Physical>>> {
        let pending_damage = self.pending_damage.read().map_err(|_| {
            CompositorError::RenderError("Failed to acquire read lock on pending damage".to_string())
        })?;
        
        Ok(pending_damage.clone())
    }
    
    /// Creates a texture from a buffer
    pub fn create_texture(&self, buffer: &AttachedBufferInfo) -> CompositorResult<GlesTexture> {
        let renderer = self.renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?;
        
        // Implementation would create a texture from the buffer
        // This is a placeholder
        Err(CompositorError::RenderError("Texture creation not yet implemented".to_string()))
    }
}

impl FrameRenderer for DrmGbmRenderer {
    fn initialize(&mut self) -> CompositorResult<()> {
        // Initialize the renderer
        let mut compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        compositor.use_renderer(|renderer| {
            // Initialize renderer state
            renderer.bind().map_err(|e| {
                CompositorError::RenderError(format!("Failed to bind renderer: {}", e))
            })?;
            
            Ok(())
        }).map_err(|e| {
            CompositorError::RenderError(format!("Failed to use renderer: {}", e))
        })?
    }
    
    fn begin_frame(&mut self) -> CompositorResult<()> {
        let mut compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        // Get the damage
        let damage = self.get_damage()?;
        
        // Begin a new frame
        let res = compositor.render_frame(damage).map_err(|e| {
            CompositorError::RenderError(format!("Failed to render frame: {}", e))
        })?;
        
        // Store the frame data
        let mut current_frame = self.current_frame.write().map_err(|_| {
            CompositorError::RenderError("Failed to acquire write lock on current frame".to_string())
        })?;
        
        *current_frame = Some(FrameData {
            buffer: res.buffer,
            surface: res.surface,
            time: res.time,
        });
        
        // Clear the damage
        self.clear_damage()?;
        
        Ok(())
    }
    
    fn render_surface(&mut self, surface: &SurfaceData, position: (i32, i32)) -> CompositorResult<()> {
        let compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        // Get the buffer info
        let buffer_info = match surface.get_buffer_info()? {
            Some(info) => info,
            None => return Ok(()),
        };
        
        // Get the surface state
        let state = surface.get_state()?;
        
        // Skip if not visible
        if !state.visible {
            return Ok(());
        }
        
        // Create a texture if needed
        let texture = match buffer_info.texture {
            Some(texture) => texture,
            None => self.create_texture(&buffer_info)?,
        };
        
        // Render the surface
        compositor.use_renderer(|renderer| {
            // Calculate the destination rectangle
            let dst = Rectangle::from_loc_and_size(
                Point::from(position),
                state.size,
            );
            
            // Render the texture
            renderer.render_texture_at(
                &texture,
                dst.to_f64(),
                Transform::Normal,
                state.opacity,
            ).map_err(|e| {
                CompositorError::RenderError(format!("Failed to render texture: {}", e))
            })?;
            
            Ok(())
        }).map_err(|e| {
            CompositorError::RenderError(format!("Failed to use renderer: {}", e))
        })?;
        
        // Add damage for the rendered area
        self.add_damage(Rectangle::from_loc_and_size(
            Point::from(position),
            state.size.to_physical(self.output_scale as f32),
        ))?;
        
        Ok(())
    }
    
    fn render_texture(&mut self, texture: &dyn RenderableTexture, position: (i32, i32), size: (i32, i32)) -> CompositorResult<()> {
        let compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        // Get the GLES texture
        let gles_texture = match texture.gles_texture() {
            Some(texture) => texture,
            None => return Ok(()),
        };
        
        // Render the texture
        compositor.use_renderer(|renderer| {
            // Calculate the destination rectangle
            let dst = Rectangle::from_loc_and_size(
                Point::from(position),
                Size::from(size),
            );
            
            // Render the texture
            renderer.render_texture_at(
                gles_texture,
                dst.to_f64(),
                Transform::Normal,
                1.0,
            ).map_err(|e| {
                CompositorError::RenderError(format!("Failed to render texture: {}", e))
            })?;
            
            Ok(())
        }).map_err(|e| {
            CompositorError::RenderError(format!("Failed to use renderer: {}", e))
        })?;
        
        // Add damage for the rendered area
        self.add_damage(Rectangle::from_loc_and_size(
            Point::from(position),
            Size::from(size).to_physical(self.output_scale as f32),
        ))?;
        
        Ok(())
    }
    
    fn end_frame(&mut self) -> CompositorResult<()> {
        let mut compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        // Submit the frame
        compositor.queue_frame().map_err(|e| {
            CompositorError::RenderError(format!("Failed to queue frame: {}", e))
        })?;
        
        Ok(())
    }
    
    fn cleanup(&mut self) -> CompositorResult<()> {
        // Clean up resources
        let mut compositor = self.compositor.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on compositor".to_string())
        })?;
        
        compositor.use_renderer(|renderer| {
            renderer.unbind().map_err(|e| {
                CompositorError::RenderError(format!("Failed to unbind renderer: {}", e))
            })?;
            
            Ok(())
        }).map_err(|e| {
            CompositorError::RenderError(format!("Failed to use renderer: {}", e))
        })?;
        
        Ok(())
    }
}
