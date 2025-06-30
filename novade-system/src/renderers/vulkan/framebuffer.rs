use ash::{vk, Device as AshDevice};
use std::sync::Arc;

// ANCHOR: VulkanFramebuffer Struct Definition
pub struct VulkanFramebuffer {
    device: Arc<AshDevice>,
    framebuffer: vk::Framebuffer,
}

/// Creates a Vulkan Framebuffer.
///
/// A framebuffer defines the collection of attachments that will be used by a render pass instance.
/// For the simple case in WP-302, this typically means attaching a single swapchain image view
/// as the color target.
///
/// This function aligns with `Rendering Vulkan.md` (Spec 6.5 - Framebuffer-Erstellung).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `render_pass`: The `vk::RenderPass` with which this framebuffer must be compatible.
///   The attachments specified here must match those expected by the `render_pass`.
/// * `image_view`: The `vk::ImageView` to be attached. For swapchain rendering, this will be
///   an image view of a swapchain image.
/// * `extent`: The `vk::Extent2D` (width and height) of the framebuffer. This should match
///   the extent of the attached image views and the render area of the render pass.
///
/// # Returns
/// A `Result` containing the created `vk::Framebuffer` handle, or an error string
/// if creation fails.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 6.5):
/// - `renderPass`: Matches the `render_pass` parameter.
/// - `attachmentCount`, `pAttachments`: Configured with a single `image_view`.
/// - `width`, `height`: Matches the `extent` parameter.
/// - `layers`: Set to 1, appropriate for standard 2D rendering.
pub fn create_vulkan_framebuffer(
    device: &AshDevice,
    render_pass: vk::RenderPass,
/// A framebuffer references all `vk::ImageView` attachments that are used by a render pass instance.
/// For this simple case (WP-302), it will typically reference a single swapchain image view
/// as the color attachment.
/// Aligns with `Rendering Vulkan.md` (Spec 6.5).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `render_pass`: The `vk::RenderPass` with which this framebuffer will be compatible.
/// * `image_view`: The `vk::ImageView` to be used as the color attachment.
/// * `extent`: The `vk::Extent2D` (width and height) of the framebuffer.
///
/// # Returns
/// A `Result` containing the created `vk::Framebuffer` or an error string on failure.
pub fn create_vulkan_framebuffer(
    device: &AshDevice,
    render_pass: vk::RenderPass,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
) -> Result<vk::Framebuffer, String> {
    let attachments = [image_view];
    let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(&attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1); // For non-stereoscopic 2D, layers is 1

    unsafe {
        device
            .create_framebuffer(&framebuffer_create_info, None)
            .map_err(|e| format!("Failed to create framebuffer: {}", e))
    }
}

// ANCHOR: VulkanFramebuffer Implementation
impl VulkanFramebuffer {
    /// Creates a new `VulkanFramebuffer` instance.
    ///
    /// This constructor wraps the `create_vulkan_framebuffer` free function.
    ///
    /// # Arguments
    /// * `device`: An `Arc` reference to the logical `ash::Device`.
    /// * `render_pass`: The `vk::RenderPass` for compatibility.
    /// * `image_view`: The `vk::ImageView` for the color attachment.
    /// * `extent`: The `vk::Extent2D` dimensions.
    ///
    /// # Returns
    /// A `Result` containing the new `VulkanFramebuffer` or an error string.
    pub fn new(
        device: Arc<AshDevice>,
        render_pass: vk::RenderPass, // The render pass this framebuffer is compatible with
        image_view: vk::ImageView,   // The image view to attach
        extent: vk::Extent2D,        // Dimensions of the framebuffer
    ) -> Result<Self, String> {
        let framebuffer_handle =
            create_vulkan_framebuffer(device.as_ref(), render_pass, image_view, extent)?;

        // Optional: log creation
        // println!("VulkanFramebuffer created (handle: {:?}) for ImageView {:?} with extent {}x{}", framebuffer_handle, image_view, extent.width, extent.height);
        Ok(Self { device, framebuffer: framebuffer_handle })
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
