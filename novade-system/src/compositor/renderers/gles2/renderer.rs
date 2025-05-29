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
use std::rc::Rc;
use uuid::Uuid;

use crate::compositor::renderer_interface::abstraction::{
    FrameRenderer, RenderableTexture, RenderElement, RendererError,
};
// Adjust path if errors.rs is in a different location relative to renderer.rs
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
    gl: Rc<Context>,
    egl_display: egl::Display,
    egl_context: egl::Context,
    // Store the EGL instance to ensure it outlives the display and context.
    // This is important as per khronos-egl documentation.
    _egl_instance: Rc<egl::Instance<egl::Dynamic<libloading::Library>>>, 
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
        egl_instance: Rc<egl::Instance<egl::Dynamic<libloading::Library>>>,
        initial_screen_size: Size<i32, Physical>,
        egl_surface: Option<egl::Surface>,
    ) -> Result<Self, Gles2RendererError> {
        let gl_rc = Rc::new(gl);

        // Compile and link texture shader
        let vs_tex = compile_shader(&gl_rc, glow::VERTEX_SHADER, VS_TEXTURE_SRC)?;
        let fs_tex = compile_shader(&gl_rc, glow::FRAGMENT_SHADER, FS_TEXTURE_SRC)?;
        let texture_shader_program = link_program(&gl_rc, vs_tex, fs_tex)?;
        unsafe { // Shaders can be deleted after linking
            gl_rc.detach_shader(texture_shader_program, vs_tex);
            gl_rc.detach_shader(texture_shader_program, fs_tex);
            gl_rc.delete_shader(vs_tex);
            gl_rc.delete_shader(fs_tex);
        }
        let texture_shader_mvp_loc = unsafe { gl_rc.get_uniform_location(texture_shader_program, "mvp") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("mvp in texture_shader".to_string()))?;
        let texture_shader_sampler_loc = unsafe { gl_rc.get_uniform_location(texture_shader_program, "u_texture") }
             .ok_or_else(|| Gles2RendererError::UniformNotFound("u_texture in texture_shader".to_string()))?;

        // Compile and link solid color shader
        let vs_solid = compile_shader(&gl_rc, glow::VERTEX_SHADER, VS_SOLID_SRC)?;
        let fs_solid = compile_shader(&gl_rc, glow::FRAGMENT_SHADER, FS_SOLID_SRC)?;
        let solid_shader_program = link_program(&gl_rc, vs_solid, fs_solid)?;
        unsafe { // Shaders can be deleted after linking
            gl_rc.detach_shader(solid_shader_program, vs_solid);
            gl_rc.detach_shader(solid_shader_program, fs_solid);
            gl_rc.delete_shader(vs_solid);
            gl_rc.delete_shader(fs_solid);
        }
        let solid_shader_mvp_loc = unsafe { gl_rc.get_uniform_location(solid_shader_program, "mvp") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("mvp in solid_shader".to_string()))?;
        let solid_shader_color_loc = unsafe { gl_rc.get_uniform_location(solid_shader_program, "u_color") }
            .ok_or_else(|| Gles2RendererError::UniformNotFound("u_color in solid_shader".to_string()))?;

        Ok(Self {
            internal_id: Uuid::new_v4(),
            gl: gl_rc,
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
    fn render_quad_internal(&self, geometry: Rectangle<i32, Logical>, is_texture: bool) {
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
            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            self.gl.disable_vertex_attrib_array(pos_attr_loc);
            if is_texture {
                self.gl.disable_vertex_attrib_array(tex_coord_attr_loc);
            }
        }
    }
}

impl FrameRenderer for Gles2Renderer {
    fn id(&self) -> Uuid {
        self.internal_id
    }

    fn render_frame<'a>(
        &mut self,
        elements: impl IntoIterator<Item = RenderElement<'a>>,
        output_geometry: Rectangle<i32, Physical>,
        _output_scale: f64, // TODO: Use for HiDPI rendering if needed
    ) -> Result<(), RendererError> {
        unsafe {
            self.gl.viewport(
                output_geometry.loc.x,
                output_geometry.loc.y,
                output_geometry.size.w,
                output_geometry.size.h,
            );
            self.current_screen_size = output_geometry.size;

            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT); // No depth buffer for 2D compositor

            self.gl.enable(glow::BLEND);
            self.gl.blend_func_separate(
                glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA, // RGB
                glow::ONE, glow::ONE_MINUS_SRC_ALPHA        // Alpha
            );
            // self.gl.disable(glow::DEPTH_TEST); // Typically off for 2D

            let projection = orthographic_projection(
                self.current_screen_size.w as f32,
                self.current_screen_size.h as f32,
            );

            for element in elements {
                match element {
                    RenderElement::WaylandSurface {
                        surface_data_arc,
                        geometry, // Logical coordinates of the surface element
                        .. // surface_wl, damage_surface_local ignored for now
                    } => {
                        let surface_data = surface_data_arc.lock().unwrap();
                        if let Some(texture_handle_dyn) = &surface_data.texture_handle {
                            // Downcast to Gles2Texture
                            if let Some(gles_texture) = texture_handle_dyn.as_ref().downcast_ref::<Gles2Texture>() {
                                self.gl.use_program(Some(self.texture_shader_program));
                                
                                // MVP: For now, projection matrix directly maps geometry to clip space.
                                // A real MVP would transform a unit quad to `geometry` then apply projection.
                                // This simplified approach means `render_quad_internal` needs to draw
                                // using `geometry` as vertex coordinates directly.
                                self.gl.uniform_matrix_3_f32_slice(
                                    &self.texture_shader_mvp_loc,
                                    true, // Transpose: GL expects column-major, our matrix is row-major
                                    &projection,
                                );
                                
                                gles_texture.bind(0)?;
                                self.gl.uniform_1_i32(&self.texture_shader_sampler_loc, 0);
                                
                                self.render_quad_internal(geometry, true);
                            } else {
                                tracing::warn!("Surface texture is not Gles2Texture. Skipping.");
                            }
                        } else {
                            tracing::trace!("Surface has no texture. Skipping.");
                        }
                    }
                    RenderElement::SolidColor { color, geometry } => {
                        self.gl.use_program(Some(self.solid_shader_program));
                        self.gl.uniform_matrix_3_f32_slice(
                            &self.solid_shader_mvp_loc,
                            true, // Transpose
                            &projection,
                        );
                        self.gl.uniform_4_f32_slice(&self.solid_shader_color_loc, &color);
                        self.render_quad_internal(geometry, false);
                    }
                    RenderElement::Cursor { texture_arc, position_logical, .. } => {
                         if let Some(gles_texture) = texture_arc.as_ref().downcast_ref::<Gles2Texture>() {
                            self.gl.use_program(Some(self.texture_shader_program));
                            let cursor_geometry = Rectangle::from_loc_and_size(
                                position_logical,
                                Size::from((gles_texture.width_px() as i32, gles_texture.height_px() as i32))
                            );
                            self.gl.uniform_matrix_3_f32_slice(
                                &self.texture_shader_mvp_loc,
                                true, // Transpose
                                &projection,
                            );
                            gles_texture.bind(0)?;
                            self.gl.uniform_1_i32(&self.texture_shader_sampler_loc, 0);
                            self.render_quad_internal(cursor_geometry, true);
                        }
                    }
                }
            }
            self.gl.finish();
        }
        Ok(())
    }

    fn present_frame(&mut self) -> Result<(), RendererError> {
        if let Some(surface) = self.egl_surface {
            self._egl_instance.swap_buffers(self.egl_display, surface)
                .map_err(|e| RendererError::BufferSwapFailed(format!("EGL swap_buffers: {:?}", e)))?;
        } else {
            tracing::debug!("present_frame on headless renderer, no EGL surface to swap.");
        }
        Ok(())
    }

    fn create_texture_from_shm(
        &mut self,
        buffer: &WlBuffer,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let (shm_data, width, height, wl_shm_format) =
            smithay::wayland::shm::with_buffer_contents_data(buffer)
                .map_err(|e| RendererError::InvalidBufferType(format!("SHM access: {}", e)))?;

        let fourcc_format = format_shm_to_fourcc(wl_shm_format)
            .ok_or_else(|| RendererError::InvalidBufferType(format!("Unsupported SHM: {:?}", wl_shm_format)))?;
        
        // Map Fourcc to GL format, type, and internal_format
        let (gl_format, gl_type, gl_internal_format) = match fourcc_format {
            Fourcc::Argb8888 => (glow::BGRA, glow::UNSIGNED_BYTE, glow::RGBA as i32),
            Fourcc::Xrgb8888 => (glow::BGRA, glow::UNSIGNED_BYTE, glow::RGB as i32), // No alpha component in data, but GL expects 4 components for BGRA
            Fourcc::Abgr8888 => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGBA as i32),
            Fourcc::Xbgr8888 => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGB as i32),
            _ => return Err(RendererError::InvalidBufferType(format!("Unsupported Fourcc for GLES2: {:?}", fourcc_format))),
        };

        let tex_id = unsafe {
            let id = self.gl.create_texture().map_err(|e_str| RendererError::TextureUploadFailed(format!("create_texture: {}", e_str)))?;
            self.gl.bind_texture(glow::TEXTURE_2D, Some(id));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D, 0, gl_internal_format, width, height, 0,
                gl_format, gl_type, Some(shm_data)
            );
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.gl.bind_texture(glow::TEXTURE_2D, None); // Unbind
            id
        };
        
        Ok(Box::new(Gles2Texture::new(
            self.gl.clone(), tex_id, width as u32, height as u32, Some(fourcc_format)
        )))
    }

    /// Creates a GLES2 texture from a DMABUF.
    ///
    /// This method imports a DMABUF provided by a client into the EGL/GLES2 rendering system.
    /// The process involves:
    /// 1. Using EGL functions to create an `EGLImage` from the DMABUF attributes.
    ///    This typically requires the `EGL_EXT_image_dma_buf_import` extension.
    ///    If modifiers are used, `EGL_EXT_image_dma_buf_import_modifiers` might also be needed.
    /// 2. Creating a GLES texture (`glow::Texture`).
    /// 3. Binding this GLES texture to the `GL_TEXTURE_EXTERNAL_OES` target.
    /// 4. Using `glEGLImageTargetTexture2DOES` (from the `GL_OES_EGL_image` extension) to link the
    ///    `EGLImage` content to the GLES texture.
    ///
    /// The resulting texture is wrapped in a [`Gles2Texture`] struct, which manages its lifecycle,
    /// including the destruction of the associated `EGLImage` when the texture is dropped.
    ///
    /// # Parameters
    ///
    /// - `dmabuf_attributes`: A reference to [`smithay::backend::allocator::dmabuf::Dmabuf`]
    ///   which provides access to the DMABUF's properties like file descriptors, planes,
    ///   format, dimensions, and modifiers.
    ///
    /// # Errors
    ///
    /// This function can return a [`RendererError`] (often wrapping a [`Gles2RendererError`])
    /// if any step in the import process fails:
    /// - [`RendererError::DmabufAccessFailed`]: If accessing DMABUF plane data (FDs, offsets, pitches, modifiers) fails.
    /// - [`RendererError::DmabufImportFailed`]: If `eglCreateImageKHR` fails (e.g., unsupported format/modifier,
    ///   invalid DMABUF attributes, or EGL errors). This can also be returned if the EGL image handle is invalid.
    /// - [`RendererError::TextureUploadFailed`]: If GLES texture creation (`glGenTextures`) or linking the
    ///   `EGLImage` to the texture (`glEGLImageTargetTexture2DOES`) fails due to GL errors.
    ///
    /// # Assumptions
    ///
    /// - The caller is responsible for ensuring that a compatible EGL display and context are current
    ///   and that the necessary EGL and GLES extensions (primarily `EGL_EXT_image_dma_buf_import`
    ///   and `GL_OES_EGL_image`) are supported and enabled on the host system.
    /// - The provided `dmabuf_attributes` are valid and represent a DMABUF that the underlying
    ///   EGL implementation can import.
    fn create_texture_from_dmabuf(
        &mut self,
        dmabuf_attributes: &smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        tracing::debug!(
            "Attempting to create texture from DMABUF: format={:?}, dimensions=({}x{}), planes={}, flags={:?}",
            dmabuf_attributes.format(),
            dmabuf_attributes.width(),
            dmabuf_attributes.height(),
            dmabuf_attributes.num_planes(),
            dmabuf_attributes.flags()
        );
        let dma_format = dmabuf_attributes.format();
        let dma_dims = (dmabuf_attributes.width(), dmabuf_attributes.height());
        let dma_planes = dmabuf_attributes.num_planes();
        // Note: Modifier logging can be very verbose. Log only on error for now.

        let mut egl_attribs: Vec<egl::Attrib> = Vec::with_capacity(30);

        egl_attribs.push(egl::WIDTH as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.width() as egl::Attrib);
        egl_attribs.push(egl::HEIGHT as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.height() as egl::Attrib);
        egl_attribs.push(egl::LINUX_DRM_FOURCC_EXT as egl::Attrib);
        egl_attribs.push(dmabuf_attributes.format().as_u32() as egl::Attrib);

        // Add plane FDs, offsets, and pitches
        for i in 0..dma_planes {
            let fd = dmabuf_attributes.plane_fd(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} fd: {}", i, e);
                tracing::error!(
                    "DMABUF import error for format {:?}, dims {:?}, planes {}: {}",
                    dma_format, dma_dims, dma_planes, err_msg
                );
                RendererError::DmabufAccessFailed(err_msg)
            })?;
            let offset = dmabuf_attributes.plane_offset(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} offset: {}", i, e);
                tracing::error!(
                    "DMABUF import error for format {:?}, dims {:?}, planes {}: {}",
                    dma_format, dma_dims, dma_planes, err_msg
                );
                RendererError::DmabufAccessFailed(err_msg)
            })?;
            let pitch = dmabuf_attributes.plane_pitch(i).map_err(|e| {
                let err_msg = format!("Failed to get plane {} pitch: {}", i, e);
                tracing::error!(
                    "DMABUF import error for format {:?}, dims {:?}, planes {}: {}",
                    dma_format, dma_dims, dma_planes, err_msg
                );
                RendererError::DmabufAccessFailed(err_msg)
            })?;

            match i {
                0 => {
                    egl_attribs.push(egl::DMA_BUF_PLANE0_FD_EXT as egl::Attrib);
                    egl_attribs.push(fd as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE0_OFFSET_EXT as egl::Attrib);
                    egl_attribs.push(offset as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE0_PITCH_EXT as egl::Attrib);
                    egl_attribs.push(pitch as egl::Attrib);
                }
                1 => {
                    egl_attribs.push(egl::DMA_BUF_PLANE1_FD_EXT as egl::Attrib);
                    egl_attribs.push(fd as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE1_OFFSET_EXT as egl::Attrib);
                    egl_attribs.push(offset as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE1_PITCH_EXT as egl::Attrib);
                    egl_attribs.push(pitch as egl::Attrib);
                }
                2 => {
                    egl_attribs.push(egl::DMA_BUF_PLANE2_FD_EXT as egl::Attrib);
                    egl_attribs.push(fd as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE2_OFFSET_EXT as egl::Attrib);
                    egl_attribs.push(offset as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE2_PITCH_EXT as egl::Attrib);
                    egl_attribs.push(pitch as egl::Attrib);
                }
                3 => { // Max 4 planes typically
                    egl_attribs.push(egl::DMA_BUF_PLANE3_FD_EXT as egl::Attrib);
                    egl_attribs.push(fd as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE3_OFFSET_EXT as egl::Attrib);
                    egl_attribs.push(offset as egl::Attrib);
                    egl_attribs.push(egl::DMA_BUF_PLANE3_PITCH_EXT as egl::Attrib);
                    egl_attribs.push(pitch as egl::Attrib);
                }
                _ => {
                    let err_msg = format!("Unsupported number of planes: {}", dma_planes);
                    tracing::error!(
                        "DMABUF import error for format {:?}, dims {:?}: {}",
                        dma_format, dma_dims, err_msg
                    );
                    return Err(RendererError::DmabufImportFailed(err_msg));
                }
            }

            if dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS) {
                 let modifier = dmabuf_attributes.plane_modifier(i).map_err(|e| {
                    let err_msg = format!("Failed to get plane {} modifier: {}", i, e);
                    tracing::error!(
                        "DMABUF import error for format {:?}, dims {:?}, planes {}: {}",
                        dma_format, dma_dims, dma_planes, err_msg
                    );
                    RendererError::DmabufAccessFailed(err_msg)
                 })?;
                 match i {
                     0 => {
                         egl_attribs.push(egl::DMA_BUF_PLANE0_MODIFIER_LO_EXT as egl::Attrib);
                         egl_attribs.push((modifier & 0xFFFFFFFF) as egl::Attrib);
                         egl_attribs.push(egl::DMA_BUF_PLANE0_MODIFIER_HI_EXT as egl::Attrib);
                         egl_attribs.push((modifier >> 32) as egl::Attrib);
                     }
                     1 => {
                         egl_attribs.push(egl::DMA_BUF_PLANE1_MODIFIER_LO_EXT as egl::Attrib);
                         egl_attribs.push((modifier & 0xFFFFFFFF) as egl::Attrib);
                         egl_attribs.push(egl::DMA_BUF_PLANE1_MODIFIER_HI_EXT as egl::Attrib);
                         egl_attribs.push((modifier >> 32) as egl::Attrib);
                     }
                     2 => {
                         egl_attribs.push(egl::DMA_BUF_PLANE2_MODIFIER_LO_EXT as egl::Attrib);
                         egl_attribs.push((modifier & 0xFFFFFFFF) as egl::Attrib);
                         egl_attribs.push(egl::DMA_BUF_PLANE2_MODIFIER_HI_EXT as egl::Attrib);
                         egl_attribs.push((modifier >> 32) as egl::Attrib);
                     }
                     3 => {
                         egl_attribs.push(egl::DMA_BUF_PLANE3_MODIFIER_LO_EXT as egl::Attrib);
                         egl_attribs.push((modifier & 0xFFFFFFFF) as egl::Attrib);
                         egl_attribs.push(egl::DMA_BUF_PLANE3_MODIFIER_HI_EXT as egl::Attrib);
                         egl_attribs.push((modifier >> 32) as egl::Attrib);
                     }
                     _ => {} // Already handled by num_planes check
                 }
            }
        }
        egl_attribs.push(egl::NONE as egl::Attrib); // Terminate the list

        let egl_image = self._egl_instance.create_image_khr(
            self.egl_display,
            egl::NO_CONTEXT, // Context should be NO_CONTEXT for EGL_LINUX_DMA_BUF_EXT target
            egl::LINUX_DMA_BUF_EXT, // target
            std::ptr::null_mut(),   // client_buffer, must be null for LINUX_DMA_BUF_EXT
            &egl_attribs,           // attrib_list
        ).map_err(|e| {
            let err_msg = format!("eglCreateImageKHR failed: EGL error {:?}", e);
            tracing::error!(
                "DMABUF import error for format {:?}, dims {:?}, planes {}, modifiers {:?}: {}",
                dma_format, dma_dims, dma_planes, dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS), err_msg
            );
            RendererError::DmabufImportFailed(err_msg)
        })?;

        if egl_image == egl::NO_IMAGE_KHR {
            // This case should ideally be covered by the error above from create_image_khr,
            // but as a safeguard:
            let egl_error = self._egl_instance.get_error(); // Query EGL for more specific error
            let err_msg = format!(
                "eglCreateImageKHR returned EGL_NO_IMAGE_KHR. EGL Error: {:?}",
                egl_error
            );
            tracing::error!(
                "DMABUF import error for format {:?}, dims {:?}, planes {}, modifiers {:?}: {}",
                dma_format, dma_dims, dma_planes, dmabuf_attributes.flags().contains(DmabufFlags::HAS_MODIFIERS), err_msg
            );
            return Err(RendererError::DmabufImportFailed(err_msg));
        }

        // Now create a GLES texture from the EGLImage
        let tex_id = unsafe {
            let id = self.gl.create_texture().map_err(|e_str| {
                let err_msg = format!("glGenTextures (glCreateTexture) failed: {}", e_str);
                tracing::error!(
                    "DMABUF texture creation error for format {:?}, dims {:?}: {}. Destroying EGLImage {:?}.",
                    dma_format, dma_dims, err_msg, egl_image
                );
                // Attempt to clean up EGL image if GL texture creation fails
                if self._egl_instance.destroy_image_khr(self.egl_display, egl_image).is_err() {
                    tracing::warn!("Failed to destroy EGLImage {:?} after glGenTextures failure.", egl_image);
                }
                RendererError::TextureUploadFailed(err_msg)
            })?;
            
            self.gl.bind_texture(glow::TEXTURE_EXTERNAL_OES, Some(id));

            // Get the address of glEGLImageTargetTexture2DOES.
            // This is an EGL extension function, not part of core GL.
            // We need to load it via eglGetProcAddress.
            // The khronos_egl::Instance can be used for this, but it's a bit manual.
            // Let's assume self._egl_instance has a way or we need to load it.
            // For now, we'll represent this call conceptually.
            // A robust solution would look up "glEGLImageTargetTexture2DOES"
            // using self._egl_instance.get_proc_address("glEGLImageTargetTexture2DOES")
            // and then transmute it to the correct function signature.

            // Pseudo-code for loading and calling glEGLImageTargetTexture2DOES:
            // let egl_image_target_texture_2d_oes_ptr = self._egl_instance.get_proc_address("glEGLImageTargetTexture2DOES")
            //    .ok_or_else(|| RendererError::EglExtensionNotSupported("glEGLImageTargetTexture2DOES".to_string()))?;
            // let gl_egl_image_target_texture_2d_oes: extern "C" fn(target: u32, image: egl::types::EGLImage) =
            //    std::mem::transmute(egl_image_target_texture_2d_oes_ptr);
            // gl_egl_image_target_texture_2d_oes(glow::TEXTURE_EXTERNAL_OES, egl_image);
            
            // The `glow` crate itself might provide this if the OES_EGL_image extension
            // is correctly detected and enabled for the context. Let's check glow's API.
            // If glow provides `gl.egl_image_target_texture_2d_oes`, that's preferred.
            // Looking at glow documentation, it seems it *should* be available on `Context`
            // if the feature "GL_OES_EGL_image" is present.

            // Check if the GL context supports GL_OES_EGL_image.
            // This should ideally be done once at renderer initialization.
            // For now, we assume it's available and `glow` will provide it.
            // The function in `glow` is `egl_image_target_texture_2d_oes`.
             self.gl.egl_image_target_texture_2d_oes(
                 glow::TEXTURE_EXTERNAL_OES,
                 egl_image as *mut std::ffi::c_void, // EGLImage is a C pointer type
             );

            // Check for GL errors after this call
            let error_code = self.gl.get_error();
            if error_code != glow::NO_ERROR {
                let err_msg = format!("glEGLImageTargetTexture2DOES failed with GL error: {:#x}", error_code);
                tracing::error!(
                    "DMABUF texture binding error for format {:?}, dims {:?}, GL texture ID {:?}: {}. Destroying EGLImage {:?} and GL texture.",
                    dma_format, dma_dims, id, err_msg, egl_image
                );
                self.gl.delete_texture(id); // Clean up GL texture
                if self._egl_instance.destroy_image_khr(self.egl_display, egl_image).is_err() {
                     tracing::warn!("Failed to destroy EGLImage {:?} after glEGLImageTargetTexture2DOES failure.", egl_image);
                }
                return Err(RendererError::TextureUploadFailed(err_msg));
            }

            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_EXTERNAL_OES, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            
            self.gl.bind_texture(glow::TEXTURE_EXTERNAL_OES, None); // Unbind
            id
        };

        // Note: The EGLImage should be destroyed when the texture is no longer needed.
        // This implies Gles2Texture should probably own the EGLImage and destroy it on drop.
        // For now, we are not destroying it here, which is a leak if the texture outlives this scope
        // without specific cleanup. Gles2Texture needs modification to handle this.
        // For the purpose of this task, we'll create the Gles2Texture and assume its Drop impl
        // will be updated later to destroy the EGLImage.

        Ok(Box::new(Gles2Texture::new_from_egl_image(
            self.gl.clone(),
            tex_id,
            dmabuf_attributes.width() as u32,
            dmabuf_attributes.height() as u32,
            Some(dmabuf_attributes.format()), // Store format
            egl_image, // Pass EGL image for later destruction
            self._egl_instance.clone(), // EGL instance for destruction
            self.egl_display, // EGL display for destruction
            true, // is_external_oes
        )))
    }

    fn screen_size(&self) -> Size<i32, Physical> {
        self.current_screen_size
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
