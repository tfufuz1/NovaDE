// novade-system/src/renderers/vulkan/command_buffer.rs
use ash::vk;

pub fn begin_single_time_commands(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer, String> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe {
        device.allocate_command_buffers(&alloc_info)
            .map_err(|e| format!("Failed to allocate single time command buffer: {}", e))?
    }[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| format!("Failed to begin single time command buffer: {}", e))?;
    }
    Ok(command_buffer)
}

pub fn end_single_time_commands(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    queue: vk::Queue,
) -> Result<(), String> {
    unsafe {
        device.end_command_buffer(command_buffer)
            .map_err(|e| format!("Failed to end single time command buffer: {}", e))?;
    }

    let submit_info = vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));

    unsafe {
        device.queue_submit(queue, &[submit_info.build()], vk::Fence::null())
            .map_err(|e| format!("Failed to submit single time command buffer: {}", e))?;
        device.queue_wait_idle(queue) // Simplest synchronization for single time commands
            .map_err(|e| format!("Failed to wait for queue idle: {}", e))?;
        device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer));
    }
    Ok(())
}

// ANCHOR: copy_buffer_to_buffer_sync
pub fn copy_buffer_to_buffer_sync(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<(), String> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let copy_region = vk::BufferCopy::builder().size(size).build(); // src_offset and dst_offset are 0
    unsafe {
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);
    }

    end_single_time_commands(device, command_pool, command_buffer, queue)?;
    Ok(())
}

// ANCHOR: transition_image_layout_sync
pub fn transition_image_layout_sync(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    mip_levels: u32,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<(), String> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(mip_levels)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        );

    // Set access masks based on layouts
    // This is a simplified version; more specific masks might be needed for complex scenarios.
    let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            // Add other common transitions as needed
            _ => {
                return Err(format!(
                    "Unsupported layout transition from {:?} to {:?}",
                    old_layout, new_layout
                ));
            }
        };

    barrier = barrier.src_access_mask(src_access_mask).dst_access_mask(dst_access_mask);

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[], // No memory barriers
            &[], // No buffer memory barriers
            &[barrier.build()],
        );
    }

    end_single_time_commands(device, command_pool, command_buffer, queue)?;
    Ok(())
}

// ANCHOR: copy_buffer_to_image_sync
pub fn copy_buffer_to_image_sync(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
    extent: vk::Extent3D, // width, height, depth of the image region to copy
) -> Result<(), String> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0) // Tightly packed
        .buffer_image_height(0) // Tightly packed
        .image_subresource(
            vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(extent)
        .build();

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, // Image must be in this layout
            &[region],
        );
    }

    end_single_time_commands(device, command_pool, command_buffer, queue)?;
    Ok(())
}
