// novade-system/src/renderer/mod.rs

// This module will house the new WGPU renderer and related components.
// Sub-modules like wgpu_renderer, wgpu_texture, etc., will be declared here.

pub mod wgpu_renderer;
pub mod wgpu_texture; // Add this line
pub mod vulkan_frame_renderer;

pub use wgpu_renderer::NovaWgpuRenderer;
pub use wgpu_texture::WgpuRenderableTexture; // Add this line
pub use vulkan_frame_renderer::VulkanContext;
pub use vulkan_frame_renderer::FrameRenderer;
pub use vulkan_frame_renderer::VulkanTexture;
pub use vulkan_frame_renderer::RenderElement;
pub use vulkan_frame_renderer::VulkanError;
