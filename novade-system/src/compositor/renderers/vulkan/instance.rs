//! Manages the Vulkan instance and basic setup, including validation layers and debug messaging.
//!
//! This module is responsible for loading the Vulkan library, creating a `VkInstance`,
//! and setting up essential global features like validation layers (for debug builds)
//! and a debug messenger callback to log Vulkan messages. The `VulkanInstance` struct
//! is the primary entry point for these operations and holds the created Vulkan instance.

use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use ash::extensions::ext::DebugUtils;
use ash::vk::{self, make_api_version};
use log::{error, info, warn, trace};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// Name of the application, used in `VkApplicationInfo`.
const APPLICATION_NAME: &str = "NovaDE Compositor";
/// Name of the rendering engine, used in `VkApplicationInfo`.
const ENGINE_NAME: &str = "NovaDE Vulkan Renderer";
/// Standard Khronos validation layer name.
const VK_LAYER_KHRONOS_VALIDATION_NAME: &str = "VK_LAYER_KHRONOS_validation";

/// Structure holding the Vulkan instance, entry points, and debug utilities.
///
/// It encapsulates the `ash::Entry` for loading Vulkan functions, the `ash::Instance`
/// representing the active Vulkan instance, and optionally, the debug messenger
/// components if validation layers are enabled.
#[derive(Debug)]
pub struct VulkanInstance {
    /// Entry point for loading Vulkan functions.
    entry: ash::Entry,
    /// The Vulkan instance handle.
    instance: ash::Instance,
    /// The Vulkan API version used by the instance.
    api_version: u32,
    /// Optional loader for the `VK_EXT_debug_utils` extension.
    /// This is `Some` if validation layers and the debug utils extension are enabled.
    debug_utils_loader: Option<DebugUtils>,
    /// Optional handle to the debug messenger.
    /// This is `Some` if a debug messenger was successfully created.
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    /// Creates a new `VulkanInstance`.
    ///
    /// This function initializes the Vulkan library by loading the entry points,
    /// configures `VkApplicationInfo`, checks for and enables required instance extensions
    /// (like surface and Wayland surface extensions, and debug utils), and attempts to enable
    /// validation layers for debug builds. Finally, it creates the `VkInstance`.
    /// If validation is enabled, it also sets up a debug messenger.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `VulkanInstance` on success, or a `VulkanError` on failure.
    /// Possible errors include:
    /// - `VulkanError::InitializationError`: If Vulkan library loading fails.
    /// - `VulkanError::MissingExtension`: If a required instance extension is not available.
    /// - `VulkanError::VkResult`: For other Vulkan API call failures during instance creation or debug messenger setup.
    pub fn new() -> Result<Self> {
        let entry = unsafe { ash::Entry::load() }
            .map_err(|e| VulkanError::InitializationError(format!("Failed to load Vulkan entry: {}", e)))?;
        let api_version_to_use = vk::API_VERSION_1_3;

        let app_name = CString::new(APPLICATION_NAME).unwrap(); // Should not fail for valid const strings
        let engine_name_cstr = CString::new(ENGINE_NAME).unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(make_api_version(0, 0, 1, 0)) // Version 0.1.0 of NovaDE Compositor
            .engine_name(&engine_name_cstr)
            .engine_version(make_api_version(0, 0, 1, 0)) // Version 0.1.0 of NovaDE Vulkan Renderer
            .api_version(api_version_to_use);

        let mut required_extensions = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(), // For Wayland integration
            DebugUtils::name().as_ptr(), // For debug messenger
        ];

        let available_extensions = entry.enumerate_instance_extension_properties(None)?;
        for &required_extension_ptr in &required_extensions {
            let required_extension_name = unsafe { CStr::from_ptr(required_extension_ptr) }.to_str().unwrap_or("InvalidExtensionName");
            let found = available_extensions.iter().any(|ext| {
                unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }.to_str().unwrap_or("") == required_extension_name
            });
            if !found {
                error!("Required Vulkan instance extension not found: {}", required_extension_name);
                return Err(VulkanError::MissingExtension(required_extension_name.to_string()));
            }
        }
        info!("All required instance extensions are available.");

        let mut enabled_layer_names: Vec<*const c_char> = Vec::new();
        let validation_layer_name_cstr = CString::new(VK_LAYER_KHRONOS_VALIDATION_NAME).unwrap();
        #[cfg(debug_assertions)]
        {
            let available_layers = entry.enumerate_instance_layer_properties()?;
            let validation_layer_available = available_layers.iter().any(|layer| {
                unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) }.to_str().unwrap_or("") == VK_LAYER_KHRONOS_VALIDATION_NAME
            });
            if validation_layer_available {
                info!("Validation layer '{}' is available.", VK_LAYER_KHRONOS_VALIDATION_NAME);
                enabled_layer_names.push(validation_layer_name_cstr.as_ptr());
            } else {
                warn!("Validation layer '{}' requested but not available.", VK_LAYER_KHRONOS_VALIDATION_NAME);
            }
        }

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&required_extensions);
        if !enabled_layer_names.is_empty() {
            instance_create_info = instance_create_info.enabled_layer_names(&enabled_layer_names);
        }

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }?;
        info!("Vulkan instance created successfully.");

        let mut debug_utils_loader = None;
        let mut debug_messenger = None;
        if !enabled_layer_names.is_empty() {
            let loader = DebugUtils::new(&entry, &instance);
            let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO // Often useful for general info
                        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE, // Enable VERBOSE for detailed debug
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_utils_callback));

            match unsafe { loader.create_debug_utils_messenger(&messenger_create_info, None) } {
                Ok(messenger) => {
                    info!("Successfully created Vulkan debug messenger.");
                    debug_messenger = Some(messenger);
                }
                Err(e) => {
                    // Log error but don't fail instance creation, as debug messenger is not critical.
                    error!("Failed to create Vulkan debug messenger: {:?}", e);
                }
            }
            debug_utils_loader = Some(loader);
        }

        Ok(Self {
            entry, instance, api_version: api_version_to_use,
            debug_utils_loader, debug_messenger,
        })
    }

    /// Returns the Vulkan API version used by the instance.
    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    /// Destroys the Vulkan instance and related objects, such as the debug messenger.
    ///
    /// # Safety
    ///
    /// This method performs `unsafe` Vulkan calls to destroy resources.
    /// It must be called before the `VulkanInstance` is dropped if manual cleanup is preferred,
    /// though `Drop` is also implemented for automatic cleanup.
    /// Ensure that all Vulkan objects created using this instance are destroyed before
    /// calling this method or dropping the `VulkanInstance`.
    pub fn destroy(&mut self) {
        unsafe {
            if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
                loader.destroy_debug_utils_messenger(messenger, None);
                info!("Vulkan debug messenger destroyed.");
            }
            self.instance.destroy_instance(None);
            info!("Vulkan instance destroyed.");
        }
        // Set Option fields to None to prevent double-free if destroy is called manually before drop
        self.debug_messenger = None; 
        self.debug_utils_loader = None;
    }

    /// Provides raw access to the underlying `ash::Instance`.
    ///
    /// This is useful for operations that require the `ash::Instance` handle directly.
    pub fn raw(&self) -> &ash::Instance {
        &self.instance
    }

    /// Provides raw access to the `ash::Entry` loader.
    ///
    /// This is needed for some operations, particularly for loading instance-level function pointers
    /// for extensions that are not directly wrapped by `ash::Instance` (though `ash` covers many).
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }
}

impl Drop for VulkanInstance {
    /// Ensures that Vulkan resources are cleaned up when the `VulkanInstance` goes out of scope.
    ///
    /// This method calls `destroy()` to release the debug messenger (if active) and the Vulkan instance.
    fn drop(&mut self) {
        // Check if debug_messenger is Some to avoid calling destroy_debug_utils_messenger on already destroyed or None.
        // This handles cases where destroy() might have been called manually.
        if self.debug_messenger.is_some() || self.debug_utils_loader.is_some() {
             info!("Dropping VulkanInstance: Performing cleanup via destroy().");
             self.destroy(); // Call explicit destroy to ensure proper order and logging.
        } else {
            // If destroy() was already called, instance handle might still be valid if only debug utils were None.
            // However, destroy() should handle the full cleanup.
            // This path is more for if destroy() was somehow bypassed or partially completed,
            // though with current destroy(), this is less likely.
            // For safety, if instance itself is still considered "live" (not a null handle, though ash doesn't expose that),
            // one might re-call destroy_instance. But destroy() should be comprehensive.
             info!("Dropping VulkanInstance: Resources assumed already cleaned or not initialized (debug utils).");
        }
    }
}

/// Vulkan debug callback function.
///
/// This function is invoked by the Vulkan validation layers to report messages
/// (errors, warnings, information, verbose output) based on the configured severity
/// and type flags during debug messenger setup.
///
/// # Safety
///
/// This is an `unsafe extern "system" fn` as required by Vulkan for callbacks.
/// The `p_callback_data` pointer must be valid when dereferenced.
/// The function must return `vk::FALSE` to indicate that the Vulkan call that
/// triggered the validation message should not be aborted.
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let severity_str = format!("{:?}", message_severity).to_lowercase();
    let type_str = format!("{:?}", message_type).to_lowercase();

    let log_message = format!("[Vulkan][{}][{}] {:?}", severity_str, type_str, message);

    if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        error!("{}", log_message);
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        warn!("{}", log_message);
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
        info!("{}", log_message); // For INFO level messages
    } else { // VERBOSE or unknown
        trace!("{}", log_message); // For VERBOSE messages
    }

    vk::FALSE // Do not abort the Vulkan call that triggered the validation message
}
