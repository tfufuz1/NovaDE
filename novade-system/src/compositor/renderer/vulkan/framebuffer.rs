use crate::compositor::renderer::vulkan::device::LogicalDevice;
use ash::vk;
use log::{debug, info};

/// Creates framebuffers for the swapchain images.
///
/// Each framebuffer wraps a swapchain image view and a depth image view,
/// making them suitable as attachments for a render pass.
///
/// # Arguments
/// * `logical_device`: Reference to the `LogicalDevice`.
/// * `render_pass`: The `vk::RenderPass` that the framebuffers will be compatible with.
/// * `swapchain_image_views`: Slice of `vk::ImageView` handles for the swapchain images.
/// * `depth_image_view`: The `vk::ImageView` for the depth buffer.
/// * `swapchain_extent`: The dimensions of the swapchain images, also used for framebuffer dimensions.
///
/// # Returns
/// `Result<Vec<vk::Framebuffer>, String>` containing a vector of created framebuffer handles
/// or an error message.
pub fn create_framebuffers(
    logical_device: &LogicalDevice,
    render_pass: vk::RenderPass,
    swapchain_image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView, // Single depth view shared by all framebuffers
    swapchain_extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>, String> {
    info!(
        "Creating {} framebuffers for swapchain extent: {:?}",
        swapchain_image_views.len(),
        swapchain_extent
    );

    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for &swapchain_image_view in swapchain_image_views {
        let attachments = [swapchain_image_view, depth_image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);

        let framebuffer = unsafe {
            logical_device
                .raw
                .create_framebuffer(&framebuffer_create_info, None)
        }
        .map_err(|e| format!("Failed to create framebuffer: {}", e))?;
        
        debug!("Framebuffer created: {:?}", framebuffer);
        framebuffers.push(framebuffer);
    }

    info!("Successfully created {} framebuffers.", framebuffers.len());
    Ok(framebuffers)
}

// Note: Framebuffers are typically managed by the component that also manages the
// swapchain image views and depth buffer. They need to be destroyed and recreated
// when the swapchain is recreated (e.g., on window resize).
// A specific 'Framebuffers' struct with a Drop trait is not created here as they
// are often Vec<vk::Framebuffer> and their cleanup is tied to swapchain recreation logic.
// The caller of `create_framebuffers` is responsible for destroying these framebuffers
// using `logical_device.raw.destroy_framebuffer(framebuffer, None)` for each one.
