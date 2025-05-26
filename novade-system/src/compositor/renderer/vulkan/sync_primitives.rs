//! Manages synchronization primitives for coordinating GPU and CPU operations.
//!
//! This module defines the `FrameSyncPrimitives` struct, which encapsulates the
//! Vulkan synchronization objects (semaphores and fences) required for managing
//! a single frame in flight during the rendering process. These primitives are
//! essential for ensuring correct order of execution between GPU image acquisition,
//! rendering, presentation, and CPU's access to frame resources.

use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::vk;
use log::{debug, info};

/// Holds synchronization primitives for a single frame in flight.
///
/// Each instance of this struct contains:
/// - An `image_available_semaphore`: Signaled when a swapchain image is ready for rendering.
/// - A `render_finished_semaphore`: Signaled when rendering to the swapchain image is complete.
/// - An `in_flight_fence`: Signaled when the frame has finished rendering and the CPU can
///   safely reuse resources associated with this frame.
///
/// These primitives are used in a cycle to manage `MAX_FRAMES_IN_FLIGHT` frames.
#[derive(Debug)]
pub struct FrameSyncPrimitives {
    /// Semaphore signaled when a swapchain image has been acquired and is ready for rendering.
    /// The graphics queue submission will wait on this semaphore.
    pub image_available_semaphore: vk::Semaphore,
    /// Semaphore signaled when all rendering commands for a frame (including compute and graphics passes)
    /// have completed. The presentation engine will wait on this semaphore before presenting the image.
    pub render_finished_semaphore: vk::Semaphore,
    /// Fence used to signal the CPU that a frame has completed all its GPU work.
    /// The CPU waits on this fence before reusing the command buffer and other resources
    /// associated with this frame. It is created in a signaled state for the first frame
    /// to allow the render loop to start.
    pub in_flight_fence: vk::Fence,
}

impl FrameSyncPrimitives {
    /// Creates new synchronization primitives for one frame.
    ///
    /// This involves creating two `VkSemaphore` objects and one `VkFence` object.
    /// The fence can be created in an initially signaled state, which is typically
    /// required for the first frame in a render loop to prevent deadlocking on `vkWaitForFences`.
    ///
    /// # Arguments
    ///
    /// * `logical_device`: A reference to the `LogicalDevice` used for creating the Vulkan objects.
    /// * `initially_signaled_fence`: If `true`, the `in_flight_fence` is created in a signaled state.
    ///   This is usually true for the first frame's primitives when `MAX_FRAMES_IN_FLIGHT > 1`,
    ///   or always if `MAX_FRAMES_IN_FLIGHT == 1`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `FrameSyncPrimitives` on success, or a `VulkanError`
    /// (typically `VulkanError::VkResult`) if any Vulkan object creation fails.
    pub fn new(
        logical_device: &LogicalDevice,
        initially_signaled_fence: bool,
    ) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        
        let image_available_semaphore = unsafe {
            logical_device.raw.create_semaphore(&semaphore_create_info, None)
        }?; // Uses From<vk::Result> for VulkanError

        let render_finished_semaphore = unsafe {
            logical_device.raw.create_semaphore(&semaphore_create_info, None)
        }?; // Uses From<vk::Result>

        let fence_flags = if initially_signaled_fence {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };
        let fence_create_info = vk::FenceCreateInfo::builder().flags(fence_flags);
        
        let in_flight_fence = unsafe {
            logical_device.raw.create_fence(&fence_create_info, None)
        }?; // Uses From<vk::Result>

        info!(
            "Created FrameSyncPrimitives: IAS={:?}, RFS={:?}, IFF={:?} (signaled={})",
            image_available_semaphore, render_finished_semaphore, in_flight_fence, initially_signaled_fence
        );

        Ok(Self {
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        })
    }

    /// Destroys the synchronization primitives held by this struct.
    ///
    /// This method should be called when the primitives are no longer needed, typically
    /// during the cleanup phase of the `FrameRenderer` or when it's dropped.
    ///
    /// # Arguments
    ///
    /// * `logical_device_raw`: A reference to the `ash::Device` (logical device handle)
    ///   used to destroy the Vulkan objects. This is passed explicitly to avoid lifetime
    ///   complexities or needing to store a clone of `ash::Device` within `FrameSyncPrimitives`
    ///   if it were to implement `Drop` directly.
    ///
    /// # Safety
    ///
    /// The caller must ensure that these synchronization primitives are not currently in use
    /// by any pending GPU operations when this method is called.
    /// The Vulkan handles become invalid after this call.
    pub fn destroy(&self, logical_device_raw: &ash::Device) {
        debug!(
            "Destroying FrameSyncPrimitives: IAS={:?}, RFS={:?}, IFF={:?}",
            self.image_available_semaphore, self.render_finished_semaphore, self.in_flight_fence
        );
        unsafe {
            logical_device_raw.destroy_semaphore(self.image_available_semaphore, None);
            logical_device_raw.destroy_semaphore(self.render_finished_semaphore, None);
            logical_device_raw.destroy_fence(self.in_flight_fence, None);
        }
    }
}
