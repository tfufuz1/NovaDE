// novade-system/src/renderer/wgpu_texture.rs

use crate::compositor::renderer_interface::abstraction::RenderableTexture;
use crate::compositor::renderer_interface::abstraction::RendererError;
use smithay::backend::renderer::utils::Fourcc;
use uuid::Uuid;
use std::sync::Arc;
use std::any::Any; // For as_any()

#[derive(Debug)]
pub struct WgpuRenderableTexture {
    id: Uuid,
    device: Arc<wgpu::Device>,

    // For single-plane or as primary plane (e.g., Luma for YUV)
    primary_texture: Arc<wgpu::Texture>,
    primary_view: Arc<wgpu::TextureView>,

    // For multi-planar textures (e.g., Y, U, V planes or Y, UV planes)
    // If single-planar, these might be empty or contain clones of primary.
    pub plane_textures: Vec<Arc<wgpu::Texture>>,
    pub plane_views: Vec<Arc<wgpu::TextureView>>,
    pub is_multi_planar: bool,

    // Sampler is often shared or configured per draw call, not strictly per plane texture.
    // This sampler can be used for the primary texture or as a default.
    sampler: Arc<wgpu::Sampler>,

    width: u32, // Overall width of the image represented by this texture
    height: u32, // Overall height of the image

    // Format of the primary_texture, or a representative format for multi-planar.
    // For multi-planar, individual plane formats might differ (e.g., R8 for Y, U, V planes).
    format: wgpu::TextureFormat,
    fourcc_format: Option<Fourcc>,
}

impl WgpuRenderableTexture {
    #[allow(clippy::too_many_arguments)] // To accommodate all necessary fields for initialization
    pub fn new(
        device: Arc<wgpu::Device>,
        initial_texture: wgpu::Texture, // This will be the primary texture
        initial_view: wgpu::TextureView,   // View for the primary texture
        sampler: wgpu::Sampler,
        width: u32, // Overall width
        height: u32, // Overall height
        format: wgpu::TextureFormat, // Format of the initial_texture
        fourcc_format: Option<Fourcc>,
        is_multi_planar: bool,
        // For multi-planar, plane_textures & plane_views are typically populated by renderer logic
        // after calling new for the primary plane, then adding other planes.
        // Or, if all planes are created simultaneously, they could be passed here.
        // For this constructor, we'll initialize them based on is_multi_planar.
        initial_plane_textures: Option<Vec<Arc<wgpu::Texture>>>,
        initial_plane_views: Option<Vec<Arc<wgpu::TextureView>>>,
    ) -> Self {
        let primary_texture_arc = Arc::new(initial_texture);
        let primary_view_arc = Arc::new(initial_view);

        let plane_textures = initial_plane_textures.unwrap_or_else(|| {
            if is_multi_planar {
                Vec::new() // Expecting renderer to populate
            } else {
                vec![primary_texture_arc.clone()]
            }
        });

        let plane_views = initial_plane_views.unwrap_or_else(|| {
            if is_multi_planar {
                Vec::new() // Expecting renderer to populate
            } else {
                vec![primary_view_arc.clone()]
            }
        });

        Self {
            id: Uuid::new_v4(),
            device,
            primary_texture: primary_texture_arc,
            primary_view: primary_view_arc,
            plane_textures,
            plane_views,
            is_multi_planar,
            sampler: Arc::new(sampler),
            width,
            height,
            format,
            fourcc_format,
        }
    }

    // Returns the primary view (e.g., for single-plane or Luma plane)
    pub fn view(&self) -> &wgpu::TextureView {
        &self.primary_view
    }

    // Returns a specific plane view if available
    pub fn get_plane_view(&self, plane_index: usize) -> Option<&wgpu::TextureView> {
        self.plane_views.get(plane_index).map(|arc_view| &**arc_view)
    }

    // Method to get the WGPU sampler
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    // Returns the primary texture (e.g., for single-plane or Luma plane)
    pub fn texture(&self) -> &wgpu::Texture {
        &self.primary_texture
    }

    // Returns a specific plane texture if available
    pub fn get_plane_texture(&self, plane_index: usize) -> Option<&wgpu::Texture> {
        self.plane_textures.get(plane_index).map(|arc_tex| &**arc_tex)
    }
}

impl RenderableTexture for WgpuRenderableTexture {
    fn id(&self) -> Uuid {
        self.id
    }

    fn width_px(&self) -> u32 {
        self.width
    }

    fn height_px(&self) -> u32 {
        self.height
    }

    fn format(&self) -> Option<Fourcc> {
        self.fourcc_format
    }

    fn bind(&self, _slot: u32) -> Result<(), RendererError> {
        tracing::trace!("RenderableTexture::bind called on WgpuRenderableTexture (id: {}), which is a no-op for WGPU.", self.id);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Optional: Implement Drop if specific WGPU resource cleanup is needed
// impl Drop for WgpuRenderableTexture {
//     fn drop(&mut self) {
//         tracing::trace!("Dropping WgpuRenderableTexture (id: {})", self.id);
//     }
// }
