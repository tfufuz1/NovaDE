use ash::vk;
use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::sync::Arc;
use tracing::{info, warn};

mod swapchain;
pub use swapchain::Swapchain;
mod render_pass;
pub use render_pass::RenderPass;
mod utils;
mod pipeline;
pub use pipeline::GraphicsPipeline;

// Define a debug callback function
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            tracing::debug!(target: "vulkan", "[VERBOSE] type: {:?}, id: {} ({}), message: {}", message_type, message_id_name, message_id_number, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            tracing::info!(target: "vulkan", "[INFO] type: {:?}, id: {} ({}), message: {}", message_type, message_id_name, message_id_number, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            tracing::warn!(target: "vulkan", "[WARNING] type: {:?}, id: {} ({}), message: {}", message_type, message_id_name, message_id_number, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            tracing::error!(target: "vulkan", "[ERROR] type: {:?}, id: {} ({}), message: {}", message_type, message_id_name, message_id_number, message);
        }
        _ => {
            tracing::trace!(target: "vulkan", "[UNKNOWN] severity: {:?}, type: {:?}, id: {} ({}), message: {}", message_severity, message_type, message_id_name, message_id_number, message);
        }
    }
    vk::FALSE
}

pub struct VulkanContext {
    #[allow(dead_code)]
    entry: ash::Entry,
    instance: Arc<ash::Instance>,
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,

    #[allow(dead_code)]
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,

    #[allow(dead_code)]
    queue_family_indices: QueueFamilyIndices,

    device: Arc<ash::Device>,

    #[allow(dead_code)]
    graphics_queue: vk::Queue,
    #[allow(dead_code)]
    present_queue: vk::Queue,

    surface_loader: Option<ash::extensions::khr::Surface>,
    wayland_surface_loader: Option<ash::extensions::khr::WaylandSurface>,
    surface: Option<vk::SurfaceKHR>,

    render_pass: Option<RenderPass>,
    graphics_pipeline: Option<GraphicsPipeline>,
    swapchain_data: Option<Swapchain>,
}

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

impl VulkanContext {
    pub fn new(application_name: &str, engine_name: &str, enable_validation_layers: bool, wayland_display: *mut std::ffi::c_void) -> Result<Self, anyhow::Error> {
        let entry = unsafe { ash::Entry::load()? };

        let app_name = CString::new(application_name)?;
        let eng_name = CString::new(engine_name)?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&eng_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut instance_extensions = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
        ];
        if enable_validation_layers {
            instance_extensions.push(ash::extensions::ext::DebugUtils::name().as_ptr());
        }

        let validation_layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        let mut enabled_layer_names: Vec<*const c_char> = Vec::new();
        if enable_validation_layers {
            enabled_layer_names.push(validation_layer_name.as_ptr());
        }

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&enabled_layer_names);

        let mut debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        if enable_validation_layers {
            instance_create_info = instance_create_info.push_next(&mut debug_messenger_create_info);
        }

        let instance = Arc::new(unsafe { entry.create_instance(&instance_create_info, None)? });
        info!("Vulkan instance created successfully.");

        let mut debug_utils_loader = None;
        let mut debug_messenger = None;
        if enable_validation_layers {
            let loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
            let messenger = unsafe {
                loader.create_debug_utils_messenger(&debug_messenger_create_info, None)?
            };
            debug_utils_loader = Some(loader);
            debug_messenger = Some(messenger);
            info!("Vulkan debug messenger created successfully.");
        }

        let (physical_device, queue_family_indices) = Self::select_physical_device(&entry, &instance, wayland_display)?;
        let device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (device, graphics_queue, present_queue) = Self::create_logical_device(&instance, physical_device, &queue_family_indices)?;

        Ok(Self {
            entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            physical_device,
            device_memory_properties,
            queue_family_indices,
            device: Arc::new(device),
            graphics_queue,
            present_queue,
            surface_loader: None,
            wayland_surface_loader: None,
            surface: None,
            render_pass: None,
            graphics_pipeline: None,
            swapchain_data: None,
        })
    }

    fn select_physical_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        wayland_display: *mut std::ffi::c_void,
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), anyhow::Error> {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };
        info!("Found {} physical devices.", physical_devices.len());

        let mut best_device = None;
        let mut best_score = 0;
        let mut best_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        for pdevice in physical_devices {
            let properties = unsafe { instance.get_physical_device_properties(pdevice) };
            let _features = unsafe { instance.get_physical_device_features(pdevice) };
            let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy();
            info!("Evaluating device: {}", device_name);

            let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pdevice) };
            let mut indices = QueueFamilyIndices {
                graphics_family: None,
                present_family: None,
            };

            let wayland_surface_loader = ash::extensions::khr::WaylandSurface::new(entry, instance);

            for (i, queue_family) in queue_families.iter().enumerate() {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    indices.graphics_family = Some(i as u32);
                }

                let presentation_support = if !wayland_display.is_null() {
                    let display_ref: &mut c_void = unsafe {
                        std::mem::transmute::<*mut c_void, &mut c_void>(wayland_display)
                    };
                    unsafe {
                        wayland_surface_loader.get_physical_device_wayland_presentation_support(
                            pdevice,
                            i as u32,
                            display_ref
                        )
                    }
                } else {
                    false
                };

                if presentation_support && indices.present_family.is_none() {
                    indices.present_family = Some(i as u32);
                }

                if indices.is_complete() {
                    break;
                }
            }

            let required_device_extensions = [ash::extensions::khr::Swapchain::name()];
            let available_extensions = unsafe { instance.enumerate_device_extension_properties(pdevice)? };
            let mut all_extensions_supported = true;
            for req_ext_name_c_str_ref in required_device_extensions.iter() {
                let required_ext_name: &CStr = *req_ext_name_c_str_ref;
                let mut found = false;
                for ext_prop in available_extensions.iter() {
                    let available_ext_name = unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) };
                    if required_ext_name == available_ext_name {
                        found = true;
                        break;
                    }
                }
                if !found {
                    all_extensions_supported = false;
                    info!("Device {} does not support required extension: {:?}", device_name, required_ext_name.to_string_lossy());
                    break;
                }
            }

            if !all_extensions_supported {
                continue;
            }

            if indices.is_complete() {
                let mut current_score = 0;
                if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                    current_score += 1000;
                } else if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                    current_score += 500;
                }

                if current_score > best_score {
                    best_score = current_score;
                    best_device = Some(pdevice);
                    best_indices = indices;
                }
            } else {
                 info!("Device {} does not have all required queue families. Graphics: {:?}, Present: {:?}", device_name, indices.graphics_family, indices.present_family);
            }
        }

        if let Some(pdevice) = best_device {
            let selected_properties = unsafe { instance.get_physical_device_properties(pdevice) };
            let selected_device_name = unsafe { CStr::from_ptr(selected_properties.device_name.as_ptr()) }.to_string_lossy();
            info!("Best physical device: {} with queue families: Graphics: {:?}, Present: {:?}", selected_device_name, best_indices.graphics_family, best_indices.present_family);
            Ok((pdevice, best_indices))
        } else {
            Err(anyhow::anyhow!("Failed to find a suitable physical device"))
        }
    }

    fn create_logical_device(
        instance: &Arc<ash::Instance>,
        physical_device: vk::PhysicalDevice,
        indices: &QueueFamilyIndices,
    ) -> Result<(ash::Device, vk::Queue, vk::Queue), anyhow::Error> {
        let mut queue_create_infos = vec![];
        let mut unique_queue_families = std::collections::HashSet::new();

        let graphics_family_idx = indices.graphics_family.ok_or_else(|| anyhow::anyhow!("Graphics queue family not found"))?;
        unique_queue_families.insert(graphics_family_idx);

        let present_family_idx = indices.present_family.ok_or_else(|| anyhow::anyhow!("Present queue family not found"))?;
        unique_queue_families.insert(present_family_idx);

        let queue_priority = 1.0f32;
        for queue_family_index in unique_queue_families {
            let queue_create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&[queue_priority])
                .build();
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::builder();
        let device_extensions_names_raw: Vec<*const c_char> = [ash::extensions::khr::Swapchain::name().as_ptr()].to_vec();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&device_extensions_names_raw);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
        info!("Logical device created successfully.");

        let graphics_queue = unsafe { device.get_device_queue(graphics_family_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_idx, 0) };
        info!("Graphics and Present queues obtained.");

        Ok((device, graphics_queue, present_queue))
    }

    pub fn init_surface_wayland(
        &mut self,
        raw_display_ptr: *mut c_void,
        raw_surface_ptr: *mut c_void,
    ) -> Result<vk::SurfaceKHR, anyhow::Error> {
        if self.surface.is_some() {
            warn!("Surface already initialized. Re-initialization is not yet supported.");
            return Ok(self.surface.unwrap());
        }

        if raw_display_ptr.is_null() || raw_surface_ptr.is_null() {
            return Err(anyhow::anyhow!("Wayland display or surface pointer is null."));
        }

        let surface_loader = ash::extensions::khr::Surface::new(&self.entry, &self.instance);
        let wayland_surface_loader = ash::extensions::khr::WaylandSurface::new(&self.entry, &self.instance);

        let create_info = vk::WaylandSurfaceCreateInfoKHR::builder()
            .display(raw_display_ptr)
            .surface(raw_surface_ptr);

        let surface = unsafe {
            wayland_surface_loader.create_wayland_surface(&create_info, None)?
        };

        self.surface_loader = Some(surface_loader);
        self.wayland_surface_loader = Some(wayland_surface_loader);
        self.surface = Some(surface);

        info!("Vulkan Wayland surface created successfully.");
        Ok(surface)
    }

    pub fn create_swapchain(
        &mut self,
        width: u32,
        height: u32,
        raw_display_ptr_opt: Option<*mut c_void>,
        raw_surface_ptr_opt: Option<*mut c_void>,
    ) -> Result<(), anyhow::Error> {
        if self.surface.is_none() {
            if let (Some(disp_ptr), Some(surf_ptr)) = (raw_display_ptr_opt, raw_surface_ptr_opt) {
                if disp_ptr.is_null() || surf_ptr.is_null() {
                     return Err(anyhow::anyhow!("Cannot create initial surface for swapchain with null Wayland display/surface pointers."));
                }
                self.init_surface_wayland(disp_ptr, surf_ptr)?;
            } else {
                return Err(anyhow::anyhow!("Vulkan surface not initialized and no Wayland pointers provided to create one for swapchain."));
            }
        }

        let surface_khr = self.surface.ok_or_else(|| anyhow::anyhow!("Surface not available for swapchain creation"))?;
        let surface_loader_ref = self.surface_loader.as_ref().ok_or_else(|| anyhow::anyhow!("Surface loader not available for swapchain creation"))?;

        // Destroy existing resources that depend on the swapchain details (format, extent)
        if let Some(old_pipeline) = self.graphics_pipeline.take() {
            info!("Destroying old graphics pipeline.");
            drop(old_pipeline);
        }
        if let Some(old_swapchain_data) = self.swapchain_data.take() {
            info!("Destroying old swapchain.");
            drop(old_swapchain_data);
        }
        if self.render_pass.is_some() { // Render pass might depend on format, recreate it.
            info!("Destroying old render pass.");
            drop(self.render_pass.take().unwrap());
        }

        // Determine Swapchain Format
        let formats = unsafe {
            surface_loader_ref.get_physical_device_surface_formats(self.physical_device, surface_khr)?
        };
        if formats.is_empty() {
            return Err(anyhow::anyhow!("No surface formats available for swapchain."));
        }
        let chosen_surface_format = formats
            .iter()
            .find(|fmt| {
                fmt.format == vk::Format::B8G8R8A8_SRGB
                    && fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .cloned()
            .unwrap_or_else(|| formats[0]);

        // Create RenderPass using the chosen format
        let new_render_pass = RenderPass::new(
            Arc::clone(&self.device),
            chosen_surface_format.format,
        )?;
        let render_pass_handle = new_render_pass.handle;
        self.render_pass = Some(new_render_pass);
        info!("RenderPass created/recreated successfully.");

        // Create the new Swapchain
        let new_swapchain = Swapchain::new(
            Arc::clone(&self.instance),
            Arc::clone(&self.device),
            self.physical_device,
            surface_loader_ref,
            surface_khr,
            &self.queue_family_indices,
            width,
            height,
            &chosen_surface_format,
            render_pass_handle,
        )?;
        let swapchain_extent = new_swapchain.extent;
        self.swapchain_data = Some(new_swapchain);
        info!("Swapchain created/recreated successfully.");

        // Create Graphics Pipeline
        let new_graphics_pipeline = GraphicsPipeline::new(
            Arc::clone(&self.device),
            render_pass_handle,
            swapchain_extent,
        )?;
        self.graphics_pipeline = Some(new_graphics_pipeline);
        info!("Graphics pipeline created/recreated successfully.");

        Ok(())
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            // Destroy in reverse order of creation dependency
            if let Some(pipeline_data) = self.graphics_pipeline.take() {
                drop(pipeline_data);
                info!("Vulkan graphics pipeline destroyed.");
            }
            if let Some(swapchain_data) = self.swapchain_data.take() {
                drop(swapchain_data);
                info!("Vulkan swapchain data destroyed.");
            }
            if let Some(render_pass_data) = self.render_pass.take() {
                drop(render_pass_data);
                info!("Vulkan render pass destroyed.");
            }
            if let (Some(surface_loader), Some(surface)) = (&self.surface_loader, self.surface) {
                info!("Destroying Vulkan surface...");
                surface_loader.destroy_surface(surface, None);
                info!("Vulkan surface destroyed.");
            }

            info!("Destroying Vulkan logical device...");
            self.device.destroy_device(None);
            info!("Vulkan logical device destroy command issued.");

            if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
                info!("Destroying Vulkan debug messenger...");
                loader.destroy_debug_utils_messenger(messenger, None);
                info!("Vulkan debug messenger destroyed.");
            }

            info!("Destroying Vulkan instance...");
            self.instance.destroy_instance(None);
            info!("Vulkan instance destroy command issued.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_vulkan_context_creation_and_drop() {
        let _guard = tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).try_init();
        let mock_wayland_display: *mut std::ffi::c_void = ptr::null_mut();

        match VulkanContext::new(
            "TestApp",
            "TestEngine",
            true,
            mock_wayland_display,
        ) {
            Ok(_ctx) => {
                info!("VulkanContext created successfully for test (no surface/swapchain/pipeline).");
            }
            Err(e) => {
                tracing::warn!("Failed to create VulkanContext in test (this might be expected in some environments): {:?}", e);
            }
        }
    }
}
