use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo;
use crate::compositor::renderer::vulkan::device::LogicalDevice;
use ash::{vk, extensions::khr::{Surface as SurfaceLoader, Swapchain as SwapchainLoader, WaylandSurface as WaylandSurfaceLoader}};
use log::{debug, info, warn, error};
use std::os::raw::c_void;
use std::ptr;

/// Wrapper for raw Wayland window handles needed for surface creation.
#[derive(Debug, Clone, Copy)]
pub struct RawWindowHandleWrapper {
    pub wl_display: *mut c_void, // Pointer to wl_display
    pub wl_surface: *mut c_void, // Pointer to wl_surface
}

/// Manages a Vulkan surface and its associated swapchain.
#[derive(Debug)]
pub struct SurfaceSwapchain {
    // Loaders
    surface_loader: SurfaceLoader,
    swapchain_loader: SwapchainLoader,

    // Vulkan Handles
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,

    // Swapchain Properties
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,

    // Required for recreation
    // We store references/copies if they are cheap or pointers if not.
    // For this task, assuming these are passed in again during recreate or stored if simple.
    // Let's store copies of PhysicalDeviceInfo and a reference to LogicalDevice for now.
    // However, to avoid lifetime issues with LogicalDevice if SurfaceSwapchain outlives it,
    // it's better to pass LogicalDevice into methods that need it.
    // For simplicity in this task, we'll assume they are available when needed.
    // A more robust solution might involve Rc<LogicalDevice> or similar.
    physical_device_handle: vk::PhysicalDevice, // Store handle directly
    logical_device_raw: ash::Device,         // Store raw device for operations
}

/// Creates a Vulkan surface (`vk::SurfaceKHR`) from Wayland display and surface handles.
///
/// # Arguments
/// * `entry`: Reference to the `ash::Entry` for loading extensions.
/// * `instance`: Reference to the `ash::Instance` for creating the surface.
/// * `window_handles`: A `RawWindowHandleWrapper` containing pointers to `wl_display` and `wl_surface`.
///
/// # Returns
/// `Result<vk::SurfaceKHR, String>` containing the created surface or an error message.
pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window_handles: &RawWindowHandleWrapper,
) -> Result<vk::SurfaceKHR, String> {
    info!(
        "Creating Vulkan surface for Wayland display {:?} and surface {:?}",
        window_handles.wl_display, window_handles.wl_surface
    );

    if window_handles.wl_display.is_null() || window_handles.wl_surface.is_null() {
        return Err("Wayland display or surface handle is null.".to_string());
    }

    let wayland_surface_loader = WaylandSurfaceLoader::new(entry, instance);
    let surface_create_info = vk::WaylandSurfaceCreateInfoKHR::builder()
        .display(window_handles.wl_display)
        .surface(window_handles.wl_surface)
        .build();

    unsafe {
        wayland_surface_loader.create_wayland_surface(&surface_create_info, None)
    }.map_err(|e| format!("Failed to create Wayland surface: {}", e))
}


impl SurfaceSwapchain {
    /// Creates a new `SurfaceSwapchain` instance.
    ///
    /// # Arguments
    /// * `vulkan_instance`: Reference to the `VulkanInstance`.
    /// * `physical_device_info`: Information about the selected physical device.
    /// * `logical_device`: Reference to the `LogicalDevice`.
    /// * `surface_khr`: The `vk::SurfaceKHR` handle created by `create_surface`.
    /// * `initial_extent`: The desired initial dimensions for the swapchain.
    ///
    /// # Returns
    /// `Result<Self, String>` containing the new `SurfaceSwapchain` or an error message.
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        surface_khr: vk::SurfaceKHR,
        initial_extent: vk::Extent2D,
    ) -> Result<Self, String> {
        info!("Creating new SurfaceSwapchain with initial extent: {:?}", initial_extent);
        let surface_loader = SurfaceLoader::new(vulkan_instance.entry(), vulkan_instance.raw());
        let swapchain_loader = SwapchainLoader::new(vulkan_instance.raw(), &logical_device.raw);

        let mut surface_swapchain = Self {
            surface_loader,
            swapchain_loader,
            surface: surface_khr,
            swapchain: vk::SwapchainKHR::null(), // Will be created by create_swapchain_internal
            format: vk::Format::UNDEFINED,
            extent: vk::Extent2D::default(),
            images: Vec::new(),
            image_views: Vec::new(),
            physical_device_handle: physical_device_info.physical_device,
            logical_device_raw: logical_device.raw.clone(), // Clone the ash::Device
        };

        surface_swapchain.create_swapchain_internal(physical_device_info, logical_device, initial_extent, None)?;
        info!("SurfaceSwapchain created successfully.");
        Ok(surface_swapchain)
    }

    /// Internal helper function to create or recreate the swapchain and its associated resources.
    fn create_swapchain_internal(
        &mut self,
        physical_device_info: &PhysicalDeviceInfo, // Passed in for capabilities
        logical_device: &LogicalDevice,         // Passed in for image view creation
        desired_extent: vk::Extent2D,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Result<(), String> {
        info!("Creating/recreating internal swapchain with desired extent: {:?}, old_swapchain: {:?}", desired_extent, old_swapchain.is_some());

        // 1. Query surface capabilities
        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device_info.physical_device, self.surface)
        }.map_err(|e| format!("Failed to get surface capabilities: {}", e))?;

        let formats = unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device_info.physical_device, self.surface)
        }.map_err(|e| format!("Failed to get surface formats: {}", e))?;

        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(physical_device_info.physical_device, self.surface)
        }.map_err(|e| format!("Failed to get surface present modes: {}", e))?;

        if formats.is_empty() {
            return Err("No surface formats available.".to_string());
        }
        if present_modes.is_empty() {
            return Err("No surface present modes available.".to_string());
        }

        // 2. Choose optimal settings
        let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| {
                warn!("Desired format B8G8R8A8_SRGB not found, using first available: {:?}", formats[0]);
                &formats[0]
            });

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&m| m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO); // FIFO is guaranteed to be available

        let extent = if capabilities.current_extent.width != u32::MAX {
            debug!("Using current_extent from capabilities: {:?}", capabilities.current_extent);
            capabilities.current_extent
        } else {
            let mut actual_extent = desired_extent;
            actual_extent.width = actual_extent.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            );
            actual_extent.height = actual_extent.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            );
            debug!("Calculated extent: {:?}, based on desired: {:?}", actual_extent, desired_extent);
            actual_extent
        };

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }
        debug!("Selected image_count: {}", image_count);


        // 3. Configure VkSwapchainCreateInfoKHR
        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST) // TRANSFER_DST for post-processing/blits
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE) // Assuming graphics and present queues are the same or handled
            .queue_family_indices(&[]) // Only needed if sharing_mode is CONCURRENT
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        if let Some(old) = old_swapchain {
            create_info = create_info.old_swapchain(old);
        }
        
        // Handle potential sharing mode if graphics and present queues are different
        // This requires queue family indices from PhysicalDeviceInfo
        let qfi = &physical_device_info.queue_family_indices;
        let queue_family_indices_array;
        if qfi.graphics_family != qfi.present_family {
            queue_family_indices_array = [qfi.graphics_family.unwrap(), qfi.present_family.unwrap()];
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices_array);
            info!("Using CONCURRENT sharing mode for swapchain due to different graphics/present queue families.");
        } else {
            info!("Using EXCLUSIVE sharing mode for swapchain.");
        }


        // 4. Create swapchain
        self.swapchain = unsafe { self.swapchain_loader.create_swapchain(&create_info, None) }
            .map_err(|e| format!("Failed to create swapchain: {}", e))?;
        info!("Swapchain created/recreated successfully: {:?}", self.swapchain);

        // 5. Get swapchain images
        self.images = unsafe { self.swapchain_loader.get_swapchain_images(self.swapchain) }
            .map_err(|e| format!("Failed to get swapchain images: {}", e))?;
        info!("Retrieved {} swapchain images.", self.images.len());

        // 6. Create image views
        self.image_views = self
            .images
            .iter()
            .map(|&image| {
                let view_create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
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
                unsafe { logical_device.raw.create_image_view(&view_create_info, None) }
                    .map_err(|e| format!("Failed to create image view: {}", e))
            })
            .collect::<Result<Vec<_>, _>>()?;
        info!("Created {} image views.", self.image_views.len());

        self.format = surface_format.format;
        self.extent = extent;

        Ok(())
    }

    /// Recreates the swapchain, typically after a window resize or when it becomes outdated.
    ///
    /// # Arguments
    /// * `physical_device_info`: Must be passed again as surface capabilities might change.
    /// * `logical_device`: Needed for creating new image views.
    /// * `new_extent`: The new dimensions for the swapchain.
    ///
    /// # Returns
    /// `Result<(), String>` indicating success or failure.
    pub fn recreate(
        &mut self,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        new_extent: vk::Extent2D,
    ) -> Result<(), String> {
        info!("Recreating swapchain with new extent: {:?}", new_extent);
        unsafe { logical_device.raw.device_wait_idle() }
            .map_err(|e| format!("Failed to wait for device idle: {}", e))?;

        let old_swapchain_handle = self.swapchain;
        self.cleanup_swapchain_resources(); // Only cleans up views and images, keeps old_swapchain handle for recreation

        self.create_swapchain_internal(physical_device_info, logical_device, new_extent, Some(old_swapchain_handle))?;
        
        // After successful recreation with the old_swapchain handle, destroy the actual old swapchain object.
        // The new swapchain creation call in create_swapchain_internal already implicitly replaces self.swapchain.
        // The old_swapchain handle passed to create_info.old_swapchain() allows resources to be reused.
        // We still need to destroy the old KHR object itself.
        unsafe { self.swapchain_loader.destroy_swapchain(old_swapchain_handle, None); }
        info!("Old swapchain object {:?} destroyed after successful recreation.", old_swapchain_handle);


        info!("Swapchain recreated successfully.");
        Ok(())
    }
    
    /// Cleans up resources associated with the current swapchain (images, image views).
    /// The swapchain handle itself is kept for `old_swapchain` during recreation.
    fn cleanup_swapchain_resources(&mut self) {
        info!("Cleaning up swapchain resources (image views).");
        for &image_view in self.image_views.iter() {
            unsafe {
                self.logical_device_raw.destroy_image_view(image_view, None);
            }
        }
        self.image_views.clear();
        self.images.clear(); // Images are owned by the swapchain, no explicit destruction here.
        // self.swapchain is not destroyed here, it's passed as old_swapchain
        debug!("Swapchain image views cleared.");
    }


    /// Cleans up all resources owned by this `SurfaceSwapchain` instance,
    /// including image views and the swapchain itself.
    /// This is typically called before dropping the struct or when fully destroying the swapchain.
    fn cleanup_full_swapchain(&mut self) {
        info!("Cleaning up full swapchain (image views and swapchain object).");
        for &image_view in self.image_views.iter() {
            unsafe {
                self.logical_device_raw.destroy_image_view(image_view, None);
            }
        }
        self.image_views.clear();
        self.images.clear(); // Images are owned by the swapchain

        if self.swapchain != vk::SwapchainKHR::null() {
            unsafe {
                self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            }
            self.swapchain = vk::SwapchainKHR::null();
            info!("Swapchain object destroyed.");
        }
    }

    // Accessors
    pub fn format(&self) -> vk::Format { self.format }
    pub fn extent(&self) -> vk::Extent2D { self.extent }
    pub fn image_views(&self) -> &[vk::ImageView] { &self.image_views }
    pub fn swapchain_khr(&self) -> vk::SwapchainKHR { self.swapchain }
    pub fn surface_khr(&self) -> vk::SurfaceKHR { self.surface }
    pub fn image_count(&self) -> usize { self.images.len() }

}

impl Drop for SurfaceSwapchain {
    fn drop(&mut self) {
        info!("Dropping SurfaceSwapchain...");
        self.cleanup_full_swapchain();
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
            info!("Vulkan surface {:?} destroyed.", self.surface);
        }
        // logical_device_raw is a clone of ash::Device, its Drop will be handled by ash.
        // Loaders are also handled by their respective Drop implementations.
        info!("SurfaceSwapchain dropped.");
    }
}
