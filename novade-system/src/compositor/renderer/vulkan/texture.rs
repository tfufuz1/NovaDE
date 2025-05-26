//! Manages Vulkan image and sampler resources, primarily for 2D textures.
//!
//! This module provides the `Texture` struct for loading image files (e.g., PNG, JPEG)
//! into Vulkan `VkImage` objects, complete with mipmap generation, `VkImageView`,
//! and `VkSampler`. It handles staging buffer transfers for optimal device-local
//! memory usage and includes utilities for common image operations like layout transitions
//! via one-time command buffer submissions. It also offers functionality to create
//! generic storage images suitable for compute shader inputs/outputs or as attachments.

use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    // device::LogicalDevice, // No longer needed directly by all functions here
    physical_device::PhysicalDeviceInfo,
    error::{Result, VulkanError},
};
use ash::vk;
use image::GenericImageView;
use log::{debug, info, error, warn};
use std::path::Path;
use vk_mem;

/// Helper function to execute commands within a one-time submit command buffer.
///
/// # Arguments
///
/// * `device_raw`: A reference to the `ash::Device` (logical device handle).
/// * `command_pool`: The `vk::CommandPool` from which the command buffer will be allocated.
/// * `queue`: The `vk::Queue` to which the command buffer will be submitted.
/// * `executor`: A closure that takes the allocated `vk::CommandBuffer` as an argument.
pub fn record_one_time_submit_commands<F>(
    device_raw: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    executor: F,
) -> Result<()>
where
    F: FnOnce(vk::CommandBuffer),
{
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = unsafe { device_raw.allocate_command_buffers(&allocate_info) }?[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device_raw.begin_command_buffer(command_buffer, &begin_info) }?;
    executor(command_buffer);
    unsafe { device_raw.end_command_buffer(command_buffer) }?;

    let submit_info = vk::SubmitInfo::builder().command_buffers(&[command_buffer]);
    let fence = unsafe { device_raw.create_fence(&vk::FenceCreateInfo::builder(), None)}?;

    unsafe {
        device_raw.queue_submit(queue, &[submit_info.build()], fence)?;
        device_raw.wait_for_fences(&[fence], true, u64::MAX)?;
    }

    unsafe {
        device_raw.destroy_fence(fence, None);
        device_raw.free_command_buffers(command_pool, &[command_buffer]);
    }
    debug!("One-time submit commands executed successfully.");
    Ok(())
}

#[derive(Debug)]
pub struct Texture {
    pub image: vk::Image,
    pub allocation: vk_mem::Allocation,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub extent: vk::Extent2D,
    pub mip_levels: u32,
    pub format: vk::Format,
    logical_device_raw: ash::Device,
    allocator_raw_clone: vk_mem::Allocator, 
}

impl Texture {
    /// Creates a new general-purpose Vulkan image, typically for use as a storage image or render target.
    ///
    /// # Arguments
    ///
    /// * `device_raw`: A reference to the `ash::Device` (logical device handle).
    /// * `allocator`: A reference to the VMA `Allocator` for memory management.
    /// * `width`: The width of the image to be created.
    /// * `height`: The height of the image to be created.
    /// * `format`: The `vk::Format` of the image.
    /// * `usage`: The `vk::ImageUsageFlags` specifying how the image will be used.
    pub fn new_storage_image(
        device_raw: &ash::Device,
        allocator: &Allocator,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<(vk::Image, vk_mem::Allocation, vk::ImageView)> {
        info!("Creating new storage image: {}x{}, format: {:?}, usage: {:?}", width, height, format, usage);

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D).format(format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL).usage(usage)
            .initial_layout(vk::ImageLayout::UNDEFINED).sharing_mode(vk::SharingMode::EXCLUSIVE);

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default()
        };

        let (image, allocation, _alloc_info) = allocator
            .create_image(&image_create_info, &allocation_create_info)
            .map_err(|e| VulkanError::ResourceCreationError {
                resource_type: "StorageImage".to_string(),
                message: format!("VMA failed to create storage image: {}", e)
            })?;
        debug!("Storage image created: {:?}", image);

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image).view_type(vk::ImageViewType::TYPE_2D).format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0,
                level_count: 1, base_array_layer: 0, layer_count: 1,
            });
        
        let image_view = unsafe { device_raw.create_image_view(&view_create_info, None) }?;
        debug!("Image view created for storage image: {:?}", image_view);
        
        Ok((image, allocation, image_view))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_from_file(
        device_raw: &ash::Device, // Changed from logical_device: &LogicalDevice
        physical_device_info: &PhysicalDeviceInfo,
        allocator: &Allocator,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue, // Changed from graphics_queue
        image_path: &str,
        vulkan_instance_raw: &ash::Instance,
    ) -> Result<Self> {
        info!("Loading texture from file with mipmapping: {}", image_path);

        let img = image::open(Path::new(image_path))
            .map_err(|e| VulkanError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to open image file {}: {}", image_path, e))))?
            .to_rgba8();
        let (width, height) = img.dimensions();
        let image_data = img.into_raw();
        let buffer_size = (width * height * 4) as vk::DeviceSize;
        let image_format = vk::Format::R8G8B8A8_SRGB;
        let image_extent = vk::Extent2D { width, height };
        let mip_levels = ((width.max(height) as f32).log2().floor() + 1.0) as u32;
        debug!("Image loaded: {}x{}, mip_levels: {}, size: {} bytes, format: {:?}", width, height, mip_levels, buffer_size, image_format);

        let format_properties = unsafe {
            vulkan_instance_raw.get_physical_device_format_properties(physical_device_info.physical_device, image_format)
        };
        if !format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR) {
            warn!("Device does not support linear filtering for format {:?}. Mipmap quality might be affected.", image_format);
        }
        if !format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::BLIT_SRC) {
             return Err(VulkanError::UnsupportedFormat(format!("Device does not support BLIT_SRC for format {:?} for mipmapping.", image_format)));
        }
        if !format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::BLIT_DST) {
            return Err(VulkanError::UnsupportedFormat(format!("Device does not support BLIT_DST for format {:?} for mipmapping.", image_format)));
        }

        let staging_buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size).usage(vk::BufferUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE);
        let staging_allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };
        let (staging_buffer, staging_allocation, staging_alloc_info) = allocator.create_buffer(&staging_buffer_create_info, &staging_allocation_create_info)?; 
        
        unsafe {
            let mapped_data = staging_alloc_info.get_mapped_data_mut();
            std::ptr::copy_nonoverlapping(image_data.as_ptr(), mapped_data as *mut u8, buffer_size as usize);
        }

        let texture_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D).format(image_format)
            .extent(vk::Extent3D { width, height, depth: 1 }).mip_levels(mip_levels)
            .array_layers(1).samples(vk::SampleCountFlags::TYPE_1).tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .initial_layout(vk::ImageLayout::UNDEFINED).sharing_mode(vk::SharingMode::EXCLUSIVE);
        let texture_allocation_create_info = vk_mem::AllocationCreateInfo { usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default() };
        let (texture_image, texture_allocation, _texture_alloc_info) = allocator.create_image(&texture_image_create_info, &texture_allocation_create_info)?;

        record_one_time_submit_commands(device_raw, command_pool, transfer_queue, |cmd_buffer| {
            let initial_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture_image).subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1,
                    base_array_layer: 0, layer_count: 1,
                }).src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            unsafe { device_raw.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(), &[], &[], &[initial_barrier.build()]) };

            let buffer_image_copy = vk::BufferImageCopy::builder()
                .buffer_offset(0).buffer_row_length(0).buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1,
                }).image_offset(vk::Offset3D::default()).image_extent(vk::Extent3D { width, height, depth: 1 });
            unsafe { device_raw.cmd_copy_buffer_to_image(cmd_buffer, staging_buffer, texture_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[buffer_image_copy.build()]) };
            
            let mut mip_width = width as i32; let mut mip_height = height as i32;
            for i in 1..mip_levels {
                let src_mip_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(texture_image).subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: i - 1, level_count: 1,
                        base_array_layer: 0, layer_count: 1,
                    }).src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::TRANSFER_READ);
                unsafe { device_raw.cmd_pipeline_barrier(cmd_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(), &[], &[], &[src_mip_barrier.build()]) };
                
                let dst_mip_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(texture_image).subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: i, level_count: 1,
                        base_array_layer: 0, layer_count: 1,
                    }).src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
                 unsafe { device_raw.cmd_pipeline_barrier(cmd_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(), &[], &[], &[dst_mip_barrier.build()]) };

                let next_mip_width = if mip_width > 1 { mip_width / 2 } else { 1 };
                let next_mip_height = if mip_height > 1 { mip_height / 2 } else { 1 };
                let blit = vk::ImageBlit::builder()
                    .src_subresource(vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: i - 1, base_array_layer: 0, layer_count: 1, })
                    .src_offsets([vk::Offset3D::default(), vk::Offset3D { x: mip_width, y: mip_height, z: 1 }])
                    .dst_subresource(vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: i, base_array_layer: 0, layer_count: 1, })
                    .dst_offsets([vk::Offset3D::default(), vk::Offset3D { x: next_mip_width, y: next_mip_height, z: 1 }]);
                unsafe { device_raw.cmd_blit_image(cmd_buffer, texture_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    texture_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[blit.build()], vk::Filter::LINEAR) };
                
                let src_to_shader_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(texture_image).subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: i - 1, level_count: 1,
                        base_array_layer: 0, layer_count: 1,
                    }).src_access_mask(vk::AccessFlags::TRANSFER_READ).dst_access_mask(vk::AccessFlags::SHADER_READ);
                unsafe { device_raw.cmd_pipeline_barrier(cmd_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(), &[], &[], &[src_to_shader_barrier.build()]) };
                mip_width = next_mip_width; mip_height = next_mip_height;
            }
            let last_mip_shader_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture_image).subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: mip_levels - 1, level_count: 1,
                    base_array_layer: 0, layer_count: 1,
                }).src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ);
            unsafe { device_raw.cmd_pipeline_barrier(cmd_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[last_mip_shader_barrier.build()]) };
        })?;
        allocator.destroy_buffer(staging_buffer, &staging_allocation);

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(texture_image).view_type(vk::ImageViewType::TYPE_2D).format(image_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: mip_levels,
                base_array_layer: 0, layer_count: 1,
            });
        let texture_image_view = unsafe { device_raw.create_image_view(&view_create_info, None) }?;

        let mut sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT).address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT).mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0).min_lod(0.0).max_lod(mip_levels as f32)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK).unnormalized_coordinates(false);
        if physical_device_info.features.sampler_anisotropy == vk::TRUE {
            sampler_create_info = sampler_create_info.anisotropy_enable(true)
                .max_anisotropy(physical_device_info.properties.limits.max_sampler_anisotropy.min(16.0));
        } else {
            sampler_create_info = sampler_create_info.anisotropy_enable(false).max_anisotropy(1.0);
        }
        let texture_sampler = unsafe { device_raw.create_sampler(&sampler_create_info, None) }?;

        info!("Texture {} loaded and Vulkan resources created successfully.", image_path);
        Ok(Self {
            image: texture_image, allocation: texture_allocation, view: texture_image_view,
            sampler: texture_sampler, extent: image_extent, mip_levels, format: image_format,
            logical_device_raw: device_raw.clone(),
            allocator_raw_clone: allocator.raw_allocator().clone(),
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        debug!("Dropping Texture (image: {:?}, view: {:?}, sampler: {:?})", self.image, self.view, self.sampler);
        unsafe {
            self.logical_device_raw.destroy_sampler(self.sampler, None);
            self.logical_device_raw.destroy_image_view(self.view, None);
            if let Err(e) = self.allocator_raw_clone.destroy_image(self.image, &self.allocation) {
                 error!("Failed to destroy texture image with VMA in Drop: {:?}", e);
            }
        }
        debug!("Texture resources destroyed.");
    }
}
