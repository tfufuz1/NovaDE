use ash::{vk, Device};
use std::sync::Arc;

pub struct RenderPass {
    device: Arc<Device>,
    pub handle: vk::RenderPass,
}

impl RenderPass {
    pub fn new(
        device: Arc<Device>,
        swapchain_format: vk::Format,
    ) -> Result<Self, anyhow::Error> {
        // Define Attachments
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        // Define Subpasses
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0) // Index of the color attachment
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref)) // Use std::slice::from_ref for single element
            .build();

        // Define Subpass Dependencies
        // This dependency ensures that the image layout transition happens correctly.
        // It waits for the previous operation on the color attachment (if any) to finish
        // before the render pass writes to it.
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before the render pass
            .dst_subpass(0) // Our first and only subpass
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty()) // Or vk::AccessFlags::default() if empty() is not available
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        // Create Render Pass
        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let handle = unsafe {
            device.create_render_pass(&render_pass_create_info, None)?
        };

        Ok(Self { device, handle })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.handle, None);
        }
    }
}
