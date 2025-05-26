//! Wraps the Vulkan Memory Allocator (VMA) library for efficient memory management.
//!
//! This module provides an `Allocator` struct that simplifies the creation and management
//! of Vulkan buffers and images by leveraging the VMA library. It handles the
//! initialization of the VMA allocator specific to the selected Vulkan instance,
//! physical device, and logical device. It also offers convenient wrapper functions
//! for common resource allocation tasks, ensuring proper memory usage and flags
//! are set as per VMA's recommendations.

use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo;
use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::vk;
use log::{info, debug};
use vk_mem;

/// A wrapper around the Vulkan Memory Allocator (VMA) instance.
///
/// This struct handles the creation and destruction of the VMA allocator and provides
/// methods to create Vulkan buffers and images with memory managed by VMA.
/// It is responsible for initializing the VMA library with the appropriate Vulkan
/// instance, physical device, and logical device handles.
///
/// The `Allocator` implements `Drop` to ensure that the underlying VMA allocator
/// is destroyed when this struct goes out of scope.
#[derive(Debug)]
pub struct Allocator {
    /// The raw `vk_mem::Allocator` handle from the `vk-mem-rs` crate.
    raw: vk_mem::Allocator,
}

impl Allocator {
    /// Creates a new VMA `Allocator`.
    ///
    /// Initializes the VMA library for the given Vulkan setup. This involves passing
    /// the `ash::Instance`, `ash::Device` (logical device), and `vk::PhysicalDevice`
    /// to `vk_mem::AllocatorCreateInfo`. The Vulkan API version used by the instance
    /// is also provided to VMA.
    ///
    /// # Arguments
    ///
    /// * `vulkan_instance`: A reference to the `VulkanInstance`, used to get the
    ///   `ash::Instance` handle and the Vulkan API version.
    /// * `physical_device_info`: A reference to `PhysicalDeviceInfo` containing the
    ///   `vk::PhysicalDevice` handle.
    /// * `logical_device`: A reference to the `LogicalDevice`, used to get the
    ///   `ash::Device` handle.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Allocator` on success, or a `VulkanError`
    /// (typically `VulkanError::VkMemError`) if VMA initialization fails.
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
    ) -> Result<Self> {
        let device_name = unsafe { std::ffi::CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
            .to_str().unwrap_or("Unknown Device");
        info!("Initializing Vulkan Memory Allocator (VMA) for device: {}", device_name);

        let create_info = vk_mem::AllocatorCreateInfo::new(
            vulkan_instance.raw(),
            &logical_device.raw,
            physical_device_info.physical_device,
        )
        .set_instance_api_version(vulkan_instance.api_version());
        // Note: For vk-mem 0.3.x and later, `set_instance_api_version` would be `set_vulkan_api_version`.
        // Flags like `vk_mem::AllocatorCreateFlags::KHR_BIND_MEMORY_2` could be set here if needed.

        let allocator = vk_mem::Allocator::new(&create_info)?; // Uses From<vk_mem::Error>
        info!("VMA Initialized successfully for device: {}.", device_name);
        Ok(Self { raw: allocator })
    }

    /// Creates a Vulkan buffer with memory allocated by VMA.
    ///
    /// This function simplifies buffer creation by handling both the `vkCreateBuffer`
    /// call and the memory allocation/binding process using VMA.
    ///
    /// # Arguments
    ///
    /// * `buffer_create_info`: A reference to `vk::BufferCreateInfo` describing the buffer's properties
    ///   (size, usage flags, etc.).
    /// * `allocation_create_info`: A reference to `vk_mem::AllocationCreateInfo` specifying
    ///   VMA allocation parameters, such as:
    ///     - `usage`: `vk_mem::MemoryUsage` (e.g., `GpuOnly`, `CpuToGpu`, `GpuToCpu`). This guides VMA
    ///       in selecting the optimal memory type, especially for UMA (Unified Memory Architecture).
    ///       For example:
    ///         - `GpuOnly`: For resources exclusively GPU-accessed.
    ///         - `CpuToGpu`: For resources frequently written by CPU, read by GPU (e.g., uniform buffers).
    ///         - `GpuToCpu`: For resources written by GPU, read by CPU (e.g., readback buffers).
    ///     - `flags`: `vk_mem::AllocationCreateFlags` (e.g., `MAPPED` for persistently mapped memory,
    ///       `HOST_ACCESS_SEQUENTIAL_WRITE` or `HOST_ACCESS_RANDOM` as hints for host access patterns).
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple `(vk::Buffer, vk_mem::Allocation, vk_mem::AllocationInfo)`
    /// on success, or a `VulkanError` (typically `VulkanError::VkMemError`) on failure.
    /// The `vk_mem::AllocationInfo` provides details about the allocation, including a mapped
    /// pointer if the `MAPPED` flag was used.
    ///
    /// # Safety
    ///
    /// This function involves an `unsafe` call to `self.raw.create_buffer`. The caller must ensure
    /// that the `buffer_create_info` and `allocation_create_info` are valid.
    pub fn create_buffer(
        &self,
        buffer_create_info: &vk::BufferCreateInfo,
        allocation_create_info: &vk_mem::AllocationCreateInfo,
    ) -> Result<(vk::Buffer, vk_mem::Allocation, vk_mem::AllocationInfo)> {
        debug!(
            "Creating buffer with VMA: size={}, usage={:?}, VMA usage={:?}, VMA flags={:?}",
            buffer_create_info.size, buffer_create_info.usage,
            allocation_create_info.usage, allocation_create_info.flags
        );
        let (buffer, allocation, allocation_info) =
            unsafe { self.raw.create_buffer(buffer_create_info, allocation_create_info) }?; // Uses From<vk_mem::Error>
        debug!("Buffer created successfully with VMA: {:?}, allocation: {:?}", buffer, allocation);
        Ok((buffer, allocation, allocation_info))
    }

    /// Destroys a Vulkan buffer and frees its associated VMA allocation.
    ///
    /// # Arguments
    ///
    /// * `buffer`: The `vk::Buffer` to destroy.
    /// * `allocation`: A reference to the `vk_mem::Allocation` handle associated with the buffer.
    ///
    /// # Safety
    ///
    /// This function involves an `unsafe` call to `self.raw.destroy_buffer`.
    /// The caller must ensure that the buffer and allocation are no longer in use by the GPU
    /// (e.g., by waiting for device idle or using appropriate synchronization).
    /// Both `buffer` and `allocation` become invalid after this call.
    pub fn destroy_buffer(&self, buffer: vk::Buffer, allocation: &vk_mem::Allocation) {
        debug!("Destroying buffer {:?} with VMA allocation {:?}", buffer, allocation);
        unsafe {
            // VMA's destroy_buffer internally calls vkDestroyBuffer and vkFreeMemory (or equivalent).
            // It's important that the allocator instance (`self.raw`) is the same one
            // that created this allocation.
            self.raw.destroy_buffer(buffer, allocation);
        }
        debug!("Buffer and VMA allocation destroyed successfully.");
    }

    /// Creates a Vulkan image with memory allocated by VMA.
    ///
    /// Similar to `create_buffer`, this function handles both `vkCreateImage` and
    /// the memory allocation/binding using VMA.
    ///
    /// # Arguments
    ///
    /// * `image_create_info`: A reference to `vk::ImageCreateInfo` describing the image's properties
    ///   (type, format, extent, usage, etc.).
    /// * `allocation_create_info`: A reference to `vk_mem::AllocationCreateInfo` specifying
    ///   VMA allocation parameters. See `create_buffer` documentation for details on
    ///   `vk_mem::MemoryUsage` and `vk_mem::AllocationCreateFlags`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple `(vk::Image, vk_mem::Allocation, vk_mem::AllocationInfo)`
    /// on success, or a `VulkanError` (typically `VulkanError::VkMemError`) on failure.
    ///
    /// # Safety
    ///
    /// This function involves an `unsafe` call to `self.raw.create_image`. The caller must ensure
    /// that the `image_create_info` and `allocation_create_info` are valid.
    pub fn create_image(
        &self,
        image_create_info: &vk::ImageCreateInfo,
        allocation_create_info: &vk_mem::AllocationCreateInfo,
    ) -> Result<(vk::Image, vk_mem::Allocation, vk_mem::AllocationInfo)> {
        debug!(
            "Creating image with VMA: format={:?}, extent=({:?},{:?},{:?}), usage={:?}, VMA usage={:?}, VMA flags={:?}",
            image_create_info.format, image_create_info.extent.width, image_create_info.extent.height,
            image_create_info.extent.depth, image_create_info.usage,
            allocation_create_info.usage, allocation_create_info.flags
        );
        let (image, allocation, allocation_info) =
            unsafe { self.raw.create_image(image_create_info, allocation_create_info) }?; // Uses From<vk_mem::Error>
        debug!("Image created successfully with VMA: {:?}, allocation: {:?}", image, allocation);
        Ok((image, allocation, allocation_info))
    }

    /// Destroys a Vulkan image and frees its associated VMA allocation.
    ///
    /// # Arguments
    ///
    /// * `image`: The `vk::Image` to destroy.
    /// * `allocation`: A reference to the `vk_mem::Allocation` handle associated with the image.
    ///
    /// # Safety
    ///
    /// This function involves an `unsafe` call to `self.raw.destroy_image`.
    /// The caller must ensure that the image and allocation are no longer in use by the GPU.
    /// Both `image` and `allocation` become invalid after this call.
    pub fn destroy_image(&self, image: vk::Image, allocation: &vk_mem::Allocation) {
        debug!("Destroying image {:?} with VMA allocation {:?}", image, allocation);
        unsafe {
            self.raw.destroy_image(image, allocation);
        }
        debug!("Image and VMA allocation destroyed successfully.");
    }

    /// Destroys the VMA allocator instance.
    ///
    /// This method is called automatically when the `Allocator` is dropped.
    /// It can also be called manually if explicit cleanup control is needed before `Drop`.
    ///
    /// # Safety
    ///
    /// This function performs an `unsafe` call to `self.raw.destroy()`.
    /// The caller must ensure that all resources allocated by this VMA instance
    /// (buffers, images) have been destroyed *before* calling this method or
    /// allowing the `Allocator` to be dropped. The `vk_mem::Allocator` handle
    /// becomes invalid after this call.
    pub fn destroy(&mut self) {
        info!("Destroying VMA Allocator...");
        unsafe {
            // This internally calls vmaDestroyAllocator.
            // All allocations made from this allocator should be freed before this call.
            self.raw.destroy();
        }
        info!("VMA Allocator destroyed.");
    }

    /// Provides raw access to the underlying `vk_mem::Allocator` handle.
    ///
    /// This can be useful for advanced VMA operations not directly exposed by this wrapper,
    /// or for passing the VMA allocator handle to other libraries or components that
    /// integrate with VMA.
    ///
    /// **Note:** The returned `vk_mem::Allocator` is a cloned handle (cheap to clone).
    /// It's important that the main `Allocator` struct (which owns the actual VMA instance
    /// via its `raw` field) outlives any use of this cloned handle if operations like
    /// allocation or deallocation are performed with the clone. It's generally safer
    /// to perform such operations through methods of this `Allocator` wrapper.
    /// For `Drop` implementations of other structs that need to free VMA resources,
    /// cloning the allocator handle and storing it can be a valid pattern if the main
    /// `Allocator`'s lifetime is guaranteed to exceed theirs.
    pub fn raw_allocator(&self) -> &vk_mem::Allocator { // Changed to return & to avoid confusion about cloning ownership
        &self.raw
    }
}

impl Drop for Allocator {
    /// Ensures that the VMA allocator instance is destroyed when the `Allocator`
    /// goes out of scope.
    ///
    /// This calls the `destroy()` method.
    fn drop(&mut self) {
        info!("Dropping Allocator, performing cleanup via destroy().");
        self.destroy();
    }
}
