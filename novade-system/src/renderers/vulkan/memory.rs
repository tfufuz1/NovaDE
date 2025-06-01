// novade-system/src/renderers/vulkan/memory.rs
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc, Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation; // MemoryLocation::GpuOnly, MemoryLocation::CpuToGpu, etc.

pub fn create_allocator(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
) -> Result<Allocator, String> {
    Allocator::new(&AllocatorCreateDesc {
        instance: instance.clone(), // Allocator may need to clone these
        device: device.clone(),
        physical_device,
        debug_settings: Default::default(), // Configure as needed
        buffer_device_address: false, // Enable if bufferDeviceAddress feature is used
        allocation_sizes: Default::default(), // Use default block sizes or customize
    })
    .map_err(|e| format!("Failed to create Vulkan memory allocator: {:?}", e))
}

pub fn create_buffer(
    allocator: &mut Allocator,
    device: &ash::Device, // Needed for vkCreateBuffer
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation, // e.g., MemoryLocation::GpuOnly, MemoryLocation::CpuToGpu
) -> Result<(vk::Buffer, Allocation), String> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE); // Assuming exclusive for simplicity

    let buffer = unsafe {
        device
            .create_buffer(&buffer_info, None)
            .map_err(|e| format!("Failed to create buffer: {}", e))?
    };

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator
        .allocate(&AllocationCreateDesc {
            name: "buffer", // For debugging
            requirements,
            location,
            scheme: AllocationScheme::GpuAllocatorManaged, // Let gpu-allocator manage it
            linear: true, // Buffers are typically linear
        })
        .map_err(|e| format!("Failed to allocate memory for buffer: {:?}", e))?;

    unsafe {
        device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
            .map_err(|e| format!("Failed to bind buffer memory: {}", e))?;
    }
    Ok((buffer, allocation))
}

pub fn create_image(
    allocator: &mut Allocator,
    device: &ash::Device, // Needed for vkCreateImage
    create_info: &vk::ImageCreateInfo, // Pass the full ImageCreateInfo
    location: MemoryLocation, // e.g., MemoryLocation::GpuOnly
) -> Result<(vk::Image, Allocation), String> {
    let image = unsafe {
        device
            .create_image(create_info, None)
            .map_err(|e| format!("Failed to create image: {}", e))?
    };

    let requirements = unsafe { device.get_image_memory_requirements(image) };

    let allocation = allocator
        .allocate(&AllocationCreateDesc {
            name: "image",
            requirements,
            location,
            scheme: AllocationScheme::GpuAllocatorManaged,
            linear: false, // Images are typically tiled/optimal
        })
        .map_err(|e| format!("Failed to allocate memory for image: {:?}", e))?;

    unsafe {
        device
            .bind_image_memory(image, allocation.memory(), allocation.offset())
            .map_err(|e| format!("Failed to bind image memory: {}", e))?;
    }
    Ok((image, allocation))
}

// It's important to free allocations and destroy buffers/images when they are no longer needed.
// This will typically be handled by custom structs that own the vk::Buffer/vk::Image and its Allocation,
// and implement Drop.
