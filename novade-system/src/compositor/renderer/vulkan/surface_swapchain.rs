//! Manages Vulkan surfaces (`VkSurfaceKHR`) and swapchains (`VkSwapchainKHR`).
//!
//! This module handles the interaction with the windowing system to create a Vulkan surface
//! (specifically for Wayland using `VK_KHR_wayland_surface`). It then manages the swapchain,
//! which is a collection of renderable images that are presented to this surface.
//! The `SurfaceSwapchain` struct encapsulates the surface, swapchain, associated images,
//! image views, and the logic for creating, recreating (e.g., on window resize),
//! and cleaning up these resources.

use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo;
use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::{vk, extensions::khr::{Surface as SurfaceLoader, Swapchain as SwapchainLoader, WaylandSurface as WaylandSurfaceLoader}};
use log::{debug, info, warn, error};
use std::os::raw::c_void; // For Wayland display/surface pointers
use std::ptr;

/// A simple wrapper for raw Wayland window system handles.
///
/// This struct is used to pass the necessary Wayland display (`wl_display`)
/// and surface (`wl_surface`) pointers to Vulkan for creating a `VkSurfaceKHR`
/// via the `VK_KHR_wayland_surface` extension.
#[derive(Debug, Clone, Copy)]
pub struct RawWindowHandleWrapper {
    /// Pointer to the Wayland display (`struct wl_display*`).
    pub wl_display: *mut c_void,
    /// Pointer to the Wayland surface (`struct wl_surface*`).
    pub wl_surface: *mut c_void,
}

/// Manages a Vulkan surface (`VkSurfaceKHR`) and its associated swapchain (`VkSwapchainKHR`).
///
/// This struct encapsulates all Vulkan objects related to displaying rendered images
/// on a window surface. It handles the creation of the surface itself, the swapchain,
/// retrieval of swapchain images, and creation of `VkImageView`s for these images.
/// It also provides functionality for recreating the swapchain when necessary (e.g.,
/// due to window resizing or surface properties changing) and ensures proper cleanup
/// of all resources via the `Drop` trait.
#[derive(Debug)]
pub struct SurfaceSwapchain {
    /// Loader for surface-related Vulkan functions (`VK_KHR_surface`).
    surface_loader: SurfaceLoader,
    /// Loader for swapchain-related Vulkan functions (`VK_KHR_swapchain`).
    swapchain_loader: SwapchainLoader,

    /// The Vulkan surface handle (`VkSurfaceKHR`).
    surface: vk::SurfaceKHR,
    /// The Vulkan swapchain handle (`VkSwapchainKHR`).
    swapchain: vk::SwapchainKHR,

    /// The `vk::Format` of the images in the swapchain.
    format: vk::Format,
    /// The `vk::Extent2D` (width and height) of the images in the swapchain.
    extent: vk::Extent2D,
    /// A vector of `vk::Image` handles for the swapchain images. These are owned by the swapchain.
    images: Vec<vk::Image>,
    /// A vector of `vk::ImageView` handles, one for each image in `images`.
    image_views: Vec<vk::ImageView>,

    /// Handle to the `vk::PhysicalDevice` used, stored for recreation if capabilities need re-querying.
    #[allow(dead_code)] // Potentially used in more advanced recreate logic not yet implemented
    physical_device_handle: vk::PhysicalDevice,
    /// A clone of the `ash::Device` handle, used for destroying image views and the swapchain.
    logical_device_raw: ash::Device,
}

/// Creates a Vulkan surface (`VkSurfaceKHR`) from Wayland display and surface handles.
///
/// This function uses the `VK_KHR_wayland_surface` extension to create a Vulkan surface
/// that can be rendered to and presented on a Wayland window.
///
/// # Arguments
///
/// * `entry`: A reference to the `ash::Entry` (Vulkan entry points).
/// * `instance`: A reference to the `ash::Instance` (Vulkan instance).
/// * `window_handles`: A `RawWindowHandleWrapper` containing non-null pointers to the
///   Wayland `wl_display` and `wl_surface`.
///
/// # Returns
///
/// A `Result` containing the created `vk::SurfaceKHR` on success.
/// On failure, returns a `VulkanError`. Possible errors include:
/// - `VulkanError::InitializationError`: If `wl_display` or `wl_surface` in `window_handles` is null.
/// - `VulkanError::VkResult`: If `vkCreateWaylandSurfaceKHR` fails.
///
/// # Safety
///
/// - `entry` and `instance` must be valid Vulkan handles.
/// - The `wl_display` and `wl_surface` pointers in `window_handles` must be valid Wayland
///   handles and must remain valid for the lifetime of the created `VkSurfaceKHR`.
pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window_handles: &RawWindowHandleWrapper,
) -> Result<vk::SurfaceKHR> {
    info!(
        "Creating Vulkan surface for Wayland display {:p} and surface {:p}",
        window_handles.wl_display, window_handles.wl_surface
    );

    if window_handles.wl_display.is_null() || window_handles.wl_surface.is_null() {
        let err_msg = "Wayland display or surface handle is null during Vulkan surface creation.".to_string();
        error!("{}", err_msg);
        return Err(VulkanError::InitializationError(err_msg));
    }

    let wayland_surface_loader = WaylandSurfaceLoader::new(entry, instance);
    let surface_create_info = vk::WaylandSurfaceCreateInfoKHR::builder()
        .display(window_handles.wl_display)
        .surface(window_handles.wl_surface);

    unsafe {
        wayland_surface_loader.create_wayland_surface(&surface_create_info, None)
    }.map_err(VulkanError::from)
}


impl SurfaceSwapchain {
    /// Creates a new `SurfaceSwapchain` instance.
    ///
    /// This constructor initializes the necessary Vulkan extension loaders (`SurfaceLoader`, `SwapchainLoader`)
    /// and then calls an internal helper method (`create_swapchain_internal`) to perform the
    /// initial creation of the swapchain, retrieve its images, and create image views.
    ///
    /// # Arguments
    ///
    /// * `vulkan_instance`: A reference to the `VulkanInstance` (provides `ash::Entry` and `ash::Instance`).
    /// * `physical_device_info`: A reference to `PhysicalDeviceInfo` for the selected GPU.
    /// * `logical_device`: A reference to the `LogicalDevice` used for creating swapchain-related resources.
    /// * `surface_khr`: The `vk::SurfaceKHR` handle that this swapchain will be associated with.
    ///   This surface must have been created using `create_surface` or a similar mechanism.
    /// * `initial_extent`: The desired initial `vk::Extent2D` (width and height) for the swapchain images.
    ///   This may be adjusted by the implementation based on surface capabilities.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `SurfaceSwapchain` instance on success.
    /// On failure, returns a `VulkanError` if any step in swapchain creation fails (propagated
    /// from `create_swapchain_internal`).
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        surface_khr: vk::SurfaceKHR,
        initial_extent: vk::Extent2D,
    ) -> Result<Self> {
        let device_name = unsafe { std::ffi::CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
            .to_str().unwrap_or("Unknown Device");
        info!("Creating new SurfaceSwapchain for device '{}' with initial extent: {:?}", device_name, initial_extent);
        let surface_loader = SurfaceLoader::new(vulkan_instance.entry(), vulkan_instance.raw());
        let swapchain_loader = SwapchainLoader::new(vulkan_instance.raw(), &logical_device.raw);

        let mut surface_swapchain = Self {
            surface_loader, swapchain_loader, surface: surface_khr,
            swapchain: vk::SwapchainKHR::null(), // Initialized by create_swapchain_internal
            format: vk::Format::UNDEFINED, extent: vk::Extent2D::default(),
            images: Vec::new(), image_views: Vec::new(),
            physical_device_handle: physical_device_info.physical_device,
            logical_device_raw: logical_device.raw.clone(),
        };

        surface_swapchain.create_swapchain_internal(physical_device_info, logical_device, initial_extent, None)?;
        info!("SurfaceSwapchain for device '{}' created successfully.", device_name);
        Ok(surface_swapchain)
    }

    /// Internal helper to create/recreate the swapchain and its resources. (Private)
    ///
    /// This method encapsulates the detailed logic for:
    /// 1. Querying surface capabilities, formats, and present modes.
    /// 2. Choosing optimal settings (format, present mode, extent, image count).
    /// 3. Configuring `VkSwapchainCreateInfoKHR`.
    /// 4. Creating the `VkSwapchainKHR`.
    /// 5. Retrieving `VkImage` handles from the swapchain.
    /// 6. Creating `VkImageView`s for each swapchain image.
    /// It updates the `SurfaceSwapchain` instance's fields with the new resources.
    ///
    /// # Arguments
    /// * `physical_device_info`: Needed for querying surface capabilities.
    /// * `logical_device`: Needed for creating image views.
    /// * `desired_extent`: The preferred dimensions for the swapchain.
    /// * `old_swapchain`: An `Option<vk::SwapchainKHR>` for optimized recreation. If `Some`,
    ///   it's a handle to the previous swapchain that can be reused by the driver.
    ///
    /// # Returns
    /// `Result<()>` indicating success or a `VulkanError` on failure.
    fn create_swapchain_internal(
        &mut self,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        desired_extent: vk::Extent2D,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Result<()> {
        let device_name = unsafe { std::ffi::CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
            .to_str().unwrap_or("Unknown Device");
        info!("Device '{}': Creating/recreating internal swapchain with desired extent: {:?}, old_swapchain: {:?}", 
            device_name, desired_extent, old_swapchain.is_some());

        let capabilities = unsafe { self.surface_loader.get_physical_device_surface_capabilities(physical_device_info.physical_device, self.surface) }?;
        let formats = unsafe { self.surface_loader.get_physical_device_surface_formats(physical_device_info.physical_device, self.surface) }?;
        let present_modes = unsafe { self.surface_loader.get_physical_device_surface_present_modes(physical_device_info.physical_device, self.surface) }?;

        if formats.is_empty() {
            let err_msg = "No surface formats available for swapchain.".to_string();
            error!("Device '{}': {}", device_name, err_msg);
            return Err(VulkanError::ResourceCreationError{ resource_type: "Swapchain".to_string(), message: err_msg });
        }
        if present_modes.is_empty() {
            let err_msg = "No surface present modes available for swapchain.".to_string();
            error!("Device '{}': {}", device_name, err_msg);
            return Err(VulkanError::ResourceCreationError{ resource_type: "Swapchain".to_string(), message: err_msg });
        }

        let surface_format = formats.iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or_else(|| {
                warn!("Device '{}': Desired format B8G8R8A8_SRGB not found, using first available: {:?}", device_name, formats[0]);
                &formats[0]
            });
        let present_mode = present_modes.iter().cloned().find(|&m| m == vk::PresentModeKHR::MAILBOX).unwrap_or(vk::PresentModeKHR::FIFO);
        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: desired_extent.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                height: desired_extent.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
            }
        };
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count { image_count = capabilities.max_image_count; }
        debug!("Device '{}': Swapchain chosen settings - Format: {:?}, ColorSpace: {:?}, PresentMode: {:?}, Extent: {:?}, ImageCount: {}",
            device_name, surface_format.format, surface_format.color_space, present_mode, extent, image_count);

        let mut create_info_builder = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface).min_image_count(image_count)
            .image_format(surface_format.format).image_color_space(surface_format.color_space)
            .image_extent(extent).image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Assume opaque for simplicity
            .present_mode(present_mode).clipped(true);
        if let Some(old) = old_swapchain { create_info_builder = create_info_builder.old_swapchain(old); }
        
        let qfi = &physical_device_info.queue_family_indices;
        let queue_family_indices_array;
        if qfi.graphics_family != qfi.present_family {
            queue_family_indices_array = [qfi.graphics_family.unwrap(), qfi.present_family.unwrap()]; // Should always be Some if is_complete passed
            create_info_builder = create_info_builder.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&queue_family_indices_array);
            info!("Device '{}': Using CONCURRENT sharing mode for swapchain.", device_name);
        } else {
            create_info_builder = create_info_builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
            info!("Device '{}': Using EXCLUSIVE sharing mode for swapchain.", device_name);
        }
        let create_info = create_info_builder.build();

        self.swapchain = unsafe { self.swapchain_loader.create_swapchain(&create_info, None) }?;
        info!("Device '{}': Swapchain created/recreated successfully: {:?}", device_name, self.swapchain);

        self.images = unsafe { self.swapchain_loader.get_swapchain_images(self.swapchain) }?;
        info!("Device '{}': Retrieved {} swapchain images.", device_name, self.images.len());

        self.image_views = self.images.iter().map(|&image| {
            let view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image).view_type(vk::ImageViewType::TYPE_2D).format(surface_format.format)
                .components(vk::ComponentMapping::default()) // Identity mapping for RGBA
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0,
                    level_count: 1, base_array_layer: 0, layer_count: 1,
                });
            unsafe { logical_device.raw.create_image_view(&view_create_info, None) }
        }).collect::<std::result::Result<Vec<_>, _>>()?; // Collect Results, then map_err the outer Result
        info!("Device '{}': Created {} image views.", device_name, self.image_views.len());

        self.format = surface_format.format;
        self.extent = extent;
        Ok(())
    }

    /// Recreates the swapchain, typically after a window resize or when it becomes outdated.
    ///
    /// This method first waits for the logical device to be idle to ensure no resources
    /// are in use. It then cleans up the old swapchain's image views (but keeps the old
    /// swapchain handle itself for optimized recreation). After that, it calls
    /// `create_swapchain_internal` to generate a new swapchain and its associated resources.
    /// Finally, the old swapchain handle is destroyed.
    ///
    /// # Arguments
    ///
    /// * `physical_device_info`: A reference to `PhysicalDeviceInfo`. This must be passed again as
    ///   surface capabilities might have changed (e.g., due to window resize).
    /// * `logical_device`: A reference to the `LogicalDevice`, needed for creating new image views
    ///   and for waiting for device idle.
    /// * `new_extent`: The new `vk::Extent2D` (dimensions) for the swapchain. This should be
    ///   obtained from the windowing system or surface capabilities after the event that
    ///   triggered the recreation (e.g., resize).
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure. Possible errors are propagated from
    /// `create_swapchain_internal` or if waiting for device idle fails (`VulkanError::VkResult`).
    ///
    /// # Safety
    ///
    /// The caller must ensure that the arguments (`physical_device_info`, `logical_device`)
    /// are valid. This function involves `unsafe` calls for device operations.
    pub fn recreate(
        &mut self,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        new_extent: vk::Extent2D,
    ) -> Result<()> {
        let device_name = unsafe { std::ffi::CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
            .to_str().unwrap_or("Unknown Device");
        info!("Device '{}': Recreating swapchain with new extent: {:?}", device_name, new_extent);
        unsafe { logical_device.raw.device_wait_idle() }?;

        let old_swapchain_handle = self.swapchain;
        self.cleanup_swapchain_resources(); // Cleans views, keeps old_swapchain handle

        // Attempt to create the new swapchain using the old one for potential optimization
        self.create_swapchain_internal(physical_device_info, logical_device, new_extent, Some(old_swapchain_handle))?;
        
        // If create_swapchain_internal was successful, the old_swapchain_handle is now truly old.
        // Destroy it. self.swapchain now holds the new swapchain.
        unsafe { self.swapchain_loader.destroy_swapchain(old_swapchain_handle, None); }
        info!("Device '{}': Old swapchain object {:?} destroyed after successful recreation.", device_name, old_swapchain_handle);

        info!("Device '{}': Swapchain recreated successfully.", device_name);
        Ok(())
    }
    
    /// Cleans up image views associated with the current swapchain. (Private helper)
    ///
    /// This method is called during swapchain recreation before creating new resources.
    /// It iterates through `self.image_views` and destroys each `VkImageView`.
    /// The `self.images` vector is cleared as these images are owned by the swapchain
    /// and will be invalid once the old swapchain is destroyed. The `vk::SwapchainKHR`
    /// handle itself (`self.swapchain`) is *not* destroyed here, as it's needed for
    /// `old_swapchain` in `vkCreateSwapchainKHR` for optimized recreation.
    fn cleanup_swapchain_resources(&mut self) {
        info!("Cleaning up swapchain resources (image views).");
        for &image_view in self.image_views.iter() {
            unsafe { self.logical_device_raw.destroy_image_view(image_view, None); }
        }
        self.image_views.clear();
        // Swapchain images are owned by the swapchain, so they are not destroyed individually.
        // Clearing the vector just removes our handles to them.
        self.images.clear(); 
        debug!("Swapchain image views cleared, image list cleared.");
    }

    /// Cleans up all resources owned by this `SurfaceSwapchain` instance. (Private helper)
    ///
    /// This includes destroying all image views and the `VkSwapchainKHR` object itself.
    /// This method is typically called from the `Drop` implementation.
    fn cleanup_full_swapchain(&mut self) {
        info!("Cleaning up full swapchain (image views and swapchain object).");
        for &image_view in self.image_views.iter() {
            unsafe { self.logical_device_raw.destroy_image_view(image_view, None); }
        }
        self.image_views.clear();
        self.images.clear();

        if self.swapchain != vk::SwapchainKHR::null() {
            unsafe { self.swapchain_loader.destroy_swapchain(self.swapchain, None); }
            self.swapchain = vk::SwapchainKHR::null(); // Mark as destroyed
            info!("Swapchain object destroyed.");
        } else {
            info!("Swapchain object was already null, no destruction needed.");
        }
    }

    // --- Accessors ---

    /// Returns the `vk::Format` of the swapchain images.
    pub fn format(&self) -> vk::Format { self.format }
    /// Returns the `vk::Extent2D` (dimensions) of the swapchain images.
    pub fn extent(&self) -> vk::Extent2D { self.extent }
    /// Returns a slice of `vk::ImageView` handles for the swapchain images.
    pub fn image_views(&self) -> &[vk::ImageView] { &self.image_views }
    /// Returns the raw `vk::SwapchainKHR` handle.
    pub fn swapchain_khr(&self) -> vk::SwapchainKHR { self.swapchain }
    /// Returns the raw `vk::SurfaceKHR` handle.
    pub fn surface_khr(&self) -> vk::SurfaceKHR { self.surface }
    /// Returns the number of images in the swapchain.
    pub fn image_count(&self) -> usize { self.images.len() }
}

impl Drop for SurfaceSwapchain {
    /// Ensures all Vulkan resources owned by this `SurfaceSwapchain` are released.
    ///
    /// This method calls `cleanup_full_swapchain()` to destroy image views and the
    /// swapchain object itself. Then, it destroys the `VkSurfaceKHR`.
    ///
    /// # Safety
    ///
    /// - The `logical_device_raw` and `surface_loader` members must still be valid.
    /// - All GPU operations using this swapchain or its resources must be complete
    ///   before the `SurfaceSwapchain` is dropped.
    fn drop(&mut self) {
        info!("Dropping SurfaceSwapchain...");
        self.cleanup_full_swapchain(); // Destroys image views and the swapchain
        
        if self.surface != vk::SurfaceKHR::null() {
            unsafe {
                self.surface_loader.destroy_surface(self.surface, None);
                info!("Vulkan surface {:?} destroyed.", self.surface);
            }
        }
        // `logical_device_raw` is a cloned `ash::Device`, its Drop is handled by ash if not manually managed.
        // `surface_loader` and `swapchain_loader` are just function pointer tables, no specific drop needed.
        info!("SurfaceSwapchain dropped.");
    }
}
