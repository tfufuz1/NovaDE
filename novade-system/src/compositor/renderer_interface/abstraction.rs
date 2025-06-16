use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::utils::Fourcc,
    },
    reexports::wayland_server::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    utils::{Buffer as SmithayBuffer, Logical, Physical, Point, Rectangle, Size}, // Renamed Buffer to SmithayBuffer
};

// Assuming gles2::errors will exist, adjust path if needed
// use crate::compositor::renderers::gles2::errors::Gles2RendererError; 
// For now, Gles2RendererError is not #[from] to avoid cyclic dependency issues if it also uses RendererError,
// or if Gles2RendererError is not yet fully defined. This can be refined.
use crate::compositor::surface_management::SurfaceData; // Assuming this path
use novade_compositor_core::surface::SurfaceId; // Added for SurfaceId in upload_surface_texture
use novade_core::types::geometry::Rect as NovaRect; // For clip_rect and source_rect
use crate::compositor::scene_graph::Transform as SceneGraphTransform; // For the transformation matrix

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Context creation failed: {0}")]
    ContextCreationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Texture upload failed: {0}")]
    TextureUploadFailed(String),
    #[error("Buffer swap/presentation failed: {0}")]
    BufferSwapFailed(String),
    #[error("Invalid buffer type: {0}")]
    InvalidBufferType(String),
    #[error("DMABUF import unsupported or failed: {0}")]
    DmabufImportFailed(String), 
    #[error("OpenGL ES 2 specific error: {0}")]
    Gles2Error(String), // Simplified for now, can be #[from] Gles2RendererError later
                        // Gles2Error(#[from] Gles2RendererError)
    #[error("Generic renderer error: {0}")]
    Generic(String),
    #[error("Operation unsupported by the renderer: {0}")]
    Unsupported(String), // Moved from original list to group with other errors
}

// Placeholder for buffer format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferFormat {
    Argb8888,
    Xrgb8888,
    // ... other common formats
}

// Represents raw buffer data from a client
// This will likely need to be more sophisticated in a real scenario.
pub struct ClientBuffer<'a> {
    pub id: u64, // A unique ID for this buffer content
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: BufferFormat,
}

pub trait RenderableTexture: Send + Sync + std::fmt::Debug + std::any::Any {
    fn id(&self) -> Uuid;
    fn bind(&self, slot: u32) -> Result<(), RendererError>;
    fn width_px(&self) -> u32;
    fn height_px(&self) -> u32;
    fn format(&self) -> Option<Fourcc>;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Parameters for rendering a texture, used by `RenderElement::TextureNode`.
#[derive(Debug)]
pub struct TextureRenderParams {
    pub texture: Box<dyn RenderableTexture>, // Owned texture handle
    /// Final transformation matrix to apply in world space.
    pub transform: SceneGraphTransform, // Using the Transform struct from scene_graph
    /// Overall opacity of the texture (0.0 transparent, 1.0 opaque).
    pub alpha: f32,
    /// Clipping rectangle in world/screen coordinates.
    pub clip_rect: NovaRect<f32>, // Using novade_core::types::geometry::Rect
    /// Source rectangle within the texture to sample from.
    /// Coordinates can be normalized (0.0-1.0) or pixel-based, depending on renderer convention.
    /// For now, assume normalized: Rect::new(0.0, 0.0, 1.0, 1.0) for full texture.
    pub source_rect: NovaRect<f32>,
}

#[derive(Debug)]
pub enum RenderElement<'a> {
    WaylandSurface {
        surface_wl: &'a WlSurface,
        surface_data_mutex_arc: Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>,
        geometry: Rectangle<i32, Logical>,
        damage_surface_local: Vec<Rectangle<i32, SmithayBuffer>>,
    },
    SolidColor {
        color: [f32; 4],
        geometry: Rectangle<i32, Logical>,
    },
    Cursor {
        texture_arc: Arc<dyn RenderableTexture>,
        position_logical: Point<i32, Logical>,
        hotspot_logical: Point<i32, Logical>,
    },
    /// Represents a texture node from the composition scene graph, ready for rendering.
    /// This variant owns its texture data via `Box<dyn RenderableTexture>`.
    TextureNode(TextureRenderParams),
}

pub trait FrameRenderer: 'static {
    // Constructor might vary, so it's not part of the trait directly
    // fn new(...) -> Result<Self, RendererError> where Self: Sized;

    fn id(&self) -> Uuid;

    fn render_frame<'a>(
        &mut self,
        elements: impl IntoIterator<Item = RenderElement<'a>>,
        output_geometry: Rectangle<i32, Physical>,
        output_scale: f64,
    ) -> Result<(), RendererError>;

    /// Submits all recorded commands for the current frame and schedules it for presentation.
    fn submit_and_present_frame(&mut self) -> Result<(), RendererError>;

    fn create_texture_from_shm(
        &mut self,
        buffer: &WlBuffer,
    ) -> Result<Box<dyn RenderableTexture>, RendererError>;

    fn create_texture_from_dmabuf(
        &mut self,
        dmabuf_attributes: &Dmabuf,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        Err(RendererError::Unsupported(
            "DMA-BUF texture creation is not supported by this renderer.".to_string(),
        ))
    }

    fn screen_size(&self) -> Size<i32, Physical>;

    /// Uploads or updates a texture for a given surface using client-provided buffer data.
    /// Returns a renderable texture representation.
    fn upload_surface_texture(
        &mut self,
        surface_id: SurfaceId, // Assuming SurfaceId from novade_compositor_core
        buffer: &ClientBuffer<'_>,
    ) -> Result<Box<dyn RenderableTexture>, RendererError>;

    // Optional: A method to release/destroy a surface texture when a surface is destroyed
    // fn release_surface_texture(&mut self, surface_id: SurfaceId) -> Result<(), RendererError>;

    /// Applies gamma correction to the current frame.
    fn apply_gamma_correction(&mut self, gamma_value: f32) -> Result<(), RendererError>;

    /// Applies HDR to SDR tone mapping.
    /// `max_luminance` could be the peak luminance of the HDR content or display.
    /// `exposure` is an adjustment factor.
    fn apply_hdr_to_sdr_tone_mapping(&mut self, max_luminance: f32, exposure: f32) -> Result<(), RendererError>;

    // Add more methods for other effects as needed in the future, e.g.:
    // fn apply_color_space_conversion(&mut self, target_space: ColorSpace) -> Result<(), RendererError>;
    // fn apply_anti_aliasing(&mut self, method: AntiAliasingMethod) -> Result<(), RendererError>;
    // fn apply_custom_effect(&mut self, effect_id: &str, params: &EffectParams) -> Result<(), RendererError>;
}
