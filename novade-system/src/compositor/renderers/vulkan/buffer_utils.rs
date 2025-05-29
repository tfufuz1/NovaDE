//! Provides utility functions for creating and managing Vulkan buffers.
//!
//! This module includes helpers for common buffer operations, such as creating
//! GPU-only buffers (e.g., vertex or index buffers) and filling them with data
//! from the CPU. It leverages staging buffers for efficient data transfer to
//! device-local memory, utilizing the VMA library for memory allocation and
//! the `record_one_time_submit_commands` helper for managing transfer commands.

use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice,
    texture::record_one_time_submit_commands, // Re-uses helper from texture module
    error::{Result, VulkanError},
};
use ash::vk;
use bytemuck::Pod; // Trait for types that can be safely cast to a byte slice
use log::{debug, info, error};
use vk_mem;

/// Creates a GPU-only buffer and fills it with data from a slice using a staging buffer.
///
/// This function is a generic way to create buffers on the GPU (e.g., vertex buffers,
/// index buffers, or uniform buffers that are not frequently updated) and initialize
/// them with data from the CPU. It performs the following steps:
/// 1. Creates a CPU-visible staging buffer (`CpuToGpu` memory).
/// 2. Copies the provided `data` slice into the mapped staging buffer.
/// 3. Creates the final GPU-only destination buffer (`GpuOnly` memory) with the specified `usage` flags
///    (plus `TRANSFER_DST` for the copy operation).
/// 4. Records and submits a one-time command buffer to copy data from the staging buffer
///    to the destination GPU buffer.
/// 5. Destroys the staging buffer and its allocation after the copy is complete.
///
/// # Type Parameters
///
/// * `T`: The type of data elements in the `data` slice. This type must implement `bytemuck::Pod`,
///   which ensures it's plain old data and can be safely cast to a byte slice (`&[u8]`) for copying.
///
/// # Arguments
///
/// * `allocator`: A reference to the VMA `Allocator` used for creating both staging and final buffers.
/// * `logical_device`: A reference to the `LogicalDevice` for Vulkan operations.
/// * `command_pool`: The `vk::CommandPool` from which to allocate the temporary command buffer for the transfer.
/// * `transfer_queue`: The `vk::Queue` to submit the transfer command buffer to (often the graphics queue).
/// * `data`: A slice of data of type `T` to be copied to the GPU buffer. The function will return an error
///   if this slice is empty.
/// * `usage`: The `vk::BufferUsageFlags` for the destination GPU buffer (e.g., `VERTEX_BUFFER`, `INDEX_BUFFER`).
///   The `vk::BufferUsageFlags::TRANSFER_DST` flag will be added to this automatically to allow
///   the staging buffer to copy data into it.
///
/// # Returns
///
/// A `Result` containing a tuple `(vk::Buffer, vk_mem::Allocation)` for the created GPU buffer
/// and its VMA allocation on success.
/// On failure, returns a `VulkanError`. Possible errors include:
/// - `VulkanError::ResourceCreationError`: If creating the staging or GPU buffer fails, or if `data` is empty.
/// - Errors propagated from `allocator.create_buffer()` (typically `VulkanError::VkMemError`).
/// - Errors propagated from `record_one_time_submit_commands` (typically `VulkanError::VkResult`).
///
/// # Panics
///
/// This function might panic if `std::mem::size_of_val(data)` is zero but `data` is not empty,
/// which should not happen for `Pod` types with non-zero size.
pub fn create_and_fill_gpu_buffer<T: Pod>(
    allocator: &Allocator,
    logical_device: &LogicalDevice,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    data: &[T],
    usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk_mem::Allocation)> {
    let buffer_size = (std::mem::size_of_val(data)) as vk::DeviceSize;
    if buffer_size == 0 {
        let err_msg = "Cannot create GPU buffer with empty data.".to_string();
        error!("{}", err_msg);
        return Err(VulkanError::ResourceCreationError{
            resource_type: format!("GPU Buffer (usage: {:?})", usage),
            message: err_msg,
        });
    }
    info!("Creating GPU buffer of size {} bytes for usage: {:?}", buffer_size, usage);

    // 1. Create Staging Buffer
    let staging_buffer_create_info = vk::BufferCreateInfo::builder()
        .size(buffer_size).usage(vk::BufferUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let staging_allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::CpuToGpu,
        flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        ..Default::default()
    };

    let (staging_buffer, staging_allocation, staging_alloc_info) = allocator
        .create_buffer(&staging_buffer_create_info, &staging_allocation_create_info)
        .map_err(|e| VulkanError::ResourceCreationError{
            resource_type: "StagingBuffer".to_string(),
            message: format!("Failed to create staging buffer for {:?}: {}", usage, e)
        })?;
    debug!("Staging buffer created: {:?}", staging_buffer);

    // Copy data to staging buffer
    unsafe {
        let mapped_data = staging_alloc_info.get_mapped_data_mut();
        let byte_data = bytemuck::cast_slice(data); // Safe because T: Pod
        std::ptr::copy_nonoverlapping(byte_data.as_ptr(), mapped_data as *mut u8, byte_data.len());
    }
    debug!("Data copied to staging buffer ({} bytes).", buffer_size);

    // 2. Create Destination GPU Buffer
    let final_buffer_usage = usage | vk::BufferUsageFlags::TRANSFER_DST;
    let final_buffer_create_info = vk::BufferCreateInfo::builder()
        .size(buffer_size).usage(final_buffer_usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let final_allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default()
    };

    let (gpu_buffer, gpu_allocation, _gpu_alloc_info) = allocator
        .create_buffer(&final_buffer_create_info, &final_allocation_create_info)
        .map_err(|e| VulkanError::ResourceCreationError{
             resource_type: format!("GPU Buffer (usage: {:?})", usage),
             message: format!("Failed to create GPU buffer: {}", e)
        })?;
    debug!("GPU destination buffer created: {:?}", gpu_buffer);

    // 3. Copy data from staging to GPU buffer using one-time commands
    record_one_time_submit_commands(logical_device, command_pool, transfer_queue, |cmd_buffer| {
        let buffer_copy_region = vk::BufferCopy::builder().src_offset(0).dst_offset(0).size(buffer_size);
        // # Safety for cmd_copy_buffer:
        // - `cmd_buffer` is a valid primary command buffer in the recording state.
        // - `staging_buffer` and `gpu_buffer` are valid `vk::Buffer` handles.
        // - `buffer_copy_region` defines a valid copy operation within the bounds of both buffers.
        // - Both buffers were created with appropriate usage flags (`TRANSFER_SRC` for staging, `TRANSFER_DST` for GPU).
        // - Host writes to staging buffer are implicitly made available to device via memory mapping properties or explicit flush (if needed, though MAPPED + HOST_ACCESS often doesn't).
        // - No other operations are concurrently accessing the specified regions of these buffers on the GPU.
        unsafe {
            logical_device.raw.cmd_copy_buffer(cmd_buffer, staging_buffer, gpu_buffer, &[buffer_copy_region.build()]);
        }
    })?;
    debug!("Data copied from staging to GPU buffer via command buffer.");

    // 4. Destroy Staging Buffer
    allocator.destroy_buffer(staging_buffer, &staging_allocation); // Assumes Allocator handles synchronization for this buffer
    debug!("Staging buffer destroyed.");

    info!("GPU buffer {:?} created and filled successfully for usage {:?}.", gpu_buffer, usage);
    Ok((gpu_buffer, gpu_allocation))
}
