use crate::error::{Result, VulkanError};
use crate::device::LogicalDevice;
use std::sync::Arc; // Not strictly needed here, but good for consistency if Arc<Device> was stored.
use vulkanalia::prelude::v1_0::*;

// Function to create framebuffers.
// This function is designed to be called when the swapchain is created or recreated.
// The caller (e.g., FrameRenderer or SurfaceSwapchain) will manage the lifetime of these framebuffers.
pub fn create_framebuffers(
    logical_device_wrapper: &LogicalDevice,
    swapchain_image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView, // The single depth image view
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    let device = logical_device_wrapper.raw();

    swapchain_image_views
        .iter()
        .map(|&color_image_view| {
            // The order of attachments must match the order of attachment descriptions
            // in the render pass (color attachment at index 0, depth at index 1).
            let attachments = &[color_image_view, depth_image_view]; 

            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&framebuffer_info, None) }
                .map_err(VulkanError::VkResult)
        })
        .collect::<std::result::Result<Vec<_>, _>>() // Collect into Result<Vec<Framebuffer>, VulkanError>
}

// Note: Framebuffer destruction is not handled in this module directly.
// The owner of the framebuffers (likely FrameRenderer or a component managing swapchain resources)
// will be responsible for iterating through the Vec<vk::Framebuffer> and calling
// `device.destroy_framebuffer(framebuffer, None)` when they are no longer needed
// (e.g., during swapchain recreation or application shutdown).
