// novade-system/src/renderers/mod.rs

pub mod wgpu_renderer;
pub mod wgpu_texture;
pub mod vulkan; // New Vulkan renderer module

pub use wgpu_renderer::NovaWgpuRenderer;
pub use wgpu_texture::WgpuRenderableTexture;
pub use vulkan::VulkanRenderer; // Expose VulkanRenderer
