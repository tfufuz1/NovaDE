use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use ash::extensions::khr::{Surface as SurfaceLoader, Swapchain};
use ash::vk;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::ffi::CStr;

/// Holds the indices of queue families found for a physical device.
#[derive(Debug, Default, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Checks if all essential queue families (graphics and present) have been found.
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

/// Holds information about a selected physical device.
#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub queue_family_indices: QueueFamilyIndices,
    // We will also store the memory properties, as they are often needed with the physical device.
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

/// Selects a suitable physical device for Vulkan operations.
///
/// This function enumerates available physical devices, checks their suitability
/// based on properties, features, extensions, and queue family support, and
/// returns information about the selected device.
///
/// # Arguments
/// * `vulkan_instance`: A reference to the `VulkanInstance`.
/// * `surface`: A `vk::SurfaceKHR` handle for checking presentation support.
///
/// # Returns
/// `Result<PhysicalDeviceInfo, String>` containing information about the selected
/// device or an error message.
pub fn select_physical_device(
    vulkan_instance: &VulkanInstance,
    surface: vk::SurfaceKHR,
) -> Result<PhysicalDeviceInfo, String> {
    let instance = vulkan_instance.raw();
    let physical_devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|e| format!("Failed to enumerate physical devices: {}", e))?;

    if physical_devices.is_empty() {
        return Err("No Vulkan-compatible physical devices found.".to_string());
    }

    info!("Found {} physical device(s). Evaluating suitability...", physical_devices.len());

    let mut last_suitable_device: Option<(vk::PhysicalDevice, QueueFamilyIndices, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, vk::PhysicalDeviceMemoryProperties)> = None;

    for &physical_device in physical_devices.iter() {
        match is_device_suitable(instance, vulkan_instance.entry(), physical_device, surface) {
            Ok(Some((indices, properties, features, memory_properties))) => {
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
                info!("Device suitable: {:?} (Type: {:?})", device_name, properties.device_type);
                // Prefer Integrated GPU, then Discrete GPU, then others.
                // This logic ensures that if an iGPU is suitable, it's preferred.
                // If a dGPU is found later and is also suitable, it won't override the iGPU unless no iGPU was found.
                if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                    info!("Selected Integrated GPU: {:?}", device_name);
                    return Ok(PhysicalDeviceInfo {
                        physical_device,
                        properties,
                        features,
                        queue_family_indices: indices,
                        memory_properties,
                    });
                } else if last_suitable_device.is_none() ||
                          (last_suitable_device.as_ref().unwrap().2.device_type != vk::PhysicalDeviceType::INTEGRATED_GPU &&
                           properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU) {
                    last_suitable_device = Some((physical_device, indices, properties, features, memory_properties));
                }
            }
            Ok(None) => {
                let properties = unsafe { instance.get_physical_device_properties(physical_device) };
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
                debug!("Device not suitable: {:?}", device_name);
            }
            Err(e) => {
                 let properties = unsafe { instance.get_physical_device_properties(physical_device) };
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
                warn!("Error checking suitability for device {:?}: {}", device_name, e);
            }
        }
    }

    if let Some((pd, qfi, props, feats, mem_props)) = last_suitable_device {
        let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
        if props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
             info!("Final selected device (Integrated GPU): {:?}", device_name);
        } else {
             info!("Final selected device (Type: {:?}): {:?}", props.device_type, device_name);
        }
        return Ok(PhysicalDeviceInfo {
            physical_device: pd,
            properties: props,
            features: feats,
            queue_family_indices: qfi,
            memory_properties: mem_props,
        });
    }

    Err("No suitable physical device found meeting all criteria.".to_string())
}

/// Checks if a given physical device is suitable for the application.
///
/// # Returns
/// `Result<Option<(QueueFamilyIndices, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, vk::PhysicalDeviceMemoryProperties)>, String>`
/// `Ok(Some(...))` if suitable, `Ok(None)` if not suitable but no error occurred, `Err(...)` if an error occurred.
fn is_device_suitable(
    instance: &ash::Instance,
    entry: &ash::Entry, // Needed for surface loader
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<Option<(QueueFamilyIndices, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, vk::PhysicalDeviceMemoryProperties)>, String> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let features = unsafe { instance.get_physical_device_features(physical_device) };
    let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    // 1. Check API version
    if properties.api_version < vk::API_VERSION_1_3 {
        debug!("Device {:?} does not support Vulkan 1.3 (supports {:?})", unsafe {CStr::from_ptr(properties.device_name.as_ptr())}, properties.api_version);
        return Ok(None);
    }

    // 2. Check required features (example: samplerAnisotropy)
    if features.sampler_anisotropy == vk::FALSE {
        debug!("Device {:?} does not support samplerAnisotropy.", unsafe {CStr::from_ptr(properties.device_name.as_ptr())});
        return Ok(None);
    }
    // Add more feature checks as needed by the application

    // 3. Check for required device extensions
    let required_extensions_cstrs = [
        Swapchain::name(), // VK_KHR_SWAPCHAIN_EXTENSION_NAME
        // For Wayland/DRM interop, common on Linux for compositors
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_external_memory_dma_buf\0") },
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_KHR_external_memory_fd\0") },
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_image_drm_format_modifier\0") },
    ];

    let available_extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }
        .map_err(|e| format!("Failed to enumerate device extensions for {:?}: {}", unsafe {CStr::from_ptr(properties.device_name.as_ptr())}, e))?;

    let mut available_extension_names = HashSet::new();
    for ext in available_extensions {
        let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
        available_extension_names.insert(name.to_owned());
    }

    for required_ext_name_cstr in &required_extensions_cstrs {
        if !available_extension_names.contains(required_ext_name_cstr) {
            debug!(
                "Device {:?} is missing required extension: {:?}",
                unsafe {CStr::from_ptr(properties.device_name.as_ptr())},
                required_ext_name_cstr
            );
            return Ok(None);
        }
    }
    debug!("Device {:?} has all required extensions.", unsafe {CStr::from_ptr(properties.device_name.as_ptr())});


    // 4. Find suitable queue families
    let surface_loader = SurfaceLoader::new(entry, instance);
    let queue_family_indices = find_queue_families(instance, physical_device, surface, &surface_loader)?;

    if !queue_family_indices.is_complete() {
        debug!("Device {:?} does not have all required queue families (graphics and present). Graphics: {:?}, Present: {:?}",
            unsafe {CStr::from_ptr(properties.device_name.as_ptr())},
            queue_family_indices.graphics_family,
            queue_family_indices.present_family
        );
        return Ok(None);
    }
    debug!("Device {:?} has suitable queue families: {:?}", unsafe {CStr::from_ptr(properties.device_name.as_ptr())}, queue_family_indices);

    // Device type preference is handled in the main selection loop.
    // Here, we just confirm it's suitable if all checks pass.
    Ok(Some((queue_family_indices, properties, features, memory_properties)))
}

/// Finds necessary queue families for a given physical device.
fn find_queue_families(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    surface_loader: &SurfaceLoader,
) -> Result<QueueFamilyIndices, String> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut indices = QueueFamilyIndices::default();

    let mut dedicated_transfer_family: Option<u32> = None;
    let mut combined_transfer_family: Option<u32> = None;

    for (i, queue_family) in queue_family_properties.iter().enumerate() {
        let current_index = i as u32;

        // Check for Graphics support
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_family = Some(current_index);
        }

        // Check for Compute support
        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            indices.compute_family = Some(current_index);
        }

        // Check for Transfer support
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            if !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
               !queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                // This is a dedicated transfer queue
                dedicated_transfer_family = Some(current_index);
            }
            // This could be a combined queue (Graphics/Compute + Transfer)
            combined_transfer_family = Some(current_index);
        }


        // Check for Presentation support
        // This needs to be done for each queue family, as presentation support is per-queue family.
        // However, it's common for the graphics queue to also support presentation.
        let present_support = unsafe {
            surface_loader.get_physical_device_surface_support(physical_device, current_index, surface)
        }.map_err(|e| format!("Failed to query surface support for queue family {}: {}", current_index, e))?;

        if present_support {
            indices.present_family = Some(current_index);
        }

        // Optimization: if we've found a graphics and a present family, we can potentially stop early
        // if we don't care about dedicated compute/transfer queues or finding the "best" ones.
        // For now, we iterate all to find dedicated queues if available.
    }

    // Prioritize dedicated transfer queue if available
    if dedicated_transfer_family.is_some() {
        indices.transfer_family = dedicated_transfer_family;
    } else {
        indices.transfer_family = combined_transfer_family; // Fallback to any transfer-capable queue
    }
    
    // Fallback for compute: can be the same as graphics if no dedicated compute queue is found
    if indices.compute_family.is_none() {
        indices.compute_family = indices.graphics_family;
    }

    // Fallback for transfer: can be the same as graphics or compute if no other transfer queue is found
    if indices.transfer_family.is_none() {
        if indices.compute_family.is_some() && queue_family_properties[indices.compute_family.unwrap() as usize].queue_flags.contains(vk::QueueFlags::TRANSFER) {
            indices.transfer_family = indices.compute_family;
        } else if indices.graphics_family.is_some() && queue_family_properties[indices.graphics_family.unwrap() as usize].queue_flags.contains(vk::QueueFlags::TRANSFER) {
            indices.transfer_family = indices.graphics_family;
        }
    }


    if indices.graphics_family.is_none() {
        return Err("Could not find a queue family with GRAPHICS support.".to_string());
    }
    if indices.present_family.is_none() {
         return Err("Could not find a queue family with PRESENT support for the given surface.".to_string());
    }
    // Optional queues are not strictly errors if not found, but we log them.
    if indices.compute_family.is_none() {
        warn!("Could not find a dedicated queue family with COMPUTE support. Will try to use graphics queue if needed.");
    }
    if indices.transfer_family.is_none() {
        warn!("Could not find a dedicated or suitable fallback queue family with TRANSFER support.");
    }


    Ok(indices)
}

/// Finds a supported format from a list of candidates.
///
/// # Arguments
/// * `instance`: Handle to the Vulkan instance.
/// * `physical_device`: Handle to the physical device.
/// * `candidates`: A slice of `vk::Format` candidates to check.
/// * `tiling`: The desired image tiling (`vk::ImageTiling::LINEAR` or `vk::ImageTiling::OPTIMAL`).
/// * `features`: The required `vk::FormatFeatureFlags`.
///
/// # Returns
/// `Option<vk::Format>` containing the first supported format found, or `None`.
pub fn find_supported_format(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Option<vk::Format> {
    for &format in candidates {
        let props = unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        match tiling {
            vk::ImageTiling::LINEAR => {
                if props.linear_tiling_features.contains(features) {
                    return Some(format);
                }
            }
            vk::ImageTiling::OPTIMAL => {
                if props.optimal_tiling_features.contains(features) {
                    return Some(format);
                }
            }
            _ => { // Should not happen for TILING_DRM_FORMAT_MODIFIER_EXT
                warn!("Unsupported tiling mode for find_supported_format: {:?}", tiling);
                return None;
            }
        }
    }
    None
}
