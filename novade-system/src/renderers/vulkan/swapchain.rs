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

/// Helper function to create `vk::ImageView`s for a set of swapchain `vk::Image`s.
///
/// Each image view will be a 2D view of the color aspect of the corresponding image.
///
/// # Arguments
/// * `device`: Reference to the logical `ash::Device`.
/// * `images`: Slice of `vk::Image` handles (typically from the swapchain).
/// * `format`: The `vk::Format` of the images.
///
/// # Returns
/// A `Result` containing a `Vec<vk::ImageView>` or an error string on failure.
fn create_swapchain_image_views(
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


// ANCHOR: VulkanSwapchain Implementation
impl VulkanSwapchain {
    /// Creates a new `VulkanSwapchain` instance.
    ///
    /// This constructor handles the creation of a Vulkan surface, queries for swapchain support,
    /// chooses appropriate settings (format, present mode, extent), and then creates the
    /// swapchain, its images, and image views. It uses the `create_swapchain_direct`
    /// function internally for the core swapchain creation logic.
    ///
    /// # Arguments
    /// * `context`: A reference to `VulkanContext` which provides necessary Vulkan handles
    ///   (instance, device, physical device, entry point, queue indices).
    /// * `window_handle`: A provider for `RawWindowHandle`, used to create the Vulkan surface.
    /// * `window_width`: The initial width for the swapchain extent.
    /// * `window_height`: The initial height for the swapchain extent.
    /// * `old_swapchain_khr`: An optional `vk::SwapchainKHR` handle if this is a recreation
    ///   (e.g., after a window resize).
    ///
    /// # Returns
    /// A `Result` containing the new `VulkanSwapchain` or an error string on failure.
    pub fn new(
        context: &VulkanContext, // Provides instance, device, physical_device, entry
        window_handle: &impl HasRawWindowHandle, // Used for surface creation internally
        window_width: u32,
        window_height: u32,
        old_swapchain_khr: Option<vk::SwapchainKHR>, // For recreation
    ) -> Result<Self, String> {

        // Surface creation is part of this constructor for VulkanSwapchain convenience
        let surface_loader = ash::extensions::khr::Surface::new(context.entry(), context.instance());
        let surface = unsafe {
            ash_window::create_surface(
                context.entry(),
                context.instance(),
                // context.raw_display_handle(), // Assuming context provides this
                window_handle.raw_window_handle(), // The prompt implies this is how it's passed
                None, // Allocator
            )
        }.map_err(|e| format!("VulkanSwapchain::new: Failed to create Vulkan surface: {}", e))?;

        // Ensure the selected physical device's queue family supports this surface
        // This check is crucial and relies on VulkanContext providing correct queue indices.
        let present_support = unsafe {
            surface_loader.get_physical_device_surface_support(
                context.physical_device(),
                context.present_queue_family_index(),
                surface,
            )
        }.unwrap_or(false);

        if !present_support {
            unsafe { surface_loader.destroy_surface(surface, None); } // Clean up created surface
            return Err(format!(
                "VulkanSwapchain::new: Physical device queue family {} does not support presentation to the created surface.",
                context.present_queue_family_index()
            ));
        }

        let (swapchain_loader_new, swapchain, format, extent, images, image_views) =
            create_swapchain_direct(
                context.instance(),
                context.device().as_ref(), // Get &AshDevice from Arc<AshDevice>
                context.physical_device(),
                &surface_loader, // Pass the loader created above
                surface, // Pass the surface created above
                context.graphics_queue_family_index(), // Assuming these are correct from context
                context.present_queue_family_index(),
                window_width,
                window_height,
                old_swapchain_khr,
            ).map_err(|e| {
                // If create_swapchain_direct fails, the surface it was given still needs cleanup here
                unsafe { surface_loader.destroy_surface(surface, None); }
                e
            })?;

        Ok(Self {
            surface_loader, // Store the loader used for the surface
            swapchain_loader: swapchain_loader_new,
            surface, // Store the surface
            swapchain,
            images,
            image_views,
            format,
            extent,
            device: context.device().clone(),
            instance_ref: context.instance_arc(),
        })
    }

    // ANCHOR: Accessors
    /// Returns the `vk::Format` of the swapchain images.
    pub fn format(&self) -> vk::Format {
        self.format
    }

    /// Returns the `vk::Extent2D` (dimensions) of the swapchain images.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// Returns a slice of `vk::ImageView`s for the swapchain images.
    pub fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }

    // ANCHOR: Query Swapchain Support Helper
    /// Queries the physical device for its swapchain support capabilities for a given surface.
    ///
    /// This includes surface capabilities (min/max image count, extents, etc.),
    /// supported surface formats, and available presentation modes.
    /// Aligns with `Rendering Vulkan.md` (Spec 5.2.1, 5.2.2, 5.2.3).
    /// This is a public static method to be usable by `create_swapchain_direct`.
    ///
    /// # Arguments
    /// * `instance_ash`: Reference to the `ash::Instance`.
    /// * `surface_loader`: Reference to the `ash::extensions::khr::Surface` loader.
    /// * `physical_device`: The `vk::PhysicalDevice` to query.
    /// * `surface`: The `vk::SurfaceKHR` to query support for.
    ///
    /// # Returns
    /// A `Result` containing `SwapchainSupportDetails` or an error string.
    pub fn query_swapchain_support(
        instance_ash: &AshInstance,
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
    /// Selects a suitable `vk::SurfaceFormatKHR` from the list of available formats.
    ///
    /// Prioritizes `VK_FORMAT_B8G8R8A8_SRGB` with `VK_COLOR_SPACE_SRGB_NONLINEAR_KHR`
    /// as per `Rendering Vulkan.md` (Spec 5.2.2). Falls back to the first available format if not found.
    /// This is a public static method.
    pub fn choose_surface_format(
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
                eprintln!("Warning: Desired format B8G8R8A8_SRGB with SRGB_NONLINEAR not found. Using first available: {:?}", available_formats[0]);
                available_formats[0]
            })
    }

    // ANCHOR: Choose Present Mode Helper
    /// Selects a `vk::PresentModeKHR` from the list of available modes.
    ///
    /// Prioritizes `MAILBOX` for low latency, then `FIFO_RELAXED`, then `FIFO` (guaranteed).
    /// Aligns with `Rendering Vulkan.md` (Spec 5.2.3).
    /// This is a public static method.
    pub fn choose_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        // Spec 5.2.3: Prioritize MAILBOX for low latency, then FIFO_RELAXED, then FIFO.
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if available_present_modes.contains(&vk::PresentModeKHR::FIFO_RELAXED) {
            vk::PresentModeKHR::FIFO_RELAXED
        } else {
            vk::PresentModeKHR::FIFO // Guaranteed available
        }
    }

    // ANCHOR: Choose Swap Extent Helper
    /// Determines the `vk::Extent2D` (resolution) for the swapchain images.
    ///
    /// Uses the surface capabilities' `currentExtent` if defined, otherwise clamps
    /// the provided window width/height to the min/max extents supported by the surface.
    /// Aligns with `Rendering Vulkan.md` (Spec 5.2.4 `imageExtent`).
    /// This is a public static method.
    pub fn choose_swap_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window_width: u32,
        window_height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
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

    // ANCHOR: Create Image Views Helper (moved to top-level)
    // fn create_image_views(...) - now a top-level function: create_swapchain_image_views
}


/// Creates a Vulkan swapchain and its associated resources directly.
///
/// This function encapsulates the logic for swapchain creation as per `Rendering Vulkan.md` (Spec 5.2).
/// It handles querying surface capabilities, choosing appropriate formats, present modes, and extents,
/// creating the `vk::SwapchainKHR` object, retrieving its images, and creating `vk::ImageView`s for them.
/// This is a procedural approach to setting up the swapchain components.
///
/// # Arguments
/// * `instance`: A reference to the `ash::Instance`.
/// * `device`: A reference to the logical `ash::Device`.
/// * `physical_device`: The `vk::PhysicalDevice` being used.
/// * `surface_loader`: A reference to the `ash::extensions::khr::Surface` loader.
/// * `surface`: The `vk::SurfaceKHR` for which the swapchain is created.
/// * `graphics_q_idx`: The queue family index for graphics operations.
/// * `present_q_idx`: The queue family index for presentation operations. These indices are used
///   to configure image sharing modes if they are different.
/// * `window_width`: The current width of the window/surface, used for determining swapchain extent.
/// * `window_height`: The current height of the window/surface.
/// * `old_swapchain`: An `Option<vk::SwapchainKHR>` representing a previous swapchain. This is used
///   when recreating the swapchain (e.g., after a window resize) to potentially reuse resources
///   or ensure smoother transitions. Pass `None` for initial creation.
///
/// # Returns
/// A `Result` containing a tuple with the following elements upon success:
///   - `ash::extensions::khr::Swapchain`: The loader for swapchain functions.
///   - `vk::SwapchainKHR`: The handle to the created swapchain.
///   - `vk::Format`: The image format chosen for the swapchain.
///   - `vk::Extent2D`: The extent (resolution) of the swapchain images.
///   - `Vec<vk::Image>`: A vector of `vk::Image` handles belonging to the swapchain.
///   - `Vec<vk::ImageView>`: A vector of `vk::ImageView` handles, one for each swapchain image.
/// Returns an error string on failure at any step of the process.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 5.2):
/// - **5.2.1 (Oberflächen-Fähigkeiten abfragen):** Done via `VulkanSwapchain::query_swapchain_support`.
/// - **5.2.2 (Oberflächen-Formate abfragen):** Done via `VulkanSwapchain::query_swapchain_support` and chosen by `VulkanSwapchain::choose_surface_format`.
/// - **5.2.3 (Präsentationsmodi abfragen):** Done via `VulkanSwapchain::query_swapchain_support` and chosen by `VulkanSwapchain::choose_present_mode`.
/// - **5.2.4 (`VkSwapchainCreateInfoKHR` Konfiguration):** All fields are configured:
///   - `surface`, `minImageCount`, `imageFormat`, `imageColorSpace`, `imageExtent`, `imageArrayLayers` (1).
///   - `imageUsage`: Set to `COLOR_ATTACHMENT | TRANSFER_DST` as per spec.
///   - `imageSharingMode`: Handles `CONCURRENT` vs `EXCLUSIVE` based on queue indices.
///   - `preTransform`, `compositeAlpha`, `presentMode`, `clipped`, `oldSwapchain`.
/// - **5.2.5 (`vkCreateSwapchainKHR` Aufruf):** Done via `swapchain_loader.create_swapchain`.
/// - **5.2.6 (Swapchain-Images abrufen):** Done via `swapchain_loader.get_swapchain_images`.
/// - Image view creation (related to Spec 6.5 but done here for swapchain images) is handled by `create_swapchain_image_views`.
#[allow(clippy::too_many_arguments)] // Justified by the number of Vulkan objects needed
pub fn create_swapchain_direct(
    instance: &AshInstance,
/// This function encapsulates the logic for swapchain creation as per `Rendering Vulkan.md` Spec 5.2.
/// It's a procedural way to get all necessary swapchain components.
///
/// # Arguments
/// * `instance`: The Vulkan instance.
/// * `device`: The logical device.
/// * `physical_device`: The physical device.
/// * `surface_loader`: Loader for surface-related functions.
/// * `surface`: The `vk::SurfaceKHR` to create the swapchain for.
/// * `graphics_q_idx`: Index of the graphics queue family.
/// * `present_q_idx`: Index of the presentation queue family.
/// * `window_width`: Current width of the window.
/// * `window_height`: Current height of the window.
/// * `old_swapchain`: Optional handle to an old swapchain for recreation.
///
/// # Returns
/// A tuple containing:
///   - `ash::extensions::khr::Swapchain` (the loader for swapchain functions)
///   - `vk::SwapchainKHR` (the swapchain handle)
///   - `vk::Format` (the chosen swapchain image format)
///   - `vk::Extent2D` (the chosen swapchain extent/resolution)
///   - `Vec<vk::Image>` (the swapchain images)
///   - `Vec<vk::ImageView>` (image views for the swapchain images)
#[allow(clippy::too_many_arguments)] // Justified by the number of Vulkan objects needed
pub fn create_swapchain_direct(
    instance: &AshInstance,
    device: &AshDevice,
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    graphics_q_idx: u32,
    present_q_idx: u32,
    window_width: u32,
    window_height: u32,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> Result<(
    ash::extensions::khr::Swapchain,
    vk::SwapchainKHR,
    vk::Format,
    vk::Extent2D,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
), String> {
    let support_details = VulkanSwapchain::query_swapchain_support(
        instance,
        surface_loader,
        physical_device,
        surface,
    )?;

    let surface_format = VulkanSwapchain::choose_surface_format(&support_details.formats);
    let present_mode = VulkanSwapchain::choose_present_mode(&support_details.present_modes);
    let extent = VulkanSwapchain::choose_swap_extent(&support_details.capabilities, window_width, window_height);

    let mut image_count = support_details.capabilities.min_image_count + 1;
    if support_details.capabilities.max_image_count > 0 && image_count > support_details.capabilities.max_image_count {
        image_count = support_details.capabilities.max_image_count;
    }

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);

    let mut create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST); // Spec 5.2.4

    let queue_family_indices_array = [graphics_q_idx, present_q_idx];
    if graphics_q_idx != present_q_idx {
        create_info = create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_family_indices_array);
    } else {
        create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
    }

    create_info = create_info
        .pre_transform(support_details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(old_swapchain.unwrap_or_else(vk::SwapchainKHR::null));

    let swapchain_khr = unsafe {
        swapchain_loader.create_swapchain(&create_info, None)
    }.map_err(|e| format!("Failed to create swapchain (direct): {}", e))?;

    let images = unsafe {
        swapchain_loader.get_swapchain_images(swapchain_khr)
    }.map_err(|e| format!("Failed to get swapchain images (direct): {}", e))?;

    let image_views = create_swapchain_image_views(device, &images, surface_format.format)?;

    Ok((swapchain_loader, swapchain_khr, surface_format.format, extent, images, image_views))
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
            // Only destroy surface if this VulkanSwapchain instance created it and is its sole owner.
            // In the current VulkanSwapchain::new, it does create it.
            // If create_swapchain_direct is used elsewhere, surface lifetime is managed by caller.
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
            // Spec 5.2.4: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST);

        let queue_family_indices_array = [ // Renamed to avoid conflict
            context.graphics_queue_family_index(),
            context.present_queue_family_index(),
        ];

        if context.graphics_queue_family_index() != context.present_queue_family_index() {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices_array);
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
