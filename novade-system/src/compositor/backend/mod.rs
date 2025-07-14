// novade-system/src/compositor/backend/mod.rs

use anyhow::Result;
use calloop::LoopHandle;
use smithay::reexports::wayland_server::DisplayHandle;

use crate::compositor::state::DesktopState; // Assuming DesktopState is here

// Forward declare winit_backend and drm_backend modules
pub mod winit_backend;
pub mod drm_backend;

/// Enum to select the active backend for the compositor.
#[derive(Debug, Clone, Copy)]
pub enum BackendType {
    Winit,
    Drm,
    // Headless, // Potentially for testing
}

/// Trait defining the capabilities of a compositor backend.
///
/// This trait abstracts over different ways a Smithay compositor can
/// be initialized and run, such as using Winit for development
/// or DRM/libinput for a production environment.
pub trait CompositorBackend {
    /// Initializes the backend.
    ///
    /// This typically involves setting up event loops, graphics contexts,
    /// and input backends.
    ///
    /// # Arguments
    ///
    /// * `event_loop_handle`: A handle to the Calloop event loop.
    /// * `display_handle`: A handle to the Wayland display.
    /// * `desktop_state`: The shared state of the compositor.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn init(
        event_loop_handle: LoopHandle<'static, DesktopState>,
        display_handle: DisplayHandle,
        desktop_state: &mut DesktopState, // May need to be mutable or passed differently
    ) -> Result<Self>
    where
        Self: Sized;

    /// Runs the backend's main event loop.
    ///
    /// This function will block until the compositor exits.
    ///
    /// # Arguments
    ///
    /// * `desktop_state`: The shared state of the compositor.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the run, or if it exited cleanly.
    fn run(self, desktop_state: &mut DesktopState) -> Result<()>;

    /// Returns a handle to the Calloop event loop used by this backend.
    ///
    /// This can be used to insert additional event sources into the loop.
    fn loop_handle(&self) -> LoopHandle<'static, DesktopState>;

    // Optional: Add other common methods if needed, e.g., for input handling,
    // renderer access, etc., if they are managed directly by the backend wrapper.
    // fn input_backend(&self) -> ...;
    // fn renderer(&self) -> ...;
}

// Example of how one might decide which backend to instantiate.
// This would typically live in main.rs or a similar high-level spot.
/*
pub fn init_backend_auto(
    backend_type: BackendType,
    event_loop_handle: LoopHandle<'static, DesktopState>,
    display_handle: DisplayHandle,
    desktop_state: &mut DesktopState,
) -> Result<Box<dyn CompositorBackend>> {
    match backend_type {
        BackendType::Winit => {
            // Ok(Box::new(winit_backend::WinitBackend::init(event_loop_handle, display_handle, desktop_state)?))
            todo!("Winit backend instantiation")
        }
        BackendType::Drm => {
            // Ok(Box::new(drm_backend::DrmBackend::init(event_loop_handle, display_handle, desktop_state)?))
            todo!("DRM backend instantiation")
        }
    }
}
*/
