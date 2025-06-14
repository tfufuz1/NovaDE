// novade-system/src/renderer/wgpu_texture.rs

// Use the RenderableTexture trait from compositor::state
use crate::compositor::state::RenderableTexture;
// Keep RendererError if it's defined in renderer_interface::abstraction and used by other parts of WgpuRenderer
use crate::compositor::renderer_interface::abstraction::RendererError;
use smithay::backend::renderer::utils::Fourcc; // For pixel format
use uuid::Uuid;
use std::sync::Arc;
use std::any::Any; // For as_any()

#[derive(Debug)]
pub struct WgpuRenderableTexture {
    id: Uuid,
    device: Arc<wgpu::Device>, // Keep a reference to the device for potential future operations
    texture: Arc<wgpu::Texture>, // Arc if texture might be shared or needs explicit drop control
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>, // Each texture might have its own sampler settings
    width: u32,
    height: u32,
    format: wgpu::TextureFormat, // Store the WGPU format
    fourcc_format: Option<Fourcc>, // Store the original FourCC if known (e.g. from DMABUF)
}

impl WgpuRenderableTexture {
    pub fn new(
        device: Arc<wgpu::Device>,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        fourcc_format: Option<Fourcc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            device,
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: Arc::new(sampler),
            width,
            height,
            format,
            fourcc_format,
        }
    }

    // Method to get the WGPU texture view, e.g., for binding in render pass
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    // Method to get the WGPU sampler
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    // Method to get the WGPU texture itself
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

impl RenderableTexture for WgpuRenderableTexture {
    fn id(&self) -> Uuid {
        self.id
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Optional: Implement Drop if specific WGPU resource cleanup is needed
// beyond what Arc handles, though typically Arc<wgpu::Texture>, etc. is fine.
// impl Drop for WgpuRenderableTexture {
//     fn drop(&mut self) {
//         tracing::trace!("Dropping WgpuRenderableTexture (id: {})", self.id);
//         // WGPU resources are automatically reclaimed when their Arcs are dropped.
//     }
// }
