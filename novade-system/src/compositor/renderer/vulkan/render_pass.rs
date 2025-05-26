use crate::compositor::renderer::vulkan::device::LogicalDevice;
use ash::vk;
use log::{debug, info};

/// Represents a Vulkan Render Pass.
#[derive(Debug)]
pub struct RenderPass {
    pub raw: vk::RenderPass,
    logical_device_raw: ash::Device, // Keep a clone for Drop
}

impl RenderPass {
    /// Creates a new Vulkan Render Pass.
    ///
    /// # Arguments
    /// * `logical_device`: Reference to the `LogicalDevice`.
    /// * `swapchain_format`: The format of the swapchain images (color attachment).
    /// * `depth_format`: The format used for the depth attachment.
    ///
    /// # Returns
    /// `Result<Self, String>` containing the new `RenderPass` or an error message.
    pub fn new(
        logical_device: &LogicalDevice,
        swapchain_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self, String> {
        info!(
            "Creating render pass with swapchain format: {:?}, depth format: {:?}",
            swapchain_format, depth_format
        );

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR) // For presentation
            .build();

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE) // Depth often not needed after render pass
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let attachments = [color_attachment, depth_attachment];

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0) // Index of the color attachment in the attachments array
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1) // Index of the depth attachment
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();
        
        let color_attachment_refs = [color_attachment_ref];

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs) // Reference to color attachments
            .depth_stencil_attachment(&depth_attachment_ref) // Reference to depth attachment
            .build();
        
        let subpasses = [subpass];

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before render pass
            .dst_subpass(0) // Our first and only subpass
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty()) // No access required from previous stages for clear
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .build();
            
        let dependencies = [dependency];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let raw_render_pass = unsafe {
            logical_device
                .raw
                .create_render_pass(&render_pass_create_info, None)
        }
        .map_err(|e| format!("Failed to create render pass: {}", e))?;
        debug!("Render pass created: {:?}", raw_render_pass);

        Ok(Self {
            raw: raw_render_pass,
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        debug!("Dropping render pass: {:?}", self.raw);
        unsafe {
            self.logical_device_raw.destroy_render_pass(self.raw, None);
        }
        debug!("Render pass {:?} destroyed.", self.raw);
    }
}
