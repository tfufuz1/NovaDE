use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance;
use crate::physical_device::PhysicalDeviceInfo;
use crate::device::LogicalDevice; // Assuming Arc<Device> is accessible from LogicalDevice
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use vk_mem_rs::{Allocator as VmaAllocator, AllocatorCreateInfo, Allocation, AllocationCreateInfo, MemoryUsage, AllocationCreateFlags};

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
        // self.vma_allocator.destroy(); // Not needed if VmaAllocator implements Drop
        log::debug!("VMA Allocator dropped, resources should be freed.");
    }
}
