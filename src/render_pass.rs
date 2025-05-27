use crate::error::{Result, VulkanError};
use crate::device::LogicalDevice;
use crate::instance::VulkanInstance; // Needed for find_supported_format
use crate::physical_device::PhysicalDeviceInfo; // Needed for find_supported_format indirectly
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;

pub struct RenderPass {
    device: Arc<Device>,
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(
        instance_wrapper: &VulkanInstance, // For physical device properties
        physical_device_info: &PhysicalDeviceInfo, // For physical device handle
        logical_device_wrapper: &LogicalDevice,
        swapchain_format: vk::Format,
    ) -> Result<Self> {
        let device = logical_device_wrapper.raw().clone();
        let physical_device = physical_device_info.raw(); // Get the raw physical device
        let instance = instance_wrapper.raw(); // Get the raw instance for format properties

        // 1. Color Attachment Description
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR); // Ready for presentation

        // 2. Depth Attachment Description
        let depth_format = Self::find_depth_format(instance.instance(), physical_device)?; // Pass &Instance
        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE) // We don't need to store depth after this pass (for now)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        // 3. Attachment References
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0) // Index in the attachments array
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1) // Index in the attachments array
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        // 4. Subpass Description
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref); // Set depth attachment

        // 5. Subpass Dependencies
        // This dependency ensures that the image layout transitions happen correctly
        // before and after the render pass.
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before render pass
            .dst_subpass(0)                   // Our subpass (index 0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | // Wait for color output stage
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS     // Or early fragment tests for depth
            )
            .src_access_mask(vk::AccessFlags::empty()) // No specific access from src needed if clearing
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | // Operations in our subpass
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE |        // We will write to color attachment
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE  // And depth attachment
            );
            // Note: More dependencies might be needed for complex scenarios or if PRESENT_SRC_KHR
            // transition isn't handled by this single dependency or by swapchain image ownership transfers.
            // For now, this covers the transition to a writable state.
            // The finalLayout of attachments handles the transition out of the render pass.

        let attachments = &[color_attachment, depth_attachment];
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }
            .map_err(VulkanError::VkResult)?;
        
        log::info!("Basic render pass created with color and depth attachments.");

        Ok(Self { device, render_pass })
    }

    // Helper to find a supported depth format
    fn find_depth_format(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::Format> {
        let candidates = vec![
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT, // If stencil is needed later
            vk::Format::D24_UNORM_S8_UINT,
        ];

        for format in candidates {
            let properties = unsafe {
                instance.get_physical_device_format_properties(physical_device, format)
            };
            // Check for optimal tiling features for depth attachment
            if properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                return Ok(format);
            }
        }
        Err(VulkanError::Message("Failed to find suitable depth format.".to_string()))
    }

    pub fn raw(&self) -> vk::RenderPass {
        self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe { self.device.destroy_render_pass(self.render_pass, None) };
        log::debug!("Render pass destroyed.");
    }
}
