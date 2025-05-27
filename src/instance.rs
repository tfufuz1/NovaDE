use crate::error::{Result, VulkanError};
use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_void;
use std::sync::Arc;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrWaylandSurfaceExtension};

// Optional: For portability, though Wayland is primary
#[cfg(target_os = "macos")]
use vulkanalia::vk::KhrPortabilityEnumerationExtension;


// Define a struct to hold the Vulkan entry points, instance, and debug messenger.
pub struct VulkanInstance {
    entry: Entry,
    instance: Arc<Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    pub fn new(window_title: &str, enable_validation_layers: bool) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY) }
            .map_err(|e| VulkanError::Message(format!("Failed to load Vulkan library: {}", e)))?;
        let entry = Entry::new(loader).map_err(|e| VulkanError::Message(format!("Failed to create Vulkan entry: {}", e)))?;

        let application_name = CString::new(window_title).unwrap();
        let engine_name = CString::new("NovaDE Renderer").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::API_VERSION_1_3); // As per Rendering Vulkan.md

        let mut layers = Vec::new();
        if enable_validation_layers {
            layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap().as_ptr());
        }

        // Check for layer support
        if enable_validation_layers && !Self::check_validation_layer_support(&entry, &layers)? {
            return Err(VulkanError::Message("Validation layers requested, but not available!".to_string()));
        }

        let mut extensions = vec![
            KhrSurfaceExtension::name().as_ptr(),
            KhrWaylandSurfaceExtension::name().as_ptr(), // For Wayland
            ExtDebugUtilsExtension::name().as_ptr(),
        ];

        // For macOS portability, if needed, though target is Linux/Wayland
        #[cfg(target_os = "macos")]
        {
            extensions.push(KhrPortabilityEnumerationExtension::name().as_ptr());
        }


        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);

        // Handle portability flags for macOS
        #[cfg(target_os = "macos")]
        {
            instance_create_info = instance_create_info.flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        }


        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .map_err(|e| VulkanError::VkResult(e))?;
        let instance = Arc::new(instance);

        let debug_messenger = if enable_validation_layers {
            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO // Optional
                        // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE, // Optional
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .user_callback(Some(vulkan_debug_callback));

            Some(unsafe { instance.create_debug_utils_messenger_ext(&messenger_info, None) }
                .map_err(|e| VulkanError::VkResult(e))?)
        } else {
            None
        };
        
        log::info!("Vulkan Instance created successfully.");
        if enable_validation_layers {
            log::info!("Validation layers enabled.");
        }


        Ok(Self { entry, instance, debug_messenger })
    }

    fn check_validation_layer_support(entry: &Entry, layers_cchar: &[*const c_char]) -> Result<bool> {
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
            .map_err(|e| VulkanError::VkResult(e))?
            .iter()
            .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
            .collect::<Vec<_>>();

        for required_layer_ptr in layers_cchar {
            let required_layer_name = unsafe { CStr::from_ptr(*required_layer_ptr) };
            if !available_layers.contains(&required_layer_name) {
                log::warn!("Validation layer not available: {:?}", required_layer_name);
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    // Getter for the raw Vulkan instance
    pub fn raw(&self) -> &Arc<Instance> {
        &self.instance
    }

    // Getter for the entry points
    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        if let Some(messenger) = self.debug_messenger {
            unsafe { self.instance.destroy_debug_utils_messenger_ext(messenger, None) };
            log::debug!("Debug messenger destroyed.");
        }
        unsafe { self.instance.destroy_instance(None) };
        log::debug!("Vulkan instance destroyed.");
    }
}

// Debug callback function
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*data).message).to_string_lossy();
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("[VULKAN VALIDATION] ({:?}): {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("[VULKAN VALIDATION] ({:?}): {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("[VULKAN VALIDATION] ({:?}): {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::trace!("[VULKAN VALIDATION] ({:?}): {}", message_type, message),
        _ => log::info!("[VULKAN VALIDATION] ({:?}): {}", message_type, message),
    }
    vk::FALSE // Do not abort the Vulkan call that triggered the callback
}
