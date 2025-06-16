use ash::{vk, Instance as AshInstance, Device as AshDevice};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::Arc;

// Assuming VulkanContext is in super::context. This might need adjustment
// if VulkanContext is not directly accessible or if a more abstract interface is preferred.
use super::context::VulkanContext;

// ANCHOR: SwapchainSupportDetails Struct Definition
#[derive(Clone, Debug)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

// ANCHOR: VulkanSwapchain Struct Definition
pub struct VulkanSwapchain {
    // Loaders - obtained from VulkanContext or created here if appropriate
    surface_loader: ash::extensions::khr::Surface,
    swapchain_loader: ash::extensions::khr::Swapchain,

    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,

    // Keep a reference to the device for cleanup of image views and swapchain
    device: Arc<AshDevice>,
    // Keep a reference to the instance for cleanup of surface (if surface_loader is instance-level)
    // Or, if surface_loader is entry-level, then entry might be needed.
    // ash::extensions::khr::Surface is new(&entry, &instance). So instance is needed.
    instance_ref: Arc<AshInstance>, // Keep a reference to instance for surface cleanup
}

// ANCHOR: VulkanSwapchain Implementation
impl VulkanSwapchain {
    pub fn new(
        context: &VulkanContext, // Provides instance, device, physical_device, entry
        window_handle: &impl HasRawWindowHandle,
        window_width: u32,
        window_height: u32,
        old_swapchain_khr: Option<vk::SwapchainKHR>,
    ) -> Result<Self, String> {

        // ANCHOR_EXT: Surface Creation
        let surface_loader = ash::extensions::khr::Surface::new(context.entry(), context.instance());
        let surface = unsafe {
            ash_window::create_surface(
                context.entry(),
                context.instance(),
                window_handle.raw_window_handle(),
                None,
            )
        }.map_err(|e| format!("Failed to create Vulkan surface: {}", e))?;

        // Check for Wayland surface support on the physical device's queue family
        // This check was simplified in context.rs, but should be more robust here or in context.
        // For now, assume the queue family selected in VulkanContext supports presentation to this surface.
        let present_support = unsafe {
            surface_loader.get_physical_device_surface_support(
                context.physical_device(),
                context.present_queue_family_index(), // Assuming present_queue_family_index is correct for this surface
                surface,
            )
        }.unwrap_or(false);

        if !present_support {
            // Cleanup surface before erroring
            unsafe { surface_loader.destroy_surface(surface, None); }
            return Err(format!(
                "Physical device queue family {} does not support presentation to this surface.",
                context.present_queue_family_index()
            ));
        }

        // ANCHOR_EXT: Query Swapchain Support
        let support_details = Self::query_swapchain_support(
            context.instance(),
            &surface_loader,
            context.physical_device(),
            surface,
        )?;

        // ANCHOR_EXT: Choose Swapchain Settings
        let surface_format = Self::choose_surface_format(&support_details.formats);
        let present_mode = Self::choose_present_mode(&support_details.present_modes);
        let extent = Self::choose_swap_extent(&support_details.capabilities, window_width, window_height);

        let mut image_count = support_details.capabilities.min_image_count + 1;
        if support_details.capabilities.max_image_count > 0 && image_count > support_details.capabilities.max_image_count {
            image_count = support_details.capabilities.max_image_count;
        }

        // ANCHOR_EXT: Swapchain Creation
        let swapchain_loader = ash::extensions::khr::Swapchain::new(context.instance(), context.device().as_ref());

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT); // Add other usages if needed (e.g., TRANSFER_DST)

        let queue_family_indices = [
            context.graphics_queue_family_index(),
            context.present_queue_family_index(),
        ];

        if context.graphics_queue_family_index() != context.present_queue_family_index() {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        create_info = create_info
            .pre_transform(support_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Or choose from capabilities
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(old_swapchain_khr.unwrap_or_else(vk::SwapchainKHR::null));

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info, None)
        }.map_err(|e| {
            // Cleanup surface if swapchain creation fails
            unsafe { surface_loader.destroy_surface(surface, None); }
            format!("Failed to create swapchain: {}", e)
        })?;

        // ANCHOR_EXT: Retrieve Swapchain Images
        let images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        }.map_err(|e| {
            // Cleanup swapchain and surface if image retrieval fails
            unsafe {
                swapchain_loader.destroy_swapchain(swapchain, None);
                surface_loader.destroy_surface(surface, None);
            }
            format!("Failed to get swapchain images: {}", e)
        })?;

        // ANCHOR_EXT: Create Image Views
        let image_views = Self::create_image_views(context.device().as_ref(), &images, surface_format.format)?;

        Ok(Self {
            surface_loader,
            swapchain_loader,
            surface,
            swapchain,
            images,
            image_views,
            format: surface_format.format,
            extent,
            device: context.device().clone(), // Clone the Arc for shared ownership
            instance_ref: context.instance_arc(), // Store Arc<Instance>
        })
    }

    // ANCHOR: Accessors
    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }

    // ANCHOR: Query Swapchain Support Helper
    fn query_swapchain_support(
        instance_ash: &AshInstance, // Renamed to avoid conflict with instance_ref field
        surface_loader: &ash::extensions::khr::Surface,
        physical_device: vk::PhysicalDevice,
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

            if formats.is_empty() || present_modes.is_empty() {
                return Err("No surface formats or present modes found for the physical device and surface.".to_string());
            }

            Ok(SwapchainSupportDetails {
                capabilities,
                formats,
                present_modes,
            })
        }
    }

    // ANCHOR: Choose Surface Format Helper
    fn choose_surface_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> vk::SurfaceFormatKHR {
        available_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .cloned()
            .unwrap_or_else(|| {
                // Log this choice, as it's a fallback
                println!("Warning: Desired format B8G8R8A8_SRGB with SRGB_NONLINEAR not found. Using first available: {:?}", available_formats[0]);
                available_formats[0]
            })
    }

    // ANCHOR: Choose Present Mode Helper
    fn choose_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            // FIFO is guaranteed to be available according to the Vulkan specification.
            vk::PresentModeKHR::FIFO
        }
    }

    // ANCHOR: Choose Swap Extent Helper
    fn choose_swap_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window_width: u32,
        window_height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX { // u32::max indicates dynamic size
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: window_width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    // ANCHOR: Create Image Views Helper
    fn create_image_views(
        device: &AshDevice,
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
                    device.create_image_view(&create_info, None)
                }.map_err(|e| format!("Failed to create image view: {}", e))
            })
            .collect()
    }
}

// ANCHOR: VulkanSwapchain Drop Implementation
impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            println!("Dropping VulkanSwapchain...");
            for image_view in self.image_views.drain(..) { // drain to empty the vec
                self.device.destroy_image_view(image_view, None);
            }
            println!("Swapchain image views destroyed (count: {}).", self.images.len());

            // Swapchain must be destroyed before the surface
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            println!("Swapchain destroyed.");

            // Surface is destroyed after the swapchain
            self.surface_loader.destroy_surface(self.surface, None);
            println!("Surface destroyed.");
            // The device and instance_ref (Arc<AshDevice> and Arc<AshInstance>)
            // will be dropped automatically by RAII when VulkanSwapchain goes out of scope.
            // Their respective Drop impls will handle vkDestroyDevice and vkDestroyInstance if this
            // was the last reference.
            println!("VulkanSwapchain dropped.");
        }
    }
}
