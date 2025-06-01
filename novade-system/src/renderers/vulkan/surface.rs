// novade-system/src/renderers/vulkan/surface.rs
use ash::extensions::khr::Surface as SurfaceLoader; // Alias to avoid conflict
use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    raw_display_handle_provider: &impl HasRawDisplayHandle,
    raw_window_handle_provider: &impl HasRawWindowHandle,
) -> Result<vk::SurfaceKHR, String> {
    unsafe {
        ash_window::create_surface(
            entry,
            instance,
            raw_display_handle_provider.raw_display_handle(),
            raw_window_handle_provider.raw_window_handle(),
            None,
        )
        .map_err(|e| format!("Failed to create Vulkan surface: {}", e))
    }
}
