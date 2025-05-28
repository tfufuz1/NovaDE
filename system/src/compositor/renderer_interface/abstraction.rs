use smithay::backend::renderer::gles2::GlesTexture; // Placeholder, use RenderableTexture
use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::utils::{Physical, Rectangle, Size, Transform, BufferCoords};
use smithay::backend::allocator::dmabuf::Dmabuf; // For DMABUF support later
use smithay::backend::renderer::utils::Format;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Renderer context creation failed: {0}")]
    ContextCreationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Texture upload failed: {0}")]
    TextureUploadFailed(String),
    #[error("Buffer swap/present failed: {0}")]
    BufferSwapFailed(String),
    #[error("Invalid buffer type: {0}")]
    InvalidBufferType(String),
    #[error("DRM error: {0}")]
    DrmError(String),
    #[error("EGL error: {0}")]
    EglError(String),
    #[error("Generic renderer error: {0}")]
    Generic(String),
}

pub trait RenderableTexture: Debug + Send + Sync {
    fn id(&self) -> Uuid; // For tracking/comparison
    fn bind(&self, slot: u32) -> Result<(), RendererError>; // For use in shaders
    fn width_px(&self) -> u32;
    fn height_px(&self) -> u32;
    fn format(&self) -> Option<Format>; // Pixel format
}

// Placeholder for a generic render element, to be fleshed out
// This will likely align with Smithay's Element or be a custom version
pub trait RenderElement<'a> {
    fn id(&self) -> Uuid; // Unique ID for this element in a frame, changed from u64 to Uuid for consistency
    fn geometry(&self, scale: f64) -> Rectangle<i32, Physical>;
    fn texture(&self, scale: f64) -> &'a dyn RenderableTexture; // Reference to our RenderableTexture
    fn damage(&self, scale: f64, space_size: Size<i32, Physical>) -> Vec<Rectangle<i32, Physical>>; // Damage in physical coordinates
    fn transform(&self) -> Transform; // Surface transform
    fn opaque_regions(&self, scale: f64) -> Option<Vec<Rectangle<i32, Physical>>>; // Opaque regions in physical coordinates
    fn alpha(&self) -> f32 { 1.0 } // Element alpha
    fn location(&self, scale: f64) -> smithay::utils::Point<i32, Physical>; // Location of the element
}


pub trait FrameRenderer: Send + Sync {
    // fn new(...) -> Result<Self, RendererError> where Self: Sized; // new is hard with traits

    fn render_frame<'a>(
        &mut self,
        output_geometry: Rectangle<i32, Physical>,
        output_scale: f64,
        elements: impl IntoIterator<Item = &'a (dyn RenderElement<'a> + 'a)>,
    ) -> Result<Vec<Rectangle<i32, Physical>>, RendererError>; // Return damage from rendering

    fn present_frame(&mut self, surface_id_to_present_on: Option<u32 /* crtc id or similar */>) -> Result<(), RendererError>;

    fn create_texture_from_shm(
        &mut self,
        buffer: &wl_buffer::WlBuffer,
    ) -> Result<Box<dyn RenderableTexture>, RendererError>;

    fn create_texture_from_dmabuf(
         &mut self,
         dmabuf: &Dmabuf,
    ) -> Result<Box<dyn RenderableTexture>, RendererError>;

    fn screen_size(&self) -> Size<i32, Physical>;
    
    // Add a method to handle cursor import specifically if it differs from general texture import
    fn import_cursor_buffer(
        &mut self,
        buffer: &wl_buffer::WlBuffer,
        hotspot: (i32, i32) // Include hotspot if it's part of texture creation/attributes
    ) -> Result<Box<dyn RenderableTexture>, RendererError>;

    // Method to update hardware cursor if supported
    fn update_hardware_cursor(
        &mut self,
        texture: Option<Box<dyn RenderableTexture>>, // Use our RenderableTexture
        hotspot: (i32, i32)
    ) -> Result<(), RendererError>;

    // Optional: Method to register a new output/surface target for rendering
    // fn register_output_target(&mut self, /* details of the output */) -> Result<(), RendererError>;

    // Optional: Method to unregister an output/surface target
    // fn unregister_output_target(&mut self, /* identifier of the output */) -> Result<(), RendererError>;

    // Optional: Method to resize rendering buffers if applicable (e.g. for a windowed backend)
    // fn resize_target(&mut self, new_size: Size<i32, Physical>) -> Result<(), RendererError>;
}
