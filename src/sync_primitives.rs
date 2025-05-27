use crate::error::{Result, VulkanError};
use crate::device::LogicalDevice;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;

pub struct FrameSyncPrimitives {
    device: Arc<Device>, // For destruction
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl FrameSyncPrimitives {
    pub fn new(logical_device_wrapper: &LogicalDevice) -> Result<Self> {
        let device = logical_device_wrapper.raw().clone();

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        
        let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }
            .map_err(VulkanError::VkResult)?;
            
        let render_finished_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }
            .map_err(VulkanError::VkResult)?;

        // Create fence in signaled state for the first frame
        // vk::FenceCreateFlags::SIGNALED_BIT is the correct enum variant name in vulkanalia
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None) }
            .map_err(VulkanError::VkResult)?;
            
        log::debug!("Created FrameSyncPrimitives (2 semaphores, 1 signaled fence).");

        Ok(Self {
            device,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        })
    }
}

impl Drop for FrameSyncPrimitives {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_semaphore(self.image_available_semaphore, None);
            self.device.destroy_semaphore(self.render_finished_semaphore, None);
            self.device.destroy_fence(self.in_flight_fence, None);
        }
        log::debug!("FrameSyncPrimitives destroyed.");
    }
}

// Helper function to create a list of sync primitives, typically called by FrameRenderer.
// The `MAX_FRAMES_IN_FLIGHT` constant would be defined elsewhere (e.g., in FrameRenderer).
pub fn create_sync_objects_list(
    logical_device_wrapper: &LogicalDevice,
    max_frames_in_flight: usize,
) -> Result<Vec<FrameSyncPrimitives>> {
    (0..max_frames_in_flight)
        .map(|_| FrameSyncPrimitives::new(logical_device_wrapper))
        .collect()
}
