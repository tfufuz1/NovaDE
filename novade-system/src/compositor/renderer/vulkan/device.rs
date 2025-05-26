//! Manages the Vulkan logical device (`VkDevice`) and its associated queues.
//!
//! This module provides functionality to create a logical device from a selected
//! physical device. The logical device is the primary interface for most Vulkan
//! operations, such as creating resources, recording command buffers, and managing queues.
//! The module defines structures to hold the logical device handle and references
//! to its important queues (graphics, present, compute, transfer).

use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::{PhysicalDeviceInfo, QueueFamilyIndices};
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::extensions::khr::{ExternalMemoryFd, Swapchain};
use ash::vk;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

/// Holds the handles to the Vulkan queues retrieved from the logical device.
///
/// These queues are used for submitting various types of commands to the GPU,
/// such as rendering commands (graphics), presentation commands, compute shaders,
/// and data transfers.
#[derive(Debug, Clone)]
pub struct Queues {
    /// Queue for submitting graphics and rendering commands.
    pub graphics_queue: vk::Queue,
    /// Queue for submitting presentation commands (displaying images on a surface).
    /// This might be the same as `graphics_queue`.
    pub present_queue: vk::Queue,
    /// Optional queue for submitting compute shader commands.
    /// If `None`, compute operations might need to be submitted to the graphics queue.
    pub compute_queue: Option<vk::Queue>,
    /// Optional queue specifically for data transfer operations (e.g., buffer copies).
    /// If `None`, transfers might use the graphics or compute queue.
    pub transfer_queue: Option<vk::Queue>,
}

/// Represents the logical Vulkan device and its associated queues.
///
/// The `LogicalDevice` struct encapsulates the `ash::Device` handle, which is
/// the main interface for interacting with the GPU at a logical level. It also
/// stores an instance of `Queues` containing handles to the relevant command queues.
///
/// The struct implements `Drop` to ensure that the `ash::Device` is destroyed
/// when the `LogicalDevice` instance goes out of scope.
#[derive(Debug)]
pub struct LogicalDevice {
    /// The raw `ash::Device` handle, representing the logical device.
    pub raw: ash::Device,
    /// Handles to the various queues (graphics, present, etc.) associated with this logical device.
    pub queues: Queues,
}

impl LogicalDevice {
    /// Destroys the logical device.
    ///
    /// This method should be called to explicitly clean up the Vulkan logical device.
    /// It is also called automatically when the `LogicalDevice` instance is dropped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all resources created with this logical device
    /// (e.g., buffers, images, pipelines, command pools) are destroyed *before*
    /// calling this method or allowing the `LogicalDevice` to be dropped.
    /// The `ash::Device` handle becomes invalid after this call.
    pub fn destroy(&self) { // Changed to &self as drop takes &mut self, and destroy_device takes &Device
        info!("Destroying logical device...");
        unsafe {
            self.raw.destroy_device(None);
        }
        info!("Logical device destroyed successfully.");
    }
}

impl Drop for LogicalDevice {
    /// Ensures that the Vulkan logical device is destroyed when the `LogicalDevice`
    /// instance goes out of scope.
    fn drop(&mut self) {
        info!("Dropping LogicalDevice, destroying associated Vulkan device.");
        // `destroy_device` takes `&Device`, so we don't need `&mut self` for this part,
        // but Drop requires `&mut self`.
        unsafe {
            self.raw.destroy_device(None);
        }
        info!("Vulkan logical device destroyed via Drop.");
    }
}

/// Creates a Vulkan logical device (`VkDevice`) and retrieves its queue handles.
///
/// This function configures and creates a logical device based on the provided
/// `PhysicalDeviceInfo`. It specifies:
/// - Which queue families to create queues from (based on unique indices from `physical_device_info`).
/// - Which device features to enable (e.g., samplerAnisotropy), ensuring they are supported by the physical device.
/// - Required device extensions (e.g., swapchain, external memory).
///
/// After creating the `ash::Device`, it retrieves `vk::Queue` handles for the
/// graphics, present, and optionally compute/transfer queues.
///
/// # Arguments
///
/// * `vulkan_instance`: A reference to the `VulkanInstance` (used to get the `ash::Instance` for device creation).
/// * `physical_device_info`: A reference to `PhysicalDeviceInfo` containing details about the selected
///   physical device, including its handle, queue family indices, and supported features.
///
/// # Returns
///
/// A `Result` containing the `LogicalDevice` (which includes the `ash::Device` and `Queues`) on success.
/// On failure, returns a `VulkanError`, typically `VulkanError::VkResult` if `vkCreateDevice` fails.
pub fn create_logical_device(
    vulkan_instance: &VulkanInstance,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<LogicalDevice> {
    let device_name = unsafe { CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
        .to_str().unwrap_or("Unknown Device");
    info!("Creating logical device for physical device: {}", device_name);

    let queue_family_indices = &physical_device_info.queue_family_indices;
    let mut unique_queue_family_indices = HashSet::new();
    if let Some(index) = queue_family_indices.graphics_family { unique_queue_family_indices.insert(index); }
    if let Some(index) = queue_family_indices.present_family { unique_queue_family_indices.insert(index); }
    if let Some(index) = queue_family_indices.compute_family { unique_queue_family_indices.insert(index); }
    if let Some(index) = queue_family_indices.transfer_family { unique_queue_family_indices.insert(index); }

    let queue_priorities = [1.0f32]; // Standard priority for all queues
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

    for &queue_family_index in unique_queue_family_indices.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);
        queue_create_infos.push(queue_create_info.build());
        debug!("Adding DeviceQueueCreateInfo for queue family index: {}", queue_family_index);
    }

    let mut enabled_features = vk::PhysicalDeviceFeatures::default();
    if physical_device_info.features.sampler_anisotropy == vk::TRUE {
        enabled_features.sampler_anisotropy = vk::TRUE;
        debug!("Enabling feature on logical device: samplerAnisotropy");
    }
    if physical_device_info.features.fill_mode_non_solid == vk::TRUE {
        enabled_features.fill_mode_non_solid = vk::TRUE;
         debug!("Enabling feature on logical device: fillModeNonSolid");
    }
    if physical_device_info.features.wide_lines == vk::TRUE {
        enabled_features.wide_lines = vk::TRUE;
        debug!("Enabling feature on logical device: wideLines");
    }
    // Add other features as needed, always checking `physical_device_info.features`.

    let device_extension_names_cstr: Vec<&CStr> = vec![
        Swapchain::name(), // VK_KHR_SWAPCHAIN_EXTENSION_NAME
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_external_memory_dma_buf\0") },
        ExternalMemoryFd::name(), // VK_KHR_EXTERNAL_MEMORY_FD_EXTENSION_NAME
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_image_drm_format_modifier\0") },
    ];
    let device_extension_names_ptr: Vec<*const c_char> = device_extension_names_cstr
        .iter().map(|s| s.as_ptr()).collect();
    info!("Requesting device extensions for {}:", device_name);
    for ext_name in &device_extension_names_cstr { info!("  - {:?}", ext_name); }

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&enabled_features)
        .enabled_extension_names(&device_extension_names_ptr);
    // Validation layers are typically instance-level. Device-specific layers are rare.

    let device_ash: ash::Device = unsafe {
        vulkan_instance.raw().create_device(physical_device_info.physical_device, &device_create_info, None)
    }?; // Uses From<vk::Result>
    info!("Logical device for '{}' created successfully.", device_name);

    let graphics_queue = unsafe { device_ash.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };
    debug!("Retrieved graphics queue handle for {}.", device_name);
    let present_queue = unsafe { device_ash.get_device_queue(queue_family_indices.present_family.unwrap(), 0) };
    debug!("Retrieved present queue handle for {}.", device_name);

    let mut compute_queue: Option<vk::Queue> = None;
    if let Some(index) = queue_family_indices.compute_family {
        compute_queue = Some(unsafe { device_ash.get_device_queue(index, 0) });
        debug!("Retrieved compute queue handle (family index {}) for {}.", index, device_name);
    } else { warn!("No dedicated compute queue family index found for {}. Compute operations might share graphics queue.", device_name); }

    let mut transfer_queue: Option<vk::Queue> = None;
    if let Some(index) = queue_family_indices.transfer_family {
        transfer_queue = Some(unsafe { device_ash.get_device_queue(index, 0) });
         debug!("Retrieved transfer queue handle (family index {}) for {}.", index, device_name);
    } else { warn!("No dedicated or fallback transfer queue family index found for {}.", device_name); }
    
    // Handle aliased queues if they were not explicitly distinct but resolved to same family
    if queue_family_indices.compute_family.is_some() && 
       queue_family_indices.compute_family == queue_family_indices.graphics_family && 
       compute_queue.is_none() { // This condition might be redundant if index was Some
        compute_queue = Some(graphics_queue);
        debug!("Compute queue aliased to graphics queue for {}.", device_name);
    }
    if queue_family_indices.transfer_family.is_some() &&
       (queue_family_indices.transfer_family == queue_family_indices.graphics_family || 
        queue_family_indices.transfer_family == queue_family_indices.compute_family) &&
       transfer_queue.is_none() {
        if queue_family_indices.compute_family.is_some() && 
           queue_family_indices.compute_family != queue_family_indices.graphics_family &&
           queue_family_indices.transfer_family == queue_family_indices.compute_family {
            transfer_queue = compute_queue; // Which might be Some(graphics_queue) or Some(dedicated_compute_queue)
            debug!("Transfer queue aliased to compute queue for {}.", device_name);
        } else {
            transfer_queue = Some(graphics_queue);
            debug!("Transfer queue aliased to graphics queue for {}.", device_name);
        }
    }

    let queues = Queues { graphics_queue, present_queue, compute_queue, transfer_queue };
    info!("Successfully retrieved all required queue handles for {}.", device_name);
    Ok(LogicalDevice { raw: device_ash, queues })
}
