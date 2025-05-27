use crate::error::{Result, VulkanError};
use crate::instance::VulkanInstance;
use crate::physical_device::{PhysicalDeviceInfo, QueueFamilyIndices};
use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;

// Required device extensions (as per physical_device.rs and Rendering Vulkan.md)
use vulkanalia::vk::{KhrSwapchainExtension, ExtExternalMemoryDmaBufExtension, KhrExternalMemoryFdExtension, ExtImageDrmFormatModifierExtension};


#[derive(Debug, Clone)]
pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue, // May be the same as graphics_queue
    pub compute_queue: Option<vk::Queue>, // Optional
    pub transfer_queue: Option<vk::Queue>, // Optional, may be same as graphics or compute
}

pub struct LogicalDevice {
    device: Arc<Device>,
    queues: Queues,
}

impl LogicalDevice {
    pub fn new(
        instance_wrapper: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<Self> {
        let instance = instance_wrapper.raw();
        let physical_device = physical_device_info.raw();
        let indices = physical_device_info.queue_family_indices();

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());
        if let Some(compute_family) = indices.compute_family {
            unique_queue_families.insert(compute_family);
        }
        if let Some(transfer_family) = indices.transfer_family {
            unique_queue_families.insert(transfer_family);
        }

        let queue_priority = 1.0;
        let mut queue_create_infos = Vec::new();
        for queue_family_index in unique_queue_families {
            let info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&[queue_priority]);
            queue_create_infos.push(info);
        }

        // Specify required device features. Enable only what's needed and available.
        let available_features = physical_device_info.features();
        let mut features_to_enable = vk::PhysicalDeviceFeatures::builder();
        if available_features.sampler_anisotropy == vk::TRUE {
            features_to_enable = features_to_enable.sampler_anisotropy(true);
        }
        if available_features.geometry_shader == vk::TRUE {
            // Enable if actually used by a pipeline later
            // features_to_enable = features_to_enable.geometry_shader(true);
        }
        // Add other features as needed, e.g., for robustBufferAccess, tessellation shaders etc.
        // features_to_enable = features_to_enable.robust_buffer_access(true);


        let device_extensions_cptr = [
            KhrSwapchainExtension::name().as_ptr(),
            ExtExternalMemoryDmaBufExtension::name().as_ptr(),
            KhrExternalMemoryFdExtension::name().as_ptr(),
            ExtImageDrmFormatModifierExtension::name().as_ptr(),
        ];
        
        // Check if all these extensions are actually supported by the selected physical device.
        // This check should ideally be part of PhysicalDeviceInfo or done before this stage.
        // For simplicity here, we assume PhysicalDeviceInfo already vetted them.

        let mut device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&features_to_enable)
            .enabled_extension_names(&device_extensions_cptr);

        // For compatibility with older Vulkan versions or specific driver requirements,
        // validation layers might be specified at device level too, but it's generally
        // recommended at instance level. We'll assume instance-level for now.
        // If validation layers are needed at device level:
        // .enabled_layer_names(&validation_layers_cptr_if_any)

        // Vulkan 1.3 features like dynamic_rendering and synchronization2
        // are enabled via pNext chains if not part of core for the target API version.
        // Assuming API_VERSION_1_3 was requested for the instance, these are core.
        // If specific 1.1 or 1.2 features are needed and not core in 1.0, they go in pNext.
        // e.g. let mut features12 = vk::PhysicalDeviceVulkan12Features::builder().timeline_semaphore(true);
        // device_create_info = device_create_info.push_next(&mut features12);


        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }
            .map_err(VulkanError::VkResult)?;
        let device = Arc::new(device);
        log::info!("Logical device created.");

        let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };
        
        let compute_queue = indices.compute_family.map(|idx| unsafe { device.get_device_queue(idx, 0) });
        let transfer_queue = indices.transfer_family.map(|idx| unsafe { device.get_device_queue(idx, 0) });

        log::info!("Retrieved device queues: Graphics, Present. Compute: {:?}, Transfer: {:?}",
            compute_queue.is_some(), transfer_queue.is_some());

        Ok(Self {
            device,
            queues: Queues {
                graphics_queue,
                present_queue,
                compute_queue,
                transfer_queue,
            },
        })
    }

    // Getter for the raw Vulkan device
    pub fn raw(&self) -> &Arc<Device> {
        &self.device
    }
    
    // Getter for the queues
    pub fn queues(&self) -> &Queues {
        &self.queues
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
        log::debug!("Logical device destroyed.");
    }
}

```
