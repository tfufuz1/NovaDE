// novade-system/src/renderers/vulkan/surface.rs
// use ash::extensions::khr::Surface as SurfaceLoader; // Not directly used, ash_window handles it.
use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

/// Creates a Vulkan surface (`vk::SurfaceKHR`) for rendering.
///
/// This function uses `ash_window::create_surface` to abstract the platform-specific
/// surface creation details. It relies on the `raw-window-handle` traits
/// (`HasRawDisplayHandle`, `HasRawWindowHandle`) to obtain the necessary native
/// display and window handles (e.g., `wl_display` and `wl_surface` for Wayland)
/// from the provided providers.
///
/// This aligns with `Rendering Vulkan.md` (Spec 5.1) for Wayland surface creation.
///
/// # Arguments
/// * `entry`: A reference to the loaded `ash::Entry` point.
/// * `instance`: A reference to the `ash::Instance`.
/// * `raw_display_handle_provider`: A provider that implements `HasRawDisplayHandle`
///   (e.g., a Smithay Wayland compositor object for `wl_display`).
/// * `raw_window_handle_provider`: A provider that implements `HasRawWindowHandle`
///   (e.g., a Smithay Wayland surface object for `wl_surface`).
///
/// # Returns
/// A `Result` containing the created `vk::SurfaceKHR` handle, or an error string
/// if surface creation fails.
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
            None, // Allocator callbacks, not used here
        )
        .map_err(|e| format!("Failed to create Vulkan surface: {}", e))
    }
}
