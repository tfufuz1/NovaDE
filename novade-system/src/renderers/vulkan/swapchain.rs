// novade-system/src/renderers/vulkan/swapchain.rs
use ash::extensions::khr::Swapchain as SwapchainLoader;
use ash::vk;
use super::device::QueueFamilyIndices; // Assuming this path is correct

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<SwapchainSupportDetails, String> {
    unsafe {
        let capabilities = surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|e| format!("Failed to get surface capabilities: {}", e))?;
        let formats = surface_loader
            .get_physical_device_surface_formats(physical_device, surface)
            .map_err(|e| format!("Failed to get surface formats: {}", e))?;
        let present_modes = surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface)
            .map_err(|e| format!("Failed to get surface present modes: {}", e))?;
        Ok(SwapchainSupportDetails {
            capabilities,
            formats,
            present_modes,
        })
    }
}

fn choose_swap_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    available_formats
        .iter()
        .cloned()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB // Or R8G8B8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| available_formats[0])
}

fn choose_swap_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    // Prefer Mailbox for low latency, FIFO is guaranteed fallback
    if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO // Guaranteed to be available
    }
}

fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR, actual_width: u32, actual_height: u32) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: actual_width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: actual_height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub fn create_swapchain(
    instance: &ash::Instance, // Needed for SwapchainLoader
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    queue_family_indices: &QueueFamilyIndices,
    window_width: u32, // Current window width
    window_height: u32, // Current window height
    old_swapchain: Option<vk::SwapchainKHR>, // For recreation
) -> Result<(vk::SwapchainKHR, vk::Format, vk::Extent2D, Vec<vk::Image>), String> {
    let support_details = query_swapchain_support(physical_device, surface_loader, surface)?;

    let surface_format = choose_swap_surface_format(&support_details.formats);
    let present_mode = choose_swap_present_mode(&support_details.present_modes);
    let extent = choose_swap_extent(&support_details.capabilities, window_width, window_height);

    let mut image_count = support_details.capabilities.min_image_count + 1;
    if support_details.capabilities.max_image_count > 0 && image_count > support_details.capabilities.max_image_count {
        image_count = support_details.capabilities.max_image_count;
    }

    let mut create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST) // TRANSFER_DST for potential clear or post-processing blit
        .pre_transform(support_details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Or choose based on capabilities
        .present_mode(present_mode)
        .clipped(true); // Performance improvement

    let indices = [
        queue_family_indices.graphics_family.unwrap(),
        queue_family_indices.present_family.unwrap(),
    ];

    if queue_family_indices.graphics_family != queue_family_indices.present_family {
        create_info = create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&indices);
    } else {
        create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
    }

    if let Some(old) = old_swapchain {
        create_info = create_info.old_swapchain(old);
    }

    let swapchain_loader = SwapchainLoader::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&create_info, None)
            .map_err(|e| format!("Failed to create swapchain: {}", e))?
    };

    let images = unsafe {
        swapchain_loader.get_swapchain_images(swapchain)
        .map_err(|e| format!("Failed to get swapchain images: {}", e))?
    };

    Ok((swapchain, surface_format.format, extent, images))
}

pub fn create_image_views(
    device: &ash::Device,
    images: &[vk::Image],
    format: vk::Format,
) -> Result<Vec<vk::ImageView>, String> {
    images
        .iter()
        .map(|&image| {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            unsafe {
                device
                    .create_image_view(&create_info, None)
                    .map_err(|e| format!("Failed to create image view: {}", e))
            }
        })
        .collect()
}
