// novade-system/src/renderer/mod.rs

// This module will house the new WGPU renderer and related components.
// Sub-modules like wgpu_renderer, wgpu_texture, etc., will be declared here.

pub mod wgpu_renderer;
pub mod wgpu_texture; // Add this line

pub use wgpu_renderer::NovaWgpuRenderer;
pub use wgpu_texture::WgpuRenderableTexture; // Add this line
