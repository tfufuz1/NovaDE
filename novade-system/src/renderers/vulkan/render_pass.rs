use ash::{vk, Device as AshDevice};
use std::sync::Arc;

// ANCHOR: VulkanRenderPass Struct Definition
pub struct VulkanRenderPass {
    device: Arc<AshDevice>,
    render_pass: vk::RenderPass,
}

/// Creates a simple Vulkan Render Pass suitable for basic rendering operations.
///
/// This render pass is configured with a single color attachment. It defines how the
/// attachment is handled at the beginning (load operation: clear) and end (store operation: store)
/// of the pass, as well as its layout transitions. The final layout is set to
/// `PRESENT_SRC_KHR`, making it suitable for images that will be presented to a swapchain.
///
/// This implementation directly follows the requirements outlined in
/// `Rendering Vulkan.md` (Spec 6.3 - Render-Pass-Definition).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `color_format`: The `vk::Format` of the color attachment. This should typically match
///   the format of the swapchain images.
///
/// # Returns
/// A `Result` containing the created `vk::RenderPass` handle, or an error string
/// if creation fails.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 6.3):
/// - **`VkAttachmentDescription`**:
///   - `format`: Matches `color_format` parameter.
///   - `samples`: `VK_SAMPLE_COUNT_1_BIT` (no multisampling).
///   - `loadOp`: `VK_ATTACHMENT_LOAD_OP_CLEAR` (to clear the screen).
///   - `storeOp`: `VK_ATTACHMENT_STORE_OP_STORE` (to store the result for presentation).
///   - `stencilLoadOp`/`stencilStoreOp`: `VK_ATTACHMENT_LOAD_OP_DONT_CARE` (no stencil).
///   - `initialLayout`: `VK_IMAGE_LAYOUT_UNDEFINED` (previous content is not needed).
///   - `finalLayout`: `VK_IMAGE_LAYOUT_PRESENT_SRC_KHR` (for swapchain presentation).
/// - **`VkAttachmentReference`**:
///   - `attachment`: 0 (references the single color attachment).
///   - `layout`: `VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL` (optimal for rendering).
/// - **`VkSubpassDescription`**:
///   - `pipelineBindPoint`: `VK_PIPELINE_BIND_POINT_GRAPHICS`.
///   - `colorAttachments`: Points to the color attachment reference.
/// - **`VkSubpassDependency`**: A basic dependency is set up to handle layout transitions
///   from an external source to the color attachment write state.
pub fn create_vulkan_render_pass(
    device: &AshDevice,
    color_format: vk::Format,
/// This render pass is configured with a single color attachment, suitable for
/// clearing the screen and presenting to a swapchain. It defines load operations,
/// store operations, and layout transitions for the attachment.
/// Aligns with `Rendering Vulkan.md` (Spec 6.3).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `color_format`: The `vk::Format` of the color attachment (typically the swapchain image format).
///
/// # Returns
/// A `Result` containing the created `vk::RenderPass` or an error string on failure.
pub fn create_vulkan_render_pass(
    device: &AshDevice,
    color_format: vk::Format,
) -> Result<vk::RenderPass, String> {
    // Attachment Description (Spec 6.3)
    // Describes the properties of the color attachment.
    let color_attachment = vk::AttachmentDescription::builder()
        .format(color_format) // Format of the swapchain images
        .samples(vk::SampleCountFlags::TYPE_1) // No multisampling
        .load_op(vk::AttachmentLoadOp::CLEAR) // Clear the attachment at the beginning of the pass
        .store_op(vk::AttachmentStoreOp::STORE) // Store the rendered content for presentation
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE) // No stencil buffer
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE) // No stencil buffer
        .initial_layout(vk::ImageLayout::UNDEFINED) // We don't care about the previous content
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR); // Transition to present layout after the pass

    // Attachment Reference (Spec 6.3)
    // Specifies which attachment to use for the subpass and its layout during the subpass.
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0) // Index of the attachment in the `attachments` array
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL); // Layout optimal for color rendering

    // Subpass Description (Spec 6.3)
    // Defines a single subpass that uses the color attachment.
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS) // Graphics pipeline
        .color_attachments(std::slice::from_ref(&color_attachment_ref));

    // Subpass Dependency (Spec 6.3, though details may vary)
    // Ensures proper synchronization between operations outside and inside the render pass.
    // This dependency handles the transition from an external state (e.g., swapchain image acquired)
    // to the state where the color attachment can be written to.
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before this render pass
        .dst_subpass(0) // Index of our subpass
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) // Stage where previous writes to image might have occurred (or semaphore signal)
        .src_access_mask(vk::AccessFlags::empty()) // No specific access from src needed if initialLayout is UNDEFINED and loadOp is CLEAR
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) // Stage where our subpass writes
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE); // Access type for our subpass

    // Render Pass Create Info (Spec 6.3)
    // Combines attachments, subpasses, and dependencies to define the render pass.
    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .map_err(|e| format!("Failed to create render pass: {}", e))
    }
}


// ANCHOR: VulkanRenderPass Implementation
impl VulkanRenderPass {
    /// Creates a new `VulkanRenderPass` instance.
    ///
    /// This constructor wraps the `create_vulkan_render_pass` free function.
    ///
    /// # Arguments
    /// * `device`: An `Arc` reference to the logical `ash::Device`.
    /// * `color_format`: The `vk::Format` of the color attachment.
    ///
    /// # Returns
    /// A `Result` containing the new `VulkanRenderPass` or an error string.
    pub fn new(device: Arc<AshDevice>, color_format: vk::Format) -> Result<Self, String> {
        let render_pass_handle = create_vulkan_render_pass(device.as_ref(), color_format)?;
        println!("VulkanRenderPass created successfully (via new method wrapping free function).");
        Ok(Self { device, render_pass: render_pass_handle })
    }

    // ANCHOR: Accessor for vk::RenderPass
    pub fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }
}

// ANCHOR: VulkanRenderPass Drop Implementation
impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
        }
        println!("VulkanRenderPass dropped.");
    }
}
