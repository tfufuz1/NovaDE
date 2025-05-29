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

    // Helper to find memory type index, adapted from vulkanalia version
    fn find_memory_type_index(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        instance_ash: &ash::Instance,
        physical_device_vk: vk::PhysicalDevice,
    ) -> Result<u32> {
        let memory_properties = unsafe { instance_ash.get_physical_device_memory_properties(physical_device_vk) };
        for i in 0..memory_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0 && memory_properties.memory_types[i as usize].property_flags.contains(properties) {
                return Ok(i);
            }
        }
        Err(VulkanError::Message("Failed to find suitable memory type.".to_string()))
    }
}


// The actual implementation of import_dma_buf_as_image
#[cfg(target_os = "linux")]
impl Allocator {
    #[allow(clippy::too_many_arguments)]
    pub fn import_dma_buf_as_image( 
        &self,
        fd: std::os::unix::io::RawFd,
        _width: u32,
        _height: u32,
        _vulkan_format: vk::Format,
        _drm_format_modifier: Option<u64>,
        _usage: vk::ImageUsageFlags,
        _instance: &ash::Instance, // Raw ash::Instance
        _physical_device: vk::PhysicalDevice, // Raw vk::PhysicalDevice
        _device: &ash::Device, // Raw ash::Device for actual import calls
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        // THIS IS A PLACEHOLDER IMPLEMENTATION
        // A real implementation would involve:
        // 1. Querying memory FD properties using vkGetMemoryFdPropertiesKHR.
        // 2. Creating a vk::Image with appropriate tiling and usage.
        // 3. Getting image memory requirements (vkGetImageMemoryRequirements2).
        // 4. Allocating vk::DeviceMemory with vk::ImportMemoryFdInfoKHR in pNext chain.
        // 5. Binding the image to the imported memory (vkBindImageMemory2).
        // 6. Handling format modifiers if present and supported.
        error!(
            "Placeholder: Allocator::import_dma_buf_as_image is not fully implemented. \
            Returning VulkanError::FeatureNotSupported. DMABUF import will fail."
        );
        // To make it compile, we could return dummy handles, but an error is more honest.
        Err(VulkanError::FeatureNotSupported(
            "DMABUF import via Allocator::import_dma_buf_as_image".to_string()
        ))
        width: u32,
        height: u32,
        vulkan_format: vk::Format,
        drm_format_modifier: Option<u64>,
        image_usage: vk::ImageUsageFlags,
        instance_ash: &ash::Instance, 
        physical_device_vk: vk::PhysicalDevice,
        device_ash: &ash::Device,
        // We need enabled extension checks here, or assume they are checked by caller/globally
        // For example, by checking fields in PhysicalDeviceInfo or LogicalDevice that store this.
        // For now, proceeding with direct calls assuming extensions are loaded if functions are callable.
    ) -> Result<(vk::Image, vk::DeviceMemory)> {

        // Check required instance extensions (usually done at instance creation)
        // e.g. VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME
        // Check required device extensions (usually done at device creation)
        // e.g. VK_KHR_EXTERNAL_MEMORY_EXTENSION_NAME, VK_KHR_EXTERNAL_MEMORY_FD_EXTENSION_NAME, 
        //      VK_EXT_EXTERNAL_MEMORY_DMA_BUF_EXTENSION_NAME
        //      VK_EXT_IMAGE_DRM_FORMAT_MODIFIER_EXTENSION_NAME (if modifier is Some)

        let mut external_memory_image_info = vk::ExternalMemoryImageCreateInfo::builder()
            .handle_types(vk::ExternalMemoryHandleTypeFlagsKHR::DMA_BUF_EXT); // Using KHR version for ash typically

        let mut image_info_builder = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vulkan_format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            // Tiling depends on whether a modifier is used.
            // If modifier is Some, TILING_DRM_FORMAT_MODIFIER_EXT is required.
            // Otherwise, OPTIMAL is common.
            .tiling(if drm_format_modifier.is_some() { vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT } else { vk::ImageTiling::OPTIMAL })
            .usage(image_usage) // e.g., vk::ImageUsageFlags::SAMPLED
            .initial_layout(vk::ImageLayout::UNDEFINED) // Or PREINITIALIZED if data is already there (not for import)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
            // .push_next(&mut external_memory_image_info); // This should be added to image_info

        let mut image_drm_format_modifier_info;
        if let Some(modifier) = drm_format_modifier {
            // TODO: Check if VK_EXT_image_drm_format_modifier is enabled on device_ash
            // This would typically be stored in LogicalDevice or PhysicalDeviceInfo
            // if !logical_device.enabled_extensions.contains(&ash::extensions::ext::ImageDrmFormatModifier::name()) {
            //    return Err(VulkanError::Message("DRM format modifier provided, but extension not enabled.".to_string()));
            // }
            log::debug!("Using DRM format modifier: {}", modifier);
            image_drm_format_modifier_info = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::builder()
                .drm_format_modifier(modifier)
                .drm_format_modifier_plane_count(1); // Assuming single plane for simplicity here
            
            // Chain external_memory_image_info and image_drm_format_modifier_info
            external_memory_image_info = external_memory_image_info.push_next(&mut image_drm_format_modifier_info);
        }
        image_info_builder = image_info_builder.push_next(&mut external_memory_image_info);
        
        let image_info = image_info_builder.build();
        let image = unsafe { device_ash.create_image(&image_info, None) }.map_err(|e| {
            tracing::error!(
                "Vulkan API error: Failed to create image for DMABUF import (fd: {}, format: {:?}, modifier: {:?}, dims: {}x{}). Error: {:?}",
                fd, vulkan_format, drm_format_modifier, width, height, e
            );
            VulkanError::VkResult(e)
        })?;

        // Get memory requirements
        // Adapting the dedicated allocation check from vulkanalia version
        let mut dedicated_allocation_required = false;
        let memory_requirements: vk::MemoryRequirements;

        // TODO: Extension check for VK_KHR_get_memory_requirements2 and VK_KHR_dedicated_allocation
        // (core in Vulkan 1.1, but good practice if targeting 1.0 + extensions)
        // For ash, these might be part of KhrGetPhysicalDeviceProperties2 or similar.
        // Simplified check:
        let mut dedicated_reqs = vk::MemoryDedicatedRequirements::builder();
        let mut memory_requirements2_builder = vk::MemoryRequirements2::builder().push_next(&mut dedicated_reqs);
        
        unsafe { device_ash.get_image_memory_requirements2(
            &vk::ImageMemoryRequirementsInfo2::builder().image(image).build(),
            &mut memory_requirements2_builder
        )};
        
        let memory_requirements2_result = memory_requirements2_builder.build(); // Finalize the builder to get the struct
        if dedicated_reqs.prefers_dedicated_allocation == vk::TRUE || dedicated_reqs.requires_dedicated_allocation == vk::TRUE {
            dedicated_allocation_required = true;
            log::debug!("DMA-BUF image {} dedicated allocation.", if dedicated_reqs.requires_dedicated_allocation != vk::FALSE {"requires"} else {"prefers"});
        }
        memory_requirements = memory_requirements2_result.memory_requirements;


        // Allocate memory
        let mut import_memory_info = vk::ImportMemoryFdInfoKHR::builder()
            .handle_type(vk::ExternalMemoryHandleTypeFlagsKHR::DMA_BUF_EXT)
            .fd(fd);

        let mut memory_allocate_info_builder = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(self.find_memory_type_index(
                memory_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL, // DMA-BUFs are typically device local
                instance_ash,
                physical_device_vk,
            ).map_err(|e| {
                tracing::error!(
                    "Failed to find suitable memory type for DMABUF image (fd: {}, format: {:?}, modifier: {:?}, image_req_bits: {:#x}). Error: {:?}",
                    fd, vulkan_format, drm_format_modifier, memory_requirements.memory_type_bits, e
                );
                // It's important to clean up the created image if memory allocation fails
                unsafe { device_ash.destroy_image(image, None); }
                e // Return original error from find_memory_type_index
            })?);
        
        memory_allocate_info_builder = memory_allocate_info_builder.push_next(&mut import_memory_info);

        let mut dedicated_alloc_info;
        if dedicated_allocation_required {
            dedicated_alloc_info = vk::MemoryDedicatedAllocateInfo::builder().image(image);
            memory_allocate_info_builder = memory_allocate_info_builder.push_next(&mut dedicated_alloc_info);
        }
        
        let memory_allocate_info = memory_allocate_info_builder.build();
        let memory = unsafe { device_ash.allocate_memory(&memory_allocate_info, None) }.map_err(|e| {
            tracing::error!(
                "Vulkan API error: Failed to allocate memory for DMABUF import (fd: {}, format: {:?}, modifier: {:?}, size: {}). Error: {:?}",
                fd, vulkan_format, drm_format_modifier, memory_requirements.size, e
            );
            unsafe { device_ash.destroy_image(image, None); } // Clean up image
            VulkanError::VkResult(e)
        })?;

        // Bind image memory
        if let Err(e) = unsafe { device_ash.bind_image_memory(image, memory, 0) } {
            tracing::error!(
                "Vulkan API error: Failed to bind memory for DMABUF image (fd: {}, format: {:?}, modifier: {:?}). Error: {:?}",
                fd, vulkan_format, drm_format_modifier, e
            );
            unsafe {
                device_ash.free_memory(memory, None); // Clean up allocated memory
                device_ash.destroy_image(image, None); // Clean up image
            }
            return Err(VulkanError::VkResult(e));
        }
        
        log::info!("Imported DMA-BUF (fd: {}) as VkImage ({:?}) with format {:?}, modifier {:?}", fd, image, vulkan_format, drm_format_modifier);

        Ok((image, memory))
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
