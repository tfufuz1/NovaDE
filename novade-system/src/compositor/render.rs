// This is novade-system/src/compositor/render.rs
// Abstraction for rendering backends (OpenGL ES 2.0, Vulkan).

use smithay::{
    backend::{
        allocator::{dmabuf::Dmabuf, Allocator, Buffer as BackendBuffer}, // For DMABUF import
        renderer::{
            gles2::Gles2Renderer, // Smithay's GLES2 Renderer
            // Import Vulkan related types from Smithay if/when available, e.g., VulkanRenderer
            // For now, Vulkan might be a more manual implementation via `ash`.
            Texture, Bind, Frame, Offscreen, RendererNode, // Common renderer traits
            damage::OutputDamageTracker, // For tracking damage per output
            element::{RenderElement, surface::WaylandSurfaceRenderElement}, // For rendering Wayland surfaces
            // Import specific error types like Gles2Error, BindError, FrameError, etc.
        },
    },
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    utils::{Buffer as SmithayBufferCoords, Physical, Point, Rectangle, Scale, Size, Transform},
};
use tracing::{error, info, warn};
use std::any::Any;


use crate::compositor::{
    errors::CompositorError,
    state::DesktopState, // To access space, outputs, etc.
};

// If using ash directly for Vulkan:
// use ash::vk;

/// Generic error type for renderer operations, convertible to CompositorError.
#[derive(Debug, thiserror::Error)]
pub enum NovaRendererError {
    #[error("Failed to bind EGL/GL context: {0}")]
    ContextBindError(String),
    #[error("Texture creation failed: {0}")]
    TextureCreationError(String),
    #[error("Shader compilation/linking failed: {0}")]
    ShaderError(String),
    #[error("Frame submission failed: {0}")]
    FrameSubmitError(String),
    #[error("Buffer import failed: {0}")]
    BufferImportError(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("Underlying GLES error: {0}")]
    GlesError(#[from] smithay::backend::renderer::gles2::Gles2Error),
    // TODO: Add Vulkan specific errors
    #[error("Generic renderer error: {0}")]
    Generic(String),
}

impl From<NovaRendererError> for CompositorError {
    fn from(err: NovaRendererError) -> Self {
        CompositorError::RenderingError(err.to_string())
    }
}


/// Trait abstracting over specific rendering backend implementations (GLES, Vulkan).
/// Smithay's `Renderer` trait is extensive. This `NovaRenderer` trait will be simpler
/// and focus on NovaDE's specific needs, potentially wrapping Smithay's renderers.
pub trait NovaRendererBackend: 'static + Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static; // Backend-specific error type
    type TextureId: Texture + Clone + Send + Sync + 'static; // Backend-specific texture type

    /// Initializes the renderer on a given rendering node (e.g., DRM fd).
    /// `node` could be Option<RendererNode> or a specific type like DrmNode.
    fn init_on_node(&mut self, node: &RendererNode) -> Result<(), Self::Error>;

    /// Imports a `WlBuffer` (typically SHM) into a renderable texture.
    fn import_shm_buffer(&mut self, buffer: &WlBuffer, surface_attributes: Option<&smithay::wayland::compositor::SurfaceAttributes>, damage: &[Rectangle<i32, SmithayBufferCoords>]) -> Result<Self::TextureId, Self::Error>;

    /// Imports a `Dmabuf` into a renderable texture.
    fn import_dmabuf(&mut self, dmabuf: &Dmabuf, damage: Option<&[Rectangle<i32, Physical>]>) -> Result<Self::TextureId, Self::Error>;

    /// Renders a single frame for a given output.
    /// `elements` are the items to draw (windows, layers, cursor).
    /// `damage_tracker` helps optimize rendering by only redrawing changed parts.
    fn render_output_frame<'a, E>(
        &mut self,
        output_node: &RendererNode, // Node associated with the output being rendered to
        output_size: Size<i32, Physical>,
        output_scale: Scale<f64>,
        output_transform: Transform,
        elements: &'a [E],
        clear_color: [f32; 4],
        damage_tracker: &mut OutputDamageTracker,
    ) -> Result<Option<Rectangle<i32, Physical>>, Self::Error> // Returns computed damage on the output
    where E: RenderElement<Self, Self::TextureId, Error = Self::Error> + 'a;


    /// Presents the rendered frame to the specified output node.
    /// This might involve swapping buffers or other backend-specific actions.
    fn present_output_frame(&mut self, output_node: &RendererNode) -> Result<(), Self::Error>;

    fn id(&self) -> &str; // For logging/identification
}


// --- OpenGL ES 2.0 Renderer Wrapper ---
pub struct GlesNovaRenderer {
    pub inner: Gles2Renderer,
    // EGL context, device, surface, etc. managed by backend (e.g. DrmBackend with GbmDevice)
    // Or, if winit backend, winit handles this.
    // This struct might need to hold the EGLDisplay, EGLContext if not managed by Smithay's RendererNode.
    // Smithay's Gles2Renderer often takes these via its `bind` method.
    // The `RendererNode` from DrmBackend already encapsulates the necessary GBM/EGL surface.
}

impl GlesNovaRenderer {
    pub fn new(renderer: Gles2Renderer) -> Self {
        info!("OpenGL ES 2.0 NovaRenderer backend initialized.");
        Self { inner: renderer }
    }
}

impl NovaRendererBackend for GlesNovaRenderer {
    type Error = smithay::backend::renderer::gles2::Gles2Error; // Gles2Error can be the error type
    type TextureId = <Gles2Renderer as smithay::backend::renderer::Renderer>::TextureId; // Gles2Texture

    fn id(&self) -> &str { "OpenGL ES 2.0" }

    fn init_on_node(&mut self, node: &RendererNode) -> Result<(), Self::Error> {
        // With Smithay's Gles2Renderer, context binding to a specific output's surface (RendererNode)
        // is typically handled when initiating a render pass or frame for that output,
        // e.g., inside `Frame::render_output` or if `Gles2Renderer::bind` is called explicitly.
        // For the `NovaRendererBackend` trait, this method signifies that the renderer should
        // prepare itself for operations on this node if any explicit per-node setup is needed
        // beyond what the main render_output_frame will do.
        // In many cases with Gles2Renderer used with Smithay's backends (like DRM),
        // this might be a no-op as the backend and `render_output_frame` handle context activation.
        info!("GlesNovaRenderer: init_on_node called for node: {:?}. Explicit binding might not be needed here if using Frame::render_output.", node.id());
        // If direct GL calls were to be made outside of a Frame, one might call:
        // self.inner.bind(node.egl_surface().ok_or(Gles2Error::EglError(EglError::BadSurface))?)?;
        // But for now, we assume Frame::render_output handles this.
        Ok(())
    }

    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface_attributes: Option<&smithay::wayland::compositor::SurfaceAttributes>, // Needed for shm_buffer_to_texture
        damage: &[Rectangle<i32, SmithayBufferCoords>]
    ) -> Result<Self::TextureId, Self::Error> {
        // Gles2Renderer has shm_buffer_to_texture
        if surface_attributes.is_none() {
            return Err(Self::Error::Generic("SHM buffer import requires surface attributes".into()));
        }
        self.inner.import_shm_buffer(buffer, surface_attributes, damage)
    }

    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        damage: Option<&[Rectangle<i32, Physical>]>
    ) -> Result<Self::TextureId, Self::Error> {
        // Gles2Renderer has import_dmabuf
        self.inner.import_dmabuf(dmabuf, damage)
    }

    fn render_output_frame<'a, E>(
        &mut self,
        output_node: &RendererNode,
        output_size: Size<i32, Physical>,
        output_scale: Scale<f64>,
        output_transform: Transform,
        elements: &'a [E],
        clear_color: [f32; 4],
        damage_tracker: &mut OutputDamageTracker,
    ) -> Result<Option<Rectangle<i32, Physical>>, Self::Error>
    where E: RenderElement<Self, Self::TextureId, Error = Self::Error> + 'a
    {
        // Use Gles2Renderer's render_output method
        // This requires `self` to be `&mut Gles2Renderer`, so we use `&mut self.inner`.
        // The elements need to be compatible with Gles2Renderer.
        // WaylandSurfaceRenderElement is generic and should work.
        // The `Frame::render_output` method is what we need.

        // `damage_tracker.render_output` is the helper that wraps Frame::render_output
        damage_tracker.render_output(
            &mut self.inner,      // The Gles2Renderer
            output_node,          // The node to render to (contains target usually)
            0,                    // age (for buffer age tracking, 0 means redraw everything)
            output_size,
            output_scale,
            output_transform,
            elements,
            clear_color,
        )
    }

    fn present_output_frame(&mut self, output_node: &RendererNode) -> Result<(), Self::Error> {
        // This is usually handled by the backend (e.g. DrmBackend swaps buffers).
        // If Gles2Renderer needs an explicit swap/flush that isn't part of Frame::finish(), it goes here.
        // Typically, Frame::finish (called by damage_tracker.render_output) handles submission.
        // If rendering to an FBO for effects first, then that FBO is blitted to screen,
        // then the final swap is by the backend.
        // For now, assume DrmBackend handles swap after render_output.
        info!("GlesNovaRenderer: Frame presentation (usually handled by backend like DRM).");
        Ok(())
    }
}


// --- Vulkan Renderer Wrapper (Placeholder) ---
use ash::{vk, Entry, Instance, Device};
use std::ffi::{CStr, CString};
use std::sync::Arc;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle}; // For surface creation

// Placeholder for a Vulkan texture ID. In a real scenario, this would be more complex.
#[derive(Clone)]
pub struct VulkanTextureId(vk::Image, vk::ImageView, vk::DeviceMemory); // Simplified

impl Texture for VulkanTextureId {
    fn width(&self) -> u32 { 0 } // Placeholder
    fn height(&self) -> u32 { 0 } // Placeholder
    fn format(&self) -> Option<smithay::backend::renderer::buffer_formats::Fourcc> { None } // Placeholder
}


pub struct VulkanNovaRenderer {
    entry: Option<Entry>,
    instance: Option<Instance>,
    physical_device: vk::PhysicalDevice, // Option might be better if selection fails
    device: Option<Arc<Device>>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    graphics_queue_family_index: u32,
    present_queue_family_index: u32,
    // TODO: Add memory allocator (e.g. gpu-allocator or vk-mem-rs)
    // TODO: Per-output data (swapchains, command pools, framebuffers, etc.)
    //       This might be stored in a HashMap<OutputId, VulkanOutputData>
}

impl VulkanNovaRenderer {
    pub fn new() -> Result<Self, NovaRendererError> {
        info!("Initializing Vulkan NovaRenderer backend...");
        unsafe { // Ash calls are unsafe
            let entry = Entry::load().map_err(|e| NovaRendererError::Generic(format!("Failed to load Vulkan entry: {}", e)))?;

            // --- Instance Creation ---
            let app_name = CString::new("NovaDE Compositor").unwrap();
            let engine_name = CString::new("Smithay-NovaDE").unwrap();
            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 0, 1, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .api_version(vk::API_VERSION_1_1); // Or 1.0, 1.2 depending on needs

            let mut instance_extensions = vec![
                ash::extensions::khr::Surface::name().as_ptr(),
                ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ];
            // TODO: Add debug utils extension if in debug mode
            // instance_extensions.push(ash::extensions::ext::DebugUtils::name().as_ptr());

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&instance_extensions);
                // TODO: Add layers (e.g., validation layers) for debug builds

            let instance = entry.create_instance(&create_info, None)
                .map_err(|e| NovaRendererError::Generic(format!("Failed to create Vulkan instance: {}", e)))?;
            info!("Vulkan instance created successfully.");

            // --- Physical Device Selection ---
            let physical_devices = instance.enumerate_physical_devices()
                .map_err(|e| NovaRendererError::Generic(format!("Failed to enumerate physical devices: {}", e)))?;
            let physical_device = physical_devices.into_iter().find(|pd| {
                // TODO: Implement more robust physical device selection (e.g., prefer discrete GPU)
                // For now, pick the first one that supports graphics and Wayland surface.
                let props = instance.get_physical_device_properties(*pd);
                info!("Found Vulkan Physical Device: {:?}", CStr::from_ptr(props.device_name.as_ptr()));
                // Check for queue families and required extensions later
                true
            }).ok_or_else(|| NovaRendererError::Generic("No suitable Vulkan physical device found.".to_string()))?;
            info!("Selected Vulkan Physical Device.");

            // --- Queue Family Indices ---
            let queue_family_properties = instance.get_physical_device_queue_family_properties(physical_device);
            let graphics_queue_family_index = queue_family_properties.iter().enumerate()
                .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|(index, _)| index as u32)
                .ok_or_else(|| NovaRendererError::Generic("No graphics queue family found.".to_string()))?;

            // For presentation, we'll need a surface later. For now, assume graphics queue can present.
            // A more robust solution checks for Wayland surface support for presentation.
            let present_queue_family_index = graphics_queue_family_index; // Simplification
            info!("Vulkan graphics queue family index: {}", graphics_queue_family_index);


            // --- Logical Device Creation ---
            let device_extensions = vec![
                ash::extensions::khr::Swapchain::name().as_ptr(),
                // Add other required device extensions: e.g., for DMABUF import
                // ash::extensions::khr::ExternalMemoryFd::name().as_ptr(),
                // ash::extensions::ext::ExternalMemoryDmaBuf::name().as_ptr(),
            ];
            let priorities = [1.0f32];
            let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_queue_family_index)
                .queue_priorities(&priorities)
                .build()];

            // TODO: Enable physical device features as needed (e.g., samplerAnisotropy)
            let features = vk::PhysicalDeviceFeatures::builder().build();

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&device_extensions)
                .enabled_features(&features);

            let device = instance.create_device(physical_device, &device_create_info, None)
                .map_err(|e| NovaRendererError::Generic(format!("Failed to create Vulkan logical device: {}", e)))?;
            info!("Vulkan logical device created successfully.");

            let graphics_queue = device.get_device_queue(graphics_queue_family_index, 0);
            let present_queue = device.get_device_queue(present_queue_family_index, 0);


            Ok(Self {
                entry: Some(entry),
                instance: Some(instance),
                physical_device,
                device: Some(Arc::new(device)),
                graphics_queue: Some(graphics_queue),
                present_queue: Some(present_queue),
                graphics_queue_family_index,
                present_queue_family_index,
            })
        }
    }
}

impl Drop for VulkanNovaRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = self.device.take() {
                // Wait for device to be idle before destroying
                // This is a simplified wait, proper synchronization is needed for in-flight commands.
                if Arc::strong_count(&device) == 1 { // Ensure we are the last owner
                     device.device_wait_idle().unwrap_or_else(|e| error!("Vulkan device_wait_idle failed: {}", e));
                     info!("Destroying Vulkan logical device.");
                     device.destroy_device(None);
                }
            }
            if let Some(instance) = self.instance.take() {
                info!("Destroying Vulkan instance.");
                instance.destroy_instance(None);
            }
            self.entry = None;
            info!("VulkanNovaRenderer dropped.");
        }
    }
}


impl NovaRendererBackend for VulkanNovaRenderer {
    type Error = NovaRendererError;
    type TextureId = VulkanTextureId; // Use the placeholder

    fn id(&self) -> &str { "Vulkan" }

    fn init_on_node(&mut self, _node: &RendererNode) -> Result<(), Self::Error> {
        // For Vulkan, this would typically involve:
        // 1. Creating a vk::SurfaceKHR using the Wayland display/surface from the RendererNode.
        //    Requires `VK_KHR_wayland_surface` instance extension.
        //    `ash::extensions::khr::WaylandSurface::create_wayland_surface()`
        // 2. Checking for surface support on the physical device for the chosen queue family.
        // 3. Creating a vk::SwapchainKHR for this surface.
        // This is complex and output-specific. It will be part of per-output data.
        warn!("VulkanNovaRenderer::init_on_node: Swapchain creation per output not yet implemented.");
        Err(NovaRendererError::Unsupported("Vulkan init_on_node".into()))
    }

    fn import_shm_buffer(&mut self, _buffer: &WlBuffer, _surface_attributes: Option<&smithay::wayland::compositor::SurfaceAttributes>, _damage: &[Rectangle<i32, SmithayBufferCoords>]) -> Result<Self::TextureId, Self::Error> {
        // This is complex: copy SHM data to a staging buffer, then to a vk::Image.
        // Requires knowledge of buffer format and manual conversion if not directly mappable.
        Err(NovaRendererError::Unsupported("Vulkan import_shm_buffer".into()))
    }

    fn import_dmabuf(&mut self, _dmabuf: &Dmabuf, _damage: Option<&[Rectangle<i32, Physical>]>) -> Result<Self::TextureId, Self::Error> {
        // Requires VK_KHR_external_memory_fd, VK_EXT_external_memory_dma_buf.
        // Involves creating a vk::Image and vk::DeviceMemory from the DMABUF FDs.
        Err(NovaRendererError::Unsupported("Vulkan import_dmabuf".into()))
    }

    fn render_output_frame<'a, E>(
        &mut self,
        _output_node: &RendererNode, // Used to get the target swapchain image
        _output_size: Size<i32, Physical>,
        _output_scale: Scale<f64>,
        _output_transform: Transform,
        _elements: &'a [E], // Vulkan-compatible RenderElements
        _clear_color: [f32; 4],
        _damage_tracker: &mut OutputDamageTracker, // Used to get damage regions
    ) -> Result<Option<Rectangle<i32, Physical>>, Self::Error>
    where E: RenderElement<Self, Self::TextureId, Error = Self::Error> + 'a
    {
        // This is the core rendering loop for one output:
        // 1. Acquire next swapchain image.
        // 2. Begin command buffer.
        // 3. Begin render pass (targetting the swapchain image's framebuffer).
        // 4. Set viewport, scissor.
        // 5. For each element:
        //    - Bind pipeline (graphics pipeline for that element type).
        //    - Bind descriptor sets (textures, uniforms).
        //    - Push constants (if any).
        //    - Draw element (e.g., vkCmdDraw).
        // 6. End render pass.
        // 7. End command buffer.
        // 8. Submit command buffer to graphics queue.
        // (Presentation is separate, in present_output_frame)
        Err(NovaRendererError::Unsupported("Vulkan render_output_frame".into()))
    }

    fn present_output_frame(&mut self, _output_node: &RendererNode) -> Result<(), Self::Error> {
        // 1. Submit a vk::PresentInfoKHR to the present queue.
        //    This requires the swapchain and the index of the image that was rendered to.
        //    Synchronization (semaphores) between render submission and presentation is crucial.
        Err(NovaRendererError::Unsupported("Vulkan present_output_frame".into()))
    }
}


/// Enum to hold the currently active renderer backend.
/// This allows switching renderers if needed, though typically one is chosen at startup.
pub enum MainNovaRenderer {
    Gles(Box<GlesNovaRenderer>),
    Vulkan(Box<VulkanNovaRenderer>),
    None, // For headless mode or if rendering fails to initialize
}

impl MainNovaRenderer {
    /// Attempts to initialize the GLES renderer.
    pub fn new_gles(gles_renderer: Gles2Renderer) -> Result<Self, CompositorError> {
        Ok(MainNovaRenderer::Gles(Box::new(GlesNovaRenderer::new(gles_renderer))))
    }

    /// Attempts to initialize the Vulkan renderer.
    pub fn new_vulkan(/* params */) -> Result<Self, CompositorError> {
        VulkanNovaRenderer::new()
            .map(|vk_renderer| MainNovaRenderer::Vulkan(Box::new(vk_renderer)))
            .map_err(|e| CompositorError::BackendCreation(format!("Vulkan renderer init failed: {}", e)))
    }

    pub fn id(&self) -> &str {
        match self {
            MainNovaRenderer::Gles(r) => r.id(),
            MainNovaRenderer::Vulkan(r) => r.id(),
            MainNovaRenderer::None => "None",
        }
    }

    // Delegate methods to the active NovaRendererBackend trait implementation
    // These would need to be carefully defined to match the trait and handle the enum dispatch.
    // Example:
    // pub fn render_output_frame<'a, E>(...) -> Result<Option<Rectangle<i32, Physical>>, CompositorError>
    // where E: RenderElement< ... > { // Complex generic bounds needed here
    //     match self {
    //         MainNovaRenderer::Gles(r) => r.render_output_frame(...).map_err(CompositorError::from),
    //         MainNovaRenderer::Vulkan(r) => r.render_output_frame(...).map_err(CompositorError::from),
    //         MainNovaRenderer::None => Err(CompositorError::RendererNotInitialized),
    //     }
    // }
    // Due to the complexity of generics with RenderElement, direct calls on the boxed trait object
    // might be challenging. It might be easier to have DesktopState store Arc<Mutex<dyn NovaRendererBackend>>
    // if dynamic dispatch is heavily used, or have specific render paths for GLES/Vulkan.
    // For now, the specific renderer instance will be retrieved and used directly where needed.
}
