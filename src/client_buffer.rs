// src/client_buffer.rs
use crate::allocator::Allocator;
use crate::device::LogicalDevice;
use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance; 
use crate::physical_device::PhysicalDeviceInfo;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use crate::buffer_utils::record_and_submit_command; // Added for layout transition

pub struct ClientVkBuffer {
    device: Arc<Device>, // For cleanup, this should be Arc<Device>
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    memory: vk::DeviceMemory, // Store the imported memory
    // pub sampler: vk::Sampler, // Optional: if this buffer is directly sampled
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
}

impl ClientVkBuffer {
    #[cfg(target_os = "linux")]
    pub fn from_dma_buf(
        instance_wrapper: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device_wrapper: &Arc<LogicalDevice>, // Expecting Arc<LogicalDevice>
        allocator: &Allocator, // Using &Allocator as per prompt
        fd: RawFd,
        width: u32,
        height: u32,
        vulkan_format: vk::Format,
        drm_format_modifier: Option<u64>,
    ) -> Result<Self> {
        let logical_device_raw_ref = logical_device_wrapper.raw().as_ref(); // Get &Device for calls
        let device_arc_for_struct = logical_device_wrapper.raw().clone(); // Get Arc<Device> for struct storage

        let (image, memory) = allocator.import_dma_buf_as_image(
            fd, width, height, vulkan_format, drm_format_modifier,
            vk::ImageUsageFlags::SAMPLED_BIT, // Assuming it will be sampled
            instance_wrapper.raw().instance(), // Pass &vulkanalia::Instance
            physical_device_info.raw(),        // Pass vk::PhysicalDevice
            logical_device_raw_ref,            // Pass &vulkanalia::Device
        )?;

        // Create an image view for it
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(vulkan_format)
            .subresource_range(vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR) // Assuming color
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1).build());
        let image_view = unsafe { logical_device_raw_ref.create_image_view(&image_view_info, None) }?;
        
        log::info!("ClientVkBuffer created from DMA-BUF fd: {}", fd);

        // TODO: Transition layout from UNDEFINED to SHADER_READ_ONLY_OPTIMAL
        // This needs a command pool and queue. This should be handled by the
        // component that uses this ClientVkBuffer (e.g., FrameRenderer before first use).
        // Example: Texture::transition_image_layout_internal_cmd(cmd_buffer, device_raw_ref, image, ...)
        // For now, the image remains in UNDEFINED layout.

        Ok(Self {
            device: device_arc_for_struct, // Store Arc<Device>
            image,
            image_view,
            memory,
            width,
            height,
            format: vulkan_format,
        })
    }
}

impl Drop for ClientVkBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.image_view, None);
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
        }
        log::debug!("ClientVkBuffer destroyed (image, view, memory).");
    }

    #[cfg(target_os = "linux")]
    pub fn transition_to_shader_read(
        &self,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
    ) -> Result<()> {
        // self.device is Arc<Device>, we need &Device for record_and_submit_command
        let device_ref = self.device.as_ref(); 

        record_and_submit_command(device_ref, command_pool, queue, |cmd_buffer| {
            let barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.image)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0).level_count(1)
                    .base_array_layer(0).layer_count(1).build())
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::SHADER_READ);

            unsafe { device_ref.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier.build()], // build() the barrier before passing as slice
            )};
            Ok(()) // Ensure the closure returns Result<()>
        })?; // Propagate error from record_and_submit_command
        log::debug!("ClientVkBuffer image transitioned to SHADER_READ_ONLY_OPTIMAL.");
        Ok(())
    }
}
