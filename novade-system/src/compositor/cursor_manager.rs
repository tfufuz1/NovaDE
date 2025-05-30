// novade-system/src/compositor/cursor_manager.rs

use smithay::{
    input::pointer::{CursorImageStatus, PointerHandle},
    utils::Serial, // For serials in set_cursor
};
use std::sync::{Arc, Mutex};
use anyhow::Result;

// Placeholder for XCursor theme if we implement actual theme loading later.
// For Smithay 0.6, direct XCursor theme loading might not be part of smithay::desktop::utils.
// We might need to use a crate like "xcursor" and manage WlBuffers for cursor images.
// For now, this manager focuses on the *logic* of when to set which status.
/*
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
struct XCursorTheme {
    // ... details of loaded theme ...
    name: String,
}

impl XCursorTheme {
    fn load(name: Option<&str>) -> Result<Self> {
        let theme_name = name.unwrap_or("default");
        tracing::info!("Loading XCursor theme: {} (placeholder)", theme_name);
        // Actual loading logic here
        Ok(XCursorTheme { name: theme_name.to_string() })
    }

    fn get_cursor(&self, cursor_name: &str) -> Option<Arc<WlSurface>> {
        tracing::info!("Getting cursor '{}' from theme '{}' (placeholder)", cursor_name, self.name);
        // Actual cursor lookup and WlSurface provision here
        None
    }
}
*/

/// Manages the cursor appearance and theme.
pub struct CursorManager {
    // theme: Option<XCursorTheme>, // Will be properly implemented later
    // current_cursor_name: Option<String>, // e.g., "left_ptr", "text", "wait"
    // Smithay's CursorImageStatus is an Arc<Mutex<...>> often stored in DesktopState.
    // This manager might interact with it or be called when it needs to change.
    // For now, it doesn't store the status directly but acts upon it via PointerHandle.
}

impl CursorManager {
    /// Creates a new `CursorManager`.
    ///
    /// `theme_name`: Optional name of the XCursor theme to load.
    pub fn new(theme_name: Option<&str>) -> Result<Self> {
        tracing::info!("Initializing CursorManager with theme: {:?} (placeholder for actual theme loading)", theme_name);
        // let theme = XCursorTheme::load(theme_name).ok();
        // if theme.is_none() && theme_name.is_some() {
        //     tracing::warn!("Failed to load XCursor theme: {}", theme_name.unwrap());
        // }
        Ok(Self {
            // theme,
            // current_cursor_name: None,
        })
    }

    /// Sets the cursor image for a given pointer.
    ///
    /// This would be called when the compositor determines the cursor should change,
    /// e.g., when a client requests a specific cursor via `wl_pointer.set_cursor`
    /// or when the pointer hovers over different UI elements.
    ///
    /// # Arguments
    /// * `pointer_handle`: The Smithay `PointerHandle` to set the cursor for.
    /// * `serial`: The serial for the set_cursor event.
    /// * `name`: Optional name of the cursor to set (e.g., "left_ptr", "text").
    ///           If `None`, it might hide the cursor or revert to default.
    ///
    /// In a full implementation, `name` would be used to look up a cursor surface
    /// from the loaded `XCursorTheme` and then provide that `WlSurface` to
    /// `pointer_handle.set_cursor(serial, Some(&surface), hotspot)`.
    /// For now, it directly uses `CursorImageStatus` variants if applicable,
    /// or logs the intent.
    pub fn set_cursor(
        &self,
        pointer_handle: &PointerHandle<impl smithay::input::SeatHandler>, // Generic over SeatHandler data
        serial: Serial,
        name: Option<&str>,
        // hotspot: (i32, i32) // Hotspot would also be needed from the theme
    ) {
        // This is where the logic for XCursor theme integration would go.
        // 1. Look up `name` in `self.theme`.
        // 2. Get the `WlSurface` and hotspot for that cursor.
        // 3. Call `pointer_handle.set_cursor(serial, Some(surface), hotspot)`.

        // For now, we'll simulate with CursorImageStatus if the names match,
        // or just log. Smithay's `set_cursor` is usually for client-provided surfaces.
        // The compositor itself managing themed cursors often involves creating
        // small WlSurfaces with the cursor images and setting those.
        // Alternatively, if the renderer handles cursor rendering directly based on a name/texture,
        // this manager would update that state.

        // The `CursorImageStatus` in `DesktopState` is what clients react to
        // if they are drawing their own cursor based on compositor hints.
        // `PointerHandle::set_cursor` is for when the compositor draws the cursor.

        if name.is_none() {
            // Request to hide cursor
            // This is often done by setting a null buffer or a fully transparent surface.
            // pointer_handle.set_cursor(serial, None, (0,0)); // Smithay 0.6 might not support None directly this way.
                                                            // Hiding is often setting a blank surface.
            tracing::info!("CursorManager: Request to hide cursor (serial: {:?}). (Placeholder - actual hiding needs surface)", serial);
            // For now, we can try to update the status that DesktopState might hold.
            // This is indirect. A full compositor cursor would directly render.
            // desktop_state.current_cursor_status.lock().unwrap() = CursorImageStatus::Hidden;
        } else {
            let cursor_name = name.unwrap();
            tracing::info!("CursorManager: Request to set cursor to '{}' (serial: {:?}). (Placeholder - actual surface setting needs theme)", cursor_name, serial);
            // Example:
            // desktop_state.current_cursor_status.lock().unwrap() = CursorImageStatus::Named(cursor_name.to_string());

            // If we had a theme and surfaces:
            // if let Some(surface_arc) = self.theme.as_ref().and_then(|t| t.get_cursor(cursor_name)) {
            //     let hotspot = (0,0); // Get from theme
            //     pointer_handle.set_cursor(serial, Some(&*surface_arc), hotspot);
            // } else {
            //     tracing::warn!("Cursor '{}' not found in theme.", cursor_name);
            //     // Fallback to a default or hide
            //     // pointer_handle.set_cursor(serial, None, (0,0)); // Or set default surface
            // }
        }
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default CursorManager")
    }
}
