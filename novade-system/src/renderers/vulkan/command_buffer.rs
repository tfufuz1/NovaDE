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
