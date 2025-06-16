// novade-system/src/compositor/render/renderer.rs

use smithay::reexports::wayland_server::protocol::{
    wl_buffer::WlBuffer,
    wl_surface::WlSurface,
};
use smithay::utils::{Physical, Rectangle, Size, Point, Transform};
use smithay::backend::allocator::{Fourcc, dmabuf::Dmabuf};
use crate::compositor::core::state::DesktopState; // Assuming DesktopState remains relevant
use raw_window_handle::DisplayHandle; // For the new() method
use std::any::Any;
use std::fmt;
use std::sync::Arc;


/// Errors that can occur during rendering operations.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Backend initialization failed: {0}")]
    BackendInitializationFailed(String),
    #[error("Failed to allocate texture: {0}")]
    TextureAllocationFailed(String),
    #[error("Failed to allocate buffer: {0}")]
    BufferAllocationFailed(String),
    #[error("Failed to import DMABUF: {0}")]
    DmabufImportFailed(String),
    #[error("Failed to import SHM buffer: {0}")]
    ShmBufferImportFailed(String),
    #[error("Invalid render state: {0}")]
    InvalidRenderState(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Draw call failed: {0}")]
    DrawCallFailed(String),
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    #[error("Vulkan error: {0}")]
    Vulkan(String),
    #[error("WGPU error: {0}")]
    Wgpu(String),
    #[error("Unsupported pixel format: {0:?}")]
    UnsupportedFormat(Option<Fourcc>),
    #[error("Invalid buffer: {0}")]
    InvalidBuffer(String),
    #[error("Smithay error: {0}")]
    SmithayError(String),
    #[error("An internal error occurred: {0}")]
    Internal(String), // Generic internal error
}

/// Trait for abstracting a texture that can be rendered by the CompositorRenderer.
pub trait RenderableTexture: fmt::Debug + Send + Sync + 'static {
    /// Returns the dimensions of the texture.
    fn dimensions(&self) -> Size<i32, Physical>;
    /// Returns the pixel format of the texture.
    fn format(&self) -> Option<Fourcc>;
    /// Returns a unique identifier for this texture instance.
    fn unique_id(&self) -> u64;
    /// Returns the texture as a dynamic type.
    fn as_any(&self) -> &dyn Any;
}

/// Represents different types of elements the compositor needs to render.
#[derive(Debug)]
pub enum RenderElement<'a, T: RenderableTexture> {
    Surface {
        texture: Arc<T>,
        geometry: Rectangle<i32, Physical>,
        damage: &'a [Rectangle<i32, Physical>], // Changed to slice
        alpha: f32,
        transform: Transform,
    },
    Cursor {
        texture: Arc<T>,
        position: Point<i32, Physical>,
        hotspot: Point<i32, Physical>, // Hotspot relative to texture's top-left
        damage: &'a [Rectangle<i32, Physical>], // Changed to slice
    },
    SolidColor {
        color: [f32; 4], // RGBA
        geometry: Rectangle<i32, Physical>,
        damage: &'a [Rectangle<i32, Physical>], // Changed to slice
    },
    CompositorUi { // Optional, for compositor's own UI
        texture: Arc<T>,
        geometry: Rectangle<i32, Physical>,
        damage: &'a [Rectangle<i32, Physical>], // Changed to slice
    }
}


/// Defines the capabilities of a compositor renderer.
pub trait CompositorRenderer: Send + 'static {
    /// The specific type of texture this renderer works with.
    type Texture: RenderableTexture + Clone + 'static;

    /// Creates a new instance of the renderer.
    fn new<'a>(display_handle: DisplayHandle<'a>, /* other necessary params */) -> Result<Self, RenderError>
    where
        Self: Sized;

    /// Called at the beginning of a frame rendering sequence.
    /// Prepares the renderer for drawing a new frame.
    fn begin_frame(&mut self, output_transform: Transform, output_physical_size: Size<i32, Physical>) -> Result<(), RenderError>;

    /// Renders a list of `RenderElement`s to the current target.
    /// `output_damage` represents the regions of the output that need repainting.
    fn render_elements<'a>(&mut self, elements: Vec<RenderElement<'a, Self::Texture>>, output_damage: &[Rectangle<i32, Physical>]) -> Result<(), RenderError>;

    /// Called after all elements for the current frame have been submitted.
    /// Finalizes the frame and presents it to the output.
    fn finish_frame(&mut self) -> Result<(), RenderError>;

    /// Imports a `WlBuffer` (typically SHM) into a renderer-specific texture.
    /// The resulting texture is wrapped in an `Arc` for shared ownership.
    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface: Option<&WlSurface>,
        desktop_state: &DesktopState, // Access to shm_state might be needed
    ) -> Result<Arc<Self::Texture>, RenderError>;

    /// Imports a `Dmabuf` into a renderer-specific texture.
    /// The resulting texture is wrapped in an `Arc` for shared ownership.
    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        surface: Option<&WlSurface>, // For context, e.g. damage tracking
    ) -> Result<Arc<Self::Texture>, RenderError>;

    /// Clears a specific texture from any internal caches, if the renderer uses them.
    /// This is important when a texture (e.g., from a client buffer) is no longer valid.
    fn clear_texture_cache(&mut self, texture_id: u64);

    /// Informs the renderer about a change in the output's scale factor.
    /// This can be used to adjust rendering parameters, like font sizes or UI scaling.
    fn set_output_scale(&mut self, scale: f64) -> Result<(), RenderError>;

    /// Returns a list of preferred FourCC codes for DMABUF formats.
    /// This helps clients choose efficient buffer formats.
    fn preferred_formats(&self) -> Option<Vec<Fourcc>>;
}
