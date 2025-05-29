use glow::{Context, HasContext, Texture};
use khronos_egl as egl; // For EGLImageKHR and related types
use libloading; // For egl::Dynamic
use smithay::backend::renderer::utils::Fourcc;
use std::rc::Rc;
use uuid::Uuid;

use crate::compositor::renderer_interface::abstraction::{RenderableTexture, RendererError};

/// Represents a GLES2 texture, potentially backed by an EGLImage for DMABUF imports.
///
/// This struct encapsulates a `glow::Texture` and manages its lifecycle.
/// For textures imported from DMABUFs via EGLImages, it also stores the necessary
/// EGL handles (`EGLImageKHR`, EGL instance, EGL display) to ensure the `EGLImage`
/// is properly destroyed when the `Gles2Texture` is dropped.
#[derive(Debug)]
pub struct Gles2Texture {
    gl: Rc<Context>,
    texture_id: Texture,
    internal_id: Uuid,
    width: u32,
    height: u32,
    format: Option<Fourcc>,
    /// True if this texture is bound to `GL_TEXTURE_EXTERNAL_OES`, typically for EGLImage imports.
    /// False if it's a standard `GL_TEXTURE_2D`.
    is_external_oes: bool,

    /// The EGLImage handle if this texture was created from a DMABUF via `eglCreateImageKHR`.
    /// `Some` for DMABUF-backed textures, `None` for others (e.g., SHM).
    egl_image: Option<egl::types::EGLImageKHR>,
    /// A reference to the EGL instance, required to destroy the `egl_image`.
    /// Only `Some` if `egl_image` is `Some`.
    egl_instance: Option<Rc<egl::Instance<egl::Dynamic<libloading::Library>>>>,
    /// The EGL display handle, required to destroy the `egl_image`.
    /// Only `Some` if `egl_image` is `Some`.
    egl_display: Option<egl::Display>,
}

impl Gles2Texture {
    // Constructor for standard GL_TEXTURE_2D textures (e.g., from SHM)
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
            is_external_oes: false,
            egl_image: None,
            egl_instance: None,
            egl_display: None,
        }
    }

    /// Creates a new `Gles2Texture` from an existing EGLImage.
    ///
    /// This constructor is used when a texture is created by importing a DMABUF.
    /// The EGLImage, EGL instance, and EGL display are stored to manage the EGLImage's lifecycle.
    ///
    /// # Parameters
    /// - `gl`: The Glow OpenGL context.
    /// - `texture_id`: The GLES texture ID (`glow::Texture`) already created and bound to the EGLImage.
    /// - `width`, `height`: Dimensions of the texture.
    /// - `format`: The underlying pixel format of the buffer, if known.
    /// - `egl_image`: The `EGLImageKHR` handle created from the DMABUF.
    /// - `egl_instance`: The EGL instance used to create and destroy the EGLImage.
    /// - `egl_display`: The EGL display used to create and destroy the EGLImage.
    /// - `is_external_oes`: Must be `true` for textures associated with EGLImages,
    ///   as they are typically bound to `GL_TEXTURE_EXTERNAL_OES`.
    pub(super) fn new_from_egl_image(
        gl: Rc<Context>,
        texture_id: Texture,
        width: u32,
        height: u32,
        format: Option<Fourcc>,
        egl_image: egl::types::EGLImageKHR,
        egl_instance: Rc<egl::Instance<egl::Dynamic<libloading::Library>>>,
        egl_display: egl::Display,
        is_external_oes: bool,
    ) -> Self {
        Self {
            gl,
            texture_id,
            internal_id: Uuid::new_v4(),
            width,
            height,
            format,
            is_external_oes,
            egl_image: Some(egl_image),
            egl_instance: Some(egl_instance),
            egl_display: Some(egl_display),
        }
    }

    // Method to get the underlying Glow texture ID
    pub(super) fn glow_id(&self) -> Texture {
        self.texture_id
    }

    /// Returns the correct GLES texture target for this texture.
    ///
    /// This will be `glow::TEXTURE_EXTERNAL_OES` if `is_external_oes` is true (typically for
    /// DMABUF/EGLImage-backed textures), or `glow::TEXTURE_2D` for standard textures (e.g., from SHM).
    pub(super) fn gl_target(&self) -> u32 {
        if self.is_external_oes {
            glow::TEXTURE_EXTERNAL_OES
        } else {
            glow::TEXTURE_2D
        }
    }
}

impl RenderableTexture for Gles2Texture {
    fn id(&self) -> Uuid {
        self.internal_id
    }

    fn bind(&self, slot: u32) -> Result<(), RendererError> {
        unsafe {
            if slot > 31 { // Arbitrary safety limit
                return Err(RendererError::Generic(format!("Texture slot {} is too high.", slot)));
            }
            self.gl.active_texture(glow::TEXTURE0 + slot);
            self.gl.bind_texture(self.gl_target(), Some(self.texture_id));
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

/// Handles the cleanup of the GLES texture.
///
/// If the texture was created from an EGLImage (i.e., `egl_image` is `Some`),
/// this implementation will also call `eglDestroyImageKHR` to release the EGLImage
/// before deleting the GLES texture. This is crucial for proper DMABUF resource management.
impl Drop for Gles2Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture_id);
        }
        if let (Some(image), Some(instance), Some(display)) =
            (self.egl_image, &self.egl_instance, self.egl_display)
        {
            if image != egl::NO_IMAGE_KHR {
                if let Err(e) = instance.destroy_image_khr(display, image) {
                    tracing::warn!(
                        "Failed to destroy EGLImage (GL ID: {:?}, EGL Image: {:?}): {:?}",
                        self.texture_id, image, e
                    );
                } else {
                    tracing::debug!(
                        "Destroyed EGLImage (GL ID: {:?}, EGL Image: {:?})",
                        self.texture_id, image
                    );
                }
            }
        }
        tracing::debug!("Dropped Gles2Texture (ID: {:?}, GL ID: {:?})", self.internal_id, self.texture_id);
    }
}
