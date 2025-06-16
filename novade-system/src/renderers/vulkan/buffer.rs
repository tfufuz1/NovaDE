use ash::{vk, Device as AshDevice};
use gpu_allocator::vulkan as vma;
use gpu_allocator::MemoryUsage as GpuMemoryUsage;
use std::sync::{Arc, Mutex};

// ANCHOR: VulkanBuffer Struct Definition
pub struct VulkanBuffer {
    device: Arc<AshDevice>,
    allocator: Arc<Mutex<vma::Allocator>>,
    pub buffer: vk::Buffer, // Made public for easy access in command recording
    allocation: Option<vma::Allocation>, // Option to allow taking it in drop
    pub size: vk::DeviceSize, // Made public for information
}

// ANCHOR: VulkanBuffer Implementation
impl VulkanBuffer {
    pub fn new(
        device: Arc<AshDevice>,
        allocator: Arc<Mutex<vma::Allocator>>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_usage: GpuMemoryUsage, // e.g., CpuToGpu, GpuOnly
    ) -> Result<Self, String> {
        if size == 0 {
            return Err("Buffer size cannot be zero.".to_string());
        }

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE) // Or CONCURRENT if needed across queue families
            .build();

        let allocation_create_info = vma::AllocationCreateInfo {
            usage: memory_usage,
            // flags, required_flags, preferred_flags, pool can be specified if needed
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            allocator
                .lock()
                .map_err(|_| "Failed to lock allocator mutex".to_string())?
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .map_err(|e| format!("VMA failed to create buffer: {:?}", e))?
        };

        Ok(Self {
            device,
            allocator,
            buffer,
            allocation: Some(allocation),
            size,
        })
    }

    // ANCHOR_EXT: fill_from_slice
    /// Fills a portion of the buffer from a slice.
    /// The buffer must have been created with memory_usage that is HOST_VISIBLE (e.g., CpuToGpu, CpuOnly).
    /// T is the type of elements in the slice.
    pub fn fill_from_slice<T: Copy>(&self, data: &[T], slice_offset_bytes: vk::DeviceSize) -> Result<(), String> {
        let data_size_bytes = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;
        if data_size_bytes == 0 {
            return Ok(()); // Nothing to copy
        }
        if slice_offset_bytes + data_size_bytes > self.size {
            return Err(format!(
                "Data slice (offset: {}, size: {}) exceeds buffer bounds (size: {}).",
                slice_offset_bytes, data_size_bytes, self.size
            ));
        }

        let allocation = self.allocation.as_ref()
            .ok_or_else(|| "Buffer allocation is None, cannot map memory.".to_string())?;

        // Check if memory is host visible (mappable)
        // This check relies on how gpu-allocator sets memory properties.
        // A more robust check might involve querying allocation.memory_properties().
        if !allocation.is_host_visible() {
             return Err("Buffer memory is not host visible, cannot map for filling.".to_string());
        }


        let mapped_ptr = unsafe {
            allocation.map(self.device.as_ref(), slice_offset_bytes, data_size_bytes)
                .map_err(|e| format!("Failed to map buffer memory: {:?}", e))?
        };

        unsafe {
            // Assuming mapped_ptr is already correctly offset by allocation.map()
            // or that slice_offset_bytes for map was 0 and we handle offset here.
            // gpu_allocator's map function signature is `map(device, offset_in_allocation, size_to_map)`
            // So mapped_ptr is already at the correct offset within the allocation.
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const std::ffi::c_void,
                mapped_ptr.as_ptr() as *mut std::ffi::c_void,
                data_size_bytes as usize,
            );
        }

        // Flush memory if it's not coherent and host visible
        if allocation.needs_flush() {
            unsafe {
                allocation.flush(self.device.as_ref(), slice_offset_bytes, data_size_bytes)
                    .map_err(|e| format!("Failed to flush mapped buffer memory: {:?}", e))?;
            }
        }

        unsafe {
            allocation.unmap(self.device.as_ref());
        }

        Ok(())
    }

    // ANCHOR: Accessor for vk::Buffer
    pub fn handle(&self) -> vk::Buffer {
        self.buffer
    }

    // ANCHOR: Accessor for size
    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }
}

// ANCHOR: VulkanBuffer Drop Implementation
impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() { // Take ownership of allocation
            //println!("Dropping VulkanBuffer: {:?}, Allocation: {:?}", self.buffer, allocation.info());
            if let Ok(mut allocator_guard) = self.allocator.lock() {
                unsafe {
                    allocator_guard.destroy_buffer(self.buffer, allocation)
                        .unwrap_or_else(|e| eprintln!("VMA failed to destroy buffer: {:?}", e));
                }
            } else {
                eprintln!("VulkanBuffer::drop: Failed to lock allocator mutex. Buffer and allocation may leak.");
                // If mutex is poisoned, resources might leak. Consider alternative cleanup or panic.
            }
        } else {
            // This case should ideally not happen if allocation is always Some after new()
            // and only taken in drop. If it can be None otherwise, ensure no vkDestroyBuffer is called
            // without an allocator context if it was VMA managed.
            // If the buffer was not created by VMA (e.g. external), then this Drop is not appropriate.
            // For now, we assume all VulkanBuffer instances here own their allocation.
            // println!("VulkanBuffer::drop called on a buffer with no VMA allocation. Buffer handle: {:?}", self.buffer);
        }
    }
}
