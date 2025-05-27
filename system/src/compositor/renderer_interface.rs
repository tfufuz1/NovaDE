use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf, // For create_texture_from_dmabuf
        renderer::utils::Format,   // For RenderableTexture::format
    },
    reexports::wayland_server::protocol::wl_buffer,
    utils::{Physical, Point, Rectangle, Size, Logical}, // For geometry and coordinates
};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

// --- Error Definition ---
#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Renderer context creation failed: {0}")]
    ContextCreationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Texture upload failed: {0}")]
    TextureUploadFailed(String),
    #[error("Buffer swap failed: {0}")]
    BufferSwapFailed(String),
    #[error("Invalid buffer type provided: {0}")]
    InvalidBufferType(String),
    #[error("DRM subsystem error: {0}")]
    DrmError(String),
    #[error("EGL subsystem error: {0}")]
    EglError(String),
    #[error("Generic renderer error: {0}")]
    Generic(String),
    // Specific error for SHM buffer access/conversion if needed
    #[error("SHM buffer processing error: {0}")]
    ShmBufferError(String),
    #[error("DMABUF not supported by this renderer")]
    DmabufNotSupported,
}

// --- Color Definition ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32, // 0.0 to 1.0
    pub g: f32, // 0.0 to 1.0
    pub b: f32, // 0.0 to 1.0
    pub a: f32, // 0.0 to 1.0 (0.0 transparent, 1.0 opaque)
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    // Add more predefined colors as needed
}

// --- RenderableTexture Trait ---
/// Represents a texture that can be used by the `FrameRenderer`.
///
/// This trait abstracts over different texture backends (e.g., SHM, DMABUF, EGLImage).
pub trait RenderableTexture: Send + Sync + std::fmt::Debug {
    /// A unique identifier for this texture.
    fn id(&self) -> Uuid;

    /// Binds the texture to a specific shader slot for rendering.
    /// Placeholder for actual shader interaction.
    fn bind(&self, slot: u32) -> Result<(), RendererError>;

    /// Width of the texture in pixels.
    fn width_px(&self) -> u32;

    /// Height of the texture in pixels.
    fn height_px(&self) -> u32;

    /// The pixel format of the texture, if known.
    /// This can be `None` if the format is opaque or managed internally by the renderer.
    fn format(&self) -> Option<Format>; // Smithay's Format or a custom one
}

// --- RenderElement Enum (Placeholder) ---
/// Represents an element to be rendered on the screen.
///
/// The lifetime `'a` is associated with data that might be borrowed for the duration
/// of a single frame's rendering pass, like damage regions.
#[derive(Debug)]
pub enum RenderElement<'a> {
    Surface {
        surface_id: Uuid, // Our internal ID for the surface (from SurfaceData.id)
        texture: Arc<dyn RenderableTexture>,
        /// Geometry of the surface in logical coordinates, relative to the output.
        geometry: Rectangle<i32, Logical>,
        /// Damage regions for this surface in surface-local logical coordinates.
        /// These are hints to the renderer about which parts of the texture have changed.
        damage_surface_coords: &'a [Rectangle<i32, Logical>], // Damage is logical, surface-relative
    },
    SolidColor {
        color: Color,
        /// Geometry of the solid color block in logical coordinates, relative to the output.
        geometry: Rectangle<i32, Logical>,
    },
    Cursor {
        texture: Arc<dyn RenderableTexture>,
        /// Position of the cursor's top-left corner in logical coordinates, relative to the output.
        position_logical: Point<i32, Logical>,
        /// Hotspot of the cursor in logical coordinates, relative to the cursor's top-left.
        /// This is used to align the actual pointing device position with the cursor image.
        hotspot_logical: Point<i32, Logical>,
    },
    // TODO: Add other element types like Text, Borders, etc., if needed.
}

// --- FrameRenderer Trait ---
/// Defines the interface for a renderer capable of drawing a frame.
///
/// This trait abstracts over specific rendering backends (e.g., OpenGL ES, Vulkan, Software).
pub trait FrameRenderer {
    /// Creates a new instance of the renderer.
    ///
    /// The parameters for `new` are highly backend-specific (e.g., requiring a GL context,
    /// a DRM device, or a window handle). As such, it's often better to have this as part
    /// of the concrete implementation's constructor rather than a trait method.
    /// If included in the trait, it would need a generic configuration struct or be very abstract.
    /// For this interface, we'll assume `new` is handled by the specific implementation.
    // fn new(config: &RendererConfig) -> Result<Self, RendererError> where Self: Sized;

    /// Renders a single frame composed of the provided `RenderElement`s.
    ///
    /// # Arguments
    /// * `elements`: An iterator over `RenderElement`s to be drawn.
    /// * `output_geometry`: The rectangle defining the output area in physical pixels.
    /// * `output_scale`: The scale factor of the output (e.g., 1.0, 2.0 for HiDPI).
    ///
    /// The renderer should transform logical coordinates from `RenderElement`s to
    /// physical coordinates for rendering, using the `output_scale` and respecting
    /// `output_geometry`.
    fn render_frame<'a, E: RenderElement<'a> + 'a>(
        &mut self,
        elements: impl IntoIterator<Item = &'a E>,
        output_geometry: Rectangle<i32, Physical>,
        output_scale: f64,
    ) -> Result<(), RendererError>;

    /// Presents the most recently rendered frame to the display.
    ///
    /// This typically involves swapping buffers in a double-buffered setup.
    fn present_frame(&mut self) -> Result<(), RendererError>;

    /// Creates a `RenderableTexture` from a Wayland SHM buffer (`wl_buffer`).
    ///
    /// The renderer will need to access the SHM buffer's contents and convert them
    /// into a GPU-usable texture format.
    fn create_texture_from_shm(
        &mut self,
        buffer: &wl_buffer::WlBuffer,
    ) -> Result<Arc<dyn RenderableTexture>, RendererError>;

    /// Creates a `RenderableTexture` from a DMABUF.
    ///
    /// This allows for zero-copy buffer sharing between the client and the compositor's GPU.
    /// Implementations can stub this by returning `Err(RendererError::DmabufNotSupported)`.
    fn create_texture_from_dmabuf(
        &mut self,
        dmabuf_attributes: &Dmabuf,
    ) -> Result<Arc<dyn RenderableTexture>, RendererError>;

    /// Returns the size of the screen or output this renderer is targeting, in physical pixels.
    fn screen_size(&self) -> Size<i32, Physical>;

    // Optional: Methods for managing shaders, pipelines, or other renderer resources
    // if they need to be exposed through the trait. For now, these are internal.

    // Optional: A method to indicate the beginning and end of a frame rendering pass,
    // if explicit control is needed (e.g., for clearing buffers, setting render targets).
    // fn begin_frame(&mut self) -> Result<(), RendererError>;
    // fn end_frame(&mut self) -> Result<(), RendererError>;
    // `render_frame` and `present_frame` can implicitly handle this.
}

// --- Placeholder concrete types for testing/early development ---

#[derive(Debug)]
pub struct DummyRenderableTexture {
    uuid: Uuid,
    width: u32,
    height: u32,
    format: Option<Format>,
}

impl DummyRenderableTexture {
    pub fn new(width: u32, height: u32, format: Option<Format>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            width,
            height,
            format,
        }
    }
}

impl RenderableTexture for DummyRenderableTexture {
    fn id(&self) -> Uuid {
        self.uuid
    }
    fn bind(&self, _slot: u32) -> Result<(), RendererError> {
        tracing::trace!("DummyRenderableTexture bound (placeholder)");
        Ok(())
    }
    fn width_px(&self) -> u32 {
        self.width
    }
    fn height_px(&self) -> u32 {
        self.height
    }
    fn format(&self) -> Option<Format> {
        self.format
    }
}

// Notes:
// - `RenderableTexture::format()` uses `smithay::backend::renderer::utils::Format`.
//   This is a good choice if integrating with Smithay's rendering utilities.
//   If a different format enum is needed, it should be defined here.
// - `FrameRenderer::new()` is omitted from the trait as its parameters are too
//   backend-specific. Concrete implementations will have their own constructors.
// - DMABUF support is included in the trait but can be stubbed in implementations.
// - `RenderElement::Surface::damage_surface_coords` is logical and surface-relative.
//   The renderer will need to transform these to physical, output-relative coordinates.
// - The lifetime `'a` in `RenderElement` and `render_frame` allows borrowing data
//   like damage regions for the scope of a single frame render, avoiding unnecessary allocations.
// - `Arc<dyn RenderableTexture>` is used in `RenderElement` to allow shared ownership
//   of textures, which is common as multiple elements (e.g., surfaces in a window stack)
//   might reference the same underlying texture resource.
// - `Color` struct defined with r, g, b, a floats.
// - `RendererError` includes common error types and uses `thiserror`.

// Further considerations for `RenderElement`:
// - Transformations: If elements need individual scaling, rotation, or opacity,
//   these would be added to `RenderElement` variants or a common wrapper struct.
//   For now, geometry is assumed to be pre-transformed to output-relative logical coords.
// - Z-ordering: The order of `elements` in the iterator passed to `render_frame`
//   implies Z-order (later elements are rendered on top). Explicit Z values could be
//   added to `RenderElement` if more complex sorting is needed by the renderer.
//   Smithay's `Space` provides z-indexing for its `Window` trait, which can inform
//   the order of elements.
// - Output association: If rendering to multiple outputs with different characteristics,
//   `FrameRenderer` might need to be associated with a specific output, or `render_frame`
//   might take an output identifier. The current `output_geometry` and `output_scale`
//   imply rendering to one output at a time, or a composite virtual output.
// - `smithay::backend::renderer::element::Element` is Smithay's own trait/enum for render elements.
//   Using it directly could provide better integration with Smithay's renderers (e.g., GLES2).
//   However, defining our own `RenderElement` provides flexibility if our needs diverge
//   or if we want to abstract over non-Smithay renderers. For now, our custom `RenderElement` is used.
//   If we were to use Smithay's Gles2Renderer, we'd likely adapt our `RenderElement`s
//   into types that `Gles2Renderer::render_elements` can consume, or implement
//   `smithay::backend::renderer::element::Element` for our surface types.Okay, I've overwritten `system/src/compositor/renderer_interface.rs` with the definitions for `RendererError`, `Color`, `RenderableTexture` trait, `RenderElement` enum, and `FrameRenderer` trait. I also included a `DummyRenderableTexture` for placeholder purposes.

The next step is to update `system/src/compositor/core/surface_management.rs` to use this new `RenderableTexture` trait for the `texture_handle` field in `SurfaceData`.
