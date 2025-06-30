// novade-system/src/renderers/vulkan/device.rs
use ash::extensions::khr::Swapchain;
use ash::vk;
use std::collections::HashSet;
use std::ffi::CStr;

/// Represents a set of Vulkan features that the application might require.
///
/// This struct is populated based on checks during physical device suitability assessment.
/// It helps in enabling only necessary and supported features when creating the logical device.
#[derive(Default, Clone, Debug)]
pub struct RequiredFeatures {
    /// Indicates if sampler anisotropy is supported and should be enabled. (Spec 3.4)
    pub sampler_anisotropy: bool,
    /// Indicates if 64-bit integers in shaders are supported and should be enabled. (Spec 3.2)
    pub shader_int64: bool,
    // Add other features as needed by the application and specified in `Rendering Vulkan.md`
    // For example: robust_buffer_access, geometry_shader, tessellation_shader, etc.
}

/// Holds the indices of queue families required by the application.
///
/// Each field is an `Option<u32>` because a single queue family might support multiple
/// operation types (e.g., graphics, compute, and transfer).
/// This structure is populated by `find_queue_families`.
#[derive(Debug, Clone)]
pub struct QueueFamilyIndices {
    /// Index of the queue family that supports graphics operations.
    pub graphics_family: Option<u32>,
    /// Index of the queue family that supports presentation to a surface.
    pub present_family: Option<u32>,
    /// Index of the queue family that supports compute operations.
    pub compute_family: Option<u32>,
    /// Index of the queue family that supports transfer operations.
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Checks if essential queue families for graphics and presentation have been found.
    pub fn is_complete_for_graphics_present(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
    // Add other completeness checks if necessary, e.g., for compute-only contexts.
}

/// Contains information about the chosen physical device.
///
/// This includes the `vk::PhysicalDevice` handle, its properties, the queue family indices identified,
/// and a summary of supported features relevant to the application.
#[derive(Clone)]
pub struct ChosenPhysicalDevice {
    /// The Vulkan physical device handle.
    pub physical_device: vk::PhysicalDevice,
    /// Features confirmed to be supported by this `physical_device` that are relevant to the application.
    pub supported_features: RequiredFeatures,
    /// Properties of the `physical_device` (e.g., name, type, limits, API version).
    pub properties: vk::PhysicalDeviceProperties,
    /// Queue family indices found on this `physical_device`.
    pub queue_family_indices: QueueFamilyIndices,
}

/// Selects the most suitable physical device (GPU) that meets the application's requirements.
///
/// This function enumerates available Vulkan-capable physical devices and evaluates them
/// based on criteria defined in `Rendering Vulkan.md` (Spec 3.2), such as API version,
/// device type (prioritizing integrated GPU for Vega 8), supported features,
/// required extensions, and queue family capabilities.
///
/// # Arguments
/// * `instance`: A reference to the `ash::Instance`.
/// * `surface_loader`: A reference to the `ash::extensions::khr::Surface` loader.
/// * `surface`: A `vk::SurfaceKHR` handle to check for presentation support.
///
/// # Returns
/// A `Result` containing a `ChosenPhysicalDevice` struct with details about the selected device,
/// or an error string if no suitable device is found.
pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<ChosenPhysicalDevice, String> {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .map_err(|e| format!("Failed to enumerate physical devices: {}", e))?
    };

    if physical_devices.is_empty() {
        return Err("No GPUs with Vulkan support found!".to_string());
    }

    let mut best_candidate: Option<ChosenPhysicalDevice> = None;
    let mut best_score = -1i32;

    for &p_device in physical_devices.iter() {
        let properties = unsafe { instance.get_physical_device_properties(p_device) };
        let all_features = unsafe { instance.get_physical_device_features(p_device) };

        // Check for Vulkan 1.3 API version support (Spec 3.2)
        if properties.api_version < vk::API_VERSION_1_3 {
            // Using eprintln for important diagnostic messages
            eprintln!("Device {} does not support Vulkan 1.3 (API version: {}.{}.{}). Skipping.",
                unsafe {CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy()},
                vk::api_version_major(properties.api_version),
                vk::api_version_minor(properties.api_version),
                vk::api_version_patch(properties.api_version)
            );
            continue;
        }

        // Determine which of the application's required features are supported by this device
        let current_supported_features = RequiredFeatures {
            sampler_anisotropy: all_features.sampler_anisotropy == vk::TRUE,
            shader_int64: all_features.shader_int64 == vk::TRUE,
            // Populate other features based on `all_features`
        };

        // Basic suitability checks (e.g., queue families, extensions)
        let queue_indices = find_queue_families(instance, p_device, surface_loader, surface, &properties);

        // For graphics applications, graphics and present queues are essential.
        if !queue_indices.is_complete_for_graphics_present() {
             eprintln!("Device {} does not have complete graphics/present queue families. Skipping.", unsafe {CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy()});
            continue;
        }

        let required_extensions = get_required_device_extensions();
        if !check_device_extension_support(instance, p_device, &required_extensions) {
            eprintln!("Device {} does not support required extensions. Skipping.", unsafe {CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy()});
            continue;
        }

        // Score the device
        let score = rate_device_suitability(&properties, &current_supported_features, &queue_indices);

        if score > best_score {
            best_score = score;
            best_candidate = Some(ChosenPhysicalDevice {
                physical_device: p_device,
                supported_features: current_supported_features,
                properties,
                queue_family_indices: queue_indices,
            });
        }
    }

    if best_score < 0 || best_candidate.is_none() { // best_score can be 0 for a minimally suitable device
        return Err("Failed to find a suitable GPU that meets all criteria!".to_string());
    }
    Ok(best_candidate.unwrap())
}

/// Rates the suitability of a physical device based on its properties, features, and queue families.
///
/// This is a helper for `pick_physical_device`.
/// Adheres to `Rendering Vulkan.md` (Spec 3.2) for scoring criteria:
/// - Prioritizes integrated GPUs (for target AMD Vega 8).
/// - Scores based on essential supported features (e.g., sampler anisotropy).
/// - Potentially penalizes or disqualifies if critical features are missing.
/// - Adds score for meeting the target API version (Vulkan 1.3).
fn rate_device_suitability(
    properties: &vk::PhysicalDeviceProperties,
    supported_features: &RequiredFeatures,
    _indices: &QueueFamilyIndices, // May be used for scoring later if specific queue configs are preferred
) -> i32 {
    let mut score = 0;

    // Prioritize Integrated GPU as per Spec 3.2 for AMD Vega 8
    if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
        score += 1000;
    } else if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 500; // Discrete GPUs are generally powerful
    }

    // Score based on essential features (Spec 3.4)
    // Sampler anisotropy is mentioned as a feature to check in Spec 3.2 and enable in Spec 3.4
    if supported_features.sampler_anisotropy {
        score += 100;
    } else {
        // If sampler anisotropy is strictly required by the application,
        // this device might be unsuitable.
        // For this example, we'll just not add score, but one could return -1.
        eprintln!("Warning: Device {} does not support samplerAnisotropy.", unsafe {CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy()});
        // return -1; // Uncomment if this feature is absolutely mandatory
    }

    // ShaderInt64 is mentioned as a feature to check in Spec 3.2
    if supported_features.shader_int64 {
        score += 50;
    }

    // Add more scoring based on limits (Spec 3.2), e.g., maxImageDimension2D, maxMemoryAllocationCount.
    // Example: score += properties.limits.max_image_dimension2d / 1000;
    // These would require defining what are "good" values for the application.

    // API version check is primarily handled in pick_physical_device.
    // Here, we confirm it's at least the targeted 1.3.
    if properties.api_version >= vk::API_VERSION_1_3 {
        score += 200;
    } else {
        // This case should ideally not be reached if pick_physical_device filters correctly.
        return -1; // Device must support at least Vulkan 1.3
    }

    score
}

/// Identifies queue families for graphics, presentation, compute, and transfer operations.
///
/// Iterates through the physical device's queue families and selects indices for each required
/// capability. It attempts to find dedicated queues where possible (especially for transfer)
/// and falls back to using shared queues (e.g., graphics queue for compute/transfer) if necessary.
/// Adheres to `Rendering Vulkan.md` (Spec 3.3).
///
/// # Arguments
/// * `instance`: Reference to the `ash::Instance`.
/// * `device`: The `vk::PhysicalDevice` to query.
/// * `surface_loader`: Reference to the `ash::extensions::khr::Surface` loader.
/// * `surface`: The `vk::SurfaceKHR` for checking presentation support.
/// * `_properties`: Physical device properties (currently unused but passed for future use).
///
/// # Returns
/// A `QueueFamilyIndices` struct populated with the found indices.
pub fn find_queue_families(
    instance: &ash::Instance,
    device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    _properties: &vk::PhysicalDeviceProperties, // Could be used for smarter selection
) -> QueueFamilyIndices {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
    let mut indices = QueueFamilyIndices {
        graphics_family: None,
        present_family: None,
        compute_family: None,
        transfer_family: None,
    };

    // Try to find a dedicated transfer queue first (Spec 3.3 suggests it might be useful)
    // A dedicated transfer queue is one that supports TRANSFER but not GRAPHICS or COMPUTE.
    for (i, queue_family) in queue_family_properties.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) &&
           !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
           !queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            indices.transfer_family = Some(i as u32);
            break;
        }
    }

    for (i, queue_family) in queue_family_properties.iter().enumerate() {
        let i_u32 = i as u32;

        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            if indices.graphics_family.is_none() { // Take the first one found
                indices.graphics_family = Some(i_u32);
            }
        }

        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            if indices.compute_family.is_none() {
                 // Prefer a compute queue that is not the graphics queue if possible
                if indices.graphics_family != Some(i_u32) {
                    indices.compute_family = Some(i_u32);
                }
            }
        }

        // If a dedicated transfer queue wasn't found, use one that supports transfer,
        // preferring one that is not graphics or compute if possible.
        if indices.transfer_family.is_none() && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
             if indices.graphics_family != Some(i_u32) && indices.compute_family != Some(i_u32) { // Prefer non-graphics/compute for transfer
                indices.transfer_family = Some(i_u32);
            }
        }

        // Check for presentation support for the current queue family index `i`
        if surface != vk::SurfaceKHR::null() { // Only check if a surface is provided
            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(device, i_u32, surface)
                    .unwrap_or(false) // Assume no support on error
            };
            if present_support && indices.present_family.is_none() { // Take the first one found
                indices.present_family = Some(i_u32);
            }
        }
    }

    // Fallbacks if dedicated queues are not found (Spec 3.3 suggests preferring combined queues if possible)
    if indices.compute_family.is_none() {
        indices.compute_family = indices.graphics_family; // Often compute can run on graphics queue
    }
    if indices.transfer_family.is_none() { // If still no transfer queue (dedicated or semi-dedicated)
        indices.transfer_family = indices.graphics_family; // Transfer can also run on graphics/compute queue
    }

    // Final check for present_family: if it's still None but we have a surface and a graphics_family,
    // check if the graphics_family can present. This is a common scenario.
    if surface != vk::SurfaceKHR::null() && indices.present_family.is_none() && indices.graphics_family.is_some() {
        let present_support_on_graphics = unsafe {
            surface_loader.get_physical_device_surface_support(device, indices.graphics_family.unwrap(), surface)
                .unwrap_or(false)
        };
        if present_support_on_graphics {
            indices.present_family = indices.graphics_family;
        }
    }

    indices
}

/// Gets a list of required device extensions based on `Rendering Vulkan.md` (Spec 3.2 & 3.4).
fn get_required_device_extensions() -> Vec<&'static CStr> {
    // As per Spec 3.2 & 3.4
    vec![
        Swapchain::name(), // VK_KHR_swapchain
        ash::extensions::ext::ExternalMemoryDmaBuf::name(), // VK_EXT_external_memory_dmabuf
        ash::extensions::khr::ExternalMemoryFd::name(),     // VK_KHR_external_memory_fd
        // ash::extensions::khr::ExternalMemory::name(), // This is implicitly supported by Vulkan 1.1+ if the others are.
        // ash::extensions::khr::GetPhysicalDeviceProperties2::name(), // Implicitly supported with Vulkan 1.1+
        // VK_EXT_image_drm_format_modifier is optional per spec (section 3.2), can be added if strictly needed by DMA-BUF import logic.
    ]
}

/// Checks if a physical device supports all required device-level extensions.
fn check_device_extension_support(instance: &ash::Instance, device: vk::PhysicalDevice, required_extensions: &[&'static CStr]) -> bool {
    let available_extensions = match unsafe { instance.enumerate_device_extension_properties(device) } {
        Ok(props) => props,
        Err(e) => {
            eprintln!("Failed to enumerate device extensions for {:?}: {}", unsafe { CStr::from_ptr(instance.get_physical_device_properties(device).device_name.as_ptr()) }, e);
            return false;
        }
    };

    let mut available_extension_names = HashSet::new();
    for ext_prop in available_extensions {
        let name = unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) };
        available_extension_names.insert(name);
    }

    for required_ext_name in required_extensions {
        if !available_extension_names.contains(required_ext_name) {
            eprintln!("Required device extension not found on device {:?}: {:?}", unsafe { CStr::from_ptr(instance.get_physical_device_properties(device).device_name.as_ptr()) }, required_ext_name);
            return false;
        }
    }
    true
}

/// Holds the handles to various device queues.
///
/// Queues for compute and transfer are optional as they might share the graphics queue.
pub struct LogicalDeviceQueues {
    /// Queue for graphics commands.
    pub graphics_queue: vk::Queue,
    /// Queue for presentation commands.
    pub present_queue: vk::Queue,
    /// Optional queue for compute commands.
    pub compute_queue: Option<vk::Queue>,
    /// Optional queue for transfer commands.
    pub transfer_queue: Option<vk::Queue>,
}

/// Creates a logical device (VkDevice) from a chosen physical device.
///
/// This function configures and creates the logical device interface to the GPU.
/// It enables specified device features (like sampler anisotropy) if supported,
/// required device extensions (like swapchain, external memory for DMA-BUF),
/// and Vulkan 1.3 features (dynamic rendering, synchronization2).
/// It also retrieves handles to the graphics, present, and optionally compute/transfer queues.
/// Adheres to `Rendering Vulkan.md` (Spec 3.4).
///
/// # Arguments
/// * `instance`: Reference to the `ash::Instance`.
/// * `chosen_device`: A reference to `ChosenPhysicalDevice` containing the selected physical device
///   and its relevant properties, features, and queue indices.
///
/// # Returns
/// A `Result` containing a tuple of the created `ash::Device` and `LogicalDeviceQueues`,
/// or an error string on failure.
pub fn create_logical_device(
    instance: &ash::Instance,
    chosen_device: &ChosenPhysicalDevice,
) -> Result<(ash::Device, LogicalDeviceQueues), String> {
    let indices = &chosen_device.queue_family_indices;
    let mut unique_queue_families = HashSet::new();

    let graphics_q_idx = indices.graphics_family.ok_or_else(|| "Graphics family index is None".to_string())?;
    unique_queue_families.insert(graphics_q_idx);

    // Presentation queue is essential for graphical applications using a surface.
    let present_q_idx = indices.present_family.ok_or_else(|| "Present family index is None but required for logical device creation with a surface".to_string())?;
    unique_queue_families.insert(present_q_idx);

    // Add compute and transfer queue indices if they are distinct and available
    if let Some(compute_idx) = indices.compute_family {
        unique_queue_families.insert(compute_idx);
    }
    if let Some(transfer_idx) = indices.transfer_family {
        unique_queue_families.insert(transfer_idx);
    }

    let queue_priorities = [1.0f32]; // Default priority
    let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
        .iter()
        .map(|&queue_family_index| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&queue_priorities)
                .build()
        })
        .collect();

    // Enable features that were confirmed supported and are required by the application (Spec 3.4)
    let mut features_to_enable_builder = vk::PhysicalDeviceFeatures::builder();
    if chosen_device.supported_features.sampler_anisotropy {
        features_to_enable_builder = features_to_enable_builder.sampler_anisotropy(true);
    }
    if chosen_device.supported_features.shader_int64 {
        features_to_enable_builder = features_to_enable_builder.shader_int64(true);
    }
    // Enable other features from chosen_device.supported_features as needed.
    let features_to_enable = features_to_enable_builder.build();


    let required_extensions_raw: Vec<*const i8> = get_required_device_extensions()
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    // Vulkan 1.3 features (dynamic_rendering, synchronization2) (Spec 3.4)
    // These should be supported if apiVersion >= 1.3 (which is checked in pick_physical_device)
    let mut features13_builder = vk::PhysicalDeviceVulkan13Features::builder();

    // Query for actual Vulkan 1.3 feature support for robustness, though device.properties.api_version >= 1.3 implies support.
    let mut supported_features13_query = vk::PhysicalDeviceVulkan13Features::default();
    let mut features2_query = vk::PhysicalDeviceFeatures2::builder().push_next(&mut supported_features13_query);
    unsafe { instance.get_physical_device_features2(chosen_device.physical_device, &mut features2_query) };

    if supported_features13_query.dynamic_rendering == vk::TRUE {
        features13_builder = features13_builder.dynamic_rendering(true);
    }
    if supported_features13_query.synchronization2 == vk::TRUE {
         features13_builder = features13_builder.synchronization2(true);
    }
    let mut features13 = features13_builder.build(); // Build here to pass a mutable reference later if needed


    let mut device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&features_to_enable)
        .enabled_extension_names(&required_extensions_raw);

    // Only add VkPhysicalDeviceVulkan13Features to pNext if any of its members are true.
    if features13.dynamic_rendering == vk::TRUE || features13.synchronization2 == vk::TRUE {
        device_create_info = device_create_info.push_next(&mut features13);
    }

    let device: ash::Device = unsafe {
        instance
            .create_device(chosen_device.physical_device, &device_create_info, None)
            .map_err(|e| format!("Failed to create logical device: {}", e))?
    };

    // Retrieve queue handles
    let graphics_queue = unsafe { device.get_device_queue(graphics_q_idx, 0) };
    let present_queue = unsafe { device.get_device_queue(present_q_idx, 0) }; // present_q_idx is guaranteed Some by earlier check

    let compute_queue = indices.compute_family.map(|idx| unsafe { device.get_device_queue(idx, 0) });
    let transfer_queue = indices.transfer_family.map(|idx| unsafe { device.get_device_queue(idx, 0) });

    Ok((device, LogicalDeviceQueues { graphics_queue, present_queue, compute_queue, transfer_queue } ))
}
