// novade-system/src/renderers/vulkan/texture_pipeline.rs
use ash::vk;
use gpu_allocator::vulkan::{Allocator, Allocation, AllocationCreateDesc};
use gpu_allocator::MemoryLocation;
use std::sync::atomic::{AtomicU64, Ordering};

use super::command_buffer::{begin_single_time_commands, end_single_time_commands}; // Assuming this will be created

// Simple Texture ID generator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureId(u64);
static NEXT_TEXTURE_ID: AtomicU64 = AtomicU64::new(0);
impl TextureId {
    pub fn new() -> Self {
        TextureId(NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct GpuTexture {
    pub id: TextureId,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub allocation: Option<Allocation>, // Option because imported DMA-BUFs might not use gpu-allocator's Allocation
    pub memory: Option<vk::DeviceMemory>, // For DMA-BUF imported memory not from allocator
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub layout: vk::ImageLayout, // Current layout
    pub mip_levels: u32,
    pub sampler: vk::Sampler, // WP-501: Added sampler
}

impl GpuTexture {
    // Placeholder for cleanup logic
    pub fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_sampler(self.sampler, None); // WP-501: Destroy sampler
            device.destroy_image_view(self.image_view, None);

            if let Some(mem) = self.memory.take() {
                device.free_memory(mem, None);
            }
            if let Some(alloc) = self.allocation.take() {
                 allocator.free(alloc).expect("Failed to free texture allocation");
            }
            device.destroy_image(self.image, None);
            // println!("Destroyed GpuTexture {:?}", self.id); // Keep logs minimal
        }
    }
}


/// Creates a Vulkan texture sampler.
///
/// Configures a `vk::Sampler` with common settings for texture sampling, including
/// linear filtering, repeat address mode, anisotropic filtering (if supported and enabled),
/// and mipmapping parameters.
/// Aligns with `Rendering Vulkan.md` (Spec 9.4 - Sampler-Konfiguration).
///
/// # Arguments
/// * `device`: Reference to the logical `ash::Device`.
/// * `pdevice_properties`: Properties of the physical device, used to query limits like max sampler anisotropy.
/// * `mip_levels`: The number of mip levels in the texture this sampler will be used with. This sets `max_lod`.
/// * `enable_anisotropy`: Flag to enable anisotropic filtering if supported.
///
/// # Returns
/// A `Result` containing the created `vk::Sampler` or an error string.
pub fn create_texture_sampler(
    device: &ash::Device,
    pdevice_properties: &vk::PhysicalDeviceProperties,
    mip_levels: u32,
    enable_anisotropy: bool,
) -> Result<vk::Sampler, String> {
    let mut sampler_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK) // Or FLOAT_OPAQUE_BLACK depending on preference
        .unnormalized_coordinates(false) // Use normalized texture coordinates (0.0 to 1.0)
        .compare_enable(false) // No depth comparison
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(mip_levels as f32); // Use all available mip levels

    if enable_anisotropy && pdevice_properties.limits.max_sampler_anisotropy > 0.0 {
        sampler_info = sampler_info
            .anisotropy_enable(true)
            .max_anisotropy(pdevice_properties.limits.max_sampler_anisotropy.min(16.0)); // Cap at 16 or use full device limit
    } else {
        sampler_info = sampler_info.anisotropy_enable(false).max_anisotropy(1.0);
    }

    unsafe {
        device.create_sampler(&sampler_info, None)
    }.map_err(|e| format!("Failed to create texture sampler: {}", e))
}


fn transition_image_layout(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    base_mip_level: u32, // New parameter
    level_count: u32,    // New parameter
) {
    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        // This might need adjustment if mipmapping depth textures,
        // but typically mipmaps are for color/sampled images.
        vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
    } else {
        vk::ImageAspectFlags::COLOR
    };

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(base_mip_level)
        .level_count(level_count)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range);

    let source_stage;
    let destination_stage;

    if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
        barrier = barrier.src_access_mask(vk::AccessFlags::empty())
                         .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::TRANSFER;
    } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
        barrier = barrier.src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                         .dst_access_mask(vk::AccessFlags::SHADER_READ);
        source_stage = vk::PipelineStageFlags::TRANSFER;
        destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    } else {
        // Basic fallback, might need more specific cases
        // For robust error handling, this should probably return a Result or panic.
        // However, for this subtask, we'll keep the provided logic which might lead to validation errors
        // if an unhandled transition is attempted.
        barrier = barrier.src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                         .dst_access_mask(vk::AccessFlags::MEMORY_READ);
        source_stage = vk::PipelineStageFlags::ALL_COMMANDS;
        destination_stage = vk::PipelineStageFlags::ALL_COMMANDS;
        println!("Warning: Using generic layout transition for {:?} -> {:?}", old_layout, new_layout);
        // panic!("Unsupported layout transition: {:?} -> {:?}", old_layout, new_layout);
    }

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            &[], // memory barriers
            &[], // buffer memory barriers
            &[barrier.build()],
        );
    }
}

fn generate_mipmaps_for_image(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    _image_format: vk::Format, // Placeholder, real check needs physical_device access
    image_width: u32,
    image_height: u32,
    mip_levels: u32,
) -> Result<(), String> {
    // Check for blit support should be done before calling this function,
    // as it requires PhysicalDevice access. For now, assume supported.

    let mut mip_width = image_width as i32; // Use i32 for calculations involving division
    let mut mip_height = image_height as i32;

    for i in 1..mip_levels {
        // Transition previous mip level (i-1) to TRANSFER_SRC_OPTIMAL
        // This was the destination of the previous blit or initial upload.
        transition_image_layout(
            device,
            command_buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, // Previous level was written to
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            i - 1, // base_mip_level for the source of the blit
            1,     // level_count
        );

        let next_mip_width = if mip_width > 1 { mip_width / 2 } else { 1 };
        let next_mip_height = if mip_height > 1 { mip_height / 2 } else { 1 };

        let blit = vk::ImageBlit::builder()
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D { x: mip_width, y: mip_height, z: 1 },
            ])
            .src_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i - 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D { x: next_mip_width, y: next_mip_height, z: 1 },
            ])
            .dst_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i, // Current mip level being generated
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            device.cmd_blit_image(
                command_buffer,
                image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, // Current mip is blit destination
                &[blit.build()],
                vk::Filter::LINEAR,
            );
        }

        // Transition the source mip level (i-1) of this blit to SHADER_READ_ONLY_OPTIMAL
        // as it's now been used as a source and is ready for sampling.
        transition_image_layout(
            device,
            command_buffer,
            image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            i - 1, // base_mip_level for the source
            1,     // level_count
        );

        mip_width = next_mip_width;
        mip_height = next_mip_height;
    }

    // After the loop, the last generated mip level (mip_levels - 1) is in TRANSFER_DST_OPTIMAL.
    // Transition it to SHADER_READ_ONLY_OPTIMAL.
    transition_image_layout(
        device,
        command_buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        mip_levels - 1, // base_mip_level for the last mip
        1,              // level_count
    );
    Ok(())
}

pub fn upload_shm_buffer_to_texture(
    device: &ash::Device,
    allocator: &mut Allocator,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    pixel_data: &[u8],
    width: u32,
    height: u32,
    format: vk::Format,
    generate_mipmaps: bool,
    pdevice_properties: &vk::PhysicalDeviceProperties, // WP-501: For sampler creation
    enable_anisotropy_for_sampler: bool, // WP-501: For sampler creation
) -> Result<GpuTexture, String> {
    // Calculate image size based on format. This is a simplification.
    let bytes_per_pixel = match format {
        vk::Format::R8G8B8A8_SRGB | vk::Format::B8G8R8A8_SRGB |
        vk::Format::R8G8B8A8_UNORM | vk::Format::B8G8R8A8_UNORM => 4,
        // Add other formats as needed
        _ => return Err(format!("Unsupported format for size calculation: {:?}", format)),
    };
    let image_size = (width * height * bytes_per_pixel) as vk::DeviceSize;

    // 1. Create Staging Buffer
    let (staging_buffer, mut staging_allocation) = super::memory::create_buffer(
        allocator,
        device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu, // Mappable by CPU
    )?;

    // Copy pixel data to staging buffer
    unsafe {
        let data_ptr = staging_allocation.mapped_ptr().ok_or("Failed to map staging buffer")?.as_ptr() as *mut u8;
        if data_ptr.is_null() { return Err("Mapped staging buffer pointer is null".to_string()); }
        if pixel_data.len() as vk::DeviceSize > image_size {
            let error_msg = format!("Pixel data size {} exceeds image size {}", pixel_data.len(), image_size);
            // Cleanup before erroring
            allocator.free(staging_allocation).map_err(|e| format!("Failed to free staging alloc on error: {:?}. Original error: {}", e, error_msg))?;
            device.destroy_buffer(staging_buffer, None);
            return Err(error_msg);
        }
        std::ptr::copy_nonoverlapping(pixel_data.as_ptr(), data_ptr, pixel_data.len());
        // Assuming CpuToGpu is coherent. If not, flush might be needed:
        // staging_allocation.flush(device, 0, vk::WHOLE_SIZE).map_err(|e| format!("Failed to flush staging buffer: {:?}", e))?;
    }

    // 2. Create Destination Image (GPU Only)
    let image_extent = vk::Extent2D { width, height };

    let mip_levels = if generate_mipmaps && width > 0 && height > 0 {
        ((width.max(height) as f32).log2().floor() as u32) + 1
    } else {
        1
    };

    let mut image_usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
    if generate_mipmaps && mip_levels > 1 {
        image_usage |= vk::ImageUsageFlags::TRANSFER_SRC; // Needed for blitting from mip levels
    }

    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(mip_levels) // Set correct mip_levels
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(image_usage) // Use updated usage
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let (dst_image, dst_allocation) = super::memory::create_image(
        allocator,
        device,
        &image_create_info,
        MemoryLocation::GpuOnly,
    )?;

    // 3. Record and submit command buffer for copy and layout transitions
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    // Transition base mip level (level 0) for initial data copy
    transition_image_layout(
        device, command_buffer, dst_image,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        0, 1, // base_mip_level: 0, level_count: 1
    );

    let copy_region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0) // 0 means tightly packed
        .buffer_image_height(0) // 0 means tightly packed
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            staging_buffer,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_region.build()],
        );
    }

    if generate_mipmaps && mip_levels > 1 {
        // Transition mip level 0 to TRANSFER_SRC_OPTIMAL for generating subsequent mips
        transition_image_layout(
            device, command_buffer, dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            0, 1, // base_mip_level: 0, level_count: 1
        );
        generate_mipmaps_for_image(device, command_buffer, dst_image, format, width, height, mip_levels)?;
        // generate_mipmaps_for_image ensures all mip levels (0 to mip_levels-1) are SHADER_READ_ONLY_OPTIMAL
    } else {
        // If no mipmaps, transition the single level (level 0) to SHADER_READ_ONLY_OPTIMAL
        transition_image_layout(
            device, command_buffer, dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            0, 1, // base_mip_level: 0, level_count: 1
        );
    }

    end_single_time_commands(device, command_pool, command_buffer, graphics_queue)?;

    // 4. Cleanup staging buffer
    unsafe {
        allocator.free(staging_allocation).map_err(|e| format!("Failed to free staging allocation: {:?}", e))?;
        device.destroy_buffer(staging_buffer, None);
    }

    // 5. Create ImageView for the destination image
    let view_create_info = vk::ImageViewCreateInfo::builder()
        .image(dst_image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: mip_levels, // ImageView covers all mip levels
            base_array_layer: 0,
            layer_count: 1,
        });
    let image_view = unsafe {
        device.create_image_view(&view_create_info, None)
            .map_err(|e| format!("Failed to create image view for GPU texture: {}", e))?
    };

    // WP-501: Create sampler for the texture
    let sampler = create_texture_sampler(device, pdevice_properties, mip_levels, enable_anisotropy_for_sampler)?;

    let texture_id = TextureId::new();
    Ok(GpuTexture {
        id: texture_id,
        image: dst_image,
        image_view,
        sampler, // Stored sampler
        allocation: Some(dst_allocation),
        memory: None,
        format,
        extent: image_extent,
        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        mip_levels,
    })
}

pub fn update_texture_region(
    device: &ash::Device,
    allocator: &mut Allocator,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    gpu_texture: &mut GpuTexture, // Mutably to update layout
    region_to_update: vk::Rect2D,
    pixel_data: &[u8],
) -> Result<(), String> {
    // Assuming pixel_data matches the size of region_to_update and texture format
    let bytes_per_pixel = match gpu_texture.format {
        vk::Format::R8G8B8A8_SRGB | vk::Format::B8G8R8A8_SRGB |
        vk::Format::R8G8B8A8_UNORM | vk::Format::B8G8R8A8_UNORM => 4,
        _ => return Err(format!("Unsupported format for region update size calculation: {:?}", gpu_texture.format)),
    };
    let expected_data_size = (region_to_update.extent.width * region_to_update.extent.height * bytes_per_pixel) as usize;
    if pixel_data.len() != expected_data_size {
        return Err(format!(
            "Pixel data size {} does not match expected size {} for region {:?} and format {:?}",
            pixel_data.len(),
            expected_data_size,
            region_to_update,
            gpu_texture.format
        ));
    }
    if region_to_update.offset.x < 0 || region_to_update.offset.y < 0 ||
       (region_to_update.offset.x as u32 + region_to_update.extent.width) > gpu_texture.extent.width ||
       (region_to_update.offset.y as u32 + region_to_update.extent.height) > gpu_texture.extent.height {
        return Err(format!("Update region {:?} is outside texture bounds {:?}", region_to_update, gpu_texture.extent));
    }

    let buffer_size = pixel_data.len() as vk::DeviceSize;

    // 1. Create Staging Buffer
    let (staging_buffer, mut staging_allocation) = super::memory::create_buffer(
        allocator,
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;

    // Copy pixel data to staging buffer
    unsafe {
        let data_ptr = staging_allocation.mapped_ptr().ok_or("Failed to map staging buffer for region update")?.as_ptr() as *mut u8;
        if data_ptr.is_null() { return Err("Mapped staging buffer pointer is null for region update".to_string()); }
        std::ptr::copy_nonoverlapping(pixel_data.as_ptr(), data_ptr, pixel_data.len());
        // Assuming CpuToGpu is coherent. If not, flush might be needed.
    }

    // 2. Record and submit command buffer for layout transitions and copy
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    // Transition only mip level 0 for the partial update
    // The current layout of the texture is stored in gpu_texture.layout
    transition_image_layout(
        device,
        command_buffer,
        gpu_texture.image,
        gpu_texture.layout, // Current layout
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        0, // base_mip_level
        1, // level_count (only mip level 0)
    );

    let copy_region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0) // Tightly packed
        .buffer_image_height(0) // Tightly packed
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0, // Update only base mip level
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_offset(vk::Offset3D { x: region_to_update.offset.x, y: region_to_update.offset.y, z: 0 })
        .image_extent(vk::Extent3D { width: region_to_update.extent.width, height: region_to_update.extent.height, depth: 1 });

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            staging_buffer,
            gpu_texture.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_region.build()],
        );
    }

    transition_image_layout(
        device,
        command_buffer,
        gpu_texture.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, // Transition back to shader read
        0, // base_mip_level
        1, // level_count
    );

    end_single_time_commands(device, command_pool, command_buffer, graphics_queue)?;

    // Cleanup staging buffer
    unsafe {
        allocator.free(staging_allocation).map_err(|e| format!("Failed to free staging allocation for region update: {:?}", e))?;
        device.destroy_buffer(staging_buffer, None);
    }

    gpu_texture.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

    Ok(())
}
