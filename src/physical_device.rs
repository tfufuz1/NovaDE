use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance; // To access the VulkanInstance
use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::Arc; // To hold VulkanInstance
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{KhrSurfaceExtension, KhrSwapchainExtension, KhrWaylandSurfaceExtension};

// For DMA-BUF extensions, as per Rendering Vulkan.md
use vulkanalia::vk::{ExtExternalMemoryDmaBufExtension, KhrExternalMemoryFdExtension, ExtImageDrmFormatModifierExtension};


// Struct to hold queue family indices
#[derive(Debug, Default, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete_for_graphics_present(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

// Struct to hold swapchain support details
#[derive(Debug, Clone)]
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

// Main struct for this module
pub struct PhysicalDeviceInfo {
    physical_device: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_indices: QueueFamilyIndices,
    // Swapchain support details are queried here but might be more relevant when a surface exists.
    // For now, we ensure the device *can* support a swapchain via extension check.
    // Specific surface support details will be checked by surface_swapchain.rs
}

impl PhysicalDeviceInfo {
    pub fn new(
        instance_wrapper: &VulkanInstance,
        surface: vk::SurfaceKHR, // Needed to check presentation support
        wayland_display: *mut std::ffi::c_void, // For vkGetPhysicalDeviceWaylandPresentationSupportKHR
    ) -> Result<Self> {
        let instance = instance_wrapper.raw();
        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(VulkanError::VkResult)?
            .into_iter()
            .filter_map(|pd| {
                let score = Self::score_device(instance, pd, surface, wayland_display);
                if score > 0 {
                    Some((score, pd))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let physical_device = physical_devices
            .into_iter()
            .max_by_key(|(score, _)| *score)
            .map(|(_, pd)| pd)
            .ok_or(VulkanError::PhysicalDeviceSelectionFailed)?;

        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_family_indices = Self::find_queue_families(instance, physical_device, surface, wayland_display)?;

        log::info!("Selected physical device: {}", unsafe {
            CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy()
        });
        log::info!("Device type: {:?}", properties.device_type);
        log::info!("Queue families: {:?}", queue_family_indices);

        Ok(Self {
            physical_device,
            properties,
            features,
            queue_family_indices,
        })
    }

    fn score_device(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        wayland_display: *mut std::ffi::c_void,
    ) -> i32 {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };

        let required_extensions = [
            KhrSwapchainExtension::name(),
            // DMA-BUF related extensions from Rendering Vulkan.md (Section 3.2 & 3.4)
            ExtExternalMemoryDmaBufExtension::name(),
            KhrExternalMemoryFdExtension::name(),
            ExtImageDrmFormatModifierExtension::name(),
        ];

        if !Self::check_device_extension_support(instance, physical_device, &required_extensions) {
            log::debug!(
                "Device {} does not support required extensions.",
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() }
            );
            return 0; // Essential extensions not supported
        }

        let indices = match Self::find_queue_families(instance, physical_device, surface, wayland_display) {
            Ok(i) => i,
            Err(_) => {
                log::debug!(
                    "Could not find required queue families for device {}.",
                    unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() }
                );
                return 0; // Could not find required queue families
            }
        };

        if !indices.is_complete_for_graphics_present() {
            log::debug!(
                "Device {} does not have complete graphics/present queues.",
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() }
            );
            return 0; // Graphics or present queue not found
        }
        
        // Check for basic swapchain support (at least one format and present mode)
        // More detailed check will be in surface_swapchain.rs
        let support = match Self::query_swap_chain_support_internal(instance, physical_device, surface) {
            Ok(s) => s,
            Err(_) => {
                log::debug!(
                    "Device {} does not support swapchain (internal query failed).",
                    unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() }
                );
                return 0;
            }
        };
        if support.formats.is_empty() || support.present_modes.is_empty() {
            log::debug!(
                "Device {} has no available swapchain formats or present modes.",
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() }
            );
            return 0;
        }


        let mut score = 0;

        // Prioritize integrated GPU as per documentation (UMA target)
        if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
            score += 1000;
        } else if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 500;
        }
        
        // API version (prefer 1.3 as per Rendering Vulkan.md)
        if properties.api_version >= vk::API_VERSION_1_3 {
            score += 200;
        }


        // Prefer devices with samplerAnisotropy
        if features.sampler_anisotropy != vk::FALSE {
            score += 100;
        }
        
        // Add more scoring based on limits or other features if needed
        score += properties.limits.max_image_dimension2d / 100;
        
        log::debug!(
            "Device {} scored: {}",
            unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy() },
            score
        );
        score
    }

    fn check_device_extension_support(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
        required_extensions_cptr: &[&CStr],
    ) -> bool {
        let available_extensions = match unsafe {
            instance.enumerate_device_extension_properties(physical_device, None)
        } {
            Ok(ext) => ext,
            Err(e) => {
                log::error!("Failed to enumerate device extensions: {}", e);
                return false;
            }
        };

        let mut available_set = HashSet::new();
        for ext_prop in available_extensions {
            available_set.insert(unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) });
        }

        for required_ext_name in required_extensions_cptr {
            if !available_set.contains(required_ext_name) {
                log::warn!("Physical device missing required extension: {:?}", required_ext_name);
                return false;
            }
        }
        true
    }

    fn find_queue_families(
        instance: &Arc<Instance>,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        wayland_display: *mut std::ffi::c_void, // Raw pointer from Smithay/Wayland
    ) -> Result<QueueFamilyIndices> {
        let mut indices = QueueFamilyIndices::default();
        let queue_family_properties = unsafe {
            instance.get_physical_device_queue_family_properties(physical_device)
        };

        for (i, properties) in queue_family_properties.iter().enumerate() {
            let i = i as u32;

            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i);
            }

            if properties.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                indices.compute_family = Some(i);
            }
            
            if properties.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                // Prefer a dedicated transfer queue if it's not also graphics or compute
                if !properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
                   !properties.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                    if indices.transfer_family.is_none() { // Take the first dedicated one
                        indices.transfer_family = Some(i);
                    }
                } else if indices.transfer_family.is_none() { // Or take any transfer queue if no dedicated one is found yet
                     indices.transfer_family = Some(i);
                }
            }

            // Check for Wayland presentation support
            // This requires the KhrWaylandSurfaceExtension to be loaded by the Instance.
            // The `wayland_display` pointer is platform specific.
            // The actual type for `wayland_display` in vulkanalia is `*mut wl_display`
            // which is often `*mut wayland_client::sys::client::wl_display`.
            // Since we receive `*mut c_void`, we cast it.
            let present_support = unsafe {
                instance.get_physical_device_wayland_presentation_support_khr(
                    physical_device,
                    i,
                    wayland_display as *mut _, // Cast to vulkanalia's expected wayland display type
                )
            } == vk::TRUE;
            
            if present_support {
                indices.present_family = Some(i);
            }
        }
        
        // If no dedicated transfer queue was found, it can often share the graphics or compute queue
        if indices.transfer_family.is_none() {
            if indices.graphics_family.is_some() { // Graphics queues usually support transfer
                indices.transfer_family = indices.graphics_family;
            } else if indices.compute_family.is_some() { // Compute queues also usually support transfer
                indices.transfer_family = indices.compute_family;
            }
        }


        if indices.is_complete_for_graphics_present() {
            Ok(indices)
        } else {
            Err(VulkanError::QueueFamilyNotFound)
        }
    }
    
    // Helper function to query swap chain support, used internally for scoring.
    // A more public version might be in surface_swapchain.rs or passed from there.
    fn query_swap_chain_support_internal(
        instance: &Arc<Instance>,
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
        
        Ok(SwapChainSupportDetails {
            capabilities,
            formats,
            present_modes,
        })
    }

    // Getters
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }
    
    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices
    }
}

// Note: The `wayland_display` parameter in `new` and `find_queue_families`
// is a raw pointer (`*mut std::ffi::c_void`). This is typical for FFI with Wayland.
// Smithay or a similar library would provide this pointer.
// For `get_physical_device_wayland_presentation_support_khr`, vulkanalia expects
// `*mut wl_display` from the `wayland_sys` crate if you were using that directly.
// Casting `*mut c_void` to `*mut wayland_sys::client::wl_display` might be needed
// if using stricter types, but for the FFI call itself `*mut c_void` is often acceptable
// if the underlying function signature in the Vulkan loader expects that.
// Here, vulkanalia's generated wrapper for `vkGetPhysicalDeviceWaylandPresentationSupportKHR`
// likely expects a specific pointer type from `wayland-client` or similar,
// so a cast `wayland_display as *mut wayland_client::sys::client::wl_display`
// might be needed if `vulkanalia` is compiled with `wayland-client` feature.
// For now, `*mut _` is used as a placeholder for the correct Wayland display pointer type
// that vulkanalia's wrapper expects. This might need adjustment based on the exact
// Wayland binding crate used in the broader project.
// The `KhrWaylandSurfaceExtension` must be enabled on the instance.
