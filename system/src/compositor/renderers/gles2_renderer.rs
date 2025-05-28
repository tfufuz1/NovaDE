use smithay::{
    backend::{
        drm::{DrmDevice, DrmError, DrmNode, DrmSurface},
        egl::{EGLContext, EGLDevice, EGLDisplay, EGLSurface},
        renderer::{
            damage::OutputDamageTracker,
            element::surface::WaylandSurfaceRenderElement,
            gles2::{Gles2Error, Gles2Frame, Gles2Renderer, Gles2Texture},
            Bind, Frame, Renderer, Texture, TextureFilter,
        },
        allocator::dmabuf::Dmabuf,
    },
    reexports::{
        calloop::timer::{Timer, TimerHandle},
        drm::control::crtc,
        wayland_server::protocol::wl_buffer,
    },
    utils::{Buffer, Physical, Rectangle, Size, Transform},
};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::Duration,
};
use uuid::Uuid;

use crate::compositor::renderer_interface::abstraction::{
    FrameRenderer, RenderableTexture, RendererError, RenderElement,
};
use crate::compositor::surface_management::get_surface_data; // For accessing surface texture

// Wrapper for Gles2Texture to implement RenderableTexture
#[derive(Debug)]
pub struct Gles2NovaTexture {
    texture: Gles2Texture,
    id: Uuid,
}

impl Gles2NovaTexture {
    pub fn new(texture: Gles2Texture) -> Self {
        Self {
            texture,
            id: Uuid::new_v4(),
        }
    }
}

impl RenderableTexture for Gles2NovaTexture {
    fn id(&self) -> Uuid {
        self.id
    }

    fn bind(&self, slot: u32) -> Result<(), RendererError> {
        self.texture
            .bind(slot)
            .map_err(|e| RendererError::Generic(e.to_string())) // Gles2Error to RendererError
    }

    fn width_px(&self) -> u32 {
        self.texture.width()
    }

    fn height_px(&self) -> u32 {
        self.texture.height()
    }

    fn format(&self) -> Option<smithay::backend::renderer::utils::Format> {
        self.texture.format()
    }
}

pub struct Gles2NovaRenderer {
    internal_renderer: Gles2Renderer,
    drm_device: DrmDevice,
    drm_surface: DrmSurface,
    egl_display: EGLDisplay,
    egl_context: EGLContext,
    // egl_surface: EGLSurface, // This is usually created per-frame or per-output from DrmSurface
    output_damage_tracker: OutputDamageTracker,
    // We need a way to map wl_buffer to Gles2NovaTexture for cursor, etc.
    // This might be better managed outside if textures are associated with surfaces directly.
    // For now, a simple map for cursors or standalone textures.
    imported_textures: Mutex<HashMap<wl_buffer::WlBuffer, Box<dyn RenderableTexture>>>,
    screen_size: Size<i32, Physical>, // Store the screen size
    drm_node: DrmNode,
}

impl Gles2NovaRenderer {
    pub fn new(
        drm_device: DrmDevice,
        drm_surface: DrmSurface,
        drm_node: DrmNode, // Pass DrmNode
        egl_display: EGLDisplay,
        logger: slog::Logger,
    ) -> Result<Self, RendererError> {
        let egl_context = EGLContext::new(&egl_display, logger.clone())
            .map_err(|e| RendererError::EglError(format!("Failed to create EGL context: {}", e)))?;

        let internal_renderer = unsafe {
            Gles2Renderer::new(egl_context.clone(), logger.clone())
                .map_err(|e| RendererError::ContextCreationFailed(e.to_string()))?
        };
        
        let screen_size = drm_surface.size(); // Get screen size from DrmSurface

        Ok(Self {
            internal_renderer,
            drm_device,
            drm_surface,
            egl_display,
            egl_context,
            output_damage_tracker: OutputDamageTracker::new_legacy(screen_size),
            imported_textures: Mutex::new(HashMap::new()),
            screen_size,
            drm_node,
        })
    }

    fn gles_error_to_renderer_error(err: Gles2Error) -> RendererError {
        match err {
            Gles2Error::ContextLost => RendererError::ContextCreationFailed("EGL Context Lost".to_string()),
            Gles2Error::DisplayLost => RendererError::EglError("EGL Display Lost".to_string()),
            Gles2Error::ShaderCompileError(msg) => RendererError::ShaderCompilationFailed(msg),
            Gles2Error::ShaderLinkError(msg) => RendererError::ShaderCompilationFailed(format!("Link error: {}", msg)),
            Gles2Error::FramebufferBindingError(msg) => RendererError::Generic(format!("Framebuffer binding error: {}", msg)),
            Gles2Error::BufferMappingError(msg) => RendererError::Generic(format!("Buffer mapping error: {}", msg)),
            Gles2Error::UnsupportedBuffer => RendererError::InvalidBufferType("Unsupported buffer".to_string()),
            Gles2Error::EglError(egl_err) => RendererError::EglError(egl_err.to_string()),
            _ => RendererError::Generic(err.to_string()),
        }
    }
}

impl FrameRenderer for Gles2NovaRenderer {
    fn render_frame<'a>(
        &mut self,
        output_geometry: Rectangle<i32, Physical>, // This is the output geometry, not used directly by Gles2Frame::render_output
        output_scale: f64, // Smithay's Gles2Renderer handles scale internally via RenderElement
        elements: impl IntoIterator<Item = &'a (dyn RenderElement<'a> + 'a)>,
    ) -> Result<Vec<Rectangle<i32, Physical>>, RendererError> {
        // Create an EGLSurface for the current rendering operation
        // This assumes self.drm_surface is the active one.
        // For multi-monitor, this needs to be more dynamic.
        let egl_surface = EGLSurface::new(
            &self.egl_display,
            self.drm_surface.pending_framebuffer().map_err(|e| RendererError::DrmError(format!("Failed to get pending framebuffer: {}", e)))?,
            self.egl_context.clone_context(), // Clone for the surface
            slog_scope::logger(),
        )
        .map_err(|e| RendererError::EglError(format!("Failed to create EGL surface: {}", e)))?;

        egl_surface.make_current().map_err(|e| RendererError::EglError(format!("Failed to make EGL surface current: {}", e)))?;

        let mut frame = self
            .internal_renderer
            .render(self.screen_size, Transform::Normal) // Output size and transform
            .map_err(Self::gles_error_to_renderer_error)?;

        frame.clear([0.1, 0.1, 0.1, 1.0], &[output_geometry]) // Clear the target area
            .map_err(Self::gles_error_to_renderer_error)?;

        let mut render_elements: Vec<WaylandSurfaceRenderElement<Gles2Texture>> = Vec::new();

        for element_trait_obj in elements {
            // Downcast or access the underlying Gles2NovaTexture
            // This is a bit tricky with trait objects. One way is to have RenderElement provide a method
            // to get the specific texture type or use an enum dispatcher if you have fixed types.
            // For now, let's assume RenderElement gives us what Gles2Renderer needs.
            // This part needs to align with how SurfaceData provides its texture.

            // Conceptual: this assumes RenderElement can give us a Smithay WaylandSurfaceRenderElement
            // This part will need careful implementation based on how RenderElement is structured
            // and how it interfaces with SurfaceData's texture.
            //
            // let smithay_element = element_trait_obj.as_smithay_element();
            // frame.render_element(smithay_element, ...);

            // More realistically, you'd iterate through your WindowElements or similar,
            // get their SurfaceData, and then get the texture from SurfaceData.
            // The `elements` iterator should probably yield something that gives access to `WlSurface`
            // or `Window` from which we can get `SurfaceData` and then the texture.
            
            // Example: If RenderElement is a wrapper around a WlSurface:
            // if let Some(surface) = element_trait_obj.get_wl_surface() {
            //     let surface_data_guard = get_surface_data(surface).lock().unwrap();
            //     if let Some(texture) = &surface_data_guard.texture {
            //         // This texture is GlesTexture, so we need to wrap it or RenderElement does
            //         let location = element_trait_obj.location(output_scale);
            //         let geometry = element_trait_obj.geometry(output_scale);
            //
            //         // This is where it gets tricky. Smithay's WaylandSurfaceRenderElement
            //         // is usually created from a Window or directly with a GlesTexture.
            //         // We need to ensure our RenderElement trait can provide the necessary inputs
            //         // or be convertible to a type Smithay's renderer understands.
            //
            //         // For now, let's skip the direct rendering part as RenderElement is a placeholder.
            //         // The actual implementation would involve:
            //         // 1. Iterating `elements`.
            //         // 2. For each, getting its `WlSurface`.
            //         // 3. Calling `smithay::desktop::wayland_surface_render_element` or similar
            //         //    to create a `WaylandSurfaceRenderElement`.
            //         // 4. Passing that to `frame.render_element_from_surface_render_element`.
            //     }
            // }
        }
        
        // The damage tracking and rendering would look more like Smithay's examples:
        // self.output_damage_tracker.render_output(
        // &mut self.internal_renderer,
        // 0, // age
        // &render_elements, // These must be Smithay's WaylandSurfaceRenderElement<Gles2Texture>
        // [0.1, 0.1, 0.1, 1.0]
        // ).map_err(Self::gles_error_to_renderer_error)?;


        // For now, returning empty damage. This needs to be calculated based on elements.
        let damage = Vec::new(); // Placeholder
        frame.finish().map_err(|e| RendererError::Generic(format!("Failed to finish frame: {:?}", e)))?;
        Ok(damage)
    }

    fn present_frame(&mut self, _surface_id_to_present_on: Option<u32>) -> Result<(), RendererError> {
        self.drm_surface
            .queue_frame()
            .map_err(|e| RendererError::DrmError(format!("Failed to queue frame: {}", e)))?;
        // Page flip will happen on next vblank if successful
        Ok(())
    }

    fn create_texture_from_shm(
        &mut self,
        buffer: &wl_buffer::WlBuffer,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let gles_texture = self
            .internal_renderer
            .import_shm_buffer(buffer, None, &[]) // No damage initially
            .map_err(|e| {
                if let Gles2Error::UnsupportedBuffer = e {
                    RendererError::InvalidBufferType("Unsupported SHM buffer".to_string())
                } else {
                    Self::gles_error_to_renderer_error(e)
                }
            })?;
        let nova_texture = Gles2NovaTexture::new(gles_texture);
        Ok(Box::new(nova_texture))
    }

    fn create_texture_from_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let gles_texture = self
            .internal_renderer
            .import_dmabuf(dmabuf, None) // No damage initially
            .map_err(|e| {
                if let Gles2Error::UnsupportedBuffer = e {
                     RendererError::InvalidBufferType("Unsupported DMABUF".to_string())
                } else {
                    Self::gles_error_to_renderer_error(e)
                }
            })?;
        let nova_texture = Gles2NovaTexture::new(gles_texture);
        Ok(Box::new(nova_texture))
    }

    fn screen_size(&self) -> Size<i32, Physical> {
        self.screen_size
    }

    fn import_cursor_buffer(
        &mut self,
        buffer: &wl_buffer::WlBuffer,
        _hotspot: (i32, i32), // Hotspot might be used by update_hardware_cursor
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        // Check if buffer is already imported
        let mut textures = self.imported_textures.lock().unwrap();
        if let Some(texture) = textures.get(buffer) {
            // This is tricky because Box<dyn RenderableTexture> is not Clone.
            // For now, we re-import. A better approach might involve Arc<dyn RenderableTexture>
            // or a more robust caching mechanism if these textures are meant to be shared.
        }

        let gles_texture = self
            .internal_renderer
            .import_shm_buffer(buffer, None, &[])
            .map_err(|e| Self::gles_error_to_renderer_error(e))?;
        
        let nova_texture = Gles2NovaTexture::new(gles_texture);
        let boxed_texture: Box<dyn RenderableTexture> = Box::new(nova_texture);
        
        // We cannot clone boxed_texture directly into the map.
        // This caching strategy needs rethinking, perhaps store Arc<Gles2NovaTexture>.
        // textures.insert(buffer.clone(), boxed_texture); // Error: use of moved value

        Ok(boxed_texture) // For now, just return the new texture.
    }

    fn update_hardware_cursor(
        &mut self,
        texture: Option<Box<dyn RenderableTexture>>, // Changed from WlBuffer to our texture
        hotspot: (i32, i32),
    ) -> Result<(), RendererError> {
        if let Some(renderable_texture) = texture {
            // We need to get the underlying Gles2Texture.
            // This requires RenderableTexture to provide a way to get the underlying Gles2Texture,
            // or Gles2NovaRenderer to only accept Gles2NovaTexture.
            // Assuming Gles2NovaTexture for now and downcasting (which is not ideal).
            
            // This downcast is unsafe and not robust.
            // A better way would be for RenderableTexture to have an `as_gles_texture()` method
            // or for this method to take `Option<&Gles2NovaTexture>`.
            // For the purpose of this abstraction, we'll assume we can get it.
            // This part of the abstraction needs refinement.
            
            // Let's assume `renderable_texture` is `Gles2NovaTexture` and we can access its `texture` field.
            // This is not directly possible with `Box<dyn RenderableTexture>`.
            //
            // A common pattern is to have an `as_any()` method on the trait to allow downcasting.
            // trait RenderableTexture: Debug + Send + Sync {
            //     fn as_any(&self) -> &dyn std::any::Any;
            //     ...
            // }
            // impl Gles2NovaTexture { fn as_any(&self) -> &dyn std::any::Any { self } }
            //
            // Then:
            // if let Some(gles_nova_texture) = renderable_texture.as_any().downcast_ref::<Gles2NovaTexture>() {
            //     self.drm_device
            //         .set_cursor_representation(&self.drm_surface, &gles_nova_texture.texture, hotspot)
            //         .map_err(|e| RendererError::DrmError(format!("Failed to set hardware cursor: {}", e)))?;
            // } else {
            //     return Err(RendererError::InvalidBufferType("Cursor texture is not a Gles2NovaTexture".to_string()));
            // }
            // For now, this part is commented out as it depends on RenderableTexture modification.
             Err(RendererError::Generic("Hardware cursor update with RenderableTexture not fully implemented due to downcasting needs.".to_string()))

        } else {
            // Disable hardware cursor
            self.drm_device
                .clear_cursor_representation(&self.drm_surface)
                .map_err(|e| RendererError::DrmError(format!("Failed to clear hardware cursor: {}", e)))
        }
    }
}

// Make Gles2NovaRenderer Send + Sync. GlesRenderer is Send. EGLContext, EGLDisplay might not be.
// Smithay's Gles2Renderer and EGLContext are Send + Sync. DrmDevice/Surface are also Send + Sync.
// So, Gles2NovaRenderer should be Send + Sync.
unsafe impl Send for Gles2NovaRenderer {}
unsafe impl Sync for Gles2NovaRenderer {}
