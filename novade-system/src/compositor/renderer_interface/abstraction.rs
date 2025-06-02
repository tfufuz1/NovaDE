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

pub trait RenderableTexture: Send + Sync + std::fmt::Debug {
    fn id(&self) -> Uuid;
    fn bind(&self, slot: u32) -> Result<(), RendererError>;
    fn width_px(&self) -> u32;
    fn height_px(&self) -> u32;
    fn format(&self) -> Option<Fourcc>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug)]
pub enum RenderElement<'a> {
    WaylandSurface {
        surface_wl: &'a WlSurface,
        // surface_data_arc: Arc<SurfaceData>, // OLD
        surface_data_mutex_arc: Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>, // NEW
        geometry: Rectangle<i32, Logical>,
        damage_surface_local: Vec<Rectangle<i32, SmithayBuffer>>, // Use the renamed SmithayBuffer
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

    fn present_frame(&mut self) -> Result<(), RendererError>;

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
}
