//! Manages texture atlases for efficient rendering of multiple small textures.
//!
//! A texture atlas combines multiple smaller images (sub-textures) into a single larger
//! Vulkan image (`VkImage`). This can improve rendering performance by reducing the
//! number of texture bindings and draw calls. This module provides the `TextureAtlas`
//! struct to create and manage such an atlas, along with `SubTextureInfo` to store
//! the UV coordinates of each sub-texture within the atlas.

use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice, // Used to pass ash::Device to record_one_time_submit_commands
    physical_device::PhysicalDeviceInfo,
    texture::record_one_time_submit_commands,
    error::{Result, VulkanError},
};
use ash::vk;
use image::GenericImageView;
use log::{debug, info, error, warn};
use std::{collections::HashMap, path::Path};
use vk_mem;

/// Default width for the texture atlas.
const ATLAS_WIDTH: u32 = 2048;
/// Default height for the texture atlas.
const ATLAS_HEIGHT: u32 = 2048;

/// Information about a sub-texture within a `TextureAtlas`.
#[derive(Debug, Clone, Copy)]
pub struct SubTextureInfo {
    /// Normalized UV coordinates `[x_min, y_min, x_max, y_max]`.
    pub uv_rect: [f32; 4],
}

/// Represents a texture atlas, a single large image containing multiple sub-textures.
#[derive(Debug)]
pub struct TextureAtlas {
    pub atlas_image: vk::Image,
    pub atlas_allocation: vk_mem::Allocation,
    pub atlas_image_view: vk::ImageView,
    pub atlas_sampler: vk::Sampler,
    pub atlas_extent: vk::Extent2D,
    pub atlas_format: vk::Format,
    pub sub_textures: HashMap<String, SubTextureInfo>,
    logical_device_raw: ash::Device,
    allocator_raw_clone: vk_mem::Allocator,
}

// Helper struct to store information needed for copy operations inside the command buffer
struct StagingCopyInfo {
    staging_buffer: vk::Buffer,
    // staging_allocation: vk_mem::Allocation, // Allocation is for cleanup, not for copy command
    copy_region: vk::BufferImageCopy,
}


impl TextureAtlas {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        logical_device: &LogicalDevice, // Provides ash::Device and Queues
        allocator: &Allocator,
        physical_device_info: &PhysicalDeviceInfo,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue,
        image_entries: &[(&str, String)],
    ) -> Result<Self> {
        info!("Creating TextureAtlas with dimensions: {}x{}", ATLAS_WIDTH, ATLAS_HEIGHT);
        let atlas_format = vk::Format::R8G8B8A8_SRGB;
        let atlas_extent = vk::Extent2D { width: ATLAS_WIDTH, height: ATLAS_HEIGHT };

        let atlas_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D).format(atlas_format)
            .extent(vk::Extent3D { width: ATLAS_WIDTH, height: ATLAS_HEIGHT, depth: 1 })
            .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .initial_layout(vk::ImageLayout::UNDEFINED).sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let atlas_allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default()
        };

        let (atlas_image, atlas_allocation, _atlas_alloc_info) = allocator
            .create_image(&atlas_image_create_info, &atlas_allocation_create_info)?;
        debug!("TextureAtlas main image created: {:?}", atlas_image);

        let mut sub_textures = HashMap::new();
        let mut staging_resources_to_cleanup: Vec<(vk::Buffer, vk_mem::Allocation)> = Vec::new();
        let mut copy_infos_for_cmd_buffer: Vec<StagingCopyInfo> = Vec::new();

        let mut current_x = 0u32;
        let mut current_y = 0u32;
        let mut current_row_max_height = 0u32;

        // 1. Load images, create and fill staging buffers, prepare copy infos
        for (id, image_path_str) in image_entries {
            let img_data = match image::open(Path::new(image_path_str)) {
                Ok(i) => i.to_rgba8(),
                Err(e) => {
                    error!("Failed to load image for atlas (id: '{}', path: '{}'): {}", id, image_path_str, e);
                    continue; 
                }
            };
            let (img_width, img_height) = img_data.dimensions();
            let raw_image_data = img_data.into_raw();
            let buffer_size = (img_width * img_height * 4) as vk::DeviceSize;

            if img_width > ATLAS_WIDTH || img_height > ATLAS_HEIGHT {
                warn!("Image {} ({}x{}) is larger than atlas dimensions ({}x{}). Skipping.", id, img_width, img_height, ATLAS_WIDTH, ATLAS_HEIGHT);
                continue;
            }

            if current_x + img_width > ATLAS_WIDTH {
                current_x = 0;
                current_y += current_row_max_height;
                current_row_max_height = 0;
            }
            if current_y + img_height > ATLAS_HEIGHT {
                error!("TextureAtlas is full. Failed to place image id: '{}' (path: '{}')", id, image_path_str);
                warn!("Atlas ran out of space. Skipping image: {}", id);
                continue;
            }

            let staging_create_info = vk::BufferCreateInfo::builder()
                .size(buffer_size).usage(vk::BufferUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE);
            let staging_alloc_info_vma = vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            };
            
            let (staging_buffer, staging_allocation, staging_mapped_info) = allocator
                .create_buffer(&staging_create_info, &staging_alloc_info_vma)?;

            unsafe {
                std::ptr::copy_nonoverlapping(raw_image_data.as_ptr(), staging_mapped_info.get_mapped_data_mut() as *mut u8, buffer_size as usize);
            }
            staging_resources_to_cleanup.push((staging_buffer, staging_allocation));

            let copy_region = vk::BufferImageCopy::builder()
                .buffer_offset(0).buffer_row_length(0).buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0,
                    base_array_layer: 0, layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: current_x as i32, y: current_y as i32, z: 0 })
                .image_extent(vk::Extent3D { width: img_width, height: img_height, depth: 1 })
                .build();
            
            copy_infos_for_cmd_buffer.push(StagingCopyInfo { staging_buffer, copy_region });

            let uv_rect = [
                current_x as f32 / ATLAS_WIDTH as f32, current_y as f32 / ATLAS_HEIGHT as f32,
                (current_x + img_width) as f32 / ATLAS_WIDTH as f32, (current_y + img_height) as f32 / ATLAS_HEIGHT as f32,
            ];
            sub_textures.insert(id.to_string(), SubTextureInfo { uv_rect });
            debug!("Prepared sub-texture id '{}' at [{}, {}], UVs: {:?}", id, current_x, current_y, uv_rect);

            current_x += img_width;
            current_row_max_height = current_row_max_height.max(img_height);
        }
        
        // 2. Record and submit commands for all copies and layout transitions
        record_one_time_submit_commands(&logical_device.raw, command_pool, transfer_queue, |cmd_buffer| {
            let atlas_initial_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(atlas_image).subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1,
                    base_array_layer: 0, layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            unsafe { logical_device.raw.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(), &[], &[], &[atlas_initial_barrier.build()]) };

            for copy_info in &copy_infos_for_cmd_buffer {
                unsafe {
                    logical_device.raw.cmd_copy_buffer_to_image(cmd_buffer, copy_info.staging_buffer, atlas_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy_info.copy_region]);
                }
            }

            let atlas_final_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(atlas_image).subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1,
                    base_array_layer: 0, layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ);
            unsafe { logical_device.raw.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[atlas_final_barrier.build()]) };
        })?;
        
        // 3. Cleanup staging resources now that command buffer has executed
        for (buffer, allocation) in staging_resources_to_cleanup {
            allocator.destroy_buffer(buffer, &allocation);
        }
        debug!("All staging buffers for atlas sub-textures destroyed.");

        // Create Atlas ImageView
        let atlas_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(atlas_image).view_type(vk::ImageViewType::TYPE_2D).format(atlas_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1,
                base_array_layer: 0, layer_count: 1,
            });
        let atlas_image_view = unsafe { logical_device.raw.create_image_view(&atlas_view_create_info, None) }?;

        // Create Atlas Sampler
        let mut sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR).max_lod(1.0)
            .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK)
            .unnormalized_coordinates(false);
        if physical_device_info.features.sampler_anisotropy == vk::TRUE {
            sampler_info = sampler_info.anisotropy_enable(true)
                .max_anisotropy(physical_device_info.properties.limits.max_sampler_anisotropy.min(16.0));
        }
        let atlas_sampler = unsafe { logical_device.raw.create_sampler(&sampler_info, None) }?;
        info!("TextureAtlas created successfully with {} sub-textures.", sub_textures.len());

        Ok(Self {
            atlas_image, atlas_allocation, atlas_image_view, atlas_sampler,
            atlas_extent, atlas_format, sub_textures,
            logical_device_raw: logical_device.raw.clone(),
            allocator_raw_clone: allocator.raw_allocator().clone(),
        })
    }

    pub fn get_sub_texture_uvs(&self, id: &str) -> Option<SubTextureInfo> {
        self.sub_textures.get(id).copied()
    }
}

impl Drop for TextureAtlas {
    fn drop(&mut self) {
        info!("Dropping TextureAtlas (image: {:?}, view: {:?}, sampler: {:?})", 
            self.atlas_image, self.atlas_image_view, self.atlas_sampler);
        unsafe {
            self.logical_device_raw.destroy_sampler(self.atlas_sampler, None);
            self.logical_device_raw.destroy_image_view(self.atlas_image_view, None);
            if let Err(e) = self.allocator_raw_clone.destroy_image(self.atlas_image, &self.atlas_allocation) {
                 error!("Failed to destroy texture atlas image with VMA in Drop: {:?}", e);
            }
        }
        debug!("TextureAtlas resources destroyed.");
    }
}
