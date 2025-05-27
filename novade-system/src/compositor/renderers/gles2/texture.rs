use glow::{Context, HasContext, Texture};
use smithay::backend::renderer::utils::Fourcc;
use std::rc::Rc; // Using Rc as OpenGL contexts are typically not Send/Sync
use uuid::Uuid;

// Assuming RendererError and RenderableTexture are correctly defined in this path
use crate::compositor::renderer_interface::abstraction::{RenderableTexture, RendererError};

#[derive(Debug)]
pub struct Gles2Texture {
    gl: Rc<Context>, // Rc for single-threaded access, common for GL contexts
    texture_id: Texture,
    internal_id: Uuid,
    width: u32,
    height: u32,
    format: Option<Fourcc>, // Store the original Fourcc format if known
}

impl Gles2Texture {
    pub fn new(
        gl: Rc<Context>,
        texture_id: Texture,
        width: u32,
        height: u32,
        format: Option<Fourcc>,
    ) -> Self {
        Self {
            gl,
            texture_id,
            internal_id: Uuid::new_v4(),
            width,
            height,
            format,
        }
    }

    // Method to get the underlying Glow texture ID if needed by the renderer internally
    pub(super) fn glow_id(&self) -> Texture {
        self.texture_id
    }
}

impl RenderableTexture for Gles2Texture {
    fn id(&self) -> Uuid {
        self.internal_id
    }

    fn bind(&self, slot: u32) -> Result<(), RendererError> {
        unsafe {
            // Ensure the slot is within reasonable limits if necessary.
            // glow::TEXTURE0 = 0x84C0, glow::TEXTURE1 = 0x84C1, etc.
            // Max texture units can be queried via gl.get_parameter_i32(glow::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            // For simplicity, we assume slot is valid (e.g., 0 or 1).
            if slot > 31 { // Arbitrary safety limit based on common GLES2 max
                return Err(RendererError::Generic(format!("Texture slot {} is too high.", slot)));
            }
            self.gl.active_texture(glow::TEXTURE0 + slot);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_id));
        }
        Ok(())
    }

    fn width_px(&self) -> u32 {
        self.width
    }

    fn height_px(&self) -> u32 {
        self.height
    }

    fn format(&self) -> Option<Fourcc> {
        self.format
    }
}

impl Drop for Gles2Texture {
    fn drop(&mut self) {
        unsafe {
            // Ensure the context is still valid if possible, though usually drop is called before context destruction.
            self.gl.delete_texture(self.texture_id);
        }
        tracing::debug!("Dropped Gles2Texture (ID: {:?}, GL ID: {:?})", self.internal_id, self.texture_id);
    }
}
