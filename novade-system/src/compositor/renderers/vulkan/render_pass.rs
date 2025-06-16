//! Manages Vulkan `VkRenderPass` objects.
//!
//! A render pass in Vulkan describes the set of attachments, subpasses, and dependencies
//! used during rendering operations. It specifies how color, depth/stencil, and other
///! attachments are used, cleared, stored, and transitioned between layouts.
//! This module provides a `RenderPass` struct to encapsulate a `VkRenderPass` handle
//! and its creation logic.

use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::vk;
use log::{debug, info};

/// Represents a Vulkan Render Pass object.
///
/// A `VkRenderPass` defines a collection of attachments, subpasses, and dependencies
/// between the subpasses. It describes how the attachments are used over the course
/// of the subpasses. For example, it specifies which attachments will be read from
/// and written to, and what their layouts should be at the beginning and end of each subpass.
///
/// This struct holds the `vk::RenderPass` handle and a clone of the `ash::Device`
/// handle for automatic cleanup via the `Drop` trait.
#[derive(Debug)]
pub struct RenderPass {
    /// The raw Vulkan `vk::RenderPass` handle.
    pub raw: vk::RenderPass,
    /// A clone of the `ash::Device` handle used to create this render pass,
    /// kept for resource cleanup in the `Drop` implementation.
    logical_device_raw: ash::Device,
}

impl RenderPass {
    /// Creates a new Vulkan Render Pass.
    ///
    /// This function configures and creates a `VkRenderPass` suitable for typical
    /// forward rendering, including a color attachment (for the swapchain image) and
    /// a depth attachment.
    ///
    /// The render pass is configured with:
    /// - **Color Attachment:**
    ///   - Format: `swapchain_format`.
    ///   - Load Operation: `CLEAR` (clears the attachment at the start of the pass).
    ///   - Store Operation: `STORE` (stores the rendered content for presentation).
    ///   - Initial Layout: `UNDEFINED` (content before the pass is not important).
    ///   - Final Layout: `PRESENT_SRC_KHR` (suitable for presenting to the swapchain).
    /// - **Depth Attachment:**
    ///   - Format: `depth_format`.
    ///   - Load Operation: `CLEAR` (clears the depth buffer at the start of the pass).
    ///   - Store Operation: `DONT_CARE` (depth values are often not needed after the pass).
    ///   - Initial Layout: `UNDEFINED`.
    ///   - Final Layout: `DEPTH_STENCIL_ATTACHMENT_OPTIMAL`.
    /// - **Subpass:** A single subpass is defined which uses the color attachment as
    ///   `COLOR_ATTACHMENT_OPTIMAL` and the depth attachment as `DEPTH_STENCIL_ATTACHMENT_OPTIMAL`.
    /// - **Subpass Dependency:** A dependency is set up for the transition from `SUBPASS_EXTERNAL`
    ///   (before the render pass) to the first subpass (index 0). This ensures that writes to
    ///   color and depth attachments can proceed after appropriate layout transitions and cache flushes.
    ///   The source stage mask includes `COLOR_ATTACHMENT_OUTPUT | EARLY_FRAGMENT_TESTS` and source access mask is empty.
    ///   The destination stage mask is also `COLOR_ATTACHMENT_OUTPUT | EARLY_FRAGMENT_TESTS` and destination access mask
    ///   is `COLOR_ATTACHMENT_WRITE | DEPTH_STENCIL_ATTACHMENT_WRITE`.
    ///
    /// # Arguments
    ///
    /// * `logical_device`: A reference to the `LogicalDevice` used to create the render pass.
    /// * `swapchain_format`: The `vk::Format` of the swapchain images, which will be used as the color attachment.
    /// * `depth_format`: The `vk::Format` to be used for the depth attachment.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RenderPass` struct on success, or a `VulkanError`
    /// (typically `VulkanError::VkResult`) if `vkCreateRenderPass` fails.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `logical_device` is a valid Vulkan logical device.
    /// The `swapchain_format` and `depth_format` must be supported by the physical device
    /// for use as color and depth/stencil attachments, respectively.
    pub fn new(
        logical_device: &LogicalDevice,
        swapchain_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self> {
        info!(
            "Creating render pass with swapchain format: {:?}, depth format: {:?}",
            swapchain_format, depth_format
        );

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE).stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE).stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let attachments = [color_attachment.build(), depth_attachment.build()];

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL); // Index into attachments array
        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1).layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL); // Index into attachments array
        
        let color_attachment_refs = [color_attachment_ref.build()];

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs) 
            .depth_stencil_attachment(&depth_attachment_ref.build());
        
        let subpasses = [subpass.build()];

        // Dependency to ensure layout transition happens before render pass begins
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL) // Implicit subpass before this render pass
            .dst_subpass(0) // Our first (and only) subpass
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty()) // No access needed from previous external operations for clear
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);
            
        let dependencies = [dependency.build()];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments).subpasses(&subpasses).dependencies(&dependencies);

        let raw_render_pass = unsafe {
            logical_device.raw.create_render_pass(&render_pass_create_info, None)
        }?; // Uses From<vk::Result> for VulkanError
        debug!("Render pass created: {:?}", raw_render_pass);

        Ok(Self {
            raw: raw_render_pass,
            logical_device_raw: logical_device.raw.clone(),
        })
    }

    /// Creates a new Vulkan Render Pass with only a color attachment.
    ///
    /// This is suitable for passes that don't require depth testing, such as a final
    /// blit to the swapchain or some 2D post-processing effects.
    ///
    /// The color attachment is configured for:
    ///   - Format: `color_format`.
    ///   - Load Operation: `CLEAR` or `DONT_CARE` (configurable, typically DONT_CARE for blit if fullscreen).
    ///     For simplicity, using CLEAR here. A more advanced version could parameterize this.
    ///   - Store Operation: `STORE`.
    ///   - Initial Layout: `UNDEFINED`.
    ///   - Final Layout: `PRESENT_SRC_KHR` (if targeting swapchain) or `SHADER_READ_ONLY_OPTIMAL` (if intermediate).
    ///     Using `PRESENT_SRC_KHR` as this is typical for a final pass to swapchain.
    pub fn new_color_only(
        logical_device: &LogicalDevice,
        color_format: vk::Format,
        final_layout: vk::ImageLayout, // e.g., PRESENT_SRC_KHR or SHADER_READ_ONLY_OPTIMAL
    ) -> Result<Self> {
        info!(
            "Creating color-only render pass with format: {:?}, final_layout: {:?}",
            color_format, final_layout
        );

        let color_attachment = vk::AttachmentDescription::builder()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE) // Common for blit target, or CLEAR if it's the first thing drawn
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED) // Or specific if transitioning from something
            .final_layout(final_layout);

        let attachments = [color_attachment.build()];

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachment_refs = [color_attachment_ref.build()];

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs); // No depth attachment

        let subpasses = [subpass.build()];

        // Dependency to ensure layout transition for color attachment
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let dependencies = [dependency.build()];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let raw_render_pass = unsafe {
            logical_device.raw.create_render_pass(&render_pass_create_info, None)
        }?;
        debug!("Color-only render pass created: {:?}", raw_render_pass);

        Ok(Self {
            raw: raw_render_pass,
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for RenderPass {
    /// Cleans up the Vulkan `VkRenderPass` resource.
    ///
    /// # Safety
    ///
    /// - The `logical_device_raw` handle stored within this struct must still be a valid
    ///   `ash::Device` when `Drop` is called.
    /// - The caller must ensure that the `VkRenderPass` is not in use by any pending
    ///   GPU operations when it's dropped.
    fn drop(&mut self) {
        debug!("Dropping render pass: {:?}", self.raw);
        unsafe {
            self.logical_device_raw.destroy_render_pass(self.raw, None);
        }
        debug!("Render pass {:?} destroyed.", self.raw);
    }
}
