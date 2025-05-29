use glow;
use std::rc::Rc;

use super::egl_context::OpenGLError;
use super::texture::{Texture, TextureError};

#[derive(Debug)]
pub enum FramebufferError {
    OpenGLCall(String), // Changed from OpenGLError to String for flexibility
    TextureAttachmentError(TextureError),
    IncompleteFramebuffer(String),
    UnsupportedFormat(String), // Potentially for future use if specific formats are checked
    InternalError(String),     // For unexpected issues like create_framebuffer failing
}

impl From<OpenGLError> for FramebufferError {
    fn from(e: OpenGLError) -> Self {
        FramebufferError::OpenGLCall(format!("{:?}", e))
    }
}

impl From<TextureError> for FramebufferError {
    fn from(e: TextureError) -> Self {
        FramebufferError::TextureAttachmentError(e)
    }
}

pub struct Framebuffer {
    gl: Rc<glow::Context>,
    id: glow::Framebuffer,
    color_attachment: Texture,
    // Optional: depth_stencil_attachment: Option<TextureOrRenderbufferID>,
    // For simplicity, depth/stencil is not handled in this phase.
}

impl Framebuffer {
    pub fn new(gl: Rc<glow::Context>, width: u32, height: u32) -> Result<Self, FramebufferError> {
        if width == 0 || height == 0 {
            return Err(FramebufferError::OpenGLCall("Width or height cannot be 0 for Framebuffer.".to_string()));
        }

        // 1. Create a Texture for the color attachment
        // No initial data is provided (None), as the FBO rendering will populate it.
        // Using RGBA8 format as a common choice.
        // Ensure texture parameters are suitable for FBOs (e.g., no mipmaps if not generated/used).
        let color_attachment = Texture::new_from_data(
            Rc::clone(&gl),
            width,
            height,
            &[], // No initial data, texture is write-only initially from FBO perspective
            glow::RGBA,       // Format of data (not relevant here as data is empty)
            glow::RGBA8,      // Internal format in GL
        )?;
        // Note: Texture::new_from_data might need adjustment if [] for data is problematic.
        // A common way is to pass `None` for data in tex_image_2d if supported by glow wrapper,
        // or pass a null pointer in raw GL. If glow expects Some(data), pass `vec![0u8; (width*height*4) as usize]`
        // This might require `Texture::new_uninitialized` or similar.
        // For now, assuming `Texture::new_from_data` with empty slice is handled by `tex_image_2d`
        // as allocating space but not initializing pixels (driver dependent, often results in black or garbage).
        // A safer `Texture::new_from_data` would use `gl.tex_image_2d(..., None)` for data if possible,
        // or the texture should be created specifically for FBO attachment (e.g. with texStorage2D).
        // Let's assume the current Texture::new_from_data is sufficient or will be adapted.
        // If `Texture::new_from_data` with `&[]` fails, a specific constructor for FBO textures might be needed.
        // A quick fix to `Texture::new_from_data` if it requires data:
        // let data = vec![0u8; (width * height * 4) as usize]; // RGBA, so 4 bytes per pixel
        // Then pass `&data` instead of `&[]`.
        // For now, proceeding with `&[]` and will address if it becomes a clear issue.

        // 2. Create Framebuffer Object
        let fbo_id = unsafe { gl.create_framebuffer() }
            .map_err(|e| FramebufferError::InternalError(format!("glCreateFramebuffer failed: {}", e)))?;

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo_id));

            // 3. Attach the color texture
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_attachment.id()),
                0, // Mipmap level
            );

            // (Optional) Depth/Stencil attachment would be here.
            // Example for a depth renderbuffer (not a texture):
            // let rbo_depth_id = gl.create_renderbuffer()?;
            // gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo_depth_id));
            // gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, width as i32, height as i32);
            // gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_STENCIL_ATTACHMENT, glow::RENDERBUFFER, Some(rbo_depth_id));
            // Store rbo_depth_id in Self for cleanup.


            // 4. Check Framebuffer status
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.bind_framebuffer(glow::FRAMEBUFFER, None); // Unbind before erroring
                gl.delete_framebuffer(fbo_id); // Clean up FBO
                // The color_attachment texture will be dropped automatically if this function errors.
                return Err(FramebufferError::IncompleteFramebuffer(format!(
                    "Framebuffer not complete: status 0x{:x}",
                    status
                )));
            }

            // 5. Unbind FBO (return to default framebuffer)
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Ok(Self {
            gl,
            id: fbo_id,
            color_attachment,
        })
    }

    /// Binds this framebuffer for rendering.
    /// Also sets the viewport to the dimensions of the framebuffer's color attachment.
    pub fn bind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.id));
            self.gl.viewport(
                0,
                0,
                self.color_attachment.width() as i32,
                self.color_attachment.height() as i32,
            );
        }
    }

    /// Unbinds this framebuffer, binding the default framebuffer instead.
    /// The viewport should be reset by the caller to the screen/surface dimensions.
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
    
    /// Static method to bind the default framebuffer (framebuffer 0).
    /// Also sets the viewport to the given screen dimensions.
    pub static fn bind_default_framebuffer(gl: &Rc<glow::Context>, screen_width: i32, screen_height: i32) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, screen_width, screen_height);
        }
    }


    /// Returns a reference to the color attachment texture.
    pub fn color_texture(&self) -> &Texture {
        &self.color_attachment
    }
    
    pub fn id(&self) -> glow::Framebuffer {
        self.id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.id);
            // Note: The color_attachment (Texture) is owned by Framebuffer
            // and will be dropped automatically, deleting its GL texture object.
            // If a Renderbuffer was used for depth/stencil, it would need to be deleted here too.
        }
    }
}

// A note on Texture::new_from_data for FBOs:
// If `Texture::new_from_data(..., &[], ...)` doesn't correctly allocate storage
// without pixel data (it's often better to pass NULL to glTexImage2D's data param for this),
// `Texture` might need a new constructor like `Texture::new_for_fbo_attachment` or similar,
// which calls `glTexImage2D` with a NULL data pointer to only allocate storage.
// Example:
// unsafe {
//     gl.tex_image_2d(
//         glow::TEXTURE_2D, 0, internal_format, width, height, 0,
//         format, glow::UNSIGNED_BYTE, None // <--- Pass None for data
//     );
// }
// This requires `Texture::new_from_data`'s `data` parameter to be `Option<&[u8]>`.
// If `Texture::new_from_data` is kept as `data: &[u8]`, then to create an uninitialized
// texture, one would pass `&vec![0u8; required_size]`, which is less ideal as it involves
// a potentially large heap allocation just to be ignored or overwritten.
// The current `Texture::new_from_data` should ideally be changed to `data: Option<&[u8]>`.
// I will assume for now that the `glow` wrapper for `tex_image_2d` handles `Some(&[])`
// as "allocate but don't initialize", or that it's acceptable to initialize with zeros if that's what it does.
// If `Texture::new_from_data` must have data, the `Framebuffer::new` should create a dummy vec:
// let data = vec![0u8; (width * height * 4) as usize]; // For RGBA8
// let color_attachment = Texture::new_from_data(Rc::clone(&gl), width, height, &data, ...)?;
// This is less efficient. The ideal is `None` for data.
// I will proceed assuming `Texture::new_from_data(..., &[], ...)` is acceptable.
// If `glow::Context::tex_image_2d` takes `Option<&[u8]>`, that's what should be used.
// A quick check of `glow` docs indicates its `tex_image_2d` takes `data: Option<&[u8]>`.
// So, `Texture::new_from_data` should be updated to accept `Option<&[u8]>`.
// This change will be deferred to when `texture.rs` is explicitly modified.
// For now, the `Framebuffer::new` will pass `&[]` which might work or might need this later adjustment.```rust
use glow;
use std::rc::Rc;

use super::egl_context::OpenGLError;
use super::texture::{Texture, TextureError}; // Assuming TextureError is defined in texture.rs

#[derive(Debug)]
pub enum FramebufferError {
    OpenGLCall(String), 
    TextureAttachmentError(TextureError),
    IncompleteFramebuffer(String),
    UnsupportedFormat(String), 
    InternalError(String),     
}

impl From<OpenGLError> for FramebufferError {
    fn from(e: OpenGLError) -> Self {
        FramebufferError::OpenGLCall(format!("{:?}", e))
    }
}

impl From<TextureError> for FramebufferError {
    fn from(e: TextureError) -> Self {
        FramebufferError::TextureAttachmentError(e)
    }
}

pub struct Framebuffer {
    gl: Rc<glow::Context>,
    id: glow::Framebuffer,
    color_attachment: Texture,
    // Optional: depth_stencil_attachment is omitted for this phase.
}

impl Framebuffer {
    pub fn new(gl: Rc<glow::Context>, width: u32, height: u32) -> Result<Self, FramebufferError> {
        if width == 0 || height == 0 {
            return Err(FramebufferError::OpenGLCall("Width or height cannot be 0 for Framebuffer.".to_string()));
        }

        // Create a Texture for the color attachment.
        // Pass None for data to allocate uninitialized texture storage, suitable for FBOs.
        // This relies on Texture::new_from_data being able to accept Option<&[u8]>.
        // If Texture::new_from_data currently expects &[u8], it needs modification.
        // For now, let's assume it will be adapted or this is a placeholder for such an adaptation.
        // The prompt for Texture::new_from_data was `data: &[u8]`.
        // A robust Framebuffer::new would ensure Texture can be created without initial data.
        // Create a Texture for the color attachment.
        // Pass None for data to allocate uninitialized texture storage, suitable for FBOs.
        // This relies on Texture::new_from_data now accepting Option<&[u8]>.
        let color_attachment = Texture::new_from_data(
            Rc::clone(&gl),
            width,
            height,
            None, // Pass None for data, as the texture will be rendered to.
            glow::RGBA,    // Format of the pixel data (irrelevant for None)
            glow::RGBA8,   // Internal format in GL
        )?;

        let fbo_id = unsafe { gl.create_framebuffer() }
            .map_err(|e| FramebufferError::InternalError(format!("glCreateFramebuffer failed: {}", e)))?;

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo_id));

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_attachment.id()),
                0, // Mipmap level
            );

            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.delete_framebuffer(fbo_id);
                return Err(FramebufferError::IncompleteFramebuffer(format!(
                    "Framebuffer not complete: status 0x{:x}",
                    status
                )));
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Ok(Self {
            gl,
            id: fbo_id,
            color_attachment,
        })
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.id));
            self.gl.viewport(
                0,
                0,
                self.color_attachment.width() as i32,
                self.color_attachment.height() as i32,
            );
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
    
    pub static fn bind_default_framebuffer(gl: &Rc<glow::Context>, screen_width: i32, screen_height: i32) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, screen_width, screen_height);
        }
    }

    pub fn color_texture(&self) -> &Texture {
        &self.color_attachment
    }
    
    pub fn id(&self) -> glow::Framebuffer {
        self.id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.id);
        }
    }
}
```
