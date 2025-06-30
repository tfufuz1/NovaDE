// novade-system/src/renderers/vulkan/instance.rs
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::vk;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
// use std::ptr; // ptr is not directly used, CString::as_ptr is used.
// use raw_window_handle::{WaylandDisplayHandle, HasRawDisplayHandle}; // Not directly used in this file after previous changes

// Temporary struct to satisfy HasRawDisplayHandle for testing instance extensions.
// In a real scenario, this would come from Smithay's Wayland integration.
// struct DummyWaylandDisplay; // Not strictly needed for instance creation logic itself
// unsafe impl HasRawDisplayHandle for DummyWaylandDisplay {
//     fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
//         let mut wh = WaylandDisplayHandle::empty();
//         raw_window_handle::RawDisplayHandle::Wayland(wh)
//     }
// }


const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

/// unsafe extern "system" fn vulkan_debug_callback
///
/// This is the callback function invoked by the Vulkan validation layers when a debug message is generated.
/// It prints the message to stderr, prefixed with severity and type.
///
/// # Arguments
/// * `message_severity`: The severity of the message (e.g., ERROR, WARNING, INFO, VERBOSE).
/// * `message_type`: The type of the message (e.g., GENERAL, VALIDATION, PERFORMANCE).
/// * `p_callback_data`: Pointer to a `VkDebugUtilsMessengerCallbackDataEXT` struct containing details about the message.
/// * `_p_user_data`: User data pointer, not used in this implementation.
///
/// # Returns
/// `vk::Bool32` (effectively `vk::FALSE`) indicating that the Vulkan call triggering the message should not be aborted.
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;
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

    // TODO: Integrate with a proper logger, e.g., tracing crate
    eprintln!( // Changed to eprintln for debug messages
        "VULKAN DEBUG: [{:?} {:?}] ({}: {}) {}",
        message_severity,
        message_type,
        message_id_name,
        message_id_number,
        message
    );
    vk::FALSE
}

/// Creates and loads a Vulkan entry point.
///
/// The entry point is required to initialize the Vulkan library and load global function pointers.
/// It's the very first step in using the Vulkan API.
pub fn create_entry() -> Result<ash::Entry, String> {
    unsafe {
        ash::Entry::load().map_err(|e| format!("Failed to load Vulkan entry: {}", e))
    }
}

/// Checks if all required instance extensions are supported by the Vulkan implementation.
///
/// # Arguments
/// * `entry`: A reference to the loaded `ash::Entry` point.
/// * `required_extensions_cstrs`: A slice of `CStr` references representing the names of required extensions.
///
/// # Returns
/// `Ok(())` if all required extensions are supported, otherwise an `Err` with a message.
fn check_required_extensions_support(entry: &ash::Entry, required_extensions_cstrs: &[&CStr]) -> Result<(), String> {
    let available_extensions = entry
        .enumerate_instance_extension_properties(None)
        .map_err(|e| format!("Failed to enumerate instance extensions: {}", e))?;

    for required_ext_name_cstr in required_extensions_cstrs {
        let found = available_extensions.iter().any(|props| {
            let avail_ext_name = unsafe { CStr::from_ptr(props.extension_name.as_ptr()) };
            avail_ext_name == *required_ext_name_cstr
        });

        if !found {
            return Err(format!(
                "Required instance extension not supported: {:?}",
                required_ext_name_cstr
            ));
        }
    }
    Ok(())
}

/// Creates a Vulkan instance.
///
/// This function initializes the core Vulkan API, sets up application information,
/// enables necessary instance-level extensions (like surface and Wayland integration),
/// and configures validation layers with a debug messenger if enabled in debug builds.
/// It adheres to the specifications outlined in `Rendering Vulkan.md`, particularly section 3.1.
///
/// # Arguments
///
/// * `entry` - A reference to the loaded `ash::Entry` point.
///
/// # Returns
///
/// A `Result` containing a tuple with:
/// 1. The created `ash::Instance`.
/// 2. An `Option<ash::extensions::ext::DebugUtils>` loader (Some if validation enabled).
/// 3. An `Option<vk::DebugUtilsMessengerEXT>` handle (Some if validation enabled).
/// Returns an error string on failure (e.g., validation layers not supported, required extensions missing).
///
/// # `Rendering Vulkan.md` Specification Mapping:
/// - **Spec 3.1 (Vulkan-Instanz-Erstellung):**
///   - `VkApplicationInfo`: Configured with "SmithayCompositor" as application name,
///     "SmithayVulkanBackend" as engine name. `apiVersion` is set to `VK_API_VERSION_1_3`.
///     Application and engine versions are set to `0.1.0.0` (project specific) rather than `1.0.0.0` from spec
///     to maintain consistency with potential overall project versioning.
///   - `VkInstanceCreateInfo`: Enables `VK_KHR_surface`, `VK_KHR_wayland_surface` (platform-specific),
///     and `VK_EXT_debug_utils` (if validation is active). Support for these extensions is verified.
///   - Validation Layer: `VK_LAYER_KHRONOS_validation` is enabled in debug builds (`ENABLE_VALIDATION_LAYERS`).
///     Its availability is checked via `check_validation_layer_support`.
///   - Debug Callback (`vulkan_debug_callback`): Configured with message severities ERROR, WARNING, and VERBOSE,
///     and message types GENERAL, VALIDATION, and PERFORMANCE, as per Spec II.A and 3.1.
pub fn create_instance(entry: &ash::Entry) -> Result<(ash::Instance, Option<DebugUtils>, Option<vk::DebugUtilsMessengerEXT>), String> {
    if ENABLE_VALIDATION_LAYERS && !check_validation_layer_support(entry) {
        return Err("Validation layers requested, but not available!".to_string());
    }

    // According to Spec 3.1
    let app_name = CString::new("SmithayCompositor").unwrap();
    let engine_name = CString::new("SmithayVulkanBackend").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 1, 0, 0)) // Spec says 1,0,0 but project is 0.1.0
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))   // Spec says 1,0,0
        .api_version(vk::API_VERSION_1_3);

    // Define CString names for extensions to check
    let surface_ext_name = Surface::name();
    let wayland_surface_ext_name = ash::extensions::khr::WaylandSurface::name();
    let debug_utils_ext_name = DebugUtils::name();

    let mut required_extensions_cstrs_to_check = vec![surface_ext_name, wayland_surface_ext_name];
    if ENABLE_VALIDATION_LAYERS {
        required_extensions_cstrs_to_check.push(debug_utils_ext_name);
    }
    check_required_extensions_support(entry, &required_extensions_cstrs_to_check)?;


    let required_extensions_ptrs: Vec<*const c_char> = required_extensions_cstrs_to_check
        .iter()
        .map(|c_str| c_str.as_ptr())
        .collect();


    let layers_names_raw: Vec<*const c_char> = VALIDATION_LAYERS
        .iter()
        .map(|raw_name| raw_name.as_ptr() as *const c_char)
        .collect();

    let mut create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&required_extensions_ptrs);

    if ENABLE_VALIDATION_LAYERS {
        create_info = create_info.enabled_layer_names(&layers_names_raw);
    }

    // According to Spec "Instanz- und Debug-Messenger-Einrichtung", II.A and 3.1
    let mut debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE, // Added VERBOSE
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    if ENABLE_VALIDATION_LAYERS {
        create_info = create_info.push_next(&mut debug_utils_messenger_create_info);
    }

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(|e| format!("Failed to create Vulkan instance: {}", e))?
    };

    let (debug_utils, debug_utils_messenger) = if ENABLE_VALIDATION_LAYERS {
        let debug_utils_loader = DebugUtils::new(entry, &instance);
        let messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_utils_messenger_create_info, None)
                .map_err(|e| format!("Failed to create debug messenger: {}", e))?
        };
        (Some(debug_utils_loader), Some(messenger))
    } else {
        (None, None)
    };

    Ok((instance, debug_utils, debug_utils_messenger))
}

/// Checks if the required validation layers (specifically `VK_LAYER_KHRONOS_validation`) are supported.
///
/// # Arguments
/// * `entry`: A reference to the loaded `ash::Entry` point.
///
/// # Returns
/// `true` if all required validation layers are supported, `false` otherwise.
fn check_validation_layer_support(entry: &ash::Entry) -> bool {
    let available_layers = match entry.enumerate_instance_layer_properties() {
        Ok(layers) => layers,
        Err(e) => {
            eprintln!("Failed to enumerate instance layer properties: {}", e);
            return false;
        }
    };

    for layer_name_str in VALIDATION_LAYERS.iter() {
        // Safe to unwrap, VALIDATION_LAYERS are constant strings
        let layer_name_cstr = CString::new(*layer_name_str).unwrap();

        let found = available_layers.iter().any(|layer_properties| {
            let available_layer_name = unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) };
            available_layer_name == layer_name_cstr.as_ref() // Compare CStr directly
        });

        if !found {
            eprintln!("Validation layer not found: {}", layer_name_str);
            return false;
        }
    }
    true
}
