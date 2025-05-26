use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo;
use crate::compositor::renderer::vulkan::device::LogicalDevice;
use ash::vk;
use log::{info, debug};
use vk_mem; // Use the vk_mem crate

/// A wrapper around the Vulkan Memory Allocator (VMA).
///
/// This struct handles the creation and destruction of the VMA allocator instance
/// and provides convenient wrapper functions for creating and destroying Vulkan
/// buffers and images with associated memory allocations.
#[derive(Debug)]
pub struct Allocator {
    raw: vk_mem::Allocator,
}

impl Allocator {
    /// Creates a new VMA Allocator.
    ///
    /// # Arguments
    /// * `vulkan_instance`: A reference to the `VulkanInstance`, used to get the
    ///   `ash::Instance` and the Vulkan API version.
    /// * `physical_device_info`: Information about the selected physical device.
    /// * `logical_device`: A reference to the `LogicalDevice` (ash::Device).
    ///
    /// # Returns
    /// `Result<Self, vk_mem::Error>` containing the new `Allocator` or an error.
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
    ) -> Result<Self, vk_mem::Error> {
        info!("Initializing Vulkan Memory Allocator (VMA)...");

        let create_info = vk_mem::AllocatorCreateInfo::new(
            vulkan_instance.raw(),      // ash::Instance
            &logical_device.raw,         // ash::Device
            physical_device_info.physical_device, // vk::PhysicalDevice
        )
        .set_instance_api_version(vulkan_instance.api_version());
        // For vk-mem 0.2.3, set_instance_api_version is correct.
        // For vk-mem 0.3.x and later, it would be .set_vulkan_api_version(vulkan_instance.api_version())
        // We can add flags here if needed, for example, if KHR_BIND_MEMORY2 or BufferDeviceAddress are used.
        // Example:
        // let mut flags = vk_mem::AllocatorCreateFlags::empty();
        // if some_condition_for_bind_memory2 {
        //     flags |= vk_mem::AllocatorCreateFlags::KHR_BIND_MEMORY_2;
        // }
        // create_info = create_info.set_flags(flags);

        let allocator = vk_mem::Allocator::new(&create_info)?;
        info!("VMA Initialized successfully.");
        Ok(Self { raw: allocator })
    }

    /// Creates a Vulkan buffer with memory allocated by VMA.
    ///
    /// # Arguments
    /// * `buffer_create_info`: A reference to `vk::BufferCreateInfo` describing the buffer.
    /// * `allocation_create_info`: A reference to `vk_mem::AllocationCreateInfo` describing
    ///   the memory allocation. This is where you specify `vk_mem::MemoryUsage`
    ///   and `vk_mem::AllocationCreateFlags`.
    ///
    /// # Memory Usage (`vk_mem::MemoryUsage` in `allocation_create_info.usage`):
    /// VMA uses these flags to select the optimal memory type, especially for UMA (Unified Memory Architecture):
    /// * `vk_mem::MemoryUsage::GpuOnly`: For resources exclusively accessed by the GPU (e.g., render targets,
    ///   most textures, vertex/index buffers). On UMA, VMA often maps this to `DEVICE_LOCAL | HOST_VISIBLE` memory.
    /// * `vk_mem::MemoryUsage::CpuToGpu`: For resources frequently written by the CPU and read by the GPU
    ///   (e.g., uniform buffers, dynamic vertex/index buffers). VMA aims for `HOST_VISIBLE` and `HOST_COHERENT`
    ///   or `HOST_CACHED`.
    /// * `vk_mem::MemoryUsage::GpuToCpu`: For resources written by the GPU and read by the CPU (e.g., readback buffers,
    ///   query results). VMA aims for `HOST_VISIBLE` and `HOST_CACHED`.
    /// * `vk_mem::MemoryUsage::CpuOnly`: Primarily for staging buffers that are only accessed by the CPU. VMA
    ///   typically uses `HOST_VISIBLE` and `HOST_COHERENT` memory.
    ///
    /// # Allocation Flags (`vk_mem::AllocationCreateFlags` in `allocation_create_info.flags`):
    /// * `vk_mem::AllocationCreateFlags::MAPPED`: Request the allocation to be persistently mapped.
    ///   The mapped pointer can be retrieved from `vk_mem::AllocationInfo.get_mapped_data_mut()`.
    /// * `vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE`: Hint for memory that will be written sequentially by the host.
    /// * `vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM`: Hint for memory that will be accessed randomly by the host.
    ///
    /// # Returns
    /// A tuple containing the `vk::Buffer`, `vk_mem::Allocation` handle, and `vk_mem::AllocationInfo`.
    pub fn create_buffer(
        &self,
        buffer_create_info: &vk::BufferCreateInfo,
        allocation_create_info: &vk_mem::AllocationCreateInfo,
    ) -> Result<(vk::Buffer, vk_mem::Allocation, vk_mem::AllocationInfo), vk_mem::Error> {
        debug!(
            "Creating buffer with VMA: size={}, usage={:?}, VMA usage={:?}, VMA flags={:?}",
            buffer_create_info.size,
            buffer_create_info.usage,
            allocation_create_info.usage,
            allocation_create_info.flags
        );
        let (buffer, allocation, allocation_info) =
            unsafe { self.raw.create_buffer(buffer_create_info, allocation_create_info)? };
        debug!("Buffer created successfully with VMA: {:?}, allocation: {:?}", buffer, allocation);
        Ok((buffer, allocation, allocation_info))
    }

    /// Destroys a Vulkan buffer and its associated VMA allocation.
    ///
    /// # Arguments
    /// * `buffer`: The `vk::Buffer` to destroy.
    /// * `allocation`: A reference to the `vk_mem::Allocation` to free.
    pub fn destroy_buffer(&self, buffer: vk::Buffer, allocation: &vk_mem::Allocation) {
        debug!("Destroying buffer {:?} with VMA allocation {:?}", buffer, allocation);
        unsafe {
            self.raw.destroy_buffer(buffer, allocation);
        }
        debug!("Buffer and allocation destroyed.");
    }

    /// Creates a Vulkan image with memory allocated by VMA.
    ///
    /// See `create_buffer` for documentation on `vk_mem::MemoryUsage` and `vk_mem::AllocationCreateFlags`.
    ///
    /// # Arguments
    /// * `image_create_info`: A reference to `vk::ImageCreateInfo` describing the image.
    /// * `allocation_create_info`: A reference to `vk_mem::AllocationCreateInfo` describing
    ///   the memory allocation.
    ///
    /// # Returns
    /// A tuple containing the `vk::Image`, `vk_mem::Allocation` handle, and `vk_mem::AllocationInfo`.
    pub fn create_image(
        &self,
        image_create_info: &vk::ImageCreateInfo,
        allocation_create_info: &vk_mem::AllocationCreateInfo,
    ) -> Result<(vk::Image, vk_mem::Allocation, vk_mem::AllocationInfo), vk_mem::Error> {
        debug!(
            "Creating image with VMA: format={:?}, extent=({:?},{:?},{:?}), usage={:?}, VMA usage={:?}, VMA flags={:?}",
            image_create_info.format,
            image_create_info.extent.width, image_create_info.extent.height, image_create_info.extent.depth,
            image_create_info.usage,
            allocation_create_info.usage,
            allocation_create_info.flags
        );
        let (image, allocation, allocation_info) =
            unsafe { self.raw.create_image(image_create_info, allocation_create_info)? };
        debug!("Image created successfully with VMA: {:?}, allocation: {:?}", image, allocation);
        Ok((image, allocation, allocation_info))
    }

    /// Destroys a Vulkan image and its associated VMA allocation.
    ///
    /// # Arguments
    /// * `image`: The `vk::Image` to destroy.
    /// * `allocation`: A reference to the `vk_mem::Allocation` to free.
    pub fn destroy_image(&self, image: vk::Image, allocation: &vk_mem::Allocation) {
        debug!("Destroying image {:?} with VMA allocation {:?}", image, allocation);
        unsafe {
            self.raw.destroy_image(image, allocation);
        }
        debug!("Image and allocation destroyed.");
    }

    /// Destroys the VMA allocator instance.
    /// This is called automatically when the `Allocator` is dropped.
    pub fn destroy(&mut self) {
        info!("Destroying VMA Allocator...");
        unsafe {
            self.raw.destroy();
        }
        info!("VMA Allocator destroyed.");
    }

    /// Provides access to the raw `vk_mem::Allocator` if needed for more advanced operations.
    pub fn raw_allocator(&self) -> &vk_mem::Allocator {
        &self.raw
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        self.destroy();
    }
}
