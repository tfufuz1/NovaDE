// novade-system/src/compositor/render/renderer.rs

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Physical, Rectangle, Size}; // Added Size
use smithay::output::Output; // Smithay's Output type

// Assuming DmabufImporter is accessible from the parent module (super)
use super::dmabuf_importer::DmabufImporter;
// Assuming DesktopState is accessible from crate::compositor::core::state
// This creates a dependency from render module to core module.
// Consider if DesktopState access should be more limited or passed differently.
use crate::compositor::core::state::DesktopState;
// Assuming CompositorError is accessible
use crate::compositor::core::errors::CompositorError;
// Assuming RenderableTexture and other renderer-specific types might be defined here or in a sub-module
// For now, let's assume RenderableTexture will be part of the renderer's responsibility.
// use smithay::backend::renderer::Texture as SmithayTexture; // Example if using Smithay's Texture trait directly
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;


/// Trait for abstracting a texture that can be rendered by the CompositorRenderer.
/// This mirrors smithay::backend::renderer::Texture but is defined here for clarity
/// and potential future extensions if needed.
use std::any::Any; // Required for as_any

pub trait RenderableTexture: std::fmt::Debug + Send + Sync + 'static {
    /// Returns the dimensions of the texture.
    fn dimensions(&self) -> Size<i32, Physical>;
    // Add other methods like format, id, etc., if they become necessary for the trait.
    // fn format(&self) -> Option<Fourcc>; // Example
    fn as_any(&self) -> &dyn Any;
}

/// Defines the capabilities of a compositor renderer.
///
/// This trait abstracts over different rendering backends (e.g., GLES2, Vulkan, WGPU)
/// allowing the compositor to be relatively backend-agnostic in its core logic.
pub trait CompositorRenderer: Send + 'static {
    /// The specific type of texture this renderer works with.
    /// Must implement the `RenderableTexture` trait.
    type Texture: RenderableTexture + Clone; // Added Clone bound

    /// Creates a new instance of the renderer.
    ///
    /// This might involve initializing the graphics API, loading shaders,
    /// and setting up initial rendering resources.
    fn new() -> Result<Self, CompositorError>
    where
        Self: Sized;

    /// Renders a single frame.
    ///
    /// # Arguments
    ///
    /// * `output`: A reference to smithay's `Output` object, representing the display
    ///   or screen area onto which the frame will be rendered. It provides geometry,
    ///   scale, and other relevant output properties.
    /// * `surfaces`: A slice of tuples, where each tuple contains a reference to a
    ///   `WlSurface` to be rendered and its `Rectangle` defining its position and size
    ///   in logical coordinates on the output.
    /// * `dmabuf_importer`: A reference to the `DmabufImporter`, which can be used by
    ///   the renderer to import DMABUF-backed buffers from clients if not already handled.
    /// * `desktop_state`: A reference to the global `DesktopState` of the compositor.
    ///   This provides access to shared memory (SHM) buffers, client data, and other
    ///   compositor-wide information that might be needed for rendering.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `CompositorError` if rendering fails.
    fn render_frame(
        &mut self,
        output: &Output,
        surfaces: &[(&WlSurface, Rectangle<i32, Physical>)], // Changed to Physical for geometry
        dmabuf_importer: &DmabufImporter, // Kept for potential direct use or if renderer needs it
        desktop_state: &DesktopState,   // For accessing SHM buffers, etc.
    ) -> Result<(), CompositorError>;

    /// Imports a `WlBuffer` (typically SHM) into a renderer-specific texture.
    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface: Option<&WlSurface>, // For context, like damage or scale
        desktop_state: &DesktopState, // For shm_state access
    ) -> Result<Self::Texture, CompositorError>;

    /// Imports a `Dmabuf` into a renderer-specific texture.
    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        surface: Option<&WlSurface>, // For context
    ) -> Result<Self::Texture, CompositorError>;

    /// Optional: Method to perform any necessary actions before client buffers are processed,
    /// e.g., beginning a render pass.
    fn begin_frame(&mut self, output_geometry: Rectangle<i32, Physical>) -> Result<(), CompositorError>;

    /// Optional: Method to perform any necessary actions after all client surfaces are rendered,
    /// e.g., rendering compositor UI, cursors.
    fn finish_frame(&mut self) -> Result<(), CompositorError>;

    // TODO: Consider adding methods for:
    // - Resizing (if the renderer needs to explicitly handle output size changes)
    // - Querying capabilities (e.g., supported DMABUF formats)
    // - Managing renderer-specific resources (e.g., shader programs, pipelines)
    // - Cursor rendering (if not handled by render_frame directly)
}
