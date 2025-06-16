use glow::{Context, HasContext, Program, Shader, Texture, UniformLocation};
use khronos_egl as egl; // Assuming khronos-egl is the chosen EGL crate
use smithay::{
    backend::{
        allocator::dmabuf::{Dmabuf, DmabufFlags}, // Added DmabufFlags
        renderer::utils::{Fourcc, format_shm_to_fourcc},
    },
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    utils::{Buffer as SmithayBufferCoord, Logical, Physical, Point, Rectangle, Size, Transform},
};
use std::sync::Arc; // <<<< CHANGED FROM std::rc::Rc
use uuid::Uuid;

// New imports for CompositorRenderer
use crate::compositor::render::renderer::{CompositorRenderer, RenderableTexture as NewRenderableTexture};
use crate::compositor::core::state::DesktopState;
use crate::compositor::render::dmabuf_importer::DmabufImporter;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::output::Output;
use smithay::backend::allocator::dmabuf::Dmabuf;
use crate::compositor::core::errors::CompositorError;

// Old imports
use crate::compositor::renderer_interface::abstraction::{
    FrameRenderer as OldFrameRenderer, RenderableTexture as OldRenderableTexture, RenderElement as OldRenderElement, RendererError as OldRendererError,
};
use super::errors::Gles2RendererError; 
use super::shaders::{
    compile_shader, link_program, FS_SOLID_SRC, FS_TEXTURE_SRC, VS_SOLID_SRC, VS_TEXTURE_SRC,
};
use super::texture::Gles2Texture;

// Minimal matrix math for MVP (orthographic projection)
// For a proper solution, a math library like `cgmath` or `nalgebra` would be used.
#[rustfmt::skip]
fn orthographic_projection(width: f32, height: f32) -> [f32; 9] {
    // Creates a 2D orthographic projection matrix:
    // Maps (0,0)-(width,height) to (-1,-1)-(1,1) clip space, with Y flipped.
    // [ 2/w   0    -1 ]
    // [ 0    -2/h   1 ]  (Y is flipped: positive Y in screen space is downwards)
    // [ 0     0     1 ]
    [
        2.0 / width, 0.0,           0.0, // Column 1
        0.0,        -2.0 / height,  0.0, // Column 2 (Y is flipped)
        -1.0,        1.0,           1.0, // Column 3 (Translation)
    ]
}


pub struct Gles2Renderer {
    internal_id: Uuid,
    gl: Arc<Context>, // <<<< CHANGED FROM Rc
    egl_display: egl::Display,
    egl_context: egl::Context,
    // Store the EGL instance to ensure it outlives the display and context.
    // This is important as per khronos-egl documentation.
    _egl_instance: Arc<egl::Instance<egl::Dynamic<libloading::Library>>>,  // <<<< CHANGED FROM Rc
    egl_surface: Option<egl::Surface>, 
    
    texture_shader_program: Program,
    texture_shader_mvp_loc: UniformLocation, // Not Option, expect it to be found
    texture_shader_sampler_loc: UniformLocation,
    
    solid_shader_program: Program,
    solid_shader_mvp_loc: UniformLocation,
    solid_shader_color_loc: UniformLocation,
    
    current_screen_size: Size<i32, Physical>,
    // Quad VBO and EBO can be added here for efficiency
    // quad_vbo: Option<glow::Buffer>,
    // quad_ebo: Option<glow::Buffer>,
}

impl Gles2Renderer {
    pub fn new(
        gl: Context, // Take raw glow::Context
        egl_display: egl::Display,
        egl_context: egl::Context,
        egl_instance: Arc<egl::Instance<egl::Dynamic<libloading::Library>>>, // <<<< CHANGED FROM Rc
        initial_screen_size: Size<i32, Physical>,
        egl_surface: Option<egl::Surface>,
    ) -> Result<Self, Gles2RendererError> {
        let gl_arc = Arc::new(gl); // <<<< CHANGED FROM gl_rc = Rc::new(gl)

        // Compile and link texture shader
        let vs_tex = compile_shader(&gl_arc, glow::VERTEX_SHADER, VS_TEXTURE_SRC)?;
        let fs_tex = compile_shader(&gl_arc, glow::FRAGMENT_SHADER, FS_TEXTURE_SRC)?;
        let texture_shader_program = link_program(&gl_arc, vs_tex, fs_tex)?;
        unsafe { // Shaders can be deleted after linking
            gl_arc.detach_shader(texture_shader_program, vs_tex);
            gl_arc.detach_shader(texture_shader_program, fs_tex);
            gl_arc.delete_shader(vs_tex);
            gl_arc.delete_shader(fs_tex);
        }
        let texture_shader_mvp_loc = unsafe { gl_arc.get_uniform_location(texture_shader_program, "mvp") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("mvp in texture_shader".to_string()))?;
        let texture_shader_sampler_loc = unsafe { gl_arc.get_uniform_location(texture_shader_program, "u_texture") }
             .ok_or_else(|| Gles2RendererError::UniformNotFound("u_texture in texture_shader".to_string()))?;

        // Compile and link solid color shader
        let vs_solid = compile_shader(&gl_arc, glow::VERTEX_SHADER, VS_SOLID_SRC)?;
        let fs_solid = compile_shader(&gl_arc, glow::FRAGMENT_SHADER, FS_SOLID_SRC)?;
        let solid_shader_program = link_program(&gl_arc, vs_solid, fs_solid)?;
        unsafe { // Shaders can be deleted after linking
            gl_arc.detach_shader(solid_shader_program, vs_solid);
            gl_arc.detach_shader(solid_shader_program, fs_solid);
            gl_arc.delete_shader(vs_solid);
            gl_arc.delete_shader(fs_solid);
        }
        let solid_shader_mvp_loc = unsafe { gl_arc.get_uniform_location(solid_shader_program, "mvp") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("mvp in solid_shader".to_string()))?;
        let solid_shader_color_loc = unsafe { gl_arc.get_uniform_location(solid_shader_program, "u_color") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("u_color in solid_shader".to_string()))?;

        Ok(Self {
            internal_id: Uuid::new_v4(),
            gl: gl_arc, // <<<< CHANGED FROM gl_rc
            egl_display,
            egl_context,
            _egl_instance: egl_instance,
            egl_surface,
            texture_shader_program,
            texture_shader_mvp_loc,
            texture_shader_sampler_loc,
            solid_shader_program,
            solid_shader_mvp_loc,
            solid_shader_color_loc,
            current_screen_size: initial_screen_size,
        })
    }

    // Helper for rendering a quad (simplified for now)
    // This would typically involve binding a VBO and setting up vertex attribute pointers.
    // For this plan, we'll keep it minimal.
    fn render_quad_internal(&self, _geometry: Rectangle<i32, Logical>, _is_texture: bool) {
        // In a real renderer, this would involve:
        // 1. A VBO with unit quad vertices: e.g., (0,0), (1,0), (0,1), (1,1) for position
        //    and texture coordinates.
        // 2. Binding the VBO.
        // 3. Setting up vertex_attrib_pointer for 'position' and (if is_texture) 'tex_coord'.
        // 4. Calculating the model matrix to scale the unit quad to `geometry.size`
        //    and translate it to `geometry.loc`.
        // 5. Multiplying model matrix with projection to get MVP.
        // 6. Uploading MVP to shader.
        // 7. gl.draw_arrays or gl.draw_elements.

        // Simplified: Assume vertices are already set up or this is a placeholder for such logic.
        // The actual vertex data and MVP matrix calculation for each element will be
        // handled within `render_frame` for now.
        // This function's role is just to call draw_arrays.
        unsafe {
            // Example of what would be needed if sending raw vertex data (less efficient)
            // This is conceptual. Actual vertex data setup is complex.
            let (pos_attr_loc, tex_coord_attr_loc) = if is_texture {
                (
                    self.gl.get_attrib_location(self.texture_shader_program, "position").unwrap_or(0),
                    self.gl.get_attrib_location(self.texture_shader_program, "tex_coord").unwrap_or(1)
                )
            } else {
                (
                    self.gl.get_attrib_location(self.solid_shader_program, "position").unwrap_or(0),
                    0 // No tex_coord for solid
                )
            };

            // Vertices for a quad at (x, y) with size (w, h)
            // Vertex order: Top-left, Top-right, Bottom-left, Bottom-right (for TRIANGLE_STRIP)
            // Y is often flipped in texture coordinates depending on source.
            // Smithay surfaces typically have (0,0) at top-left.
            let x1 = geometry.loc.x as f32;
            let y1 = geometry.loc.y as f32;
            let x2 = (geometry.loc.x + geometry.size.w) as f32;
            let y2 = (geometry.loc.y + geometry.size.h) as f32;

            // Position vertices (for TRIANGLE_STRIP: TL, TR, BL, BR or TL, BL, TR, BR)
            // Let's use TL, BL, TR, BR for strip (or two triangles)
            // Triangle 1: TL, BL, TR
            // Triangle 2: TR, BL, BR (if using GL_TRIANGLES)
            // For TRIANGLE_STRIP: (TL, BL, TR, BR) -> (TL-BL-TR), (TR-BL-BR) -> (TL,BL,TR), (BL,TR,BR)
            // Vertices: (x, y)
            // TL: x1, y1
            // BL: x1, y2
            // TR: x2, y1
            // BR: x2, y2
            let vertices_pos: [f32; 8] = [
                x1, y1, // Top-left
                x1, y2, // Bottom-left
                x2, y1, // Top-right
                x2, y2, // Bottom-right
            ];

            // Texture coordinates (Y might be flipped: 0,1 for top-left if texture origin is bottom-left)
            // Standard: (0,0) top-left, (1,1) bottom-right
            let vertices_tex: [f32; 8] = [
                0.0, 0.0, // Top-left
                0.0, 1.0, // Bottom-left
                1.0, 0.0, // Top-right
                1.0, 1.0, // Bottom-right
            ];
            
            // This is NOT how you typically render with Glow efficiently.
            // This is a placeholder for VBO-based rendering.
            // It attempts to use client-side arrays, which is slow and deprecated in modern GL.
            // For the purpose of this plan, we'll note this is a major simplification.
            self.gl.vertex_attrib_pointer_f32(pos_attr_loc, 2, glow::FLOAT, false, 0, 0); // Problematic without VBO
            self.gl.enable_vertex_attrib_array(pos_attr_loc);
            if is_texture {
                self.gl.vertex_attrib_pointer_f32(tex_coord_attr_loc, 2, glow::FLOAT, false, 0, 0); // Problematic
                self.gl.enable_vertex_attrib_array(tex_coord_attr_loc);
            }

            // The actual data needs to be in a buffer for the above to work correctly.
            // This call will likely fail or draw garbage without proper VBO setup.
            // For now, we assume the MVP matrix handles all transformations.
            // self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4); // Simplified out

            // self.gl.disable_vertex_attrib_array(pos_attr_loc); // Simplified out
            // if is_texture {
            //     self.gl.disable_vertex_attrib_array(tex_coord_attr_loc); // Simplified out
            // }
        }
    }
}

/* Commenting out the old FrameRenderer implementation
impl OldFrameRenderer for Gles2Renderer {
    fn id(&self) -> Uuid { // This is part of OldFrameRenderer
        self.internal_id // This is part of OldFrameRenderer
    }

    // fn render_frame ... (Commented out)

    // fn present_frame ... (Commented out)

    // This is the old create_texture_from_shm, to be refactored into import_shm_buffer_internal
    fn import_shm_buffer_internal(
        &mut self,
        buffer: &WlBuffer,
    ) -> Result<Gles2Texture, Gles2RendererError> { // Returns Gles2Texture now
        let (shm_data, width, height, wl_shm_format) =
            smithay::wayland::shm::with_buffer_contents_data(buffer)
                .map_err(|e| Gles2RendererError::WrappedRendererError(OldRendererError::InvalidBufferType(format!("SHM access: {}", e))))?;

        let fourcc_format = format_shm_to_fourcc(wl_shm_format)
            .ok_or_else(|| Gles2RendererError::WrappedRendererError(OldRendererError::InvalidBufferType(format!("Unsupported SHM: {:?}", wl_shm_format))))?;
        
        let (gl_format, gl_type, gl_internal_format) = match fourcc_format {
            Fourcc::Argb8888 => (glow::BGRA, glow::UNSIGNED_BYTE, glow::RGBA as i32),
            Fourcc::Xrgb8888 => (glow::BGRA, glow::UNSIGNED_BYTE, glow::RGB as i32),
            Fourcc::Abgr8888 => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGBA as i32),
            Fourcc::Xbgr8888 => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGB as i32),
            _ => return Err(Gles2RendererError::WrappedRendererError(OldRendererError::InvalidBufferType(format!("Unsupported Fourcc for GLES2: {:?}", fourcc_format)))),
        };

        let tex_id = unsafe {
            let id = self.gl.create_texture().map_err(|e_str| Gles2RendererError::WrappedRendererError(OldRendererError::TextureUploadFailed(format!("create_texture: {}", e_str))))?;
            self.gl.bind_texture(glow::TEXTURE_2D, Some(id));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D, 0, gl_internal_format, width, height, 0,
                gl_format, gl_type, Some(shm_data)
            );
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.gl.bind_texture(glow::TEXTURE_2D, None);
            id
        };
        
        Ok(Gles2Texture::new( // Returns Gles2Texture directly
            self.gl.clone(), tex_id, width as u32, height as u32, Some(fourcc_format)
        ))
    }

    // This is the old create_texture_from_dmabuf, to be refactored into import_dmabuf_internal
    fn import_dmabuf_internal(
        &mut self,
        dmabuf_attributes: &Dmabuf,
    ) -> Result<Gles2Texture, Gles2RendererError> { // Returns Gles2Texture now
        tracing::debug!(
            "Attempting to create texture from DMABUF (internal): format={:?}, dimensions=({}x{}), planes={}, flags={:?}",
            dmabuf_attributes.format(),
            dmabuf_attributes.width(),
            dmabuf_attributes.height(),
            dmabuf_attributes.num_planes(),
            dmabuf_attributes.flags()
        );
        let dma_format = dmabuf_attributes.format();
        let dma_dims = (dmabuf_attributes.width(), dmabuf_attributes.height());
        let dma_planes = dmabuf_attributes.num_planes();

        let mut egl_attribs: Vec<egl::Attrib> = Vec::with_capacity(30);

        egl_attribs.push(egl::WIDTH as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.width() as egl::Attrib);
        egl_attribs.push(egl::HEIGHT as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.height() as egl::Attrib);
        egl_attribs.push(egl::LINUX_DRM_FOURCC_EXT as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.format().as_u32() as egl::Attrib);

        for i in 0..dma_planes {
            let fd = dmabuf_attributes.plane_fd(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} fd: {}", i, e);
                tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}: {}", dma_format, dma_dims, dma_planes, err_msg );
                OldRendererError::DmabufAccessFailed(err_msg)
            })?;
            let offset = dmabuf_attributes.plane_offset(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} offset: {}", i, e);
                tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}: {}", dma_format, dma_dims, dma_planes, err_msg );
                OldRendererError::DmabufAccessFailed(err_msg)
            })?;
            let pitch = dmabuf_attributes.plane_pitch(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} pitch: {}", i, e);
                tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}: {}", dma_format, dma_dims, dma_planes, err_msg );
                OldRendererError::DmabufAccessFailed(err_msg)
            })?;

            match i {
                0 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE0_FD_EXT as egl::Attrib, fd as egl::Attrib, egl::DMA_BUF_PLANE0_OFFSET_EXT as egl::Attrib, offset as egl::Attrib, egl::DMA_BUF_PLANE0_PITCH_EXT as egl::Attrib, pitch as egl::Attrib]); }
                1 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE1_FD_EXT as egl::Attrib, fd as egl::Attrib, egl::DMA_BUF_PLANE1_OFFSET_EXT as egl::Attrib, offset as egl::Attrib, egl::DMA_BUF_PLANE1_PITCH_EXT as egl::Attrib, pitch as egl::Attrib]); }
                2 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE2_FD_EXT as egl::Attrib, fd as egl::Attrib, egl::DMA_BUF_PLANE2_OFFSET_EXT as egl::Attrib, offset as egl::Attrib, egl::DMA_BUF_PLANE2_PITCH_EXT as egl::Attrib, pitch as egl::Attrib]); }
                3 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE3_FD_EXT as egl::Attrib, fd as egl::Attrib, egl::DMA_BUF_PLANE3_OFFSET_EXT as egl::Attrib, offset as egl::Attrib, egl::DMA_BUF_PLANE3_PITCH_EXT as egl::Attrib, pitch as egl::Attrib]); }
                _ => {
                    let err_msg = format!("Unsupported number of planes: {}", dma_planes);
                    tracing::error!( "DMABUF import error for format {:?}, dims {:?}: {}", dma_format, dma_dims, err_msg );
                    return Err(OldRendererError::DmabufImportFailed(err_msg));
                }
            }

            if dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS) {
                 let modifier = dmabuf_attributes.plane_modifier(i).map_err(|e| {
                    let err_msg = format!("Failed to get plane {} modifier: {}", i, e);
                    tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}: {}", dma_format, dma_dims, dma_planes, err_msg );
                    OldRendererError::DmabufAccessFailed(err_msg)
                 })?;
                 match i {
                     0 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE0_MODIFIER_LO_EXT as egl::Attrib, (modifier & 0xFFFFFFFF) as egl::Attrib, egl::DMA_BUF_PLANE0_MODIFIER_HI_EXT as egl::Attrib, (modifier >> 32) as egl::Attrib]); }
                     1 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE1_MODIFIER_LO_EXT as egl::Attrib, (modifier & 0xFFFFFFFF) as egl::Attrib, egl::DMA_BUF_PLANE1_MODIFIER_HI_EXT as egl::Attrib, (modifier >> 32) as egl::Attrib]); }
                     2 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE2_MODIFIER_LO_EXT as egl::Attrib, (modifier & 0xFFFFFFFF) as egl::Attrib, egl::DMA_BUF_PLANE2_MODIFIER_HI_EXT as egl::Attrib, (modifier >> 32) as egl::Attrib]); }
                     3 => { egl_attribs.extend_from_slice(&[egl::DMA_BUF_PLANE3_MODIFIER_LO_EXT as egl::Attrib, (modifier & 0xFFFFFFFF) as egl::Attrib, egl::DMA_BUF_PLANE3_MODIFIER_HI_EXT as egl::Attrib, (modifier >> 32) as egl::Attrib]); }
                     _ => {}
                 }
            }
        }
        egl_attribs.push(egl::NONE as egl::Attrib);

        let egl_image = self._egl_instance.create_image_khr(
            self.egl_display,
            egl::NO_CONTEXT,
            egl::LINUX_DMA_BUF_EXT,
            std::ptr::null_mut(),
            &egl_attribs,
        ).map_err(|e| {
            let err_msg = format!("eglCreateImageKHR failed: EGL error {:?}", e);
            tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}, modifiers {:?}: {}", dma_format, dma_dims, dma_planes, dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS), err_msg );
            OldRendererError::DmabufImportFailed(err_msg)
        })?;

        if egl_image == egl::NO_IMAGE_KHR {
            let egl_error = self._egl_instance.get_error();
            let err_msg = format!( "eglCreateImageKHR returned EGL_NO_IMAGE_KHR. EGL Error: {:?}", egl_error );
            tracing::error!( "DMABUF import error for format {:?}, dims {:?}, planes {}, modifiers {:?}: {}", dma_format, dma_dims, dma_planes, dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS), err_msg );
            return Err(OldRendererError::DmabufImportFailed(err_msg));
        }

        let tex_id = unsafe {
            let id = self.gl.create_texture().map_err(|e_str| {
                let err_msg = format!("glGenTextures (glCreateTexture) failed: {}", e_str);
                tracing::error!( "DMABUF texture creation error for format {:?}, dims {:?}: {}. Destroying EGLImage {:?}.", dma_format, dma_dims, err_msg, egl_image );
                if self._egl_instance.destroy_image_khr(self.egl_display, egl_image).is_err() {
                    tracing::warn!("Failed to destroy EGLImage {:?} after glGenTextures failure.", egl_image);
                }
                OldRendererError::TextureUploadFailed(err_msg)
            })?;
            
            self.gl.bind_texture(glow::TEXTURE_EXTERNAL_OES, Some(id));
             self.gl.egl_image_target_texture_2d_oes(
                 glow::TEXTURE_EXTERNAL_OES,
                 egl_image as *mut std::ffi::c_void,
             );

            let error_code = self.gl.get_error();
            if error_code != glow::NO_ERROR {
                let err_msg = format!("glEGLImageTargetTexture2DOES failed with GL error: {:#x}", error_code);
                tracing::error!( "DMABUF texture binding error for format {:?}, dims {:?}, GL texture ID {:?}: {}. Destroying EGLImage {:?} and GL texture.", dma_format, dma_dims, id, err_msg, egl_image );
                self.gl.delete_texture(id);
                if self._egl_instance.destroy_image_khr(self.egl_display, egl_image).is_err() {
                     tracing::warn!("Failed to destroy EGLImage {:?} after glEGLImageTargetTexture2DOES failure.", egl_image);
                }
                return Err(OldRendererError::TextureUploadFailed(err_msg));
            }

            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            
            self.gl.bind_texture(glow::TEXTURE_EXTERNAL_OES, None);
            id
        };

        Ok(Box::new(Gles2Texture::new_from_egl_image(
            self.gl.clone(),
            tex_id,
            dmabuf_attributes.width() as u32,
            dmabuf_attributes.height() as u32,
            Some(dmabuf_attributes.format()),
            egl_image,
            self._egl_instance.clone(),
            self.egl_display,
            true,
        )))
    }

    fn screen_size(&self) -> Size<i32, Physical> {
        self.current_screen_size
    }
}
*/

impl CompositorRenderer for Gles2Renderer {
    type Texture = Arc<Gles2Texture>;

    fn new() -> Result<Self, CompositorError> {
        Err(CompositorError::FeatureUnavailable(
            "Gles2Renderer cannot be created through CompositorRenderer::new directly. It must be initialized by a backend.".to_string()
        ))
    }

    fn render_frame(
        &mut self,
        output: &Output,
        surfaces: &[(&WlSurface, Rectangle<i32, Physical>)],
        _dmabuf_importer: &DmabufImporter, // Not directly used if import is done via import_dmabuf method
        desktop_state: &DesktopState,
    ) -> Result<(), CompositorError> {
        // output_geometry_physical derived from output, used for projection
        let output_geometry_physical = output.current_mode().unwrap_or_else(|| {
            let size_logical = output.current_geometry().unwrap().size;
            let mode_size_physical = output.current_scale().scale_coords(size_logical);
            smithay::output::Mode { size: mode_size_physical, refresh: 60_000 }
        }).size;

        let projection = orthographic_projection(
            output_geometry_physical.w as f32,
            output_geometry_physical.h as f32,
        );

        unsafe {
            self.gl.use_program(Some(self.texture_shader_program));
            // Set projection matrix once for all textured quads (surfaces, cursor)
            self.gl.uniform_matrix_3_f32_slice(
                &self.texture_shader_mvp_loc,
                false, // Smithay GLES2 examples use false for transpose
                &projection,
            );
            self.gl.uniform_1_i32(&self.texture_shader_sampler_loc, 0); // Use texture unit 0
        }

        for (surface_wl, geometry_physical) in surfaces {
            if !surface_wl.is_alive() {
                continue;
            }

            let surface_data_dyn = surface_wl.data_map().get::<Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>>();

            if let Some(surface_data_arc_mutex) = surface_data_dyn {
                let surface_data = surface_data_arc_mutex.lock().unwrap(); // TODO: Handle Mutex poison
                if let Some(texture_dyn_arc) = &surface_data.texture_handle {
                    if let Some(gles_texture_arc) = texture_dyn_arc.as_any().downcast_ref::<Arc<Gles2Texture>>() {
                        let gles_texture = gles_texture_arc.as_ref();

                        if let Err(e) = gles_texture.bind(0) { // bind is from the old AbstractionRenderableTexture
                            tracing::error!("Failed to bind GLES2 texture for surface {:?}: {:?}", surface_wl.id(), e);
                            continue;
                        }

                        // Placeholder for actual drawing logic.
                        // This needs to use geometry_physical to position/scale the quad.
                        // The current `render_quad_internal` is too basic and assumes direct vertex coords.
                        // A real solution involves setting up a model matrix per surface based on `geometry_physical`,
                        // then combining with `projection` for the final MVP, or sending transformed vertices.
                        // For now, this is a conceptual draw call.
                        unsafe {
                            // This is a simplified draw call. A proper implementation would setup VBOs,
                            // vertex attributes, and use a model matrix within the shader or by transforming CPU-side.
                            // The `geometry_physical` contains the location and size for the surface.
                            // This current setup implies that the MVP matrix (`projection`) alone is used,
                            // and vertices are expected in clip space, or `render_quad_internal` handles this.
                            // The `render_quad_internal` is not equipped for this yet.
                            // For now, we'll just call a placeholder draw.
                            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
                        }
                        tracing::trace!("Conceptually rendered surface {:?} at {:?}", surface_wl.id(), geometry_physical);

                    } else {
                         tracing::warn!("Surface texture for {:?} is not Arc<Gles2Texture>", surface_wl.id());
                    }
                } else {
                    // It's normal for a surface to not have a texture if it hasn't committed one yet.
                    tracing::trace!("Surface {:?} has no texture_handle. Skipping.", surface_wl.id());
                }
            } else {
                tracing::warn!("Surface {:?} has no SurfaceData (crate::compositor::surface_management). Skipping.", surface_wl.id());
            }
        }

        // Render cursor
        if desktop_state.current_cursor_status.lock().unwrap().is_visible() { // TODO: Handle Mutex poison
            if let Some(cursor_texture_dyn_arc) = &desktop_state.active_cursor_texture {
                if let Some(cursor_gles_texture_arc) = cursor_texture_dyn_arc.as_any().downcast_ref::<Arc<Gles2Texture>>() {
                    let cursor_gles_texture = cursor_gles_texture_arc.as_ref();
                    let cursor_size_physical = cursor_gles_texture.dimensions();

                    let output_scale_f64 = output.current_scale().fractional_scale();
                    let pointer_output_local_logical = desktop_state.pointer_location - output.current_geometry().unwrap().loc.to_f64();
                    let cursor_pos_physical_on_output = pointer_output_local_logical.to_physical(output_scale_f64).to_i32_round();

                    let hotspot_physical = desktop_state.cursor_hotspot.to_physical(output_scale_f64).to_i32_round();
                    let final_cursor_pos_on_output = Point::from((cursor_pos_physical_on_output.x - hotspot_physical.x, cursor_pos_physical_on_output.y - hotspot_physical.y));

                    let _cursor_rect_physical = Rectangle::from_loc_and_size(final_cursor_pos_on_output, cursor_size_physical);

                    if let Err(e) = cursor_gles_texture.bind(0) {
                        tracing::error!("Failed to bind cursor GLES2 texture: {:?}", e);
                    } else {
                        unsafe {
                            // Placeholder draw call for cursor, similar to surfaces.
                            // Needs proper geometry transformation based on _cursor_rect_physical.
                            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
                        }
                        tracing::trace!("Conceptually rendered cursor at {:?}", _cursor_rect_physical);
                    }
                }
            }
        }
        Ok(())
    }

    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer, // Removed _
        _surface: Option<&WlSurface>,
        _desktop_state: &DesktopState,
    ) -> Result<Self::Texture, CompositorError> {
        self.import_shm_buffer_internal(buffer) // Call the refactored internal method
            .map(Arc::new) // Wrap the Gles2Texture in Arc
            .map_err(|e| CompositorError::Generic(format!("GLES2 SHM import failed: {:?}", e)))
    }

    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf, // Removed _
        _surface: Option<&WlSurface>,
    ) -> Result<Self::Texture, CompositorError> {
        self.import_dmabuf_internal(dmabuf) // Call the refactored internal method
            .map(Arc::new) // Wrap the Gles2Texture in Arc
            .map_err(|e| CompositorError::DmabufImportFailed(format!("GLES2 DMABUF import failed: {:?}", e)))
    }

    fn begin_frame(&mut self, output_geometry: Rectangle<i32, Physical>) -> Result<(), CompositorError> {
        unsafe {
            self.gl.viewport(
                output_geometry.loc.x,
                output_geometry.loc.y,
                output_geometry.size.w,
                output_geometry.size.h,
            );
            // self.current_screen_size = output_geometry.size; // If Gles2Renderer still tracks this, update here.
            let background_color = [0.1f32, 0.1, 0.1, 1.0];
            self.gl.clear_color(background_color[0], background_color[1], background_color[2], background_color[3]);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.enable(glow::BLEND);
            self.gl.blend_func_separate(
                glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE, glow::ONE_MINUS_SRC_ALPHA
            );
        }
        Ok(())
    }

    fn finish_frame(&mut self) -> Result<(), CompositorError> {
        unsafe {
            self.gl.finish();
        }
        Ok(())
    }
}

impl Drop for Gles2Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.texture_shader_program);
            self.gl.delete_program(self.solid_shader_program);
            // Delete VBOs/EBOs if created
        }
        tracing::debug!("Dropped Gles2Renderer (ID: {:?})", self.internal_id);
        // Note: EGL display, context, and surface are not owned by Gles2Renderer here.
        // Their lifecycle is managed by the entity that creates Gles2Renderer.
    }
}
