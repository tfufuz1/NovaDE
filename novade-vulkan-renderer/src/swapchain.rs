use ash::{vk, Instance, Device};
use std::sync::Arc;
// use crate::VulkanContext; // Removed unused import
use crate::QueueFamilyIndices; // Made public in lib.rs

pub struct Swapchain {
    // instance: Arc<Instance>, // Not strictly needed if not creating new resources with it directly
    device: Arc<Device>, // Needed for creating image views, destroying swapchain
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>, // Added
    // physical_device: vk::PhysicalDevice, // Passed to new()
    // surface: vk::SurfaceKHR, // Passed to new()
}

impl Swapchain {
    #[allow(clippy::too_many_arguments)] // Common for Vulkan setup functions
    pub fn new(
        instance_arc: Arc<Instance>,
        device_arc: Arc<Device>,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        queue_family_indices: &QueueFamilyIndices, // Renamed from _queue_family_indices
        width: u32,
        height: u32,
        chosen_surface_format: &vk::SurfaceFormatKHR, // Passed in
        render_pass_handle: vk::RenderPass,        // Passed in
    ) -> Result<Self, anyhow::Error> {
        // Query surface capabilities and present modes (formats are already chosen)
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        // Formats are passed via chosen_surface_format
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        // if formats.is_empty() { // Not needed, format is passed in
        //     return Err(anyhow::anyhow!("No surface formats available for swapchain."));
        // }
        if present_modes.is_empty() {
            return Err(anyhow::anyhow!("No present modes available for swapchain."));
        }

        // Select optimal configuration
        // 1. Format - already provided by chosen_surface_format
        let selected_format_info = *chosen_surface_format;

        // 2. Present Mode
        let selected_present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO); // FIFO is guaranteed to be available

        // 3. Extent
        let selected_extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        // 4. Image Count
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }
        if capabilities.min_image_count == 0 && image_count == 0 { // Should not happen if min_image_count is valid
             return Err(anyhow::anyhow!("Swapchain min_image_count is zero, cannot create swapchain."));
        }


        // Create VkSwapchainKHR
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance_arc, &device_arc);

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(selected_format_info.format) // Use from chosen_surface_format
            .image_color_space(selected_format_info.color_space) // Use from chosen_surface_format
            .image_extent(selected_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST) // Added TRANSFER_DST
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Assuming opaque for now
            .present_mode(selected_present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        // Handle image sharing mode
        let graphics_family_idx = queue_family_indices.graphics_family.ok_or_else(|| anyhow::anyhow!("Graphics queue family index not found for swapchain"))?;
        let present_family_idx = queue_family_indices.present_family.ok_or_else(|| anyhow::anyhow!("Present queue family index not found for swapchain"))?;

        let actual_queue_family_indices = [graphics_family_idx, present_family_idx];

        if graphics_family_idx != present_family_idx {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&actual_queue_family_indices);
        } else {
            swapchain_create_info = swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None)?
        };

        // Retrieve images
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let mut framebuffers = Vec::with_capacity(images.len()); // Added

        // Create image views and framebuffers
        let mut image_views = Vec::with_capacity(images.len());
        for image in &images {
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(selected_format_info.format) // Use from chosen_surface_format
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

            let image_view = unsafe {
                device_arc.create_image_view(&image_view_create_info, None)?
            };
            image_views.push(image_view);

            // Create Framebuffer
            let fb_attachments = [image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass_handle)
                .attachments(&fb_attachments)
                .width(selected_extent.width)
                .height(selected_extent.height)
                .layers(1);

            let framebuffer = unsafe {
                device_arc.create_framebuffer(&framebuffer_create_info, None)?
            };
            framebuffers.push(framebuffer);
        }

        Ok(Self {
            device: device_arc.clone(),
            swapchain_loader,
            swapchain,
            format: selected_format_info.format,
            color_space: selected_format_info.color_space,
            extent: selected_extent,
            images,
            image_views,
            framebuffers, // Added
        })
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for fb in self.framebuffers.iter() {
                self.device.destroy_framebuffer(*fb, None);
            }
            for image_view in self.image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
