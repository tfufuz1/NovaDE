// novade-system/src/renderer/wgpu_texture.rs

use crate::compositor::renderer_interface::abstraction::{RenderableTexture, RendererError};
use smithay::backend::renderer::utils::Fourcc; // For pixel format
use uuid::Uuid;
use std::sync::Arc; // If WGPU resources need to be shared

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

    fn bind(&self, _slot: u32) -> Result<(), RendererError> {
        // For WGPU, binding is typically handled by creating BindGroups
        // with the texture view and sampler. This method might not be directly
        // used in the same way as it is in OpenGL.
        // The renderer will use the view() and sampler() methods to set up BindGroups.
        tracing::trace!("WgpuRenderableTexture::bind called (slot {}), but actual binding is via BindGroups.", _slot);
        Ok(())
    }

    fn width_px(&self) -> u32 {
        self.width
    }

    fn height_px(&self) -> u32 {
        self.height
    }

    fn format(&self) -> Option<Fourcc> {
        self.fourcc_format
        // Or convert self.format (wgpu::TextureFormat) back to Fourcc if needed,
        // but storing the original Fourcc is more direct if available.
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
