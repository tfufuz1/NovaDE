// novade-system/src/renderers/vulkan/instance.rs
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::vk::{self, make_api_version, API_VERSION_1_3};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Defines errors that can occur during Vulkan instance creation and debug messenger setup.
/// As per the detailed specification for WP-101.
#[derive(Debug, Clone)]
pub enum InstanceError {
    /// Indicates that the required validation layer (`VK_LAYER_KHRONOS_validation`) is not supported.
    ValidationLayerNotSupported,
    /// Indicates that a required instance extension is not supported.
    ExtensionNotSupported(String),
    /// Wraps a `VkResult` from a failed `vkCreateInstance` call.
    InstanceCreationFailed(vk::Result),
    /// Wraps a `VkResult` from a failed `vkCreateDebugUtilsMessengerEXT` call.
    DebugMessengerCreationFailed(vk::Result),
    /// Failure during Vulkan library loading.
    LoadingError(String),
    /// Failure to convert string to CString.
    CStringError(std::ffi::NulError),
}

impl std::fmt::Display for InstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceError::ValidationLayerNotSupported => write!(f, "Required validation layer VK_LAYER_KHRONOS_validation not supported."),
            InstanceError::ExtensionNotSupported(ext) => write!(f, "Required instance extension not supported: {}", ext),
            InstanceError::InstanceCreationFailed(res) => write!(f, "vkCreateInstance failed with VkResult: {:?}", res),
            InstanceError::DebugMessengerCreationFailed(res) => write!(f, "vkCreateDebugUtilsMessengerEXT failed with VkResult: {:?}", res),
            InstanceError::LoadingError(msg) => write!(f, "Failed to load Vulkan library: {}", msg),
            InstanceError::CStringError(e) => write!(f, "Failed to create CString: {}", e),
        }
    }
}
impl std::error::Error for InstanceError {}


const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS_FLAG: bool = true; // Use a distinct name from the new spec's fn param
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS_FLAG: bool = false;

/// Vulkan debug callback function.
/// Prints validation layer messages to stderr.
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = &*p_callback_data; // Use reference
    let message_id_number: i32 = callback_data.message_id_number; // No cast needed
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("null")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("null")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    eprintln!(
        "VULKAN DEBUG: [{:?} {:?}] ({}: {}) {}",
        message_severity, message_type, message_id_name, message_id_number, message
    );
    vk::FALSE
}

/// Creates and loads a Vulkan entry point.
pub fn create_entry() -> Result<ash::Entry, InstanceError> {
    unsafe { ash::Entry::load().map_err(|e| InstanceError::LoadingError(e.to_string())) }
}

/// Checks if required validation layers are supported.
/// Conforms to Step 1 of WP-101 detailed specification.
fn check_validation_layer_support_internal(entry: &ash::Entry) -> Result<(), InstanceError> {
    let available_layers = entry.enumerate_instance_layer_properties()
        .map_err(|e| InstanceError::InstanceCreationFailed(e))?; // Or a more specific error

    for layer_name_str in VALIDATION_LAYERS.iter() {
        let layer_name_cstr = CString::new(*layer_name_str).map_err(InstanceError::CStringError)?;
        let found = available_layers.iter().any(|layer_properties| {
            let available_layer_name = unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) };
            available_layer_name == layer_name_cstr.as_c_str()
        });
        if !found {
            return Err(InstanceError::ValidationLayerNotSupported);
        }
    }
    Ok(())
}

/// Checks if required instance extensions are supported.
fn check_required_extensions_support_internal(entry: &ash::Entry, required_extensions_cstrs: &[CString]) -> Result<(), InstanceError> {
    let available_extensions = entry.enumerate_instance_extension_properties(None)
        .map_err(|e| InstanceError::InstanceCreationFailed(e))?; // Or a more specific error

    for required_ext_name_cstr in required_extensions_cstrs {
        let found = available_extensions.iter().any(|props| {
            let avail_ext_name = unsafe { CStr::from_ptr(props.extension_name.as_ptr()) };
            avail_ext_name == required_ext_name_cstr.as_c_str()
        });
        if !found {
            return Err(InstanceError::ExtensionNotSupported(required_ext_name_cstr.to_string_lossy().into_owned()));
        }
    }
    Ok(())
}

/// Initializes the Vulkan library by creating a `VkInstance`.
/// Optionally configures and registers a `VkDebugUtilsMessengerEXT`.
/// This function is the first step for Vulkan API interaction.
/// Conforms to the detailed specification for WP-101.
///
/// # Arguments
/// * `entry`: The loaded Vulkan entry point.
/// * `app_name_str`: The name of the application.
/// * `engine_name_str`: The name of the engine.
/// * `enable_validation_layers_param`: Flag to enable validation layers.
///
/// # Returns
/// A tuple containing the `ash::Instance` and an optional tuple of
/// `(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)` if validation is enabled.
pub fn init_vulkan_instance(
    entry: &ash::Entry,
    app_name_str: &str,
    engine_name_str: &str,
    enable_validation_layers_param: bool, // Parameter from new spec
) -> Result<(ash::Instance, Option<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)>), InstanceError> {

    // Step 1: Check validation layers (if requested by param)
    if enable_validation_layers_param {
        check_validation_layer_support_internal(entry)?;
    }

    // Step 2: Configure VkApplicationInfo
    let app_name_c = CString::new(app_name_str).map_err(InstanceError::CStringError)?;
    let engine_name_c = CString::new(engine_name_str).map_err(InstanceError::CStringError)?;

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name_c)
        .application_version(make_api_version(0, 1, 0, 0)) // Kept project version
        .engine_name(&engine_name_c)
        .engine_version(make_api_version(0, 1, 0, 0)) // Kept project version
        .api_version(API_VERSION_1_3);

    // Step 3: Configure VkInstanceCreateInfo
    // Step 3.d, 3.e: Collect required instance extensions
    let mut required_extensions_cstrings = vec![
        Surface::name().to_owned(), // CString from CStr
        ash::extensions::khr::WaylandSurface::name().to_owned(),
    ];
    if enable_validation_layers_param {
        required_extensions_cstrings.push(DebugUtils::name().to_owned());
    }
    check_required_extensions_support_internal(entry, &required_extensions_cstrings)?;

    let required_extensions_ptrs: Vec<*const c_char> = required_extensions_cstrings
        .iter()
        .map(|c_str| c_str.as_ptr())
        .collect();

    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&required_extensions_ptrs);

    // Step 3.g: Debug Messenger for instance creation
    let mut debug_messenger_create_info_for_instance = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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

    if enable_validation_layers_param {
        instance_create_info = instance_create_info.push_next(&mut debug_messenger_create_info_for_instance);

        // Step 3.h: Assign validation layers
        let layers_names_raw: Vec<*const c_char> = VALIDATION_LAYERS
            .iter()
            .map(|raw_name| raw_name.as_ptr() as *const c_char) // Assuming VALIDATION_LAYERS are CStr compatible
            .collect();
        instance_create_info = instance_create_info.enabled_layer_names(&layers_names_raw);
    }

    // Step 4: Create Vulkan Instance
    let instance = unsafe {
        entry.create_instance(&instance_create_info, None)
    }.map_err(InstanceError::InstanceCreationFailed)?;

    // Step 5: Setup Debug Messenger (if validation enabled)
    // This part is slightly different from the spec's "load vkCreateDebugUtilsMessengerEXT dynamically"
    // because ash provides a loader struct `DebugUtils`.
    // The spec's step 3.g.ii already sets up a messenger for create/destroy.
    // This second messenger is the main one for runtime messages.
    let mut debug_loader_and_messenger = None;
    if enable_validation_layers_param {
        let debug_utils_loader = DebugUtils::new(entry, &instance);
        // Use the same create info struct as for instance creation pNext chain
        let messenger = unsafe {
            debug_utils_loader.create_debug_utils_messenger(&debug_messenger_create_info_for_instance, None)
        }.map_err(InstanceError::DebugMessengerCreationFailed)?;
        debug_loader_and_messenger = Some((debug_utils_loader, messenger));
    }

    // Step 6: Success
    Ok((instance, debug_loader_and_messenger))
}
