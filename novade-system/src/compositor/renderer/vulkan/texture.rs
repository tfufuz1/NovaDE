use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice,
    physical_device::PhysicalDeviceInfo,
};
use ash::vk;
use image::GenericImageView; // For image dimensions and pixel data
use log::{debug, info, error};
use std::path::Path;
use vk_mem;

/// Helper function to execute commands in a one-time submit command buffer.
///
/// This function allocates a temporary command buffer, begins recording,
/// executes the provided closure to record commands, ends recording,
/// submits the command buffer to the given queue, waits for completion,
/// and finally frees the command buffer and related synchronization objects.
///
/// # Arguments
/// * `logical_device`: Reference to the `LogicalDevice`.
/// * `command_pool`: The command pool from which to allocate the command buffer.
/// * `graphics_queue`: The queue to submit the command buffer to.
/// * `executor`: A closure that takes the allocated `vk::CommandBuffer` and records commands into it.
///
/// # Returns
/// `Result<(), String>` indicating success or failure of the operation.
pub fn record_one_time_submit_commands<F>(
    logical_device: &LogicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    executor: F,
) -> Result<(), String>
where
    F: FnOnce(vk::CommandBuffer),
{
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = unsafe { logical_device.raw.allocate_command_buffers(&allocate_info) }
        .map_err(|e| format!("Failed to allocate one-time command buffer: {}", e))?[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { logical_device.raw.begin_command_buffer(command_buffer, &begin_info) }
        .map_err(|e| format!("Failed to begin one-time command buffer: {}", e))?;

    executor(command_buffer); // Record commands using the provided closure

    unsafe { logical_device.raw.end_command_buffer(command_buffer) }
        .map_err(|e| format!("Failed to end one-time command buffer: {}", e))?;

    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&[command_buffer])
        .build();
    
    let fence_create_info = vk::FenceCreateInfo::builder();
    let fence = unsafe { logical_device.raw.create_fence(&fence_create_info, None)}
        .map_err(|e| format!("Failed to create fence for one-time submit: {}", e))?;

    unsafe {
        logical_device.raw.queue_submit(graphics_queue, &[submit_info], fence)
            .map_err(|e| format!("Failed to submit one-time command buffer: {}", e))?;
        logical_device.raw.wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|e| format!("Failed to wait for one-time submit fence: {}", e))?;
    }

    unsafe {
        logical_device.raw.destroy_fence(fence, None);
        logical_device.raw.free_command_buffers(command_pool, &[command_buffer]);
    }
    debug!("One-time submit commands executed successfully.");
    Ok(())
}


/// Represents a Vulkan texture, including its image, memory, view, and sampler.
#[derive(Debug)]
pub struct Texture {
    pub image: vk::Image,
    pub allocation: vk_mem::Allocation,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub extent: vk::Extent2D,
    pub mip_levels: u32,
    pub format: vk::Format,
    // Keep logical_device_raw and allocator_raw for Drop
    logical_device_raw: ash::Device,
    allocator_raw_clone: vk_mem::Allocator, // Clone of the allocator handle for Drop
}

impl Texture {
    /// Creates a new texture from an image file.
    ///
    /// # Arguments
    /// * `logical_device`: Reference to the `LogicalDevice`.
    /// * `physical_device_info`: Information about the physical device (for sampler anisotropy).
    /// * `allocator`: Reference to the VMA `Allocator`.
    /// * `command_pool`: Command pool for allocating temporary command buffers.
    /// * `graphics_queue`: Queue for submitting image copy commands.
    /// * `image_path`: Path to the image file.
    ///
    /// # Returns
    /// `Result<Self, String>` containing the new `Texture` or an error message.
    #[allow(clippy::too_many_arguments)]
    pub fn new_from_file(
        logical_device: &LogicalDevice,
        physical_device_info: &PhysicalDeviceInfo,
        allocator: &Allocator,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        image_path: &str,
    ) -> Result<Self, String> {
        info!("Loading texture from file: {}", image_path);

        // 1. Load image using 'image' crate
        let img = image::open(Path::new(image_path))
            .map_err(|e| format!("Failed to open image file {}: {}", image_path, e))?
            .to_rgba8(); // Convert to RGBA8 for consistency
        let (width, height) = img.dimensions();
        let image_data = img.into_raw();
        let buffer_size = (width * height * 4) as vk::DeviceSize; // RGBA8 = 4 bytes per pixel
        let image_format = vk::Format::R8G8B8A8_SRGB; // Assuming SRGB format for color data
        let image_extent = vk::Extent2D { width, height };
        let mip_levels = 1; // Simple case: no mipmaps for now

        debug!("Image loaded: {}x{}, size: {} bytes, format: {:?}", width, height, buffer_size, image_format);

        // 2. Create Staging Buffer
        let staging_buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let staging_allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let (staging_buffer, staging_allocation, staging_alloc_info) = allocator
            .create_buffer(&staging_buffer_create_info, &staging_allocation_create_info)
            .map_err(|e| format!("Failed to create staging buffer for texture: {}", e))?;
        debug!("Staging buffer created: {:?}", staging_buffer);

        // Copy image data to staging buffer
        unsafe {
            let mapped_data = staging_alloc_info.get_mapped_data_mut();
            std::ptr::copy_nonoverlapping(image_data.as_ptr(), mapped_data as *mut u8, buffer_size as usize);
        }
        debug!("Image data copied to staging buffer.");


        // 3. Create Vulkan Image (VkImage)
        let texture_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(image_format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let texture_allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };

        let (texture_image, texture_allocation, _texture_alloc_info) = allocator
            .create_image(&texture_image_create_info, &texture_allocation_create_info)
            .map_err(|e| format!("Failed to create texture image with VMA: {}", e))?;
        debug!("Texture image created: {:?}", texture_image);


        // 4. Copy data and transition layouts
        record_one_time_submit_commands(logical_device, command_pool, graphics_queue, |cmd_buffer| {
            // Transition UNDEFINED -> TRANSFER_DST_OPTIMAL
            let barrier_to_transfer_dst = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: mip_levels,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::empty()) // Or HOST_WRITE if data written before barrier
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .build();
            
            unsafe {
                logical_device.raw.cmd_pipeline_barrier(
                    cmd_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE, // Or HOST if data written before
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[], &[], &[barrier_to_transfer_dst],
                );
            }

            // Copy buffer to image
            let buffer_image_copy = vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0) // Tightly packed
                .buffer_image_height(0) // Tightly packed
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D { width, height, depth: 1 })
                .build();
            
            unsafe {
                logical_device.raw.cmd_copy_buffer_to_image(
                    cmd_buffer,
                    staging_buffer,
                    texture_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[buffer_image_copy],
                );
            }

            // Transition TRANSFER_DST_OPTIMAL -> SHADER_READ_ONLY_OPTIMAL
            let barrier_to_shader_read = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: mip_levels,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build();

            unsafe {
                logical_device.raw.cmd_pipeline_barrier(
                    cmd_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER, // Wait in fragment shader stage
                    vk::DependencyFlags::empty(),
                    &[], &[], &[barrier_to_shader_read],
                );
            }
        })?;
        debug!("Texture data copied and layout transitioned.");

        // 5. Destroy Staging Buffer
        allocator.destroy_buffer(staging_buffer, &staging_allocation);
        debug!("Staging buffer destroyed.");

        // 6. Create Image View
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(texture_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            });
        let texture_image_view = unsafe { logical_device.raw.create_image_view(&view_create_info, None) }
            .map_err(|e| format!("Failed to create texture image view: {}", e))?;
        debug!("Texture image view created: {:?}", texture_image_view);

        // 7. Create Sampler
        let mut sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(mip_levels as f32) // Use mip_levels
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK) // Or FLOAT_OPAQUE_BLACK
            .unnormalized_coordinates(false);

        if physical_device_info.features.sampler_anisotropy == vk::TRUE {
            sampler_create_info = sampler_create_info
                .anisotropy_enable(true)
                .max_anisotropy(physical_device_info.properties.limits.max_sampler_anisotropy.min(16.0)); // Cap at 16 or use device limit
            debug!("Sampler anisotropy enabled with max: {}", sampler_create_info.max_anisotropy);
        } else {
            sampler_create_info = sampler_create_info.anisotropy_enable(false).max_anisotropy(1.0);
            debug!("Sampler anisotropy not supported or not enabled.");
        }
        
        let texture_sampler = unsafe { logical_device.raw.create_sampler(&sampler_create_info, None) }
            .map_err(|e| format!("Failed to create texture sampler: {}", e))?;
        debug!("Texture sampler created: {:?}", texture_sampler);

        info!("Texture {} loaded and Vulkan resources created successfully.", image_path);
        Ok(Self {
            image: texture_image,
            allocation: texture_allocation,
            view: texture_image_view,
            sampler: texture_sampler,
            extent: image_extent,
            mip_levels,
            format: image_format,
            logical_device_raw: logical_device.raw.clone(),
            allocator_raw_clone: allocator.raw_allocator().clone(), // Clone VMA allocator handle for Drop
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        debug!("Dropping Texture (image: {:?}, view: {:?}, sampler: {:?})", self.image, self.view, self.sampler);
        unsafe {
            self.logical_device_raw.destroy_sampler(self.sampler, None);
            self.logical_device_raw.destroy_image_view(self.view, None);
            // Image and allocation are destroyed via the cloned VMA allocator handle
            self.allocator_raw_clone.destroy_image(self.image, &self.allocation);
        }
        debug!("Texture resources destroyed.");
    }
}
