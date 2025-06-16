use ash::{vk, Device as AshDevice};
use std::sync::Arc;

// ANCHOR: VulkanRenderPass Struct Definition
pub struct VulkanRenderPass {
    device: Arc<AshDevice>,
    render_pass: vk::RenderPass,
}

// ANCHOR: VulkanRenderPass Implementation
impl VulkanRenderPass {
    pub fn new(device: Arc<AshDevice>, color_format: vk::Format) -> Result<Self, String> {
        // ANCHOR_EXT: Attachment Description
        let color_attachment = vk::AttachmentDescription::builder()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR) // For direct presentation
            .build();

        // ANCHOR_EXT: Color Attachment Reference
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0) // Index in the pAttachments array of RenderPassCreateInfo
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        // ANCHOR_EXT: Subpass Description
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            // No depth/stencil, input, resolve, or preserve attachments for this simple pass
            .build();

        // ANCHOR_EXT: Subpass Dependency
        // This dependency ensures that the render pass waits for the image to be available
        // (e.g., from the swapchain) before writing to it, and that writing is finished
        // before the image is presented.
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before render pass
            .dst_subpass(0) // Our subpass (index 0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty()) // No access required from previous stage for this simple case
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        // ANCHOR_EXT: Render Pass Create Info
        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency))
            .build();

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass_create_info, None)
                .map_err(|e| format!("Failed to create render pass: {}", e))?
        };

        println!("VulkanRenderPass created successfully.");
        Ok(Self { device, render_pass })
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
