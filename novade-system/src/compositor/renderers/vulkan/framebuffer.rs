//! Manages Vulkan `VkFramebuffer` objects.
//!
//! Framebuffers in Vulkan are collections of image views that serve as attachments
//! for a render pass. Each framebuffer typically includes one or more color attachments
//! (e.g., from the swapchain) and a depth/stencil attachment. This module provides
//! a utility function to create a set of framebuffers, one for each swapchain image view,
//! all compatible with a given render pass and sharing a common depth buffer view.

use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::vk;
use log::{debug, info};

/// Creates a set of Vulkan framebuffers.
///
/// This function generates a `Vec<vk::Framebuffer>`, where each framebuffer corresponds
/// to one of the provided `swapchain_image_views`. All created framebuffers will be
/// compatible with the specified `render_pass` and will share the same `depth_image_view`.
/// The dimensions of the framebuffers are determined by `swapchain_extent`.
///
/// Framebuffers are essential for defining the render targets for a `VkRenderPass`.
/// They must be recreated if the swapchain (and thus its image views or extent)
/// or the depth buffer changes.
///
/// # Arguments
///
/// * `logical_device`: A reference to the `LogicalDevice` used to create the framebuffers.
/// * `render_pass`: The `vk::RenderPass` with which these framebuffers must be compatible.
///   The attachments specified in the `render_pass` (e.g., color, depth) must match
///   the image views provided here in type and order.
/// * `swapchain_image_views`: A slice of `vk::ImageView` handles, typically one for each
///   image in the swapchain. Each of these will be used as the color attachment (attachment 0)
///   in a corresponding framebuffer.
/// * `depth_image_view`: The `vk::ImageView` for the depth buffer. This single view will be
///   used as the depth attachment (attachment 1) for all created framebuffers.
/// * `swapchain_extent`: The `vk::Extent2D` (width and height) of the swapchain images.
///   This also defines the dimensions of the created framebuffers.
///
/// # Returns
///
/// A `Result` containing a `Vec<vk::Framebuffer>` on success.
/// On failure, returns a `VulkanError` (typically `VulkanError::ResourceCreationError` or
/// `VulkanError::VkResult`) if any framebuffer creation fails.
///
/// # Safety
///
/// - The `logical_device`, `render_pass`, `swapchain_image_views`, and `depth_image_view`
///   must all be valid Vulkan handles.
/// - The image views must be compatible with the `render_pass` (e.g., format, sample count).
/// - The `swapchain_extent` must match the extent of the images referenced by the views.
/// - The caller is responsible for destroying each `vk::Framebuffer` in the returned vector
///   when they are no longer needed or before the associated image views/render pass are destroyed.
///   This is typically done when the swapchain is recreated.
pub fn create_framebuffers(
    logical_device: &LogicalDevice,
    render_pass: vk::RenderPass,
    swapchain_image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView,
    swapchain_extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    info!(
        "Creating {} framebuffers for swapchain extent: {:?}, render pass: {:?}",
        swapchain_image_views.len(), swapchain_extent, render_pass
    );

    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for (i, &swapchain_image_view) in swapchain_image_views.iter().enumerate() {
        // The order of attachments must match the order in the render pass definition.
        // Attachment 0: Color (swapchain image view)
        // Attachment 1: Depth (depth image view)
        let attachments = [swapchain_image_view, depth_image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1); // For non-stereoscopic 2D rendering

        let framebuffer = unsafe {
            logical_device.raw.create_framebuffer(&framebuffer_create_info, None)
        }.map_err(|e| VulkanError::ResourceCreationError {
            resource_type: "Framebuffer".to_string(),
            message: format!("Failed to create framebuffer for swapchain image view index {}: {:?}", i, e),
        })?; 
        
        debug!("Framebuffer {} created: {:?}", i, framebuffer);
        framebuffers.push(framebuffer);
    }

    info!("Successfully created {} framebuffers.", framebuffers.len());
    Ok(framebuffers)
}
