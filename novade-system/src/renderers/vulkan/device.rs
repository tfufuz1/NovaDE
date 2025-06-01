// novade-system/src/renderers/vulkan/device.rs
use ash::extensions::khr::Swapchain;
use ash::vk;
use std::collections::HashSet;
use std::ffi::CStr;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    // TODO: Add compute_family, transfer_family if needed
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub fn pick_physical_device(instance: &ash::Instance, surface_loader: &ash::extensions::khr::Surface, surface: vk::SurfaceKHR) -> Result<vk::PhysicalDevice, String> {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .map_err(|e| format!("Failed to enumerate physical devices: {}", e))?
    };

    if physical_devices.is_empty() {
        return Err("No GPUs with Vulkan support found!".to_string());
    }

    let mut best_device: Option<vk::PhysicalDevice> = None;
    let mut best_score = -1;

    for &p_device in physical_devices.iter() {
        let score = rate_device_suitability(instance, p_device, surface_loader, surface);
        if score > best_score {
            best_score = score;
            best_device = Some(p_device);
        }
    }

    if best_score < 0 { // Changed from if best_score == -1 to allow 0 as a valid score
        return Err("Failed to find a suitable GPU!".to_string());
    }
    Ok(best_device.unwrap())
}

fn rate_device_suitability(instance: &ash::Instance, device: vk::PhysicalDevice, surface_loader: &ash::extensions::khr::Surface, surface: vk::SurfaceKHR) -> i32 {
    let properties = unsafe { instance.get_physical_device_properties(device) };
    let _features = unsafe { instance.get_physical_device_features(device) }; // Check specific features if needed

    // Basic checks: must support graphics and presentation for this phase
    let indices = find_queue_families(instance, device, surface_loader, surface);
    if !indices.is_complete() {
        return -1;
    }

    // Check for required extensions
    let required_extensions = get_required_device_extensions();
    if !check_device_extension_support(instance, device, &required_extensions) {
        return -1;
    }

    // TODO: Check swapchain adequacy (formats, present modes) - will be done in swapchain.rs

    let mut score = 0;
    if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
        score += 1000; // Prioritize iGPU as per docs/Rendering Vulkan.md for AMD Vega 8
    } else if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 500;
    }
    // Add more scoring based on limits, features, Vulkan version etc.
    // e.g. score += properties.limits.max_image_dimension2d;
    if properties.api_version >= vk::API_VERSION_1_3 {
        score += 200;
    } else if properties.api_version >= vk::API_VERSION_1_2 {
        score += 100;
    }


    score
}

pub fn find_queue_families(instance: &ash::Instance, device: vk::PhysicalDevice, surface_loader: &ash::extensions::khr::Surface, surface: vk::SurfaceKHR) -> QueueFamilyIndices {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
    let mut indices = QueueFamilyIndices {
        graphics_family: None,
        present_family: None,
    };

    for (i, queue_family) in queue_family_properties.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_family = Some(i as u32);
        }

        // Check for presentation support if a surface is provided
        // This check might be deferred if surface is not available during initial device picking
        if surface != vk::SurfaceKHR::null() {
            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(device, i as u32, surface)
                    .unwrap_or(false)
            };
            if present_support {
                indices.present_family = Some(i as u32);
            }
        }


        if indices.is_complete() { // This might not be complete if surface is null
            break;
        }
    }
    // If present_family is still None because surface was null, it can be set to graphics_family
    // as a placeholder, but this needs careful handling later when a real surface is available.
    if surface == vk::SurfaceKHR::null() && indices.graphics_family.is_some() && indices.present_family.is_none() {
        // This is a temporary measure for headless/initialization scenarios.
        // Real presentation capability must be verified with a surface.
        // For now, we assume if it has graphics, it might present. This is often true for primary GPUs.
        // indices.present_family = indices.graphics_family;
        // The subtask description in vulkan/mod.rs suggests setting present_family = graphics_family as a temporary fix.
    }

    indices
}

fn get_required_device_extensions() -> Vec<&'static CStr> {
    vec![
        Swapchain::name(),
        ash::extensions::khr::ExternalMemoryFd::name(),
        ash::extensions::ext::ExternalMemoryDmaBuf::name(),
        ash::extensions::khr::ExternalMemory::name(), // Dependency for the above
        ash::extensions::khr::GetPhysicalDeviceProperties2::name(), // Often used with external memory
        // VK_EXT_image_drm_format_modifier is more complex, handled separately if needed.
    ]
}

fn check_device_extension_support(instance: &ash::Instance, device: vk::PhysicalDevice, required_extensions: &[&'static CStr]) -> bool {
    let available_extensions = match unsafe { instance.enumerate_device_extension_properties(device) } {
        Ok(props) => props,
        Err(_) => return false,
    };

    let mut available_extension_names = HashSet::new();
    for ext_prop in available_extensions {
        let name = unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) };
        available_extension_names.insert(name);
    }

    for required_ext_name in required_extensions {
        if !available_extension_names.contains(required_ext_name) {
            println!("Required device extension not found: {:?}", required_ext_name);
            return false;
        }
    }
    true
}

pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
) -> Result<(ash::Device, vk::Queue, vk::Queue), String> {
    let mut unique_queue_families = HashSet::new();
    unique_queue_families.insert(indices.graphics_family.ok_or_else(|| "Graphics family index is None".to_string())?);
    if let Some(present_idx) = indices.present_family { // Only add if present_family is Some
        unique_queue_families.insert(present_idx);
    } else {
        // This case should ideally be handled before calling create_logical_device
        // if presentation is strictly required at this stage.
        // For now, proceeding with graphics only if present is None, which means present_queue will be same as graphics_queue.
        // Or, we can error out if present_family is None but required.
        // The updated VulkanRenderer::new logic in the prompt assumes present_family will be Some (even if it's same as graphics).
        return Err("Present family index is None, cannot create distinct present queue.".to_string());
    }


    let queue_priorities = [1.0f32];
    let mut queue_create_infos = Vec::new();

    for queue_family_index in unique_queue_families {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)
            .build();
        queue_create_infos.push(queue_create_info);
    }

    let mut physical_device_features = vk::PhysicalDeviceFeatures::builder();
    // Enable specific features if needed, e.g. physical_device_features.sampler_anisotropy(true);
    // Check availability first via instance.get_physical_device_features(physical_device)

    let required_extensions_raw: Vec<*const i8> = get_required_device_extensions()
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    let mut features13 = vk::PhysicalDeviceVulkan13Features::builder()
        .dynamic_rendering(true)
        .synchronization2(true);

    // Check actual support for these 1.3 features
    let mut supported_features13_query = vk::PhysicalDeviceVulkan13Features::default();
    let mut features2_query = vk::PhysicalDeviceFeatures2::builder().push_next(&mut supported_features13_query);
    unsafe { instance.get_physical_device_features2(physical_device, &mut features2_query) };

    if !supported_features13_query.dynamic_rendering { features13.dynamic_rendering = false; }
    if !supported_features13_query.synchronization2 { features13.synchronization2 = false; }


    let mut device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&physical_device_features)
        .enabled_extension_names(&required_extensions_raw);

    // Only push_next if at least one 1.3 feature is enabled and supported
    if features13.dynamic_rendering == vk::TRUE || features13.synchronization2 == vk::TRUE {
        device_create_info = device_create_info.push_next(&mut features13);
    }


    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .map_err(|e| format!("Failed to create logical device: {}", e))?
    };

    let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
    // Ensure present_family has a value before unwrapping
    let present_queue_family = indices.present_family.ok_or_else(|| "Present family index is None after device creation".to_string())?;
    let present_queue = unsafe { device.get_device_queue(present_queue_family, 0) };

    Ok((device, graphics_queue, present_queue))
}
