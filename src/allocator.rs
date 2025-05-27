use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance;
use crate::physical_device::PhysicalDeviceInfo;
use crate::device::LogicalDevice;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use vk_mem_rs::{Allocator as VmaAllocator, AllocatorCreateInfo, Allocation, AllocationCreateInfo, MemoryUsage, AllocationCreateFlags};
use std::os::unix::io::RawFd;
use vulkanalia::vk::{ExtExternalMemoryDmaBufExtension, KhrExternalMemoryFdExtension, ExtImageDrmFormatModifierExtension, KhrExternalMemoryExtension, KhrGetPhysicalDeviceProperties2Extension}; // For dedicated allocation check

pub struct Allocator {
    vma_allocator: VmaAllocator,
    device: Arc<Device>, // Keep a reference to the device for destruction if VMA needs it implicitly
}

impl Allocator {
    pub fn new(
        instance_wrapper: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device_wrapper: &LogicalDevice,
    ) -> Result<Self> {
        let instance_ref: &Arc<Instance> = instance_wrapper.raw();
        let physical_device = physical_device_info.raw();
        let device_ref: &Arc<Device> = logical_device_wrapper.raw();
        let device_clone_for_struct = device_ref.clone(); // Clone Arc for struct ownership

        // For vk-mem-rs 0.2.3, we need to populate VulkanFunctions.
        let vulkan_functions = vk_mem_rs::VulkanFunctions::new(
            instance_wrapper.entry().static_fn(), // Entry static functions
            instance_ref.static_fn(),             // Instance static functions
            device_ref.static_fn(),               // Device static functions
        );

        // As per Rendering Vulkan.md (Section 4.1)
        // preferredLargeHeapBlockSize: Empfohlen 256 MiB, oder heap_size / 8 für Heaps <= 1 GiB.
        // VMA default is 256MB, so we can omit it or set it explicitly.
        let create_info = AllocatorCreateInfo::new(
            instance_ref,    // Pass &Arc<Instance>
            device_ref,      // Pass &Arc<Device>
            physical_device,
        )
        .set_vulkan_functions(vulkan_functions); // For vk-mem-rs 0.2.3
        // .vulkan_api_version(vk::API_VERSION_1_3) // If needed by VMA version


        let vma_allocator = VmaAllocator::new(create_info)
            .map_err(VulkanError::VmaResult)?;
        
        log::info!("Vulkan Memory Allocator (VMA) initialized.");

        Ok(Self { vma_allocator, device: device_clone_for_struct })
    }

    pub fn create_buffer(
        &self,
        buffer_info: &vk::BufferCreateInfo,
        usage: MemoryUsage,
        flags: Option<AllocationCreateFlags>, // e.g., MAPPED, HOST_ACCESS_SEQUENTIAL_WRITE
    ) -> Result<(vk::Buffer, Allocation)> {
        // Example: Prefer DEVICE_LOCAL | HOST_VISIBLE for UMA for dynamic data
        // `usage` flag in AllocationCreateInfo helps VMA pick the right memory type.
        // For UMA, CpuToGpu often results in DEVICE_LOCAL | HOST_VISIBLE.
        // GpuOnly for static vertex/index buffers.
        
        let allocation_create_info = AllocationCreateInfo {
            usage,
            flags: flags.unwrap_or_else(AllocationCreateFlags::empty), // Default to no special flags
            ..Default::default() // required_flags, preferred_flags, memory_type_bits, pool, user_data
        };

        self.vma_allocator
            .create_buffer(buffer_info, &allocation_create_info)
            .map_err(VulkanError::VmaResult)
    }

    pub fn create_image(
        &self,
        image_info: &vk::ImageCreateInfo,
        usage: MemoryUsage, // Typically GpuOnly for textures, render targets
        flags: Option<AllocationCreateFlags>,
    ) -> Result<(vk::Image, Allocation)> {
        let allocation_create_info = AllocationCreateInfo {
            usage,
            flags: flags.unwrap_or_else(AllocationCreateFlags::empty),
            ..Default::default()
        };
        
        self.vma_allocator
            .create_image(image_info, &allocation_create_info)
            .map_err(VulkanError::VmaResult)
    }
    
    // It's important to destroy buffers/images created with VMA using VMA functions
    pub fn destroy_buffer(&self, buffer: vk::Buffer, allocation: Allocation) -> Result<()> {
        self.vma_allocator.destroy_buffer(buffer, allocation);
        Ok(())
    }

    pub fn destroy_image(&self, image: vk::Image, allocation: Allocation) -> Result<()> {
        self.vma_allocator.destroy_image(image, allocation);
        Ok(())
    }

    // Map memory
    pub fn map_memory(&self, allocation: &mut Allocation) -> Result<*mut u8> {
        self.vma_allocator.map_memory(allocation).map_err(VulkanError::VmaResult)
    }

    // Unmap memory
    pub fn unmap_memory(&self, allocation: &mut Allocation) -> Result<()> {
        self.vma_allocator.unmap_memory(allocation); // This function is void in vk-mem-rs
        Ok(())
    }
    
    // Flush allocation (if not HOST_COHERENT and HOST_VISIBLE)
    pub fn flush_allocation(&self, allocation: &Allocation, offset: vk::DeviceSize, size: vk::DeviceSize) -> Result<()> {
        self.vma_allocator.flush_allocation(allocation, offset, size).map_err(VulkanError::VmaResult)
    }
    
    // Invalidate allocation (if not HOST_COHERENT and HOST_VISIBLE)
    pub fn invalidate_allocation(&self, allocation: &Allocation, offset: vk::DeviceSize, size: vk::DeviceSize) -> Result<()> {
        self.vma_allocator.invalidate_allocation(allocation, offset, size).map_err(VulkanError::VmaResult)
    }

    // Getter for the raw VMA allocator if needed elsewhere (e.g. for stats or custom pools)
    pub fn raw_vma(&self) -> &VmaAllocator {
        &self.vma_allocator
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        // VMA allocator is destroyed when it goes out of scope.
        // Its Drop trait handles cleanup.
        log::debug!("VMA Allocator dropped, resources should be freed.");
    }

    // Helper to find memory type index
    fn find_memory_type_index(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<u32> {
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        for i in 0..memory_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0 && memory_properties.memory_types[i as usize].property_flags.contains(properties) {
                return Ok(i);
            }
        }
        Err(VulkanError::Message("Failed to find suitable memory type.".to_string()))
    }

    #[cfg(target_os = "linux")]
    pub fn import_dma_buf_as_image(
        &self,
        fd: RawFd,
        width: u32,
        height: u32,
        vulkan_format: vk::Format,
        drm_format_modifier: Option<u64>,
        image_usage: vk::ImageUsageFlags,
        instance_vulkanalia: &Instance, // Renamed to avoid conflict with struct field
        physical_device: vk::PhysicalDevice,
        device_vulkanalia: &Device,    // Renamed to avoid conflict
    ) -> Result<(vk::Image, vk::DeviceMemory)> {

        let mut external_memory_image_info = vk::ExternalMemoryImageCreateInfo::builder()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

        let mut image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .format(vulkan_format)
            .extent(vk::Extent3D::builder().width(width).height(height).depth(1).build())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::_1)
            .tiling(if drm_format_modifier.is_some() { vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT } else { vk::ImageTiling::OPTIMAL })
            .usage(image_usage)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .push_next(&mut external_memory_image_info);

        let mut modifier_info; 
        if let Some(modifier) = drm_format_modifier {
            if !device_vulkanalia.enabled_extensions().contains(&ExtImageDrmFormatModifierExtension::name()) {
               return Err(VulkanError::Message("DRM format modifier provided, but VK_EXT_image_drm_format_modifier extension is not enabled on device.".to_string()));
            }
            log::debug!("Using DRM format modifier: {}", modifier);
            modifier_info = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::builder()
                .drm_format_modifier(modifier)
                .drm_format_modifier_plane_count(1);
            image_info = image_info.push_next(&mut modifier_info);
        }

        let image = unsafe { device_vulkanalia.create_image(&image_info, None) }
            .map_err(VulkanError::VkResult)?;

        let mut dedicated_allocation_required = false;
        let mut memory_requirements2 = vk::MemoryRequirements2::builder();
        let mut dedicated_reqs = vk::MemoryDedicatedRequirements::builder();
        
        // Assuming KHR_get_physical_device_properties2 is enabled on instance if KHR_external_memory is enabled on device.
        // KHR_dedicated_allocation is core in 1.1. Device extensions imply certain instance extensions.
        // This check is simplified; a robust app would track enabled instance/device extensions.
        if device_vulkanalia.enabled_extensions().contains(&KhrExternalMemoryExtension::name()) {
            memory_requirements2 = memory_requirements2.push_next(&mut dedicated_reqs);
            let image_mem_req_info = vk::ImageMemoryRequirementsInfo2::builder().image(image);
            unsafe { device_vulkanalia.get_image_memory_requirements2(&image_mem_req_info, &mut memory_requirements2); }
            if dedicated_reqs.prefers_dedicated_allocation != vk::FALSE || dedicated_reqs.requires_dedicated_allocation != vk::FALSE {
                dedicated_allocation_required = true;
                log::debug!("DMA-BUF image {} dedicated allocation.", if dedicated_reqs.requires_dedicated_allocation != vk::FALSE {"requires"} else {"prefers"});
            }
        }
        let memory_requirements = if dedicated_allocation_required {
             memory_requirements2.memory_requirements // Use this if dedicated check was done
        } else {
            unsafe { device_vulkanalia.get_image_memory_requirements(image) }
        };


        let mut import_memory_info = vk::ImportMemoryFdInfoKHR::builder()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
            .fd(fd);

        let mut memory_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(self.find_memory_type_index(
                memory_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                instance_vulkanalia, physical_device
            )?)
            .push_next(&mut import_memory_info);

        let mut dedicated_alloc_info;
        if dedicated_allocation_required {
            dedicated_alloc_info = vk::MemoryDedicatedAllocateInfo::builder().image(image);
            memory_allocate_info = memory_allocate_info.push_next(&mut dedicated_alloc_info);
        }

        let memory = unsafe { device_vulkanalia.allocate_memory(&memory_allocate_info, None) }
            .map_err(VulkanError::VkResult)?;

        unsafe { device_vulkanalia.bind_image_memory(image, memory, 0) }
            .map_err(VulkanError::VkResult)?;
        
        log::info!("Imported DMA-BUF (fd: {}) as VkImage with format {:?}, modifier {:?}", fd, vulkan_format, drm_format_modifier);

        Ok((image, memory))
    }
}
