use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::physical_device::{PhysicalDeviceInfo, QueueFamilyIndices};
use ash::extensions::khr::{ExternalMemoryFd, Swapchain};
use ash::vk;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

/// Holds the handles to the Vulkan queues retrieved from the logical device.
#[derive(Debug, Clone)]
pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub compute_queue: Option<vk::Queue>, // Optional, as it might share with graphics
    pub transfer_queue: Option<vk::Queue>, // Optional, as it might be dedicated or shared
}

/// Represents the logical Vulkan device and its associated queues.
#[derive(Debug)]
pub struct LogicalDevice {
    pub raw: ash::Device,
    pub queues: Queues,
}

impl LogicalDevice {
    /// Destroys the logical device.
    pub fn destroy(&self) {
        unsafe {
            self.raw.destroy_device(None);
            info!("Logical device destroyed.");
        }
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Creates a Vulkan logical device (`VkDevice`) and retrieves queue handles.
///
/// # Arguments
/// * `vulkan_instance`: A reference to the `VulkanInstance`.
/// * `physical_device_info`: Information about the selected physical device,
///   including queue family indices and supported features.
///
/// # Returns
/// `Result<LogicalDevice, String>` containing the created logical device and its queues,
/// or an error message.
pub fn create_logical_device(
    vulkan_instance: &VulkanInstance,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<LogicalDevice, String> {
    info!("Creating logical device...");

    let queue_family_indices = &physical_device_info.queue_family_indices;
    let mut unique_queue_family_indices = HashSet::new();
    if let Some(index) = queue_family_indices.graphics_family {
        unique_queue_family_indices.insert(index);
    }
    if let Some(index) = queue_family_indices.present_family {
        unique_queue_family_indices.insert(index);
    }
    if let Some(index) = queue_family_indices.compute_family {
        unique_queue_family_indices.insert(index);
    }
    if let Some(index) = queue_family_indices.transfer_family {
        unique_queue_family_indices.insert(index);
    }

    let queue_priorities = [1.0f32];
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

    for &queue_family_index in unique_queue_family_indices.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)
            .build();
        queue_create_infos.push(queue_create_info);
        debug!("Adding DeviceQueueCreateInfo for queue family index: {}", queue_family_index);
    }

    // Configure VkPhysicalDeviceFeatures
    // Only enable features that are both needed AND supported.
    let mut enabled_features = vk::PhysicalDeviceFeatures::default();
    if physical_device_info.features.sampler_anisotropy == vk::TRUE {
        enabled_features.sampler_anisotropy = vk::TRUE;
        debug!("Enabling feature: samplerAnisotropy");
    }
    if physical_device_info.features.fill_mode_non_solid == vk::TRUE {
        enabled_features.fill_mode_non_solid = vk::TRUE;
         debug!("Enabling feature: fillModeNonSolid");
    }
    if physical_device_info.features.wide_lines == vk::TRUE {
        enabled_features.wide_lines = vk::TRUE;
        debug!("Enabling feature: wideLines");
    }
    // Add more features to enable as needed by the application, always checking physical_device_info.features first.


    // Define required device extensions
    let device_extension_names_cstr: Vec<&CStr> = vec![
        Swapchain::name(),
        // VK_EXT_EXTERNAL_MEMORY_DMA_BUF_EXTENSION_NAME
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_external_memory_dma_buf\0") },
        ExternalMemoryFd::name(), // VK_KHR_EXTERNAL_MEMORY_FD_EXTENSION_NAME
        // VK_EXT_IMAGE_DRM_FORMAT_MODIFIER_EXTENSION_NAME
        unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_EXT_image_drm_format_modifier\0") },
    ];

    let device_extension_names_ptr: Vec<*const c_char> = device_extension_names_cstr
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    info!("Required device extensions:");
    for ext_name in &device_extension_names_cstr {
        info!("  - {:?}", ext_name);
    }


    // Configure VkDeviceCreateInfo
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&enabled_features)
        .enabled_extension_names(&device_extension_names_ptr);
    // Validation layers are typically set at instance level.
    // If specific device layers were needed (rare), they would be added here.

    let device: ash::Device = unsafe {
        vulkan_instance
            .raw()
            .create_device(physical_device_info.physical_device, &device_create_info, None)
    }
    .map_err(|e| format!("Failed to create logical device: {}", e))?;
    info!("Logical device created successfully.");

    // Retrieve queue handles
    let graphics_queue = unsafe {
        device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0)
    };
    debug!("Retrieved graphics queue handle.");

    let present_queue = unsafe {
        device.get_device_queue(queue_family_indices.present_family.unwrap(), 0)
    };
    debug!("Retrieved present queue handle.");

    let mut compute_queue: Option<vk::Queue> = None;
    if let Some(index) = queue_family_indices.compute_family {
        compute_queue = Some(unsafe { device.get_device_queue(index, 0) });
        debug!("Retrieved compute queue handle (family index {}).", index);
    } else {
        warn!("No dedicated compute queue family index found. Compute operations might share graphics queue.");
    }

    let mut transfer_queue: Option<vk::Queue> = None;
    if let Some(index) = queue_family_indices.transfer_family {
        transfer_queue = Some(unsafe { device.get_device_queue(index, 0) });
         debug!("Retrieved transfer queue handle (family index {}).", index);
    } else {
        warn!("No dedicated or fallback transfer queue family index found.");
    }
    
    // Special handling if compute or transfer queues are intended to be the same as graphics
    // and were not explicitly found as separate families but were aliased in physical_device.rs
    if queue_family_indices.compute_family.is_some() && 
       queue_family_indices.compute_family == queue_family_indices.graphics_family && 
       compute_queue.is_none() {
        compute_queue = Some(graphics_queue);
        debug!("Compute queue aliased to graphics queue.");
    }

    if queue_family_indices.transfer_family.is_some() &&
       (queue_family_indices.transfer_family == queue_family_indices.graphics_family || 
        queue_family_indices.transfer_family == queue_family_indices.compute_family) &&
       transfer_queue.is_none() {
        // Prefer aliasing to compute if compute is distinct from graphics, else graphics
        if queue_family_indices.compute_family.is_some() && 
           queue_family_indices.compute_family != queue_family_indices.graphics_family &&
           queue_family_indices.transfer_family == queue_family_indices.compute_family {
            transfer_queue = compute_queue;
            debug!("Transfer queue aliased to compute queue.");
        } else {
            transfer_queue = Some(graphics_queue);
            debug!("Transfer queue aliased to graphics queue.");
        }
    }


    let queues = Queues {
        graphics_queue,
        present_queue,
        compute_queue,
        transfer_queue,
    };

    info!("Successfully retrieved all required queue handles.");

    Ok(LogicalDevice { raw: device, queues })
}
