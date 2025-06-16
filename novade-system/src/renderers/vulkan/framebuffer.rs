use ash::{vk, Device as AshDevice};
use std::sync::Arc;

// ANCHOR: VulkanFramebuffer Struct Definition
pub struct VulkanFramebuffer {
    device: Arc<AshDevice>,
    framebuffer: vk::Framebuffer,
}

// ANCHOR: VulkanFramebuffer Implementation
impl VulkanFramebuffer {
    pub fn new(
        device: Arc<AshDevice>,
        render_pass: vk::RenderPass, // The render pass this framebuffer is compatible with
        image_view: vk::ImageView,   // The image view to attach
        extent: vk::Extent2D,        // Dimensions of the framebuffer
    ) -> Result<Self, String> {

        // ANCHOR_EXT: Framebuffer Create Info
        let attachments = [image_view];
        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1)
            .build();

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_create_info, None)
                .map_err(|e| format!("Failed to create framebuffer: {}", e))?
        };

        // It might be useful to log creation, perhaps with extent or image_view handle
        // println!("VulkanFramebuffer created for ImageView {:?} with extent {}x{}", image_view, extent.width, extent.height);
        Ok(Self { device, framebuffer })
    }

    // ANCHOR: Accessor for vk::Framebuffer
    pub fn handle(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

// ANCHOR: VulkanFramebuffer Drop Implementation
impl Drop for VulkanFramebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.framebuffer, None);
        }
        // To avoid spamming logs if many framebuffers are created/destroyed (e.g. per swapchain image)
        // consider making this log conditional or less verbose.
        // println!("VulkanFramebuffer dropped (handle: {:?}).", self.framebuffer);
    }
}
