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

// --- VulkanRenderableTexture for DMABUF/SHM imports ---

use std::sync::Arc;
use std::any::Any;
use uuid::Uuid;
use smithay::reexports::drm_fourcc::DrmFourcc;
use crate::compositor::renderer_interface::abstraction::{RenderableTexture, RendererError};
use crate::compositor::renderer::vulkan::error::VulkanError; // For error conversion

/// Represents a Vulkan texture imported from a client buffer (DMABUF or SHM).
///
/// This struct manages the lifetime of Vulkan resources associated with the imported buffer,
/// such as `vk::Image`, `vk::ImageView`, and the underlying `vk::DeviceMemory` or `vk_mem::Allocation`.
#[derive(Debug)]
pub struct VulkanRenderableTexture {
    internal_id: Uuid,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    /// Raw device memory, typically for DMABUF imports where memory is externally owned/imported.
    pub memory: Option<vk::DeviceMemory>, 
    /// VMA allocation, typically for SHM-backed textures where memory is allocated by VMA.
    pub allocation: Option<vk_mem::Allocation>,
    /// Sampler to be used with this texture. Often a shared/default sampler.
    pub sampler: vk::Sampler, // Assuming this is a non-owning handle for now (e.g. default sampler)
    pub format: vk::Format,
    pub width: u32,
    pub height: u32,
    pub current_layout: vk::ImageLayout, // Current layout of the vk::Image
    logical_device: Arc<ash::Device>,
    /// VMA allocator instance, needed if `allocation` is Some and needs to be freed.
    vma_allocator: Option<Arc<Allocator>>, 
}

impl VulkanRenderableTexture {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        memory: Option<vk::DeviceMemory>,
        allocation: Option<vk_mem::Allocation>,
        sampler: vk::Sampler,
        format: vk::Format,
        width: u32,
        height: u32,
        initial_layout: vk::ImageLayout,
        logical_device: Arc<ash::Device>,
        vma_allocator: Option<Arc<Allocator>>,
    ) -> Self {
        Self {
            internal_id: Uuid::new_v4(),
            image,
            image_view,
            memory,
            allocation,
            sampler,
            format,
            width,
            height,
            current_layout: initial_layout,
            logical_device,
            vma_allocator,
        }
    }

    // Helper to map Vulkan formats to DRM FourCC codes.
    // This can be expanded as more formats are supported.
    fn vk_format_to_drm_fourcc(vk_format: vk::Format) -> DrmFourcc {
        match vk_format {
            vk::Format::B8G8R8A8_UNORM => DrmFourcc::Argb8888,
            vk::Format::B8G8R8A8_SRGB => DrmFourcc::Argb8888, // DrmFourcc typically doesn't distinguish sRGB
            vk::Format::R8G8B8A8_UNORM => DrmFourcc::Rgba8888,
            vk::Format::R8G8B8A8_SRGB => DrmFourcc::Rgba8888,
            // TODO: Add more mappings as needed (e.g., for Xrgb8888, common planar formats for DMABUF)
            // vk::Format::G8_B8_R8_3PLANE_420_UNORM, etc. -> DrmFourcc::Nv12, Yuyv, etc.
            _ => DrmFourcc::Invalid, // Default for unmapped formats
        }
    }
}

impl RenderableTexture for VulkanRenderableTexture {
    fn id(&self) -> Uuid {
        self.internal_id
    }

    fn bind(&self, _texture_unit: u32) -> Result<(), RendererError> {
        // For Vulkan, "binding" a texture is different from OpenGL.
        // Textures are typically bound via descriptor sets.
        // This method could:
        // 1. Ensure the texture is in the correct layout for sampling (e.g., SHADER_READ_ONLY_OPTIMAL).
        //    (This should ideally be handled by the render pass or commands using the texture).
        // 2. Return information needed to update a descriptor set, like (ImageView, Sampler, Layout).
        // For now, as a basic implementation, we'll just acknowledge the call.
        // The actual usage will depend on how the VulkanFrameRenderer manages descriptor sets.
        debug!("VulkanRenderableTexture::bind called for texture ID {:?}. (ImageView: {:?}, Sampler: {:?})", 
               self.internal_id, self.image_view, self.sampler);
        Ok(())
    }

    fn width_px(&self) -> u32 {
        self.width
    }

    fn height_px(&self) -> u32 {
        self.height
    }

    fn drm_format(&self) -> Option<DrmFourcc> {
        let drm_fmt = Self::vk_format_to_drm_fourcc(self.format);
        if drm_fmt == DrmFourcc::Invalid {
            None
        } else {
            Some(drm_fmt)
        }
    }

    fn downcast_ref<T: 'static + Any>(&self) -> Option<&T> {
        if Any::type_id(self) == Any::type_id::<T>() {
            unsafe { Some(&*(self as *const Self as *const T)) }
        } else {
            None
        }
    }
     // TODO: Implement downcast_mut if mutable access to the concrete type is needed.
}

impl Drop for VulkanRenderableTexture {
    fn drop(&mut self) {
        debug!(
            "Dropping VulkanRenderableTexture (ID: {:?}, Image: {:?}, ImageView: {:?})",
            self.internal_id, self.image, self.image_view
        );
        unsafe {
            self.logical_device.destroy_image_view(self.image_view, None);
            // Image and memory destruction depends on how they were created.
            if let Some(allocation) = self.allocation.take() {
                if let Some(allocator_arc) = &self.vma_allocator {
                    // This image was created and allocated by VMA (e.g., for SHM)
                     if let Err(e) = allocator_arc.destroy_image(self.image, &allocation) {
                        error!("Failed to destroy VMA-allocated image {:?}: {:?}", self.image, e);
                     } else {
                        debug!("Destroyed VMA-allocated image {:?}", self.image);
                     }
                } else {
                    // This is problematic: allocation exists but no allocator to free it.
                    error!("VMA allocation for image {:?} present but no VMA allocator to free it.", self.image);
                }
            } else if let Some(memory) = self.memory.take() {
                // This image's memory was imported or manually bound (e.g., DMABUF)
                // The image itself might still need to be destroyed if it wasn't solely an imported resource.
                // If ClientVkBuffer::from_dma_buf creates the vk::Image handle for an imported memory,
                // that vk::Image handle should be destroyed.
                self.logical_device.destroy_image(self.image, None);
                debug!("Destroyed vk::Image handle {:?}", self.image);
                self.logical_device.free_memory(memory, None);
                debug!("Freed vk::DeviceMemory for image {:?}", self.image);
            } else {
                // If neither allocation nor memory is set, it might be an image whose lifecycle is managed elsewhere
                // or an error in state. For imported DMABUFs where only an image view is created for an
                // externally managed image, only the view should be destroyed.
                // However, our current plan for DMABUF involves creating a vk::Image via ClientVkBuffer.
                 warn!("VulkanRenderableTexture dropped for image {:?}, but it had neither VMA allocation nor direct DeviceMemory. Ensure lifecycle is managed if this is intended.", self.image);
                 // If the image handle itself was created by us (even for imported memory), it should be destroyed.
                 // This path might be hit if memory was None but image was still created.
                 // self.logical_device.destroy_image(self.image, None); // Consider if this is always safe.
            }
        }
        debug!("VulkanRenderableTexture (ID: {:?}) resources destroyed.", self.internal_id);
    }
}
