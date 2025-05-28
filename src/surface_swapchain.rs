use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance;
use crate::physical_device::{PhysicalDeviceInfo, QueueFamilyIndices, SwapChainSupportDetails};
use crate::device::LogicalDevice;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window; // For create_surface

// These are raw pointers obtained from Smithay or a similar Wayland integration.
// They are opaque handles to Wayland objects.
pub type WaylandDisplayPtr = *mut std::ffi::c_void;
pub type WaylandSurfacePtr = *mut std::ffi::c_void;

pub struct SurfaceSwapchain {
    instance: Arc<Instance>, // To destroy surface
    device: Arc<Device>,     // To destroy swapchain and image views
    
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    
    format: vk::Format,
    extent: vk::Extent2D,

    // Keep physical_device_info and queue_indices for recreation
    physical_device: vk::PhysicalDevice, // No Arc needed, it's just a handle
    queue_indices: QueueFamilyIndices,
    wayland_display: WaylandDisplayPtr,
    wayland_surface_ptr: WaylandSurfacePtr, // The wl_surface pointer for recreation
}

impl SurfaceSwapchain {
    pub fn new(
        instance_wrapper: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device_wrapper: &LogicalDevice,
        wayland_display: WaylandDisplayPtr,    // e.g., from smithay::reexports::client::wl_display
        wayland_surface_ptr: WaylandSurfacePtr, // e.g., from smithay::reexports::client::wl_surface
        preferred_width: u32,
        preferred_height: u32,
    ) -> Result<Self> {
        let instance = instance_wrapper.raw().clone();
        let physical_device_handle = physical_device_info.raw(); // Renamed to avoid conflict
        let device = logical_device_wrapper.raw().clone();
        let queue_indices = physical_device_info.queue_family_indices();

        // 1. Create Surface
        // The `wayland_display` and `wayland_surface_ptr` are crucial here.
        // `vulkanalia::window::create_surface` handles the platform-specific call.
        let surface = unsafe {
            vk_window::create_surface(
                &instance,
                wayland_display as *mut _, // Casts might be needed depending on exact types
                wayland_surface_ptr as *mut _,
                None,
            )
        }
        .map_err(VulkanError::VkResult)?;
        log::info!("Vulkan surface created for Wayland window.");

        // Verify presentation support for the chosen graphics queue family on this new surface
        // This was partially checked in PhysicalDeviceInfo, but this is the definitive check.
        let present_support = unsafe {
            instance.get_physical_device_surface_support_khr(
                physical_device_handle, // Use renamed variable
                queue_indices.present_family.ok_or(VulkanError::QueueFamilyNotFound)?,
                surface,
            )
        } .map_err(VulkanError::VkResult)?;
        if present_support != vk::TRUE {
            return Err(VulkanError::Message("Selected queue family does not support presentation to this surface.".to_string()));
        }


        // 2. Create Swapchain (initial creation)
        let (swapchain, format, extent, images) = Self::create_swapchain_internal(
            &instance,
            physical_device_handle, // Use renamed variable
            &device,
            surface,
            queue_indices,
            Some((preferred_width, preferred_height)), // Initial preferred dimensions
            None, // old_swapchain
        )?;

        // 3. Create Image Views
        let image_views = Self::create_image_views_internal(&device, &images, format)?;
        
        log::info!("Swapchain created with format {:?} and extent {}x{}", format, extent.width, extent.height);

        Ok(Self {
            instance,
            device,
            surface,
            swapchain,
            images,
            image_views,
            format,
            extent,
            physical_device: physical_device_handle, // Store the handle
            queue_indices,
            wayland_display,
            wayland_surface_ptr,
        })
    }

    fn query_support(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<SwapChainSupportDetails> {
        let capabilities = unsafe {
            instance.get_physical_device_surface_capabilities_khr(physical_device, surface)
        }.map_err(VulkanError::VkResult)?;
        let formats = unsafe {
            instance.get_physical_device_surface_formats_khr(physical_device, surface, None)
        }.map_err(VulkanError::VkResult)?;
        let present_modes = unsafe {
            instance.get_physical_device_surface_present_modes_khr(physical_device, surface, None)
        }.map_err(VulkanError::VkResult)?;
        Ok(SwapChainSupportDetails { capabilities, formats, present_modes })
    }
    
    fn choose_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        available_formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB // As per Rendering Vulkan.md
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .cloned()
            .unwrap_or_else(|| available_formats[0]) // Fallback to the first available
    }

    fn choose_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        // As per Rendering Vulkan.md (Section 5.2) and general best practices
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX // Good for low latency
        } else if available_present_modes.contains(&vk::PresentModeKHR::FIFO_RELAXED) {
            vk::PresentModeKHR::FIFO_RELAXED // V-Sync, but allows tearing if app is late
        } else {
            vk::PresentModeKHR::FIFO // Standard V-Sync (guaranteed to be available)
        }
    }

    fn choose_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        preferred_width: u32,
        preferred_height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent // Window system dictates extent
        } else {
            vk::Extent2D::builder()
                .width(preferred_width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(preferred_height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }

    #[allow(clippy::too_many_arguments)] // This function has many arguments by necessity for swapchain creation
    fn create_swapchain_internal(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        device: &Device,
        surface: vk::SurfaceKHR,
        indices: QueueFamilyIndices,
        preferred_dimensions: Option<(u32,u32)>, // (width, height)
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Result<(vk::SwapchainKHR, vk::Format, vk::Extent2D, Vec<vk::Image>)> {
        let support = Self::query_support(instance, physical_device, surface)?;
        if support.formats.is_empty() || support.present_modes.is_empty() {
            return Err(VulkanError::SwapchainCreationError("No formats or present modes available".to_string()));
        }

        let surface_format = Self::choose_surface_format(&support.formats);
        let present_mode = Self::choose_present_mode(&support.present_modes);
        
        let extent = if let Some((w,h)) = preferred_dimensions {
            Self::choose_extent(&support.capabilities, w, h)
        } else {
            // If no preferred dimensions, use current extent from capabilities
            // This path is usually for recreation where we want current surface size
            support.capabilities.current_extent 
        };


        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0 // 0 means no limit
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }

        let mut queue_family_indices_vec = Vec::new();
        let image_sharing_mode = if indices.graphics_family != indices.present_family {
            queue_family_indices_vec.push(indices.graphics_family.unwrap());
            queue_family_indices_vec.push(indices.present_family.unwrap());
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST) // TRANSFER_DST for clear or post-proc blit
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices_vec)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Assuming opaque window
            .present_mode(present_mode)
            .clipped(true); // Discard pixels obscured by other windows

        if let Some(old) = old_swapchain {
            create_info = create_info.old_swapchain(old);
        }
        
        let swapchain = unsafe { device.create_swapchain_khr(&create_info, None) }
            .map_err(VulkanError::VkResult)?;

        let images = unsafe { device.get_swapchain_images_khr(swapchain, None) }
            .map_err(VulkanError::VkResult)?;

        Ok((swapchain, surface_format.format, extent, images))
    }

    fn create_image_views_internal(
        device: &Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
        images
            .iter()
            .map(|&image| {
                let components = vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY);
                let subresource_range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(format)
                    .components(components)
                    .subresource_range(subresource_range);
                unsafe { device.create_image_view(&create_info, None) }
                    .map_err(VulkanError::VkResult)
            })
            .collect()
    }
    
    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.image_views.iter().for_each(|&view| {
                self.device.destroy_image_view(view, None);
            });
            self.device.destroy_swapchain_khr(self.swapchain, None);
        }
        log::debug!("Previous swapchain and image views destroyed.");
    }

    pub fn recreate_swapchain(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        log::info!("Recreating swapchain with new dimensions: {}x{}", new_width, new_height);
        unsafe { self.device.device_wait_idle() }.map_err(VulkanError::VkResult)?; // Ensure all GPU operations are finished.

        self.cleanup_swapchain();

        let (swapchain, format, extent, images) = Self::create_swapchain_internal(
            &self.instance,
            self.physical_device,
            &self.device,
            self.surface,
            self.queue_indices,
            Some((new_width, new_height)),
            None, // old_swapchain is destroyed
        )?;
        
        let image_views = Self::create_image_views_internal(&self.device, &images, format)?;

        self.swapchain = swapchain;
        self.format = format;
        self.extent = extent;
        self.images = images;
        self.image_views = image_views;
        
        log::info!("Swapchain recreated successfully.");
        Ok(())
    }

    // Getters
    pub fn surface(&self) -> vk::SurfaceKHR { self.surface }
    pub fn swapchain(&self) -> vk::SwapchainKHR { self.swapchain }
    pub fn images(&self) -> &[vk::Image] { &self.images }
    pub fn image_views(&self) -> &[vk::ImageView] { &self.image_views }
    pub fn format(&self) -> vk::Format { self.format }
    pub fn extent(&self) -> vk::Extent2D { self.extent }
    pub fn physical_device_handle(&self) -> vk::PhysicalDevice { self.physical_device } // Added getter

}

impl Drop for SurfaceSwapchain {
    fn drop(&mut self) {
        self.cleanup_swapchain();
        unsafe {
            self.instance.destroy_surface_khr(self.surface, None);
        }
        log::debug!("Vulkan surface destroyed.");
    }
}
