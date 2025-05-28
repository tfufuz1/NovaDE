use crate::allocator::{Allocator, Allocation};
use crate::device::LogicalDevice;
use crate::error::{Result, VulkanError};
use std::mem::size_of;
// Arc is not directly used in this file's function signatures or struct, but often
// LogicalDevice itself is an Arc, so it's good to keep in mind for context.
// No, Arc is not needed here.
use vulkanalia::prelude::v1_0::*;


// Helper function for single-time command buffer operations (moved from texture.rs)
// Now public and part of buffer_utils
pub fn record_and_submit_command<F>(
    device: &Device,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    record_cb: F,
) -> Result<()>
where
    F: FnOnce(vk::CommandBuffer) -> Result<()>,
{
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);
    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }
        .map_err(VulkanError::VkResult)?[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
        .map_err(VulkanError::VkResult)?;

    record_cb(command_buffer)?;

    unsafe { device.end_command_buffer(command_buffer) }.map_err(VulkanError::VkResult)?;

    let submits = vk::SubmitInfo::builder().command_buffers(&[command_buffer]);
    let fence_info = vk::FenceCreateInfo::builder();
    let fence = unsafe { device.create_fence(&fence_info, None) }.map_err(VulkanError::VkResult)?;

    unsafe {
        device.queue_submit(transfer_queue, &[submits], fence)
            .map_err(VulkanError::VkResult)?;
        device.wait_for_fences(&[fence], true, u64::MAX)
            .map_err(VulkanError::VkResult)?;
        device.destroy_fence(fence, None);
        device.free_command_buffers(command_pool, &[command_buffer]);
    }
    Ok(())
}


/// Creates a GPU-accessible buffer (typically device-local) and fills it with the provided data.
/// Uses a temporary staging buffer for the data transfer if the target is GPU-only.
pub fn create_gpu_buffer_with_data<T: Copy>(
    logical_device_wrapper: &LogicalDevice,
    allocator: &Allocator,
    command_pool: vk::CommandPool, // Command pool for transfer operations
    transfer_queue: vk::Queue,     // Queue to submit transfer commands
    data: &[T],
    usage: vk::BufferUsageFlags,   // e.g., VERTEX_BUFFER_BIT, INDEX_BUFFER_BIT
) -> Result<(vk::Buffer, Allocation)> {
    // `device` is &Device, from `logical_device_wrapper.raw()` which returns &Arc<Device>
    // and then dereferencing it.
    let device = logical_device_wrapper.raw().as_ref(); 
    let buffer_size = (size_of::<T>() * data.len()) as vk::DeviceSize;

    if buffer_size == 0 {
        return Err(VulkanError::Message("Cannot create a buffer with zero size.".to_string()));
    }

    // 1. Create Staging Buffer (CPU visible)
    let staging_buffer_info = vk::BufferCreateInfo::builder()
        .size(buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE); // Exclusive for graphics/transfer queue family

    // VMA's CpuOnly or CpuToGpu is suitable for staging. CpuToGpu might pick BAR if available.
    // CpuOnly ensures it's host RAM.
    // HOST_ACCESS_SEQUENTIAL_WRITE is a hint for VMA to optimize memory type and mapping.
    let (staging_buffer, mut staging_allocation) = allocator.create_buffer(
        &staging_buffer_info,
        vk_mem_rs::MemoryUsage::CpuOnly, 
        Some(vk_mem_rs::AllocationCreateFlags::MAPPED | vk_mem_rs::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE)
    )?;

    // 2. Copy data to staging buffer
    unsafe {
        // VMA provides a mapped pointer if MAPPED flag was used at allocation.
        // This pointer is available via allocation.mapped_ptr().
        // If not mapped at creation, map_memory would be needed.
        // The provided code uses map_memory, which is fine, VMA will just return existing ptr if already mapped.
        let mapped_ptr = allocator.map_memory(&mut staging_allocation)?;
        std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_ptr as *mut T, data.len());
        
        // If the memory type chosen by VMA is not HOST_COHERENT, a flush is necessary.
        // HOST_ACCESS_SEQUENTIAL_WRITE might lead VMA to pick a write-combined (WC) memory
        // which is often not coherent and needs flushing.
        if !staging_allocation.get_memory_type_properties().contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
            allocator.flush_allocation(&staging_allocation, 0, vk::WHOLE_SIZE)?;
            log::trace!("Staging buffer flushed as it's not HOST_COHERENT.");
        }
        // No explicit unmap needed if MAPPED flag is used with VMA for allocation.
        // VMA handles unmapping on destroy if it was mapped during creation.
        // allocator.unmap_memory(&mut staging_allocation)?; // Only if mapped manually and not persistently
    }


    // 3. Create Device Local Buffer (GPU only)
    let device_buffer_info = vk::BufferCreateInfo::builder()
        .size(buffer_size)
        .usage(usage | vk::BufferUsageFlags::TRANSFER_DST) // Ensure TRANSFER_DST for copying
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let (device_buffer, device_allocation) = allocator.create_buffer(
        &device_buffer_info,
        vk_mem_rs::MemoryUsage::GpuOnly, // Target GPU-only memory
        None,
    )?;

    // 4. Copy from Staging to Device Local Buffer
    copy_buffer(
        logical_device_wrapper, // Pass the wrapper which contains Arc<Device>
        command_pool,
        transfer_queue,
        staging_buffer,
        device_buffer,
        buffer_size,
    )?;

    // 5. Cleanup Staging Buffer
    // This also implicitly unmaps if VMA had it mapped.
    allocator.destroy_buffer(staging_buffer, staging_allocation)?;
    
    log::debug!("Created GPU buffer with data (size: {}, usage: {:?})", buffer_size, usage);

    Ok((device_buffer, device_allocation))
}

/// Helper function to copy data from one buffer to another using a temporary command buffer.
pub fn copy_buffer(
    logical_device_wrapper: &LogicalDevice,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let device = logical_device_wrapper.raw().as_ref(); // Get &Device

    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }
        .map_err(VulkanError::VkResult)?[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
        .map_err(VulkanError::VkResult)?;

    let copy_region = vk::BufferCopy::builder().size(size); // srcOffset and dstOffset default to 0
    unsafe { device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]) };

    unsafe { device.end_command_buffer(command_buffer) }.map_err(VulkanError::VkResult)?;

    let submits = vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer)); // Use slice
    
    // Using a fence to wait for completion
    let fence_info = vk::FenceCreateInfo::builder();
    let fence = unsafe { device.create_fence(&fence_info, None) }.map_err(VulkanError::VkResult)?;

    unsafe {
        device.queue_submit(transfer_queue, std::slice::from_ref(&submits), fence) // Use slice
            .map_err(VulkanError::VkResult)?;
        device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) // Use slice
            .map_err(VulkanError::VkResult)?;
    }
    
    unsafe {
        device.destroy_fence(fence, None);
        device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer)); // Use slice
    }
    
    log::trace!("Buffer copied (size: {}) via temporary command buffer.", size);
    Ok(())
}

// TODO: Potentially add other buffer utilities:
// - create_uniform_buffer (CPU_TO_GPU, persistently mapped)
// - create_empty_device_local_buffer (e.g. for storage buffers filled by compute)
```
