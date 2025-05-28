use crate::allocator::{Allocator, Allocation};
use crate::device::LogicalDevice;
use crate::error::{Result, VulkanError};
use std::path::Path;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use image as image_rs_crate; // Alias for the image crate
use image_rs_crate::{DynamicImage, GenericImageView}; // ImageFormat not directly used, format inferred
use crate::buffer_utils::record_and_submit_command; // Use moved helper

pub struct Texture {
    device: Arc<Device>,
    allocator: Arc<Allocator>, // Keep Arc to allocator for Drop
    pub image: vk::Image,
    image_allocation: Option<Allocation>, // Option to take it in Drop
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    vk_format: vk::Format, // Added to store the format
}

impl Texture {
    pub fn image_format_vulkan(&self) -> vk::Format { // Getter for the format
        self.vk_format
    }

    pub fn from_file(
        logical_device_wrapper: &Arc<LogicalDevice>, // Pass Arc directly
        allocator: Arc<Allocator>,                   // Pass Arc directly
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue, // Can be graphics queue
        physical_device_properties: &vk::PhysicalDeviceProperties, // For sampler anisotropy limits
        physical_device_features: &vk::PhysicalDeviceFeatures, // For sampler anisotropy enable
        file_path: &Path,
    ) -> Result<Self> {
        let device_arc = logical_device_wrapper.raw().clone(); // Clone Arc<Device> for struct
        let device_ref = device_arc.as_ref(); // Get &Device for direct calls

        // 1. Load Image File
        let img = image_rs_crate::open(file_path)
            .map_err(|e| VulkanError::ImageError(e))?;
        let (width, height) = img.dimensions();
        let image_data_rgba = img.to_rgba8().into_raw(); // Convert to RGBA8
        let image_size = image_data_rgba.len() as vk::DeviceSize;
        let image_format = vk::Format::R8G8B8A8_SRGB; // Assuming SRGB for color textures

        // 2. Mipmap Levels
        let mip_levels = ((width.max(height) as f32).log2().floor() + 1.0) as u32;

        // 3. Create Staging Buffer & Copy Data
        let staging_buffer_info = vk::BufferCreateInfo::builder()
            .size(image_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let (staging_buffer, mut staging_allocation) = allocator.create_buffer(
            &staging_buffer_info,
            vk_mem_rs::MemoryUsage::CpuOnly,
            Some(vk_mem_rs::AllocationCreateFlags::MAPPED | vk_mem_rs::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE),
        )?;
        unsafe {
            // VMA map_memory is fine even if MAPPED was used; it's idempotent or returns existing pointer.
            let mapped_ptr = allocator.map_memory(&mut staging_allocation)?;
            std::ptr::copy_nonoverlapping(image_data_rgba.as_ptr(), mapped_ptr, image_data_rgba.len());
            if !staging_allocation.get_memory_type_properties().contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
                allocator.flush_allocation(&staging_allocation, 0, vk::WHOLE_SIZE)?;
            }
            // No explicit unmap needed if MAPPED was used, VMA handles it.
        }

        // 4. Create VkImage
        let image_extent = vk::Extent3D::builder().width(width).height(height).depth(1);
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .format(image_format)
            .extent(image_extent)
            .mip_levels(mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::TRANSFER_SRC | // For mip blitting source
                vk::ImageUsageFlags::TRANSFER_DST | // For staging copy & mip blitting destination
                vk::ImageUsageFlags::SAMPLED,      // For shader sampling
            )
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let (image, image_allocation) = allocator.create_image(
            &image_info,
            vk_mem_rs::MemoryUsage::GpuOnly,
            None,
        )?;

        // 5. Transition Layout & Copy Staging to Image
        Self::transition_image_layout_internal(
            device_ref, command_pool, transfer_queue,
            image, vk::ImageAspectFlags::COLOR, mip_levels, 1,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        Self::copy_buffer_to_image_internal(
            device_ref, command_pool, transfer_queue,
            staging_buffer, image, width, height,
        )?;

        // 6. Cleanup Staging Buffer
        allocator.destroy_buffer(staging_buffer, staging_allocation)?;

        // 7. Generate Mipmaps (if mip_levels > 1)
        if mip_levels > 1 {
            Self::generate_mipmaps_internal(
                device_ref, command_pool, transfer_queue,
                image, image_format, width, height, mip_levels,
            )?;
        } else { // If no mipmaps, transition directly to shader read optimal
            Self::transition_image_layout_internal(
                device_ref, command_pool, transfer_queue,
                image, vk::ImageAspectFlags::COLOR, mip_levels, 1,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            )?;
        }
        
        // 8. Create VkImageView
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(image_format)
            .subresource_range(vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(mip_levels)
                .base_array_layer(0)
                .layer_count(1)
                .build());
        let image_view = unsafe { device_ref.create_image_view(&image_view_info, None) }
            .map_err(VulkanError::VkResult)?;

        // 9. Create VkSampler
        let max_anisotropy = if physical_device_features.sampler_anisotropy == vk::TRUE {
            physical_device_properties.limits.max_sampler_anisotropy
        } else {
            1.0
        };

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(0.0)
            .anisotropy_enable(physical_device_features.sampler_anisotropy == vk::TRUE)
            .max_anisotropy(max_anisotropy)
            .compare_enable(false) // No comparison sampler for now
            .min_lod(0.0)
            .max_lod(mip_levels as f32) // Use all mip levels
            .border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK)
            .unnormalized_coordinates(false);
        let sampler = unsafe { device_ref.create_sampler(&sampler_info, None) }
            .map_err(VulkanError::VkResult)?;
        
        log::info!("Texture loaded from {:?}: {}x{} ({} mip levels)", file_path, width, height, mip_levels);

        Ok(Self {
            device: device_arc, // Store the Arc<Device>
            allocator,
            image,
            image_allocation: Some(image_allocation),
            image_view,
            sampler,
            width,
            height,
            mip_levels,
            vk_format: image_format, // Store the format
        })
    }

    // New method for creating compute target textures
    pub fn new_compute_target(
        logical_device_wrapper: &Arc<LogicalDevice>,
        allocator: Arc<Allocator>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Result<Self> {
        let device_arc = logical_device_wrapper.raw().clone(); // Clone Arc<Device> for struct
        let device_ref = device_arc.as_ref(); // Get &Device for direct calls

        let image_extent = vk::Extent3D::builder().width(width).height(height).depth(1);

        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .format(format)
            .extent(image_extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::STORAGE_BIT | 
                vk::ImageUsageFlags::SAMPLED_BIT    
            )
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (image, image_allocation) = allocator.create_image(
            &image_info,
            vk_mem_rs::MemoryUsage::GpuOnly,
            None,
        )?;

        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1).build());
        let image_view = unsafe { device_ref.create_image_view(&image_view_info, None) }?;

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .max_lod(0.0);
        let sampler = unsafe { device_ref.create_sampler(&sampler_info, None) }?;
        
        log::info!("Compute target texture created: {}x{}, format {:?}", width, height, format);

        Ok(Self {
            device: device_arc,
            allocator,
            image,
            image_allocation: Some(image_allocation),
            image_view,
            sampler,
            width,
            height,
            mip_levels: 1,
            vk_format: format, // Store the format
        })
    }


    #[allow(clippy::too_many_arguments)] // Helper function with many specific parameters
    fn transition_image_layout_internal(
        device: &Device, command_pool: vk::CommandPool, transfer_queue: vk::Queue,
        image: vk::Image, aspect_mask: vk::ImageAspectFlags, mip_levels: u32, layer_count: u32,
        old_layout: vk::ImageLayout, new_layout: vk::ImageLayout,
    ) -> Result<()> {
        crate::buffer_utils::record_and_submit_command(device, command_pool, transfer_queue, |cmd_buffer| {
            let mut barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(mip_levels)
                    .base_array_layer(0)
                    .layer_count(layer_count)
                    .build());

            let (src_stage, dst_stage, src_access_mask, dst_access_mask) = 
                match (old_layout, new_layout) {
                    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER,
                        vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
                    ),
                    (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
                    ),
                    (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL) => (
                        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER,
                        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::TRANSFER_READ,
                    ),
                    _ => return Err(VulkanError::Message(format!("Unsupported layout transition from {:?} to {:?}", old_layout, new_layout))),
            };
            
            barrier = barrier.src_access_mask(src_access_mask).dst_access_mask(dst_access_mask);
            
            unsafe { device.cmd_pipeline_barrier(
                cmd_buffer,
                src_stage, dst_stage,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier], // Corrected slice syntax
                &[] as &[vk::BufferMemoryBarrier], // Corrected slice syntax
                &[barrier.build()], // Build the barrier and pass as slice
            )};
            Ok(())
        })
    }

    fn copy_buffer_to_image_internal(
        device: &Device, command_pool: vk::CommandPool, transfer_queue: vk::Queue,
        buffer: vk::Buffer, image: vk::Image, width: u32, height: u32,
    ) -> Result<()> {
        crate::buffer_utils::record_and_submit_command(device, command_pool, transfer_queue, |cmd_buffer| {
            let region = vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0)   // 0 means tightly packed
                .buffer_image_height(0) // 0 means tightly packed
                .image_subresource(vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build())
                .image_offset(vk::Offset3D::default())
                .image_extent(vk::Extent3D::builder().width(width).height(height).depth(1).build());
            unsafe { device.cmd_copy_buffer_to_image(
                cmd_buffer, buffer, image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region] // Pass region as slice
            )};
            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)] // Helper function with many specific parameters
    fn generate_mipmaps_internal(
        device: &Device, command_pool: vk::CommandPool, transfer_queue: vk::Queue,
        image: vk::Image, _image_format: vk::Format, width: u32, height: u32, mip_levels: u32,
    ) -> Result<()> {
        // Note: Check physical_device_format_properties for VK_FORMAT_FEATURE_BLIT_SRC_BIT
        // and VK_FORMAT_FEATURE_BLIT_DST_BIT for the image_format.
        // This should be done ideally during physical device selection or format choice.
        // For this subtask, we assume the format supports blitting as required.

        crate::buffer_utils::record_and_submit_command(device, command_pool, transfer_queue, |cmd_buffer| {
            let mut mip_width = width as i32;
            let mut mip_height = height as i32;

            for i in 1..mip_levels {
                // Transition mip level i-1 to TRANSFER_SRC_OPTIMAL
                let subresource_prev = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(i - 1)
                    .level_count(1).base_array_layer(0).layer_count(1);
                let barrier_src = vk::ImageMemoryBarrier::builder()
                    .image(image).old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL) // Mip 0 is DST_OPTIMAL from copy
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .subresource_range(subresource_prev);
                unsafe { device.cmd_pipeline_barrier(cmd_buffer,
                    vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(), &[], &[], &[barrier_src.build()]) }; // Build barrier

                // Blit from mip level i-1 to mip level i (which is already in TRANSFER_DST_OPTIMAL)
                let blit = vk::ImageBlit::builder()
                    .src_subresource(vk::ImageSubresourceLayers::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR).mip_level(i - 1)
                        .base_array_layer(0).layer_count(1).build())
                    .src_offsets([vk::Offset3D::default(), vk::Offset3D { x: mip_width, y: mip_height, z: 1 }])
                    .dst_subresource(vk::ImageSubresourceLayers::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR).mip_level(i)
                        .base_array_layer(0).layer_count(1).build())
                    .dst_offsets([vk::Offset3D::default(), vk::Offset3D {
                        x: if mip_width > 1 { mip_width / 2 } else { 1 },
                        y: if mip_height > 1 { mip_height / 2 } else { 1 },
                        z: 1,
                    }]);
                unsafe { device.cmd_blit_image(cmd_buffer,
                    image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, // Mip i is already in DST_OPTIMAL
                    &[blit.build()], vk::Filter::LINEAR)}; // Build blit

                // Transition mip level i-1 to SHADER_READ_ONLY_OPTIMAL
                let barrier_shader_ro = vk::ImageMemoryBarrier::builder()
                    .image(image).old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .subresource_range(subresource_prev);
                unsafe { device.cmd_pipeline_barrier(cmd_buffer,
                    vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(), &[], &[], &[barrier_shader_ro.build()]) }; // Build barrier
                
                if mip_width > 1 { mip_width /= 2; }
                if mip_height > 1 { mip_height /= 2; }
            }

            // Transition the last mip level (mip_levels - 1) to SHADER_READ_ONLY_OPTIMAL
            // It was left in TRANSFER_DST_OPTIMAL after being blitted to.
            let subresource_last_mip = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(mip_levels - 1)
                .level_count(1).base_array_layer(0).layer_count(1);
            let barrier_last_mip = vk::ImageMemoryBarrier::builder()
                .image(image).old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .subresource_range(subresource_last_mip);
            unsafe { device.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[barrier_last_mip.build()]) }; // Build barrier
            Ok(())
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.image_view, None);
            // Image and its allocation are destroyed using the allocator
            if let Some(alloc) = self.image_allocation.take() {
                if let Err(e) = self.allocator.destroy_image(self.image, alloc) {
                     log::error!("Failed to destroy texture image with VMA: {:?}", e);
                }
            }
        }
        log::debug!("Texture resources destroyed (sampler, image view, image via VMA).");
    }
}

```
