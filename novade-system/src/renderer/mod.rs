// novade-system/src/renderer/mod.rs
pub mod vulkan;

// Re-export key types for easier access from outside the renderer module, if desired.
// For example, if other parts of novade-system need direct access to VulkanCoreContext or VulkanError:
// pub use self::vulkan::VulkanCoreContext;
// pub use self::vulkan::error::VulkanError;
// pub use self::vulkan::QueueFamilyIndices; // If needed externally
//
// For now, direct usage will be novade_system::renderer::vulkan::VulkanCoreContext, etc.
// Re-exports can be added as the design evolves and integration points become clearer.
