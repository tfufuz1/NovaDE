// novade-system/src/renderer/vulkan/mod.rs
// This is the main module for the Vulkan renderer logic.

pub mod error; // Declares an error.rs file in the same directory (vulkan/error.rs)
pub mod texture;
pub mod frame_renderer;

// Re-export key public types from this module
pub use self::error::{Result, VulkanError}; // Use self::error for the submodule
pub use self::texture::VulkanTexture;
pub use self::frame_renderer::VulkanFrameRenderer;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions, ApplicationInfo};
use vulkano::Version; // Keep this for V1_3 etc.
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags};

/// Holds the indices of queue families found on a physical device.
#[derive(Debug, Default, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Checks if the essential queue families (graphics and present) have been found.
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

// Helper functions are now part of this module.
// They can be pub(crate) if only used by VulkanCoreContext::new within this crate,
// or pub if intended for wider use (though VulkanCoreContext is the main public API).
// For simplicity during refactoring, using `pub` and can be restricted later.

// VulkanCoreContext and its constituents like QueueFamilyIndices are primary exports.
// The individual init functions (create_instance, etc.) are kept pub for now,
// allowing potential direct use or testing, but could be made pub(super) or pub(crate)
// if VulkanCoreContext::new() becomes the sole entry point from outside this module.

/// Creates a Vulkan instance.
pub fn create_instance() -> Result<Arc<Instance>> {
    info!("(Core) Creating Vulkan instance...");

    let app_info = ApplicationInfo {
        application_name: Some("NovaDE".into()),
        application_version: Some(Version { major: 0, minor: 1, patch: 0 }),
        engine_name: Some("NovaDE-Vulkan-Renderer".into()),
        engine_version: Some(Version { major: 0, minor: 1, patch: 0 }),
        api_version: Some(Version::V1_3),
    };

    let required_extensions = InstanceExtensions {
        khr_surface: true,
        khr_wayland_surface: true,
        ..InstanceExtensions::empty()
    };
    info!("(Core) Required instance extensions: {:?}", required_extensions);

    let mut instance_create_info = InstanceCreateInfo {
        application_info: Some(app_info),
        enabled_extensions: required_extensions,
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    {
        debug!("(Core) Debug assertions enabled, attempting to enable validation layers.");
        let desired_layers = vec!["VK_LAYER_KHRONOS_validation"];
        match Instance::layers_list() {
            Ok(available_layers) => {
                let mut enabled_layer_count = 0;
                for layer_name in desired_layers {
                    if available_layers.iter().any(|l| l.name() == layer_name) {
                        instance_create_info.enabled_layers.push(layer_name.to_owned());
                        info!("(Core) Validation layer enabled: {}", layer_name);
                        enabled_layer_count += 1;
                    } else {
                        warn!("(Core) Validation layer not available: {}", layer_name);
                    }
                }
                if enabled_layer_count > 0 {
                    info!("(Core) {} validation layer(s) enabled.", enabled_layer_count);
                } else {
                    warn!("(Core) No desired validation layers were enabled. Check Vulkan SDK installation.");
                }
            }
            Err(e) => {
                warn!("(Core) Failed to query available instance layers: {}. Proceeding without validation layers.", e);
            }
        }
    }

    match Instance::new(instance_create_info) {
        Ok(instance) => {
            info!("(Core) Vulkan instance created successfully. API Version: {}", instance.api_version());
            Ok(instance)
        }
        Err(err) => {
            error!("(Core) Vulkan instance creation failed: {}", err);
            Err(VulkanError::VulkanoInstance(err))
        }
    }
}

/// Selects a suitable Vulkan physical device.
pub fn select_physical_device(instance: Arc<Instance>) -> Result<Arc<PhysicalDevice>> {
    info!("(Core) Starting physical device selection...");

    let required_device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    info!("(Core) Device selection will require extensions: {:?}", required_device_extensions);

    let devices = match PhysicalDevice::enumerate(&instance).collect::<Vec<_>>() {
        Ok(devs) => devs,
        Err(e) => return Err(VulkanError::VulkanoPhysicalDevice(e)),
    };
    
    info!("(Core) Found {} available physical device(s)", devices.len());
    if devices.is_empty() {
        warn!("(Core) No physical devices found!");
        return Err(VulkanError::NoSuitablePhysicalDevice);
    }

    let selected_device = devices.into_iter()
        .inspect(|device| {
            debug!("(Core) Evaluating device: '{}' (Type: {:?}, API: {}, Driver: {:?})",
                device.properties().device_name,
                device.properties().device_type,
                device.properties().api_version,
                device.properties().driver_version
            );
        })
        .filter(|device| {
            let supported_extensions = device.supported_extensions();
            if !supported_extensions.khr_swapchain {
                warn!("(Core) Device '{}' does not support required extension khr_swapchain.", device.properties().device_name);
                return false;
            }
            true
        })
        .filter(|device| {
            device.queue_family_properties().iter().any(|qf| qf.queue_flags.intersects(QueueFlags::GRAPHICS))
        })
        .min_by_key(|device| {
            match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5, 
            }
        });

    match selected_device {
        Some(device) => {
            info!("(Core) Selected physical device: {} (Type: {:?})", device.properties().device_name, device.properties().device_type);
            Ok(device)
        }
        None => {
            warn!("(Core) No suitable physical device found after filtering.");
            Err(VulkanError::NoSuitablePhysicalDevice)
        }
    }
}

/// Finds suitable queue families on the given physical device.
pub fn find_queue_families(physical_device: Arc<PhysicalDevice>) -> Result<QueueFamilyIndices> {
    info!("(Core) Finding queue families for device: {}", physical_device.properties().device_name);

    if !physical_device.instance().enabled_extensions().khr_wayland_surface {
        error!("(Core) Instance extension VK_KHR_wayland_surface is not enabled.");
        return Err(VulkanError::MissingExtension("VK_KHR_wayland_surface instance extension not enabled".to_string()));
    }

    let mut indices = QueueFamilyIndices::default();
    let queue_family_properties = physical_device.queue_family_properties();

    for (i, qf_props) in queue_family_properties.iter().enumerate() {
        let q_idx = i as u32;
        if qf_props.queue_flags.intersects(QueueFlags::GRAPHICS) && indices.graphics_family.is_none() {
            indices.graphics_family = Some(q_idx);
        }
        if physical_device.supports_wayland_presentation(q_idx).unwrap_or(false) && indices.present_family.is_none() {
            indices.present_family = Some(q_idx);
        }
        if qf_props.queue_flags.intersects(QueueFlags::COMPUTE) && !qf_props.queue_flags.intersects(QueueFlags::GRAPHICS) && indices.compute_family.is_none() {
            indices.compute_family = Some(q_idx);
        }
        if qf_props.queue_flags.intersects(QueueFlags::TRANSFER) && !qf_props.queue_flags.intersects(QueueFlags::GRAPHICS) && !qf_props.queue_flags.intersects(QueueFlags::COMPUTE) && indices.transfer_family.is_none() {
            indices.transfer_family = Some(q_idx);
        }
    }

    if indices.compute_family.is_none() { // Fallback for compute
        for (i, qf_props) in queue_family_properties.iter().enumerate() {
            if qf_props.queue_flags.intersects(QueueFlags::COMPUTE) { indices.compute_family = Some(i as u32); break; }
        }
    }
    if indices.transfer_family.is_none() { // Fallback for transfer
        for (i, qf_props) in queue_family_properties.iter().enumerate() {
            if qf_props.queue_flags.intersects(QueueFlags::TRANSFER) && !qf_props.queue_flags.intersects(QueueFlags::GRAPHICS) { indices.transfer_family = Some(i as u32); break; }
        }
        if indices.transfer_family.is_none() {
            for (i, qf_props) in queue_family_properties.iter().enumerate() {
                if qf_props.queue_flags.intersects(QueueFlags::TRANSFER) { indices.transfer_family = Some(i as u32); break; }
            }
        }
    }
    
    if indices.graphics_family.is_none() {
         error!("(Core) Critical: Graphics queue family not found.");
         return Err(VulkanError::QueueFamilyIdentificationError("Graphics queue family not found.".to_string()));
    }
    if indices.present_family.is_none() {
        warn!("(Core) No dedicated Wayland present queue found. Checking if graphics queue can present.");
        if let Some(gfx_idx) = indices.graphics_family {
            if physical_device.supports_wayland_presentation(gfx_idx).unwrap_or(false) {
                indices.present_family = Some(gfx_idx);
                info!("(Core) Using graphics queue family {} as Wayland present queue.", gfx_idx);
            }
        }
    }
    
    info!("(Core) Selected Queue Families: Graphics: {:?}, Present: {:?}, Compute: {:?}, Transfer: {:?}",
        indices.graphics_family, indices.present_family, indices.compute_family, indices.transfer_family);

    if !indices.is_complete() {
        error!("(Core) Critical queue families missing (Graphics: {:?}, Present: {:?})", indices.graphics_family, indices.present_family);
        return Err(VulkanError::QueueFamilyIdentificationError("Required graphics or present queue family not found or not Wayland compatible.".to_string()));
    }
    Ok(indices)
}

/// Creates a Vulkan logical device and its queues.
pub fn create_logical_device(
    physical_device: Arc<PhysicalDevice>,
    queue_indices: &QueueFamilyIndices,
) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>> + Send + Sync)> {
    info!("(Core) Creating logical device...");

    let mut unique_queue_families = HashSet::new();
    if let Some(idx) = queue_indices.graphics_family { unique_queue_families.insert(idx); }
    if let Some(idx) = queue_indices.present_family { unique_queue_families.insert(idx); }

    let queue_create_infos: Vec<QueueCreateInfo> = unique_queue_families
        .iter()
        .map(|&index| QueueCreateInfo { queue_family_index: index, queues: vec![1.0], ..Default::default() })
        .collect();

    if queue_create_infos.is_empty() {
        error!("(Core) No queue families provided for logical device creation.");
        return Err(VulkanError::QueueFamilyIdentificationError("No queues to create for logical device.".to_string()));
    }
    debug!("(Core) Requesting queues: {:?}", queue_create_infos);

    let required_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    if !physical_device.supported_extensions().contains(&required_extensions) {
        if !physical_device.supported_extensions().khr_swapchain {
             error!("(Core) Required device extension VK_KHR_swapchain not supported.");
            return Err(VulkanError::MissingExtension("VK_KHR_swapchain not supported".to_string()));
        }
    }
    info!("(Core) Enabled device extensions: {:?}", required_extensions);

    let features = Features::empty();
    info!("(Core) Enabled device features: {:?}", features);

    match Device::new( Arc::clone(&physical_device), DeviceCreateInfo {
            enabled_extensions: required_extensions,
            enabled_features: features,
            queue_create_infos,
            ..Default::default()
        },
    ) {
        Ok((device, queues)) => {
            info!("(Core) Logical device created successfully.");
            Ok((device, queues))
        }
        Err(e) => {
            error!("(Core) Logical device creation failed: {}", e);
            Err(VulkanError::VulkanoDevice(e))
        }
    }
}

/// Central struct holding all core Vulkan components.
pub struct VulkanCoreContext {
    pub instance: Arc<Instance>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue_family_indices: QueueFamilyIndices,
    pub graphics_queue: Arc<Queue>,
    pub present_queue: Arc<Queue>,
}

impl VulkanCoreContext {
    /// Initializes all Vulkan components: instance, physical device, logical device, and queues.
    pub fn new() -> Result<Self> {
        info!("Initializing NovaDE Vulkan Renderer Core Context...");

        // Calls to helper functions are now direct as they are in the same module.
        let instance = create_instance()?;
        let physical_device = select_physical_device(Arc::clone(&instance))?;
        let queue_family_indices = find_queue_families(Arc::clone(&physical_device))?;
        
        let (device, queues_iter_raw) = create_logical_device(Arc::clone(&physical_device), &queue_family_indices)?;
        let all_queues: Vec<Arc<Queue>> = queues_iter_raw.collect();

        let graphics_q_idx = queue_family_indices.graphics_family
            .ok_or_else(|| {
                error!("Graphics queue family index missing after successful identification.");
                VulkanError::QueueFamilyIdentificationError("Graphics queue index unavailable in context creation".to_string())
            })?;
        let present_q_idx = queue_family_indices.present_family
            .ok_or_else(|| {
                error!("Present queue family index missing after successful identification.");
                VulkanError::QueueFamilyIdentificationError("Present queue index unavailable in context creation".to_string())
            })?;

        let mut graphics_queue: Option<Arc<Queue>> = None;
        let mut present_queue: Option<Arc<Queue>> = None;

        for queue in all_queues {
            if queue.queue_family_index() == graphics_q_idx && graphics_queue.is_none() {
                graphics_queue = Some(queue.clone());
            }
            if queue.queue_family_index() == present_q_idx && present_queue.is_none() {
                present_queue = Some(queue.clone());
            }
        }
        
        let gq = graphics_queue.ok_or_else(|| {
            error!("Failed to retrieve graphics queue (family {}) from logical device.", graphics_q_idx);
            VulkanError::QueueFamilyIdentificationError(format!("Graphics queue (family {}) not found in logical device queues.", graphics_q_idx))
        })?;
        let pq = present_queue.ok_or_else(|| {
            error!("Failed to retrieve present queue (family {}) from logical device.", present_q_idx);
            VulkanError::QueueFamilyIdentificationError(format!("Present queue (family {}) not found in logical device queues.", present_q_idx))
        })?;
            
        debug!("Retrieved queues: Graphics (Family: {}, ID-in-family: {}), Present (Family: {}, ID-in-family: {})", 
               gq.queue_family_index(), gq.id_within_family(), 
               pq.queue_family_index(), pq.id_within_family());

        info!("Vulkan Core Context initialized successfully.");
        Ok(Self {
            instance,
            physical_device,
            device,
            queue_family_indices: queue_family_indices.clone(),
            graphics_queue: gq,
            present_queue: pq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports VulkanCoreContext, VulkanError, QueueFamilyIndices, etc.
                 // Also imports helper functions like create_instance if they are pub in super
    use vulkano::instance::InstanceCreationError; 
    use vulkano::Version; // Make sure Version is in scope for tests
    use std::sync::Once;

    static TEST_LOGGER_INIT: Once = Once::new();

    fn setup_test_logger() {
        TEST_LOGGER_INIT.call_once(|| {
            env_logger::builder().is_test(true).try_init().ok();
        });
    }

    #[test]
    fn test_vulkan_core_context_initialization() {
        setup_test_logger();
        info!("Running test: test_vulkan_core_context_initialization (within novade-system)");

        match VulkanCoreContext::new() {
            Ok(context) => {
                info!("VulkanCoreContext::new() succeeded in test environment.");
                assert_eq!(context.instance.api_version(), Version::V1_3, "Instance API version mismatch.");
                assert!(!context.physical_device.properties().device_name.is_empty(), "Physical device name should not be empty.");
                assert!(context.queue_family_indices.graphics_family.is_some(), "Graphics family index should be Some.");
                assert!(context.queue_family_indices.present_family.is_some(), "Present family index should be Some.");
                assert_eq!(context.graphics_queue.queue_family_index(), context.queue_family_indices.graphics_family.unwrap(), "Graphics queue family index mismatch.");
                assert_eq!(context.present_queue.queue_family_index(), context.queue_family_indices.present_family.unwrap(), "Present queue family index mismatch.");
                info!("test_vulkan_core_context_initialization: Assertions passed. Context seems valid.");
            }
            Err(VulkanError::NoSuitablePhysicalDevice) => {
                warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - No suitable physical device found.");
            }
            Err(VulkanError::VulkanoInstance(InstanceCreationError::InitializationFailed)) => {
                warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - Vulkan instance initialization failed (no driver/ICD).");
            }
            Err(VulkanError::VulkanoInstance(InstanceCreationError::LayerNotPresent(_))) => {
                warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - A requested validation layer was not present.");
            }
            Err(VulkanError::MissingExtension(ref ext_name)) 
                if ext_name.contains("VK_KHR_wayland_surface instance extension not enabled") || 
                   ext_name.contains("VK_KHR_surface instance extension not enabled") => {
                warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - Instance surface extension issue (Wayland/KHR_surface: {}).", ext_name);
            }
            Err(VulkanError::MissingExtension(ref ext_name)) if ext_name.contains("VK_KHR_swapchain not supported") => {
                 warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - Device extension VK_KHR_swapchain not supported.");
            }
            Err(VulkanError::QueueFamilyIdentificationError(ref msg)) if msg.contains("Present family (Wayland compatible) missing") => {
                 warn!("test_vulkan_core_context_initialization: Skipped (passed with warning) - Could not find a Wayland-compatible present queue. Msg: {}", msg);
            }
            Err(e) => {
                error!("test_vulkan_core_context_initialization: Failed with unexpected error: {:?}", e);
                panic!("VulkanCoreContext::new() failed with unexpected error: {:?}", e);
            }
        }
    }
}
