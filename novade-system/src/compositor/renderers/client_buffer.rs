use glow;
use khronos_egl as egl; // For egl::Image
use std::rc::Rc;
use smithay::reexports::wayland_server::protocol::wl_shm; // For wl_shm::Format
use smithay::backend::allocator::dmabuf::Dmabuf; // For Dmabuf type hint

use super::egl_context::OpenGLError;
use super::texture::TextureError;

#[derive(Debug)]
pub enum ClientBufferError {
    OpenGLError(OpenGLError),
    TextureError(TextureError), // If reusing Texture struct or its errors
    UnsupportedShmFormat(u32), // wl_shm::Format as u32
    DmaBufImportUnsupported,
    DmaBufImageCreationFailed(egl::types::EGLenum), // EGL error code
    DmaBufTextureLinkFailed(String), // GL error string or info log
    InvalidDmaBuf(String), // e.g. missing planes, invalid FD
    PixelUnpackError(String), // For errors during pixel unpacking (stride, etc.)
    InternalError(String),
}

impl From<OpenGLError> for ClientBufferError {
    fn from(e: OpenGLError) -> Self {
        ClientBufferError::OpenGLError(e)
    }
}

impl From<TextureError> for ClientBufferError {
    fn from(e: TextureError) -> Self {
        ClientBufferError::TextureError(e)
    }
}

// EGLImage is a pointer type in khronos_egl
type EglImage = egl::Image;

pub struct ClientTexture {
    // gl: Rc<glow::Context>, // This will be accessed via parent_gl_context.gl()
    texture_id: glow::Texture,
    width: u32,
    height: u32,
    // Store Rc<GlContext> to access EGL instance for destroying EGLImage,
    // and Glow context (gl) for GL operations.
    parent_gl_context: Rc<super::egl_context::GlContext>,
    egl_image: Option<EglImage>, // Store EGLImage if DMA-BUF
}

impl ClientTexture {
    /// Creates a new texture from a shared memory (SHM) buffer.
    ///
    /// - `gl_context`: The parent GlContext providing OpenGL and EGL resources.
    /// - `data`: Slice containing the pixel data from the SHM buffer.
    /// - `width`, `height`: Dimensions of the buffer in pixels.
    /// - `stride`: Number of bytes from the beginning of one row to the beginning of the next.
    /// - `shm_format`: The Wayland SHM format of the pixel data.
    pub fn new_from_shm(
        gl_context: Rc<super::egl_context::GlContext>,
        data: &[u8],
        width: i32,
        height: i32,
        stride: i32, // Bytes
        shm_format: wl_shm::Format,
    ) -> Result<Self, ClientBufferError> {
        if width <= 0 || height <= 0 {
            return Err(ClientBufferError::InternalError("Width and height must be positive.".to_string()));
        }

        let (gl_format, gl_internal_format, bytes_per_pixel) = shm_format_to_gl(shm_format)?;
        
        // Validate data length against dimensions and stride (basic check)
        if (height as usize * stride as usize) > data.len() {
            return Err(ClientBufferError::InternalError(
                format!("SHM data slice too small for given dimensions and stride. Expected at least {} bytes, got {}.", height as usize * stride as usize, data.len())
            ));
        }


        let texture_id = unsafe { gl.create_texture() }
            .map_err(|e_str| ClientBufferError::InternalError(format!("glCreateTexture failed: {}", e_str)))?;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
            // Standard texture parameters
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            // Handle stride: UNPACK_ROW_LENGTH is specified in pixels, not bytes.
            let expected_row_bytes = width * bytes_per_pixel as i32;
            if stride != expected_row_bytes {
                if stride < expected_row_bytes || stride % (bytes_per_pixel as i32) != 0 {
                     gl.delete_texture(texture_id); // Clean up
                    return Err(ClientBufferError::PixelUnpackError(format!(
                        "Invalid stride {} for width {} and bpp {}. Must be >= width*bpp and multiple of bpp.",
                        stride, width, bytes_per_pixel
                    )));
                }
                gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, stride / (bytes_per_pixel as i32));
            }

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0, // level
                gl_internal_format,
                width,
                height,
                0, // border (must be 0)
                gl_format,
                glow::UNSIGNED_BYTE,
                Some(data),
            );

            // Reset UNPACK_ROW_LENGTH to default (0 means tightly packed)
            if stride != expected_row_bytes {
                gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            }
            
            gl.bind_texture(glow::TEXTURE_2D, None); // Unbind
            
            // Check for GL errors after critical operations
            let error_code = gl.get_error();
            if error_code != glow::NO_ERROR {
                gl.delete_texture(texture_id); // Clean up allocated texture
                return Err(ClientBufferError::OpenGLError(OpenGLError::Other(format!(
                    "OpenGL error after SHM texture creation: 0x{:x}", error_code
                ))));
            }
        }

        Ok(Self {
            // gl: Rc::clone(&gl_context.gl()), // Store the glow::Context directly
            texture_id,
            width: width as u32,
            height: height as u32,
            parent_gl_context: gl_context,
            egl_image: None,   // Not used for SHM
        })
    }
    
    /// Updates an existing SHM-based texture with new data.
    /// Assumes dimensions and format are the same.
    pub fn update_from_shm(
        &self,
        data: &[u8],
        x_offset: i32,
        y_offset: i32,
        width: i32,    // width of the sub-image to update
        height: i32,   // height of the sub-image to update
        stride: i32,   // stride of the *source* data
        shm_format: wl_shm::Format,
    ) -> Result<(), ClientBufferError> {
        let (gl_format, _, bytes_per_pixel) = shm_format_to_gl(shm_format)?;

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_id));

            let expected_row_bytes = width * bytes_per_pixel as i32;
            if stride != expected_row_bytes {
                 if stride < expected_row_bytes || stride % (bytes_per_pixel as i32) != 0 {
                    return Err(ClientBufferError::PixelUnpackError(format!(
                        "Invalid stride {} for update width {} and bpp {}.",
                        stride, width, bytes_per_pixel
                    )));
                }
                self.gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, stride / (bytes_per_pixel as i32));
            }

            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0, // level
                x_offset,
                y_offset,
                width,
                height,
                gl_format,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );

            if stride != expected_row_bytes {
                self.gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            }
            
            self.gl.bind_texture(glow::TEXTURE_2D, None); // Unbind

            let error_code = self.gl.get_error();
            if error_code != glow::NO_ERROR {
                return Err(ClientBufferError::OpenGLError(OpenGLError::Other(format!(
                    "OpenGL error during SHM texture update: 0x{:x}", error_code
                ))));
            }
        }
        Ok(())
    }


    // Constructor for DMA-BUF will be added here
    pub fn new_from_dmabuf(
        gl_context: Rc<super::egl_context::GlContext>, // Pass GlContext by Rc
        dmabuf: &Dmabuf,
    ) -> Result<Self, ClientBufferError> {
        if !gl_context.supports_dma_buf_import {
            return Err(ClientBufferError::DmaBufImportUnsupported);
        }

        let egl_instance = gl_context.egl_instance();
        let egl_display = gl_context.display(); // EGLDisplay needed for create_image_khr and destroy_image_khr

        // --- Prepare EGLImage attributes ---
        // This part is highly dependent on the Dmabuf struct's API and the specific plane/modifier layout.
        // Smithay's Dmabuf provides access to planes, fds, offsets, strides.
        // Modifiers might need specific EGL extensions (EGL_EXT_image_dma_buf_import_modifiers).
        
        let width = dmabuf.width() as egl::Attrib;
        let height = dmabuf.height() as egl::Attrib;
        
        // Format conversion: Dmabuf format (FourCC) to EGL_LINUX_DRM_FOURCC_EXT
        // This requires a mapping. Example for a common format like ARGB8888 or XRGB8888.
        // DRM_FORMAT_ARGB8888 is 0x34325241 in FourCC.
        // DRM_FORMAT_XRGB8888 is 0x34325258.
        let fourcc_format = match dmabuf.format().code {
            // Example: Map specific FourCC codes. This needs to be comprehensive.
            0x34325241 => egl::LINUX_DRM_FOURCC_EXT_ARGB8888, // This is a conceptual constant.
                                                             // khronos_egl crate might not define these specific format tokens.
                                                             // They are usually just the FourCC value itself.
                                                             // So, this might just be `dmabuf.format().code as egl::Attrib`.
                                                             // For now, let's use a placeholder or assume direct use of code.
                                                             // The EGL spec for EGL_EXT_image_dma_buf_import says EGL_LINUX_DRM_FOURCC_EXT takes the FourCC code.
            fourcc_code => fourcc_code as egl::Attrib, // Directly use the FourCC code
        };

        // Prepare attributes for eglCreateImageKHR
        // This is simplified. Multi-planar formats need attributes for each plane (FD, offset, pitch).
        // Modifiers also add more attributes if EGL_EXT_image_dma_buf_import_modifiers is used.
        let mut attribs = vec![
            egl::WIDTH, width,
            egl::HEIGHT, height,
            egl::LINUX_DRM_FOURCC_EXT, fourcc_format,
        ];

        // Assuming single-plane for simplicity here. Dmabuf can have multiple planes.
        if dmabuf.num_planes() == 0 {
            return Err(ClientBufferError::InvalidDmaBuf("DMA-BUF has no planes.".to_string()));
        }
        
        // Plane 0 attributes (example for a common single-plane or first plane of multi-plane)
        attribs.extend(&[
            egl::DMA_BUF_PLANE0_FD_EXT, dmabuf.plane_fd(0).map_err(|_| ClientBufferError::InvalidDmaBuf("Failed to get FD for plane 0".to_string()))? as egl::Attrib,
            egl::DMA_BUF_PLANE0_OFFSET_EXT, dmabuf.plane_offset(0).map_err(|_| ClientBufferError::InvalidDmaBuf("Failed to get offset for plane 0".to_string()))? as egl::Attrib,
            egl::DMA_BUF_PLANE0_PITCH_EXT, dmabuf.plane_stride(0).map_err(|_| ClientBufferError::InvalidDmaBuf("Failed to get stride for plane 0".to_string()))? as egl::Attrib,
        ]);

        // Modifier support (if EGL_EXT_image_dma_buf_import_modifiers is present and used)
        // let modifier = dmabuf.plane_modifier(0); // This API might not exist directly on Dmabuf. Smithay provides it.
        // if modifier != DRM_FORMAT_MOD_INVALID && modifier != DRM_FORMAT_MOD_LINEAR {
        //    attribs.extend(&[
        //        egl::DMA_BUF_PLANE0_MODIFIER_LO_EXT, (modifier & 0xFFFFFFFF) as egl::Attrib,
        //        egl::DMA_BUF_PLANE0_MODIFIER_HI_EXT, (modifier >> 32) as egl::Attrib,
        //    ]);
        // }
        // Note: khronos_egl uses *const Attrib for attributes, so they need to be properly terminated with EGL_NONE.
        attribs.push(egl::NONE as egl::Attrib); // Terminator for the attribute list

        let egl_image = unsafe {
            egl_instance.create_image_khr(
                egl_display,
                egl::NO_CONTEXT, // Context should be EGL_NO_CONTEXT as per spec for EGL_LINUX_DMA_BUF_EXT target
                egl::LINUX_DMA_BUF_EXT,
                std::ptr::null_mut(), // client_buffer (not used for DMA_BUF_EXT)
                attribs.as_ptr() as *const _, // Correctly pass pointer to attribs
            )
        };
        
        if egl_image.is_none() || egl_image == Some(egl::NO_IMAGE_KHR) {
            let egl_error = egl_instance.get_error();
            return Err(ClientBufferError::DmaBufImageCreationFailed(egl_error));
        }
        let egl_image_val = egl_image.unwrap();


        // --- Create GL texture and link EGLImage ---
        let gl = Rc::clone(gl_context.gl());
        let texture_id = unsafe { gl.create_texture() }
            .map_err(|e_str| ClientBufferError::InternalError(format!("glCreateTexture for DMA-BUF failed: {}", e_str)))?;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
            // Standard texture parameters (might differ for DMA-BUF/external textures)
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            // Link EGLImage to the texture
            // This requires OES_EGL_image or similar GL extension. Glow provides this via features.
            // The function signature in glow might be `gl.egl_image_target_texture_2d(target, image)`
            // where image is `egl::ImageKHR` (which `EglImage` is an alias for).
            // The `glow` feature `oes_egl_image` needs to be enabled for this.
            // Check `glow` documentation for the exact function if `glEGLImageTargetTexture2DOES` is not found directly.
            // It might be `gl.egl_image_target_texture_2d(glow::TEXTURE_2D, egl_image_val as *mut _)`
            // or similar, depending on how `glow` wraps this extension.
            // Assuming `glow` has `egl_image_target_texture_2d_oes` available.
            gl.egl_image_target_texture_2d_oes(glow::TEXTURE_2D, egl_image_val as *mut std::ffi::c_void);

            gl.bind_texture(glow::TEXTURE_2D, None); // Unbind

            let error_code = gl.get_error();
            if error_code != glow::NO_ERROR {
                // Cleanup
                gl.delete_texture(texture_id);
                egl_instance.destroy_image_khr(egl_display, egl_image_val);
                return Err(ClientBufferError::DmaBufTextureLinkFailed(format!(
                    "OpenGL error linking EGLImage to texture: 0x{:x}", error_code
                )));
            }
        }

        Ok(Self {
            // gl: Rc::clone(gl_context.gl()),
            texture_id,
            width: dmabuf.width() as u32,
            height: dmabuf.height() as u32,
            parent_gl_context: gl_context,
            egl_image: Some(egl_image_val),
        })
    }

    
    pub fn texture_id(&self) -> glow::Texture {
        self.texture_id
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Binds the texture to a specific texture unit.
    pub fn bind(&self, texture_unit: u32) {
        unsafe {
            self.parent_gl_context.gl().active_texture(glow::TEXTURE0 + texture_unit);
            self.parent_gl_context.gl().bind_texture(glow::TEXTURE_2D, Some(self.texture_id));
        }
    }

    /// Unbinds the texture from a specific texture unit.
    pub fn unbind(&self, texture_unit: u32) {
        unsafe {
            self.parent_gl_context.gl().active_texture(glow::TEXTURE0 + texture_unit);
            self.parent_gl_context.gl().bind_texture(glow::TEXTURE_2D, None);
        }
    }
}

impl Drop for ClientTexture {
    fn drop(&mut self) {
        let gl = self.parent_gl_context.gl(); // Get glow::Context for delete_texture
        unsafe {
            gl.delete_texture(self.texture_id);
            if let Some(image) = self.egl_image.take() {
                // Use the EGL instance and display from the stored GlContext
                let egl_instance = self.parent_gl_context.egl_instance();
                let display = self.parent_gl_context.display();
                if !egl_instance.destroy_image_khr(display, image) {
                    eprintln!(
                        "Failed to destroy EGLImage. EGL error: {}",
                        super::egl_context::egl_error_string_from_instance(egl_instance)
                    );
                }
            }
        }
    }
}

// Helper function to map wl_shm::Format to OpenGL format and internal format
// Returns (gl_format, gl_internal_format, bytes_per_pixel)
pub(super) fn shm_format_to_gl(
    shm_format: wl_shm::Format,
) -> Result<(u32, i32, u8), ClientBufferError> {
    match shm_format {
        wl_shm::Format::Argb8888 => Ok((glow::BGRA, glow::RGBA8, 4)),
        wl_shm::Format::Xrgb8888 => Ok((glow::BGRA, glow::RGBA8, 4)), // Treat XRGB as BGRA, alpha will be ignored by GL / opaque
        wl_shm::Format::Rgb888 => Ok((glow::RGB, glow::RGB8, 3)),
        // Add other formats as needed. This is a minimal set.
        // For example, Bgr888 might map to (glow::BGR, glow::RGB8, 3)
        // Greyscale, etc. would need specific handling.
        _ => Err(ClientBufferError::UnsupportedShmFormat(shm_format as u32)),
    }
}
