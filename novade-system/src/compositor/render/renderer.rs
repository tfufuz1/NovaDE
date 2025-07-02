//! Defines the core abstractions for graphics rendering within the NovaDE compositor.
//!
//! This module provides the `CompositorRenderer` trait, which serves as a generic
//! interface for various rendering backends (like OpenGL, Vulkan, WGPU). It also
//! defines associated types like `RenderableTexture` for abstract texture handling,
//! `RenderElement` for describing items to be drawn, and `RenderError` for error
//! reporting. This abstraction layer is crucial for backend flexibility and facilitates
//! future migrations, such as to WGPU.

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
    /// Failure during the initialization of the rendering backend.
    #[error("Backend initialization failed: {0}")]
    BackendInitializationFailed(String),
    /// Error encountered during texture allocation.
    #[error("Failed to allocate texture: {0}")]
    TextureAllocationFailed(String),
    /// Error encountered during buffer allocation.
    #[error("Failed to allocate buffer: {0}")]
    BufferAllocationFailed(String),
    /// Failed to import a DMABUF buffer.
    #[error("Failed to import DMABUF: {0}")]
    DmabufImportFailed(String),
    /// Failed to import a shared memory (SHM) buffer.
    #[error("Failed to import SHM buffer: {0}")]
    ShmBufferImportFailed(String),
    /// The renderer was in an invalid state for the requested operation.
    #[error("Invalid render state: {0}")]
    InvalidRenderState(String),
    /// A shader failed to compile.
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    /// A draw call failed.
    #[error("Draw call failed: {0}")]
    DrawCallFailed(String),
    /// An OpenGL-specific error occurred.
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    /// A Vulkan-specific error occurred.
    #[error("Vulkan error: {0}")]
    Vulkan(String),
    /// A WGPU-specific error occurred.
    #[error("WGPU error: {0}")]
    Wgpu(String),
    /// The provided pixel format is not supported by the renderer.
    #[error("Unsupported pixel format: {0:?}")]
    UnsupportedFormat(Option<Fourcc>),
    /// The provided buffer was invalid or corrupted.
    #[error("Invalid buffer: {0}")]
    InvalidBuffer(String),
    /// An error originating from the Smithay library.
    #[error("Smithay error: {0}")]
    SmithayError(String),
    /// An invalid parameter was provided to a rendering function.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    /// A necessary rendering resource could not be created.
    #[error("Resource creation failed: {0}")]
    ResourceCreationFailed(String),
    /// A requested rendering resource was not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    /// A generic internal error occurred within the renderer.
    #[error("An internal error occurred: {0}")]
    Internal(String), // Generic internal error
}

/// Trait for abstracting a texture that can be rendered by a `CompositorRenderer`.
///
/// This trait allows different rendering backends to define their own texture representations
/// while providing a common interface for the compositor to interact with them.
pub trait RenderableTexture: fmt::Debug + Send + Sync + 'static {
    /// Returns the dimensions (width and height) of the texture in physical pixels.
    fn dimensions(&self) -> Size<i32, Physical>;
    /// Returns the pixel format (e.g., ARGB8888) of the texture, if known.
    /// Some textures might be opaque to the compositor in terms of their exact format.
    fn format(&self) -> Option<Fourcc>;
    /// Returns a unique identifier for this texture instance.
    /// This can be used for caching or tracking purposes.
    fn unique_id(&self) -> u64;
    /// Returns the texture as a dynamic type (`dyn Any`).
    /// This allows for downcasting to a concrete texture type if necessary,
    /// though direct usage should be minimized in favor of the trait's methods.
    fn as_any(&self) -> &dyn Any;
}

/// Represents different types of elements that the compositor needs to render.
///
/// Each variant holds the necessary information for a `CompositorRenderer`
/// to draw the element on the screen.
#[derive(Debug)]
pub enum RenderElement<'a, T: RenderableTexture> {
    /// A Wayland client surface to be rendered.
    Surface {
        /// The actual texture data for the surface.
        texture: Arc<T>,
        /// The geometry (position and size) where the surface should be rendered, in physical pixels.
        geometry: Rectangle<i32, Physical>,
        /// Regions of the surface that have been damaged (updated) and need repainting. Given in surface-local coordinates.
        damage: Vec<Rectangle<i32, Physical>>, // Changed to owned Vec
        /// The opacity of the surface, ranging from `0.0` (fully transparent) to `1.0` (fully opaque).
        alpha: f32,
        /// The transformation to be applied to the surface (e.g., rotation, flip).
        transform: Transform,
    },
    /// A hardware or software cursor to be rendered.
    Cursor {
        /// The texture data for the cursor.
        texture: Arc<T>,
        /// The position where the cursor's hotspot should be placed, in physical pixels.
        position: Point<i32, Physical>,
        /// The hotspot of the cursor, relative to the top-left corner of its texture.
        hotspot: Point<i32, Physical>,
        /// Regions of the output that were previously obscured by the cursor and need repainting.
        damage: Vec<Rectangle<i32, Physical>>, // Changed to owned Vec
    },
    /// A solid color block, typically used for backgrounds or debug rendering.
    SolidColor {
        /// The RGBA color components, each ranging from `0.0` to `1.0`.
        color: [f32; 4],
        /// The geometry (position and size) of the solid color block, in physical pixels.
        geometry: Rectangle<i32, Physical>,
        /// Regions of the output that need to be filled with this color.
        damage: Vec<Rectangle<i32, Physical>>, // Changed to owned Vec
    },
    /// UI elements rendered by the compositor itself (e.g., a status bar, window decorations if server-side).
    CompositorUi {
        /// The texture data for the UI element.
        texture: Arc<T>,
        /// The geometry (position and size) where the UI element should be rendered, in physical pixels.
        geometry: Rectangle<i32, Physical>,
        /// Regions of the UI element that have been damaged and need repainting.
        damage: Vec<Rectangle<i32, Physical>>, // Changed to owned Vec
    }
}


/// Defines the capabilities of a compositor graphics backend.
///
/// This trait abstracts the underlying rendering API (e.g., OpenGL, Vulkan)
/// allowing the compositor to be backend-agnostic. Implementors of this trait
/// are responsible for managing resources, shaders, and executing draw calls.
/// The design aims to simplify the integration of different rendering strategies
/// and facilitate future transitions, for example, towards WGPU.
pub trait CompositorRenderer: Send + 'static {
    /// The backend-specific concrete type that implements `RenderableTexture`.
    /// This allows each renderer to manage its own texture representations.
    type Texture: RenderableTexture + Clone + 'static;

    /// Creates a new instance of the renderer, initializing the graphics backend.
    ///
    /// # Parameters
    /// - `display_handle`: A raw display handle required by some backends (e.g., EGL) to initialize.
    /// - `_other_params`: Placeholder for any other backend-specific parameters that might be needed.
    ///
    /// # Errors
    /// Returns a `RenderError` if initialization of the backend fails.
    fn new<'a>(display_handle: DisplayHandle<'a>, /* other necessary params */) -> Result<Self, RenderError>
    where
        Self: Sized;

    /// Called at the beginning of a frame rendering sequence for a specific output.
    ///
    /// This method prepares the renderer for drawing a new frame. Operations might include
    /// making a rendering context current, setting up framebuffers, or acquiring a swapchain image.
    ///
    /// # Parameters
    /// - `output_transform`: The transformation (e.g., rotation, flip) applied to the output.
    /// - `output_physical_size`: The physical dimensions (width and height) of the output in pixels.
    ///
    /// # Errors
    /// Returns a `RenderError` if preparing the frame fails.
    fn begin_frame(&mut self, output_transform: Transform, output_physical_size: Size<i32, Physical>) -> Result<(), RenderError>;

    /// Renders a list of `RenderElement`s to the current target (e.g., screen, framebuffer).
    ///
    /// The renderer should iterate through the provided elements and draw them in order.
    ///
    /// # Parameters
    /// - `elements`: A vector of `RenderElement`s describing what to draw.
    /// - `clear_color`: If `Some(color)`, the background is cleared with this color before rendering elements. If `None`, no clear is performed.
    /// - `output_damage`: An array of rectangles representing the regions of the output that need repainting.
    ///   The renderer can use this for optimization, only redrawing damaged areas. Used by `finish_frame` typically.
    ///
    /// # Errors
    /// Returns a `RenderError` if any rendering operation fails.
    fn render_elements( // Removed 'a lifetime as RenderElement now owns its damage
        &mut self,
        elements: Vec<RenderElement<Self::Texture>>, // No 'a here
        clear_color: Option<[f32; 4]>,
        output_damage: &[Rectangle<i32, Physical>], // This is for overall frame damage, not element damage
    ) -> Result<(), RenderError>;

    /// Called after all elements for the current frame have been submitted.
    ///
    /// This method finalizes the frame and presents it to the output. Operations might include
    /// submitting command buffers, swapping buffers (for double/triple buffering), or releasing resources.
    ///
    /// # Errors
    /// Returns a `RenderError` if finalizing or presenting the frame fails.
    fn finish_frame(&mut self) -> Result<(), RenderError>;

    /// Imports a Wayland shared memory (`WlBuffer`) buffer and converts it into a backend-specific texture.
    ///
    /// This is typically used for client surfaces that provide their content via SHM.
    /// The resulting texture is wrapped in an `Arc` for shared ownership and efficient reuse.
    ///
    /// # Parameters
    /// - `buffer`: The `WlBuffer` containing the pixel data.
    /// - `surface`: Optionally, the `WlSurface` this buffer is associated with, for context.
    /// - `desktop_state`: Provides access to shared compositor state, like the SHM state for buffer access.
    ///
    /// # Errors
    /// Returns a `RenderError` if importing the SHM buffer fails (e.g., unsupported format, invalid data).
    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface: Option<&WlSurface>,
        desktop_state: &DesktopState,
    ) -> Result<Arc<Self::Texture>, RenderError>;

    /// Imports a DMABUF (`Dmabuf`) buffer and converts it into a backend-specific texture.
    ///
    /// This allows for zero-copy buffer sharing between clients (or other components) and the compositor's
    /// rendering backend. For OpenGL/EGL, this typically involves creating an `EGLImage` from the DMABUF
    /// and then texturing from that image. For Vulkan, it involves importing the DMABUF into a `VkImage`.
    /// The resulting texture is wrapped in an `Arc`.
    ///
    /// # Parameters
    /// - `dmabuf`: The `Dmabuf` object representing the buffer.
    /// - `surface`: Optionally, the `WlSurface` this buffer is associated with, for context.
    ///
    /// # Errors
    /// Returns a `RenderError` if importing the DMABUF fails (e.g., unsupported format, driver issues).
    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        surface: Option<&WlSurface>,
    ) -> Result<Arc<Self::Texture>, RenderError>;

    /// Clears a specific texture from any internal caches managed by the renderer.
    ///
    /// This is important when a texture (e.g., from a client buffer that has been released)
    /// is no longer valid and should not be reused.
    ///
    /// # Parameters
    /// - `texture_id`: The unique identifier of the texture to remove from the cache.
    fn clear_texture_cache(&mut self, texture_id: u64);

    /// Informs the renderer about a change in the output's scale factor.
    ///
    /// This can be used by the renderer to adjust rendering parameters, such as
    /// font sizes, UI element scaling, or shader parameters, to ensure crisp visuals
    /// on high-DPI displays.
    ///
    /// # Parameters
    /// - `scale`: The new output scale factor (e.g., `1.0` for standard DPI, `2.0` for HiDPI).
    ///
    /// # Errors
    /// Returns a `RenderError` if applying the scale factor change fails.
    fn set_output_scale(&mut self, scale: f64) -> Result<(), RenderError>;

    /// Returns a list of preferred FourCC codes for DMABUF formats, ordered by preference.
    ///
    /// This helps clients (or other buffer providers) choose efficient buffer formats that
    /// are well-supported by the rendering backend, potentially enabling zero-copy pathways.
    /// If `None`, the renderer has no specific preference or cannot provide this information.
    fn preferred_formats(&self) -> Option<Vec<Fourcc>>;
}
