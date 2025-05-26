use ash::extensions::ext::DebugUtils;
use ash::vk::{self, make_api_version};
use log::{error, info, warn, trace};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr;

const APPLICATION_NAME: &str = "NovaDE Compositor";
const ENGINE_NAME: &str = "NovaDE Vulkan Renderer";
const VK_LAYER_KHRONOS_VALIDATION_NAME: &str = "VK_LAYER_KHRONOS_validation";

/// Structure holding the Vulkan instance and related objects.
pub struct VulkanInstance {
    entry: ash::Entry,
    instance: ash::Instance,
    api_version: u32, // Store the API version
    debug_utils_loader: Option<DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    /// Creates a new VulkanInstance.
    ///
    /// This function initializes the Vulkan library, creates an instance,
    /// and sets up a debug messenger if validation layers are enabled.
    pub fn new() -> Result<Self, String> {
        let entry = unsafe { ash::Entry::load() }.map_err(|e| format!("Failed to load Vulkan entry: {}", e))?;
        let api_version_to_use = vk::API_VERSION_1_3; // Define it once

        // Configure VkApplicationInfo
        let app_name = CString::new(APPLICATION_NAME).unwrap();
        let engine_name_cstr = CString::new(ENGINE_NAME).unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name_cstr)
            .engine_version(make_api_version(0, 0, 1, 0))
            .api_version(api_version_to_use);

        // Define required instance extensions
        let mut required_extensions = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ];

        // Check for extension availability
        let available_extensions = entry
            .enumerate_instance_extension_properties(None)
            .map_err(|e| format!("Failed to enumerate instance extensions: {}", e))?;

        for &required_extension_ptr in &required_extensions {
            let required_extension_name = unsafe { CStr::from_ptr(required_extension_ptr) }.to_str().unwrap();
            let found = available_extensions.iter().any(|ext| {
                unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }.to_str().unwrap() == required_extension_name
            });
            if !found {
                return Err(format!("Required instance extension not found: {}", required_extension_name));
            }
        }
        info!("Required instance extensions are available.");

        // Define validation layers (for debug builds)
        let mut enabled_layer_names: Vec<*const c_char> = Vec::new();
        let validation_layer_name_cstr = CString::new(VK_LAYER_KHRONOS_VALIDATION_NAME).unwrap();

        #[cfg(debug_assertions)]
        {
            let available_layers = entry
                .enumerate_instance_layer_properties()
                .map_err(|e| format!("Failed to enumerate instance layers: {}", e))?;

            let validation_layer_available = available_layers.iter().any(|layer| {
                unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) }.to_str().unwrap() == VK_LAYER_KHRONOS_VALIDATION_NAME
            });

            if validation_layer_available {
                info!("Validation layer '{}' is available.", VK_LAYER_KHRONOS_VALIDATION_NAME);
                enabled_layer_names.push(validation_layer_name_cstr.as_ptr());
            } else {
                warn!("Validation layer '{}' requested but not available.", VK_LAYER_KHRONOS_VALIDATION_NAME);
            }
        }

        // Configure VkInstanceCreateInfo
        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&required_extensions);

        if !enabled_layer_names.is_empty() {
            instance_create_info = instance_create_info.enabled_layer_names(&enabled_layer_names);
        }

        // Create VkInstance
        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .map_err(|e| format!("Failed to create Vulkan instance: {}", e))?;
        info!("Vulkan instance created successfully.");

        let mut debug_utils_loader = None;
        let mut debug_messenger = None;

        if !enabled_layer_names.is_empty() {
            let loader = DebugUtils::new(&entry, &instance);
            let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE, // Enable VERBOSE for debug
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
                    // Not returning an error here, as debug messenger is not critical for application startup
                    error!("Failed to create Vulkan debug messenger: {}", e);
                }
            }
            debug_utils_loader = Some(loader);
        }


        Ok(Self {
            entry,
            instance,
            api_version: api_version_to_use,
            debug_utils_loader,
            debug_messenger,
        })
    }

    /// Returns the Vulkan API version used by the instance.
    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    /// Destroys the Vulkan instance and related objects.
    pub fn destroy(&mut self) {
        unsafe {
            if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
                loader.destroy_debug_utils_messenger(messenger, None);
                info!("Vulkan debug messenger destroyed.");
            }
            self.instance.destroy_instance(None);
            info!("Vulkan instance destroyed.");
        }
    }

    // Getter for the ash::Instance, needed for other Vulkan operations
    pub fn raw(&self) -> &ash::Instance {
        &self.instance
    }

    // Getter for the ash::Entry, needed for some surface operations
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Vulkan debug callback function.
///
/// This function is called by the Vulkan validation layers to report messages.
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
        info!("{}", log_message);
    } else { // VERBOSE or unknown
        trace!("{}", log_message);
    }

    vk::FALSE // Indicate that the Vulkan call causing the validation message should not be aborted
}
