//! Handles the selection and querying of Vulkan physical devices (GPUs).
//!
//! This module provides functionality to enumerate available physical devices,
//! evaluate their capabilities (properties, features, extensions, queue families),
//! and select the most suitable one for the application's needs. It defines
//! structures to hold information about queue families and the selected physical device.

use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::extensions::khr::{Surface as SurfaceLoader, Swapchain};
use ash::vk;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::ffi::CStr;

/// Holds the indices of queue families found for a physical device.
///
/// Each field is an `Option<u32>`, where `Some(index)` indicates that a queue family
/// supporting the respective operations was found, and `index` is its identifier.
/// `None` means no suitable queue family was found for that specific operation type.
#[derive(Debug, Default, Clone)]
pub struct QueueFamilyIndices {
    /// Index of the queue family supporting graphics operations. Essential for rendering.
    pub graphics_family: Option<u32>,
    /// Index of the queue family supporting presentation to a surface. Essential for displaying rendered images.
    pub present_family: Option<u32>,
    /// Index of the queue family supporting compute operations. Optional.
    pub compute_family: Option<u32>,
    /// Index of the queue family supporting transfer operations (e.g., buffer copies). Optional;
    /// may be a dedicated transfer queue or share an index with graphics/compute.
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Checks if all essential queue families (graphics and present) have been found.
    ///
    /// For basic rendering and presentation, both a graphics queue and a present queue
    /// are required. This method returns `true` if both `graphics_family` and
    /// `present_family` are `Some`.
    ///
    /// # Returns
    /// `true` if both graphics and present queue families are defined, `false` otherwise.
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

/// Holds detailed information about a selected Vulkan physical device.
///
/// This structure aggregates the `vk::PhysicalDevice` handle along with its
/// properties, supported features, queue family indices, and memory properties.
/// It serves as a convenient container for all relevant data about the chosen GPU.
#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    /// The Vulkan handle to the selected physical device.
    pub physical_device: vk::PhysicalDevice,
    /// Properties of the physical device, such as name, type, limits, etc.
    /// See `vk::PhysicalDeviceProperties`.
    pub properties: vk::PhysicalDeviceProperties,
    /// Features supported by the physical device, such as geometry shaders, tessellation, etc.
    /// See `vk::PhysicalDeviceFeatures`.
    pub features: vk::PhysicalDeviceFeatures,
    /// Indices of the queue families found on this physical device.
    pub queue_family_indices: QueueFamilyIndices,
    /// Memory properties of the physical device, detailing available memory types and heaps.
    /// See `vk::PhysicalDeviceMemoryProperties`.
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

/// Selects a suitable physical device for Vulkan operations.
///
/// This function enumerates all available Vulkan-compatible physical devices (GPUs)
/// and evaluates each one based on a set of criteria:
/// - Support for required Vulkan API version (e.g., 1.3).
/// - Availability of necessary device features (e.g., sampler anisotropy).
/// - Support for required device extensions (e.g., swapchain, DMA buffer import).
/// - Presence of suitable queue families (graphics and presentation).
///
/// It prioritizes integrated GPUs first, then discrete GPUs, and finally other types.
/// If multiple devices of the same preferred type are suitable, the last one enumerated
/// that meets all criteria is chosen.
///
/// # Arguments
///
/// * `vulkan_instance`: A reference to the initialized `VulkanInstance`.
/// * `surface`: A `vk::SurfaceKHR` handle, used to check for presentation support on the device's queues.
///
/// # Returns
///
/// A `Result` containing `PhysicalDeviceInfo` for the selected device on success.
/// On failure, returns a `VulkanError`, which could be:
/// - `VulkanError::NoSuitablePhysicalDevice`: If no device meets all criteria or no devices are found.
/// - `VulkanError::VkResult`: For Vulkan API errors during enumeration or querying.
/// - `VulkanError::ResourceCreationError`: If enumeration of device extensions fails.
pub fn select_physical_device(
    vulkan_instance: &VulkanInstance,
    surface: vk::SurfaceKHR,
) -> Result<PhysicalDeviceInfo> {
    let instance = vulkan_instance.raw();
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?; // Uses From<vk::Result>

    if physical_devices.is_empty() {
        error!("No Vulkan-compatible physical devices found.");
        return Err(VulkanError::NoSuitablePhysicalDevice);
    }

    info!("Found {} physical device(s). Evaluating suitability...", physical_devices.len());

    let mut last_suitable_device: Option<(vk::PhysicalDevice, QueueFamilyIndices, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, vk::PhysicalDeviceMemoryProperties)> = None;

    for &physical_device in physical_devices.iter() {
        match is_device_suitable(instance, vulkan_instance.entry(), physical_device, surface) {
            Ok(Some((indices, properties, features, memory_properties))) => {
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_str().unwrap_or("Unknown");
                info!("Device suitable: {} (Type: {:?})", device_name, properties.device_type);
                
                if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                    info!("Selected Integrated GPU: {}", device_name);
                    return Ok(PhysicalDeviceInfo {
                        physical_device, properties, features,
                        queue_family_indices: indices, memory_properties,
                    });
                } else if last_suitable_device.is_none() ||
                          (last_suitable_device.as_ref().unwrap().2.device_type != vk::PhysicalDeviceType::INTEGRATED_GPU &&
                           properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU) {
                    last_suitable_device = Some((physical_device, indices, properties, features, memory_properties));
                }
            }
            Ok(None) => { // Device not suitable, but no error during check
                let properties = unsafe { instance.get_physical_device_properties(physical_device) };
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_str().unwrap_or("Unknown");
                debug!("Device not suitable: {}", device_name);
            }
            Err(e) => { // Error occurred during suitability check
                 let properties = unsafe { instance.get_physical_device_properties(physical_device) };
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_str().unwrap_or("Unknown") };
                warn!("Error checking suitability for device {}: {}", device_name, e);
            }
        }
    }

    if let Some((pd, qfi, props, feats, mem_props)) = last_suitable_device {
        let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()).to_str().unwrap_or("Unknown") };
        if props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
             info!("Final selected device (Integrated GPU): {}", device_name);
        } else {
             info!("Final selected device (Type: {:?}): {}", props.device_type, device_name);
        }
        return Ok(PhysicalDeviceInfo {
            physical_device: pd, properties: props, features: feats,
            queue_family_indices: qfi, memory_properties: mem_props,
        });
    }
    error!("No suitable physical device found meeting all criteria.");
    Err(VulkanError::NoSuitablePhysicalDevice)
}

/// Checks if a given physical device is suitable for the application. (Private helper)
///
/// This function performs several checks:
/// 1.  Vulkan API version support (must be >= 1.3).
/// 2.  Required device features (e.g., samplerAnisotropy).
/// 3.  Required device extensions (e.g., swapchain, external memory).
/// 4.  Availability of necessary queue families (graphics and present).
///
/// # Arguments
/// * `instance`: Reference to the `ash::Instance`.
/// * `entry`: Reference to `ash::Entry` for loading surface-related functions.
/// * `physical_device`: The `vk::PhysicalDevice` to evaluate.
/// * `surface`: The `vk::SurfaceKHR` for checking presentation support.
///
/// # Returns
/// A `Result` containing:
/// - `Ok(Some((indices, properties, features, memory_properties)))` if the device is suitable.
/// - `Ok(None)` if the device is not suitable but no error occurred during checks.
/// - `Err(VulkanError)` if an error occurred during the checks (e.g., failed to enumerate extensions).
fn is_device_suitable(
    instance: &ash::Instance,
    entry: &ash::Entry, 
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<Option<(QueueFamilyIndices, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures, vk::PhysicalDeviceMemoryProperties)>> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let features = unsafe { instance.get_physical_device_features(physical_device) };
    let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let device_name_cstr = unsafe {CStr::from_ptr(properties.device_name.as_ptr())};
    let device_name_str = device_name_cstr.to_str().unwrap_or("Unknown Device");

    if properties.api_version < vk::API_VERSION_1_3 {
        debug!("Device {} does not support Vulkan 1.3 (supports {:?})", device_name_str, properties.api_version);
        return Ok(None);
    }

    if features.sampler_anisotropy == vk::FALSE {
        debug!("Device {} does not support samplerAnisotropy.", device_name_str);
        return Ok(None);
    }

    let required_extensions_cstrs = [
        Swapchain::name(), 
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_external_memory_dma_buf\0") },
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_KHR_external_memory_fd\0") },
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_image_drm_format_modifier\0") },
    ];

    let available_extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }
        .map_err(|e| VulkanError::ResourceCreationError{
            resource_type: "DeviceExtensions".to_string(), 
            message: format!("Failed to enumerate device extensions for {}: {}", device_name_str, e)
        })?;

    let mut available_extension_names = HashSet::new();
    for ext in available_extensions {
        let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
        available_extension_names.insert(name.to_owned());
    }

    for required_ext_name_cstr in &required_extensions_cstrs {
        if !available_extension_names.contains(required_ext_name_cstr) {
            let missing_ext_name = required_ext_name_cstr.to_str().unwrap_or("Unknown Extension");
            debug!("Device {} is missing required extension: {}", device_name_str, missing_ext_name);
            return Ok(None); 
        }
    }
    debug!("Device {} has all required extensions.", device_name_str);

    let surface_loader = SurfaceLoader::new(entry, instance);
    let queue_family_indices = find_queue_families(instance, physical_device, surface, &surface_loader, device_name_str)?;

    if !queue_family_indices.is_complete() {
        debug!("Device {} does not have all required queue families (graphics and present). Graphics: {:?}, Present: {:?}",
            device_name_str, queue_family_indices.graphics_family, queue_family_indices.present_family);
        return Ok(None);
    }
    debug!("Device {} has suitable queue families: {:?}", device_name_str, queue_family_indices);

    Ok(Some((queue_family_indices, properties, features, memory_properties)))
}

/// Finds necessary queue families for a given physical device. (Private helper)
///
/// Iterates through the queue families of the device and identifies indices for
/// graphics, present, compute (optional), and transfer (optional) operations.
///
/// # Arguments
/// * `instance`: Reference to the `ash::Instance`.
/// * `physical_device`: The `vk::PhysicalDevice` to query.
/// * `surface`: The `vk::SurfaceKHR` for checking presentation support.
/// * `surface_loader`: An instance of `ash::extensions::khr::Surface` for surface operations.
/// * `device_name_str`: The name of the device, for logging context.
///
/// # Returns
/// A `Result` containing `QueueFamilyIndices` on success.
/// On failure, returns a `VulkanError`, typically:
/// - `VulkanError::QueueFamilyNotFound`: If graphics or present queues are not found.
/// - `VulkanError::VkResult`: For Vulkan API errors during querying surface support.
fn find_queue_families(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    surface_loader: &SurfaceLoader,
    device_name_str: &str,
) -> Result<QueueFamilyIndices> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut indices = QueueFamilyIndices::default();
    let mut dedicated_transfer_family: Option<u32> = None;
    let mut combined_transfer_family: Option<u32> = None;

    for (i, queue_family) in queue_family_properties.iter().enumerate() {
        let current_index = i as u32;
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) { indices.graphics_family = Some(current_index); }
        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) { indices.compute_family = Some(current_index); }
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            if !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && !queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                dedicated_transfer_family = Some(current_index);
            }
            combined_transfer_family = Some(current_index);
        }
        let present_support = unsafe { surface_loader.get_physical_device_surface_support(physical_device, current_index, surface) }?;
        if present_support {
            indices.present_family = Some(current_index);
            debug!("Device {}: Queue family {} supports presentation.", device_name_str, current_index);
        }
    }

    indices.transfer_family = dedicated_transfer_family.or(combined_transfer_family);
    if indices.compute_family.is_none() { indices.compute_family = indices.graphics_family; }
    if indices.transfer_family.is_none() {
        if indices.compute_family.is_some() && queue_family_properties[indices.compute_family.unwrap() as usize].queue_flags.contains(vk::QueueFlags::TRANSFER) {
            indices.transfer_family = indices.compute_family;
        } else if indices.graphics_family.is_some() && queue_family_properties[indices.graphics_family.unwrap() as usize].queue_flags.contains(vk::QueueFlags::TRANSFER) {
            indices.transfer_family = indices.graphics_family;
        }
    }

    if indices.graphics_family.is_none() {
        error!("Device {}: Could not find a queue family with GRAPHICS support.", device_name_str);
        return Err(VulkanError::QueueFamilyNotFound("Graphics".to_string()));
    }
    if indices.present_family.is_none() {
        error!("Device {}: Could not find a queue family with PRESENT support for the given surface.", device_name_str);
         return Err(VulkanError::QueueFamilyNotFound("Present".to_string()));
    }
    if indices.compute_family.is_none() { warn!("Device {}: Could not find a dedicated queue family with COMPUTE support. Will try to use graphics queue if needed.", device_name_str); }
    if indices.transfer_family.is_none() { warn!("Device {}: Could not find a dedicated or suitable fallback queue family with TRANSFER support.", device_name_str); }
    Ok(indices)
}

/// Finds a supported format from a list of candidates for a given physical device.
///
/// This utility function iterates through a provided list of `vk::Format` candidates
/// and checks if the physical device supports any of them with the specified tiling
/// and features.
///
/// # Arguments
/// * `instance`: Handle to the Vulkan instance, used to query format properties.
/// * `physical_device`: Handle to the physical device to query.
/// * `candidates`: A slice of `vk::Format` candidates to check (e.g., for depth formats).
/// * `tiling`: The desired image tiling (`vk::ImageTiling::LINEAR` or `vk::ImageTiling::OPTIMAL`).
/// * `features`: The required `vk::FormatFeatureFlags` that the format must support (e.g., `DEPTH_STENCIL_ATTACHMENT`).
///
/// # Returns
/// `Option<vk::Format>` containing the first supported format found from the candidates,
/// or `None` if no candidate format meets the criteria.
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
                if props.linear_tiling_features.contains(features) { return Some(format); }
            }
            vk::ImageTiling::OPTIMAL => {
                if props.optimal_tiling_features.contains(features) { return Some(format); }
            }
            _ => { 
                warn!("Unsupported tiling mode for find_supported_format: {:?}", tiling);
                return None; // Or handle as an error if this case should be impossible.
            }
        }
    }
    None
}
