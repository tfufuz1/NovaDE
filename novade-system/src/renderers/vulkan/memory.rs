// novade-system/src/renderers/vulkan/memory.rs
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc, Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation; // MemoryLocation::GpuOnly, MemoryLocation::CpuToGpu, etc.

/// Creates a Vulkan memory allocator using the `gpu-allocator` crate.
///
/// This function initializes an `Allocator` which is responsible for managing Vulkan device memory.
/// It corresponds to the VMA initialization described in `Rendering Vulkan.md` (Spec 4.1),
/// adapted for the `gpu-allocator` API.
///
/// # Arguments
/// * `instance`: A reference to the `ash::Instance`.
/// * `physical_device`: The `vk::PhysicalDevice` that the allocator will manage memory for.
/// * `device`: A reference to the logical `ash::Device`.
///
/// # Returns
/// A `Result` containing the initialized `gpu_allocator::vulkan::Allocator`,
/// or an error string if initialization fails.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 4.1 - VMA-Initialisierung):
/// - `VmaAllocatorCreateInfo` fields are mapped to `gpu_allocator::vulkan::AllocatorCreateDesc`:
///   - `physicalDevice`, `device`, `instance`: Directly provided.
///   - `flags` (e.g., `VMA_ALLOCATOR_CREATE_KHR_BIND_MEMORY2_BIT`): Partly covered by `buffer_device_address`
///     in `AllocatorCreateDesc`. Other advanced binding features are managed internally by `gpu-allocator`.
///   - `preferredLargeHeapBlockSize`: Corresponds to `allocation_sizes` in `AllocatorCreateDesc`.
///     Currently uses `Default::default()`. Spec recommendation for VMA (256MB or heap/8) is VMA-specific.
///   - `pAllocationCallbacks`: Not directly exposed by `gpu-allocator`'s high-level `new` function.
///   - `pVulkanFunctions`: Handled internally by `gpu-allocator` when built with the `ash` feature.
/// - The function aims to create a ready-to-use `Allocator` handle.
pub fn create_allocator(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
) -> Result<Allocator, String> {
    Allocator::new(&AllocatorCreateDesc {
        instance: instance.clone(), // Allocator may need to clone these
        device: device.clone(),
        physical_device,
        debug_settings: Default::default(), // Configure as needed (e.g., for logging, validation)
        buffer_device_address: false, // Enable only if the bufferDeviceAddress Vulkan feature is used.
        allocation_sizes: Default::default(), // Use default block sizes. Can be customized for performance tuning.
    })
    .map_err(|e| format!("Failed to create Vulkan memory allocator: {:?}", e))
}

/// Creates a Vulkan buffer and allocates memory for it using the provided allocator.
///
/// This function handles the creation of a `vk::Buffer`, querying its memory requirements,
/// allocating the necessary `vk::DeviceMemory` using `gpu-allocator`, and binding
/// the memory to the buffer.
///
/// It aligns with `Rendering Vulkan.md` (Spec 4.4 for Buffer-Erstellung and Spec 4.2/4.5
/// for UMA-optimized memory selection via the `location` parameter).
///
/// # Arguments
/// * `allocator`: A mutable reference to the `gpu_allocator::vulkan::Allocator`.
/// * `device`: A reference to the logical `ash::Device` (used for `vkCreateBuffer`).
/// * `size`: The desired size of the buffer in bytes (`vk::DeviceSize`).
/// * `usage`: `vk::BufferUsageFlags` specifying how the buffer will be used (e.g., vertex, index, uniform).
/// * `location`: `gpu_allocator::MemoryLocation` indicating the desired memory properties
///   (e.g., `MemoryLocation::GpuOnly`, `MemoryLocation::CpuToGpu`). For UMA systems (like AMD Vega 8),
///   `MemoryLocation::CpuToGpu` is often preferred for staging or frequently updated buffers as it typically
///   maps to `HOST_VISIBLE | DEVICE_LOCAL` memory. `MemoryLocation::GpuOnly` is suitable for static
///   data after initial upload.
///
/// # Returns
/// A `Result` containing a tuple of the created `vk::Buffer` and the `gpu_allocator::vulkan::Allocation`,
/// or an error string on failure.
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
        .sharing_mode(vk::SharingMode::EXCLUSIVE); // Assuming exclusive for simplicity, adjust if sharing needed

    let buffer = unsafe {
        device
            .create_buffer(&buffer_info, None)
            .map_err(|e| format!("Failed to create buffer: {}", e))?
    };

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    // AllocationCreateDesc allows specifying the name for debugging purposes.
    let allocation_desc = AllocationCreateDesc {
        name: "buffer_allocation", // Descriptive name
        requirements,
        location,
        scheme: AllocationScheme::GpuAllocatorManaged, // Let gpu-allocator manage the underlying VkDeviceMemory
        linear: true, // Buffers are linear resources
    };

    let allocation = allocator
        .allocate(&allocation_desc)
        .map_err(|e| format!("Failed to allocate memory for buffer (size: {}, usage: {:?}, location: {:?}): {:?}", size, usage, location, e))?;

    // Bind the allocated memory to the buffer.
    // The `allocation.offset()` is important if the allocator sub-allocates from larger blocks.
    unsafe {
        device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
            .map_err(|e| format!("Failed to bind buffer memory: {}", e))?;
    }
    Ok((buffer, allocation))
}

/// Creates a Vulkan image and allocates memory for it using the provided allocator.
///
/// This function handles the creation of a `vk::Image` using the provided `vk::ImageCreateInfo`,
/// querying its memory requirements, allocating the necessary `vk::DeviceMemory` via `gpu-allocator`,
/// and binding the memory to the image.
///
/// It aligns with `Rendering Vulkan.md` (Spec 9.1 for Image-Erstellung and Spec 4.4 for
/// general resource creation principles). The creation of `vk::ImageView` is handled separately.
///
/// # Arguments
/// * `allocator`: A mutable reference to the `gpu_allocator::vulkan::Allocator`.
/// * `device`: A reference to the logical `ash::Device` (used for `vkCreateImage`).
/// * `create_info`: A reference to `vk::ImageCreateInfo` specifying the properties of the image
///   to be created (e.g., type, format, extent, usage, tiling).
/// * `location`: `gpu_allocator::MemoryLocation` indicating the desired memory properties
///   (e.g., `MemoryLocation::GpuOnly` for textures and render targets, `MemoryLocation::CpuToGpu`
///   if CPU access is needed, though less common for optimal images).
///
/// # Returns
/// A `Result` containing a tuple of the created `vk::Image` and the `gpu_allocator::vulkan::Allocation`,
/// or an error string on failure.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 9.1):
/// - `VkImageCreateInfo` fields (`imageType`, `format`, `extent`, `mipLevels`, `arrayLayers`, `samples`,
///   `tiling`, `usage`, `initialLayout`) are expected to be correctly set in the `create_info` argument
///   by the caller, as per Spec 9.1.
/// - Memory allocation uses `VMA_MEMORY_USAGE_GPU_ONLY` (mapped to `MemoryLocation::GpuOnly`) for typical
///   images like textures or attachments.
pub fn create_image(
    allocator: &mut Allocator,
    device: &ash::Device, // Needed for vkCreateImage
    create_info: &vk::ImageCreateInfo, // Pass the full ImageCreateInfo
    location: MemoryLocation, // e.g., MemoryLocation::GpuOnly
) -> Result<(vk::Image, Allocation), String> {
    let image = unsafe {
        device
            .create_image(create_info, None)
            .map_err(|e| format!("Failed to create image with format {:?}, extent {:?}: {}", create_info.format, create_info.extent, e))?
    };

    let requirements = unsafe { device.get_image_memory_requirements(image) };

    let allocation_desc = AllocationCreateDesc {
        name: "image_allocation", // Descriptive name
        requirements,
        location,
        scheme: AllocationScheme::GpuAllocatorManaged,
        linear: create_info.tiling == vk::ImageTiling::LINEAR, // Images are linear if tiling is LINEAR, otherwise optimal/tiled (non-linear)
    };

    let allocation = allocator
        .allocate(&allocation_desc)
        .map_err(|e| format!("Failed to allocate memory for image (format {:?}, extent {:?}, location: {:?}): {:?}", create_info.format, create_info.extent, location, e))?;

    // Bind the allocated memory to the image.
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
