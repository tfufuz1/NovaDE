// novade-system/src/renderer_interface.rs
use novade_core::types::geometry::Size2D;
// Placeholder for SurfaceRenderData, adapt as needed
// pub struct SurfaceRenderData { ... }

// #[cfg_attr(test, mockall::automock)] // For mocking in tests
pub trait RendererInterface {
    fn begin_frame(&mut self);
    // fn upload_textures_for_surfaces(&mut self, elements: &[/* SurfaceRenderData or similar */]);
    // fn draw_elements(&mut self, elements: &[/* SurfaceRenderData or similar */]);
    fn submit_frame(&mut self);
    fn present(&mut self);
    fn resize(&mut self, new_size: Size2D);
    // Add other necessary methods like:
    // fn create_texture_from_buffer(...) -> Result<TextureId, Error>;
    // fn destroy_texture(&mut self, texture_id: TextureId);
    // fn get_output_details(...) -> ...;
}

// Basic mock for testing if mockall is not yet set up
#[cfg(test)]
pub struct MockRenderer {
    pub frame_begun: bool,
    pub frame_submitted: bool,
    pub frame_presented: bool,
    pub resized_to: Option<Size2D>,
}

#[cfg(test)]
impl MockRenderer {
    pub fn new() -> Self {
        MockRenderer {
            frame_begun: false,
            frame_submitted: false,
            frame_presented: false,
            resized_to: None,
        }
    }
}

#[cfg(test)]
impl RendererInterface for MockRenderer {
    fn begin_frame(&mut self) { self.frame_begun = true; }
    fn submit_frame(&mut self) { self.frame_submitted = true; }
    fn present(&mut self) { self.frame_presented = true; }
    fn resize(&mut self, new_size: Size2D) { self.resized_to = Some(new_size); }
}
