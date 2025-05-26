//! Manages dynamic uniform buffers for efficient updates of per-object data.
//!
//! This module provides `DynamicUboManager`, a generic struct that encapsulates
//! the creation and management of large uniform buffers. These buffers are designed
//! to hold multiple instances of a Uniform Buffer Object (UBO) type `T`.
//! Each instance within the buffer can be accessed using a dynamic offset when binding
//! descriptor sets, allowing for efficient updates and usage of UBO data for many
//! objects with a single descriptor set binding per frame.

use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice,
    error::{Result, VulkanError},
    frame_renderer::MAX_FRAMES_IN_FLIGHT, // To know how many per-frame buffers to create
};
use ash::vk;
use bytemuck::{Pod, Zeroable}; // Require Pod and Zeroable for safety with mapped memory
use log::{debug, info, error};
use std::ffi::c_void;
use vk_mem;

/// Manages a set of large uniform buffers, one for each frame in flight,
/// designed to store multiple instances of a UBO type `T` accessed via dynamic offsets.
///
/// This manager simplifies the handling of dynamic UBOs by:
/// - Calculating the required alignment for UBO items based on device properties.
/// - Creating appropriately sized buffers (one per frame in flight) that can hold
///   a specified maximum number of UBO items.
/// - Providing a method to update individual UBO items within these buffers.
/// - Offering accessors for buffer handles and size information needed for descriptor set
///   binding and command recording.
/// - Ensuring proper cleanup of all allocated buffers and memory via the `Drop` trait.
///
/// # Type Parameters
///
/// * `T`: The type of the Uniform Buffer Object data structure. Must implement `Copy`
///   (to allow writing data by value), `Pod` (Plain Old Data, for safe memory casting/copying),
///   and `Zeroable` (for safety with memory operations, though not strictly used if data is always initialized).
#[derive(Debug)]
pub struct DynamicUboManager<T: Copy + Pod + Zeroable> {
    /// Vector of Vulkan buffer handles (`vk::Buffer`), one for each frame in flight.
    /// Each buffer is large enough to hold `max_items` of `T`, respecting alignment.
    buffers: Vec<vk::Buffer>,
    /// Vector of VMA allocations (`vk_mem::Allocation`) corresponding to the `buffers`.
    allocations: Vec<vk_mem::Allocation>,
    /// Vector of raw mapped memory pointers (`*mut c_void`) for each buffer,
    /// allowing direct CPU writes into the UBO data.
    mapped_pointers: Vec<*mut c_void>,
    /// The size of a single item of type `T` in bytes, padded to meet the
    /// `minUniformBufferOffsetAlignment` requirement of the physical device.
    /// This is the stride between UBO instances in the buffer.
    aligned_item_size: vk::DeviceSize,
    /// The maximum number of items of type `T` that can be stored in each per-frame buffer.
    max_items: usize,
    /// A cloned `ash::Device` handle, kept for resource cleanup in the `Drop` implementation.
    logical_device_raw: ash::Device,
    /// A cloned `vk_mem::Allocator` handle, kept for freeing VMA allocations in `Drop`.
    allocator_raw_clone: vk_mem::Allocator,
    /// The actual size of a single UBO item `T` (unaligned). Stored for use in
    /// `VkDescriptorBufferInfo.range`.
    item_size: vk::DeviceSize,
}

impl<T: Copy + Pod + Zeroable> DynamicUboManager<T> {
    /// Creates a new `DynamicUboManager`.
    ///
    /// This function initializes one large uniform buffer for each frame in flight
    /// (defined by `MAX_FRAMES_IN_FLIGHT`). Each buffer is sized to hold `max_items`
    /// of type `T`. The size of each item slot within the buffer (`aligned_item_size`)
    /// is calculated by taking `std::mem::size_of::<T>()` and padding it to meet the
    /// `minUniformBufferOffsetAlignment` requirement from `physical_device_properties`.
    /// The buffers are created with `vk::BufferUsageFlags::UNIFORM_BUFFER` and allocated
    /// from CPU-to-GPU visible memory, persistently mapped for direct writes.
    ///
    /// # Arguments
    ///
    /// * `allocator`: A reference to the VMA `Allocator` used for buffer creation.
    /// * `logical_device`: A reference to the `LogicalDevice` for Vulkan operations.
    /// * `physical_device_properties`: Vulkan `vk::PhysicalDeviceProperties`, used to query
    ///   `minUniformBufferOffsetAlignment`.
    /// * `max_items`: The maximum number of UBO instances (of type `T`) that each
    ///   per-frame buffer should be able to hold.
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized `DynamicUboManager<T>` on success.
    /// On failure, returns a `VulkanError`. Possible errors include:
    /// - `VulkanError::InitializationError`: If `std::mem::size_of::<T>()` is zero.
    /// - `VulkanError::ResourceCreationError`: If buffer creation or VMA allocation fails.
    /// - Errors propagated from `allocator.create_buffer()`.
    pub fn new(
        allocator: &Allocator,
        logical_device: &LogicalDevice,
        physical_device_properties: &vk::PhysicalDeviceProperties,
        max_items: usize,
    ) -> Result<Self> {
        let item_size_unaligned = std::mem::size_of::<T>() as vk::DeviceSize;
        if item_size_unaligned == 0 {
            return Err(VulkanError::InitializationError("UBO item size cannot be zero for DynamicUboManager.".to_string()));
        }

        let alignment = physical_device_properties.limits.min_uniform_buffer_offset_alignment;
        let aligned_item_size = if alignment > 0 {
            (item_size_unaligned + alignment - 1) & !(alignment - 1)
        } else {
            // This case should ideally not happen as minUniformBufferOffsetAlignment is usually positive.
            // If it were 0 or 1, no special alignment is needed beyond natural alignment of T.
            warn!("minUniformBufferOffsetAlignment is 0 or invalid (value: {}). Using unaligned item size.", alignment);
            item_size_unaligned 
        };
        info!(
            "DynamicUboManager: Item size unaligned: {}, minAlignment: {}, calculated aligned item size: {}, max_items per frame buffer: {}",
            item_size_unaligned, alignment, aligned_item_size, max_items
        );

        let total_buffer_size_per_frame = max_items as vk::DeviceSize * aligned_item_size;
        if total_buffer_size_per_frame == 0 {
             return Err(VulkanError::InitializationError(
                 format!("Total buffer size for dynamic UBO is zero (max_items: {}, aligned_item_size: {}).", max_items, aligned_item_size)
            ));
        }


        let mut buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut allocations = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut mapped_pointers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_create_info = vk::BufferCreateInfo::builder()
                .size(total_buffer_size_per_frame)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE); // Usually fine for UBOs

            let allocation_create_info = vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu, // For buffers written by CPU, read by GPU
                flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            };

            let (buffer, allocation, alloc_info) = allocator
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .map_err(|e| VulkanError::ResourceCreationError {
                    resource_type: "DynamicUniformBuffer".to_string(),
                    message: format!("Failed to create dynamic UBO for frame {}: {}", i, e),
                })?;
            
            buffers.push(buffer);
            allocations.push(allocation);
            mapped_pointers.push(alloc_info.get_mapped_data_mut()); // Store the mapped pointer
            debug!("Dynamic UBO for frame {} created: {:?}, total size: {}, mapped pointer: {:p}", 
                   i, buffer, total_buffer_size_per_frame, mapped_pointers.last().unwrap());
        }

        Ok(Self {
            buffers, allocations, mapped_pointers,
            aligned_item_size, max_items,
            logical_device_raw: logical_device.raw.clone(),
            allocator_raw_clone: allocator.raw_allocator().clone(),
            item_size: item_size_unaligned,
        })
    }

    /// Updates the UBO data for a specific item at a given index within a specific frame's buffer.
    ///
    /// The data is copied into the persistently mapped buffer at an offset calculated
    /// using `item_index` and `self.aligned_item_size`.
    ///
    /// # Arguments
    ///
    /// * `frame_index`: The index of the frame in flight (0 to `MAX_FRAMES_IN_FLIGHT - 1`),
    ///   determining which of the per-frame buffers to update.
    /// * `item_index`: The index of the item within the dynamic UBO array for that frame
    ///   (0 to `self.max_items - 1`).
    /// * `data`: The data of type `T` to write into the buffer.
    ///
    /// # Returns
    ///
    /// `Result<()>` indicating success, or a `VulkanError::InitializationError` if `frame_index`
    /// or `item_index` are out of bounds, or if the buffer for `frame_index` is not mapped.
    ///
    /// # Safety
    ///
    /// - `frame_index` must be less than `MAX_FRAMES_IN_FLIGHT`.
    /// - `item_index` must be less than `self.max_items`.
    /// - The memory pointed to by `self.mapped_pointers[frame_index]` must be valid and correctly mapped
    ///   by VMA. The `MAPPED` flag during allocation ensures this.
    /// - `data` must be a valid instance of type `T`.
    /// - Concurrent writes to the same memory location from multiple threads must be synchronized externally.
    pub fn update_data(&self, frame_index: usize, item_index: usize, data: T) -> Result<()> {
        if frame_index >= MAX_FRAMES_IN_FLIGHT {
            let msg = format!("Invalid frame_index {} for dynamic UBO update (max: {}).", frame_index, MAX_FRAMES_IN_FLIGHT - 1);
            error!("{}", msg); return Err(VulkanError::InitializationError(msg));
        }
        if item_index >= self.max_items {
            let msg = format!("Invalid item_index {} for dynamic UBO update (max_items: {}).", item_index, self.max_items);
            error!("{}", msg); return Err(VulkanError::InitializationError(msg));
        }

        let ptr_frame_base = self.mapped_pointers[frame_index];
        if ptr_frame_base.is_null() {
            let msg = format!("Dynamic UBO for frame_index {} is not mapped (pointer is null).", frame_index);
            error!("{}", msg); return Err(VulkanError::InitializationError(msg));
        }

        let offset = item_index as vk::DeviceSize * self.aligned_item_size;
        // # Safety:
        // - `ptr_frame_base` is a valid pointer to the start of mapped memory for this frame's buffer.
        // - `offset` is calculated to be within the bounds of this mapped buffer because `item_index < self.max_items`
        //   and `total_buffer_size_per_frame = max_items * aligned_item_size`.
        // - The pointer `final_ptr` will be correctly aligned for type `T` because `aligned_item_size`
        //   respects `minUniformBufferOffsetAlignment`, and `T` itself has natural alignment.
        // - `data` is `Copy`, so it can be safely copied. `T: Pod` ensures it's safe to treat as bytes if needed.
        unsafe {
            let final_ptr = ptr_frame_base.add(offset as usize) as *mut T;
            // *final_ptr = data; // Direct assignment is fine for `Copy` types.
            std::ptr::copy_nonoverlapping(&data, final_ptr, 1); // More explicit about the copy.
        }
        Ok(())
    }

    /// Returns the raw `vk::Buffer` handle for the specified frame index.
    ///
    /// This handle is used when configuring `VkDescriptorBufferInfo` for binding
    /// the dynamic UBO to a descriptor set. The entire buffer for the frame is bound.
    ///
    /// # Arguments
    ///
    /// * `frame_index`: The index of the frame in flight.
    ///
    /// # Returns
    /// The `vk::Buffer` handle for the given frame.
    ///
    /// # Panics
    /// Panics if `frame_index` is greater than or equal to `MAX_FRAMES_IN_FLIGHT` (or `self.buffers.len()`).
    pub fn get_buffer(&self, frame_index: usize) -> vk::Buffer {
        // This could panic if frame_index is out of bounds.
        // Consider returning Option<vk::Buffer> or Result<vk::Buffer> if called with arbitrary indices.
        // However, in the context of FrameRenderer, frame_index is always current_frame_index % MAX_FRAMES_IN_FLIGHT.
        self.buffers[frame_index]
    }
    
    /// Returns the unaligned size of a single UBO item (`T`).
    ///
    /// This value is typically used for the `range` field in `VkDescriptorBufferInfo`
    /// when updating a descriptor set for a dynamic uniform buffer, as shaders usually
    /// expect to see a single instance of the UBO structure.
    pub fn get_item_size_for_descriptor(&self) -> vk::DeviceSize {
        self.item_size
    }

    /// Returns the aligned size of a single UBO item slot in the buffer.
    ///
    /// This value (stride) is crucial for calculating the dynamic offset when binding
    /// the descriptor set using `vkCmdBindDescriptorSets`. The offset for item `i` is
    /// `i * get_aligned_item_size()`.
    pub fn get_aligned_item_size(&self) -> vk::DeviceSize {
        self.aligned_item_size
    }

    /// Returns the maximum number of UBO items (`T`) that can be stored in each
    /// per-frame buffer managed by this instance.
    pub fn get_max_items(&self) -> usize {
        self.max_items
    }
}

impl<T: Copy + Pod + Zeroable> Drop for DynamicUboManager<T> {
    /// Cleans up all Vulkan buffers and their associated VMA allocations.
    ///
    /// This is called automatically when the `DynamicUboManager` goes out of scope.
    /// It iterates through all per-frame buffers and destroys them using the
    /// cloned VMA allocator handle.
    ///
    /// # Safety
    ///
    /// - The `logical_device_raw` and `allocator_raw_clone` handles stored within this struct
    ///   must still be valid Vulkan handles when `Drop` is called.
    /// - The caller must ensure that these uniform buffers are not in use by any pending
    ///   GPU operations when the `DynamicUboManager` is dropped. Typically, this means
    ///   waiting for the device to be idle before allowing the manager to be dropped.
    fn drop(&mut self) {
        debug!("Dropping DynamicUboManager (managing {} buffers of {} items each, aligned item size {})...", 
            self.buffers.len(), self.max_items, self.aligned_item_size);
        for i in 0..self.buffers.len() {
            // Mapped pointers from VMA with MAPPED flag are typically unmapped automatically on destroy_buffer/free.
            // No explicit unmap call is usually needed with vk-mem-rs.
            debug!("Destroying dynamic UBO for frame {}: buffer {:?}, allocation {:?}", i, self.buffers[i], self.allocations[i]);
            self.allocator_raw_clone.destroy_buffer(self.buffers[i], &self.allocations[i]);
        }
        self.buffers.clear();
        self.allocations.clear();
        self.mapped_pointers.clear(); // Pointers are now invalid
        debug!("DynamicUboManager resources destroyed.");
    }
}
