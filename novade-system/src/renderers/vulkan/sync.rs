use ash::{vk, Device as AshDevice};
use std::sync::Arc;

// ANCHOR: FrameSyncPrimitives Struct Definition
pub struct FrameSyncPrimitives {
    device: Arc<AshDevice>,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

// ANCHOR: FrameSyncPrimitives Implementation
impl FrameSyncPrimitives {
    pub fn new(device: Arc<AshDevice>) -> Result<Self, String> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        // No flags needed for basic semaphores

        let image_available_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| format!("Failed to create image_available_semaphore: {}", e))?
        };

        let render_finished_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| format!("Failed to create render_finished_semaphore: {}", e))?
        };

        // Create fence in signaled state for the first frame to avoid deadlock
        // if we wait on it before the first submission.
        // However, typical frame loops wait then reset, so starting unsignaled is also common.
        // For a MAX_FRAMES_IN_FLIGHT loop, fences are usually associated with a specific frame index's
        // previous submission. If we always wait then submit, starting unsignaled is fine.
        // Let's start unsignaled, as typical render loops will wait for the *previous* use of this fence.
        let fence_create_info = vk::FenceCreateInfo::builder();
            // .flags(vk::FenceCreateFlags::SIGNALED); // Only if needed for specific first-frame logic

        let in_flight_fence = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .map_err(|e| format!("Failed to create in_flight_fence: {}", e))?
        };

        Ok(Self {
            device,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        })
    }
}

// ANCHOR: FrameSyncPrimitives Drop Implementation
impl Drop for FrameSyncPrimitives {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_semaphore(self.image_available_semaphore, None);
            self.device.destroy_semaphore(self.render_finished_semaphore, None);
            self.device.destroy_fence(self.in_flight_fence, None);
        }
        // println!("FrameSyncPrimitives dropped (IAS: {:?}, RFS: {:?}, IFF: {:?})",
        //          self.image_available_semaphore, self.render_finished_semaphore, self.in_flight_fence);
    }
}
