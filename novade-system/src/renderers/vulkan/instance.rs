// novade-system/src/renderers/vulkan/instance.rs
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::vk;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use raw_window_handle::{WaylandDisplayHandle, HasRawDisplayHandle}; // For Wayland display handle

// Temporary struct to satisfy HasRawDisplayHandle for testing instance extensions.
// In a real scenario, this would come from Smithay's Wayland integration.
struct DummyWaylandDisplay;
unsafe impl HasRawDisplayHandle for DummyWaylandDisplay {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        let mut wh = WaylandDisplayHandle::empty();
        // wh.display = /* some non-null pointer, e.g., Box::into_raw(Box::new(0)) for testing purposes only */;
        // This is tricky without a real Wayland display. For instance creation itself,
        // it's not strictly needed unless getting surface extensions.
        // Let's assume for now Wayland extensions are requested but not strictly validated here.
        raw_window_handle::RawDisplayHandle::Wayland(wh)
    }
}


const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

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

    println!(
        "[{:?} {:?}] ({}: {}) {}",
        message_severity,
        message_type,
        message_id_name,
        message_id_number,
        message
    );
    vk::FALSE
}


pub fn create_entry() -> Result<ash::Entry, String> {
    unsafe {
        ash::Entry::load().map_err(|e| format!("Failed to load Vulkan entry: {}", e))
    }
}

pub fn create_instance(entry: &ash::Entry) -> Result<(ash::Instance, Option<DebugUtils>, Option<vk::DebugUtilsMessengerEXT>), String> {
    if ENABLE_VALIDATION_LAYERS && !check_validation_layer_support(entry) {
        return Err("Validation layers requested, but not available!".to_string());
    }

    let app_name = CString::new("NovaDE Compositor").unwrap();
    let engine_name = CString::new("NovaDE Engine").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_3); // Request Vulkan 1.3

    let mut required_extensions = vec![
        Surface::name().as_ptr(),
        ash::extensions::khr::WaylandSurface::name().as_ptr(), // For Wayland surface
    ];
    if ENABLE_VALIDATION_LAYERS {
        required_extensions.push(DebugUtils::name().as_ptr());
    }

    // TODO: Check if these extensions are actually available using entry.enumerate_instance_extension_properties()

    let layers_names_raw: Vec<*const c_char> = VALIDATION_LAYERS
        .iter()
        .map(|raw_name| raw_name.as_ptr() as *const c_char)
        .collect();

    let mut create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&required_extensions);

    if ENABLE_VALIDATION_LAYERS {
        create_info = create_info.enabled_layer_names(&layers_names_raw);
    }

    let mut debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
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

fn check_validation_layer_support(entry: &ash::Entry) -> bool {
    let available_layers = match entry.enumerate_instance_layer_properties() {
        Ok(layers) => layers,
        Err(_) => return false,
    };

    for layer_name_str in VALIDATION_LAYERS.iter() {
        let layer_name_cstr = CString::new(*layer_name_str).unwrap();
        let layer_name_raw = layer_name_cstr.as_ptr();

        let found = available_layers.iter().any(|layer_properties| {
            let available_layer_name = unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) };
            available_layer_name.to_bytes_with_nul() == layer_name_cstr.as_bytes_with_nul()
        });

        if !found {
            return false;
        }
    }
    true
}
