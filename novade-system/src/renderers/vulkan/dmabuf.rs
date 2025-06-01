// novade-system/src/renderers/vulkan/dmabuf.rs
use ash::vk;
use gpu_allocator::vulkan::{Allocator, Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;
use std::os::unix::io::RawFd;

// Structure to hold DMA-BUF import parameters
pub struct DmaBufImportOptions {
    pub fd: RawFd,
    pub width: u32,
    pub height: u32,
    pub format: vk::Format, // Vulkan format corresponding to the DMA-BUF
    pub drm_format_modifier: Option<u64>, // Optional DRM format modifier
    pub allocation_size: vk::DeviceSize, // Size of the memory to import
    // Memory type index needs to be determined based on the DMA-BUF's properties
    // This is a complex part, as Vulkan needs to know which memory type is compatible.
    // For this subtask, we might have to simplify or assume a compatible memory type is found.
    pub memory_type_index: u32,
}

pub fn get_memory_type_index_for_dmabuf_placeholder(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    // For a more accurate version, you'd pass vk::ImageCreateInfo or vk::BufferCreateInfo
    // to then use vkGetPhysicalDeviceImageFormatProperties2 with VkPhysicalDeviceExternalImageFormatInfo
    // or vkGetMemoryFdPropertiesKHR for an imported FD.
) -> Result<u32, String> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    // Ideal scenario: Query based on FD properties or image format properties.
    // Simplified placeholder: Find a device-local, host-visible memory type.
    // This is a common configuration for integrated GPUs where DMA-BUFs might originate
    // or be efficiently used. WARNING: This is not universally correct for all DMA-BUF types.
    for i in 0..mem_properties.memory_type_count {
        let mt = &mem_properties.memory_types[i as usize];
        if mt.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) &&
           mt.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            // This is still a guess. A real DMA-BUF might require a specific memory type
            // that isn't necessarily HOST_VISIBLE from Vulkan's perspective,
            // or might be DEVICE_LOCAL only if it's from another GPU component.
            // The memoryTypeBits from vkGetMemoryFdPropertiesKHR is the proper source.
            println!("Warning: Using placeholder logic (DEVICE_LOCAL | HOST_VISIBLE) for DMA-BUF memory type selection. Index: {}", i);
            return Ok(i);
        }
    }
    // Fallback to first DEVICE_LOCAL if no HOST_VISIBLE one is found
    for i in 0..mem_properties.memory_type_count {
         let mt = &mem_properties.memory_types[i as usize];
        if mt.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            println!("Warning: Using fallback DEVICE_LOCAL placeholder logic for DMA-BUF memory type selection. Index: {}", i);
            return Ok(i);
        }
    }
    Err("No suitable placeholder memory type found for DMA-BUF import.".to_string())
}


pub fn import_dmabuf_as_image(
    _allocator: &mut Allocator, // Allocator is not directly used for vkImportMemoryFdKHR
    device: &ash::Device,
    options: &DmaBufImportOptions,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), String> {
    // 1. Create VkImage with external memory info
    let mut external_memory_image_info = vk::ExternalMemoryImageCreateInfo::builder()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

    let mut image_create_info_builder = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(options.format)
        .extent(vk::Extent3D { width: options.width, height: options.height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL) // Default, may be changed by DRM modifier
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT) // Adjust usage as needed
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    // Chain external memory info
    image_create_info_builder = image_create_info_builder.push_next(&mut external_memory_image_info);

    // If DRM format modifier is used, it needs VkImageDrmFormatModifierListCreateInfoEXT
    let mut drm_modifier_info_storage; // Needs to live long enough
    if let Some(modifier) = options.drm_format_modifier {
        drm_modifier_info_storage = vk::ImageDrmFormatModifierListCreateInfoEXT::builder()
            .drm_format_modifiers(std::slice::from_ref(&modifier));
        image_create_info_builder = image_create_info_builder.push_next(&mut drm_modifier_info_storage);
        // Also need to set tiling to DRM_FORMAT_MODIFIER_EXT
        image_create_info_builder.tiling = vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT;
    }

    let image_create_info = image_create_info_builder.build();


    let image = unsafe {
        device.create_image(&image_create_info, None)
            .map_err(|e| format!("Failed to create external image: {}", e))?
    };

    // 2. Allocate and bind memory for the external image
    // This is different from normal allocation; we import the FD.
    let mut import_memory_info = vk::ImportMemoryFdInfoKHR::builder()
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
        .fd(options.fd); // The actual DMA-BUF file descriptor

    // Before allocating, it's good practice to get memory requirements for the external image,
    // especially to confirm the size and required memoryTypeBits for the import.
    // However, for DMA-BUF, the size is often dictated by the buffer itself.
    // The options.allocation_size should be derived from the DMA-BUF's actual size.
    // The options.memory_type_index must be compatible with the DMA-BUF and image.

    let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(options.allocation_size)
        .memory_type_index(options.memory_type_index)
        .push_next(&mut import_memory_info);

    let device_memory = unsafe {
        device.allocate_memory(&memory_allocate_info, None)
            .map_err(|e| format!("Failed to import DMA-BUF memory: {}", e))?
    };

    unsafe {
        device.bind_image_memory(image, device_memory, 0)
            .map_err(|e| format!("Failed to bind memory to external image: {}", e))?;
    }

    // 3. Create ImageView (simplified from swapchain.rs)
    let imageview_create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(options.format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR) // Assuming color
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        );
    let image_view = unsafe {
        device.create_image_view(&imageview_create_info, None)
        .map_err(|e| format!("Failed to create image view for DMA-BUF image: {}", e))?
    };

    Ok((image, device_memory, image_view))
}
