//! Manages the core state of the NovaDE Wayland compositor.
//!
//! This module defines `NovaCompositorState`, the central struct holding all
//! compositor-wide state, including Smithay's helper states, window management
//! structures (like `Space` and `PopupManager`), output information, and input device states.
//! It also provides helper structs like `ClientData` and `SurfaceData` for associating
//! custom data with Wayland clients and surfaces, although these are currently placeholders.

use smithay::{
    desktop::{Window, WindowAutoTune, WindowSurfaceType, Space, PopupManager},
    output::{Output, Mode, PhysicalProperties, Scale},
    reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        DisplayHandle, UserDataMap
    },
    input::{SeatState, Seat},
    wayland::{
        compositor::CompositorState,
        shell::xdg::{XdgShellState, Configure, XdgPopupSurfaceData, ToplevelSurfaceData, XdgToplevelSurfaceData},
        shm::ShmState,
    },
    utils::{Log, Point, Size},
};
use std::sync::Arc;

/// Data associated with each Wayland client.
///
/// Currently a placeholder. Can be extended to store client-specific permissions,
/// tracking information, or other metadata. Smithay's `UserDataMap` on `Client`
/// objects can also be used for this purpose.
#[derive(Debug, Default)]
pub struct ClientData {
    // TODO: Add fields for client-specific data, e.g., security context.
}

/// Data associated with each Wayland surface (`wl_surface`).
///
/// Currently a placeholder. Can be used to store surface-specific state
/// not directly managed by Smithay, such as custom damage tracking information,
/// compositor-specific decorations, or window management hints.
/// Smithay's `SurfaceDataExt` (accessible via `surface.data_map()`) provides
/// storage for common surface attributes.
#[derive(Debug, Default, Clone)]
pub struct SurfaceData {
    // TODO: Add fields for surface-specific data.
}

/// The main state structure for the NovaDE compositor.
///
/// This struct aggregates all compositor-wide state, including:
/// - Handles to the Wayland display and logger.
/// - Smithay's state objects for core Wayland protocols like `wl_compositor` (`compositor_state`),
///   `wl_shm` (`shm_state`), `wl_seat` (`seat_state`), and `xdg_shell` (`xdg_shell_state`).
/// - Window management structures: `Space` for managing toplevel window positions and stacking,
///   and `PopupManager` for handling XDG popups.
/// - Output information (`output`), representing the display (e.g., a Winit window).
/// - Input device management: the `seat_name`, the main `Seat` object (`seat`), and tracking
///   for the current keyboard focus (`keyboard_focus`).
///
/// Most fields are initialized during the `new()` call, with some (like `seat`) being
/// fully initialized later in the compositor setup sequence (e.g., in `core.rs`).
#[allow(dead_code)]
// #[derive(Debug)] // Seat<Self> is not Debug, so we remove this for now.
// TODO: Implement Debug manually if needed, or wrap Seat in a way that it can be Debug.
pub struct NovaCompositorState {
    /// Handle to the Wayland display, used for interacting with global objects and clients.
    pub display_handle: DisplayHandle,
    /// Central logger instance for the compositor.
    pub logger: slog::Logger,

    // Smithay's core protocol states
    /// State for the `wl_compositor` global, manages `wl_surface` creation and roles.
    pub compositor_state: CompositorState,
    /// State for the `wl_shm` global, manages shared memory buffers.
    pub shm_state: ShmState,
    /// State for `wl_seat` globals, manages input devices and event dispatch.
    pub seat_state: SeatState<Self>,
    /// State for the `xdg_shell` global, manages XDG toplevels and popups.
    pub xdg_shell_state: XdgShellState,

    // Output and Window Management
    /// The desktop `Space`, which organizes and manages toplevel windows.
    pub space: Space<Window>,
    /// Represents the primary display output (e.g., the Winit window).
    pub output: Output,
    /// Manages XDG popups, ensuring correct placement and behavior relative to parent surfaces.
    pub popup_manager: PopupManager,

    // Input related state
    /// Name of the primary seat (e.g., "seat-0").
    pub seat_name: String,
    /// The main `Seat` object, representing a collection of input devices.
    /// Initialized in `core.rs` after `NovaCompositorState` is created.
    pub seat: Option<Seat<Self>>,
    /// The `WlSurface` that currently has keyboard focus.
    /// This is the compositor's explicit record, complementing Smithay's internal focus tracking.
    pub keyboard_focus: Option<WlSurface>,
}

impl NovaCompositorState {
    /// Creates a new instance of `NovaCompositorState`.
    ///
    /// Initializes all necessary sub-states for core Wayland protocols, window management
    /// (Space, PopupManager), and a default output configuration.
    ///
    /// # Arguments
    ///
    /// * `dh`: A `DisplayHandle` to the Wayland display.
    /// * `logger`: A `slog::Logger` instance for logging.
    pub fn new(dh: DisplayHandle, logger: slog::Logger) -> Self {
        let compositor_state = CompositorState::new::<Self>(&dh, logger.clone());
        let shm_state = ShmState::new(&dh, vec![], logger.clone().into());
        let seat_state = SeatState::new();
        let xdg_shell_state = XdgShellState::new(&dh, logger.clone());

        let space = Space::new(logger.clone());
        let popup_manager = PopupManager::new(logger.clone());
        let seat_name = "seat-0".to_string();

        let output_name = "winit-1".to_string();
        let physical_props = PhysicalProperties {
            size: (0, 0).into(), // Initial size in mm, typically updated by backend
            subpixel: smithay::output::Subpixel::Unknown,
            make: "NovaDE".into(),
            model: "Winit Output".into(),
        };
        let mode = Mode {
            size: (800, 600).into(), // Default resolution
            refresh: 60_000,         // Default refresh rate (60Hz) in mHz
        };
        let output = Output::new(output_name.clone(), physical_props, logger.clone());
        output.create_global::<Self>(&dh);
        output.change_current_state(Some(mode), None, None, Some(Point::from((0,0))));

        Self {
            display_handle: dh,
            logger: logger.clone(),
            compositor_state,
            shm_state,
            seat_state,
            xdg_shell_state,
            space,
            output,
            popup_manager,
            seat_name,
            seat: None, // Initialized later in `core.rs`
            keyboard_focus: None,
        }
    }

    /// Returns a reference to the primary output.
    ///
    /// In a single-output setup (like with Winit), this provides access to the main display.
    pub fn window_output(&self) -> &Output {
        &self.output
    }

    /// Sets or clears the keyboard focus to the given `WlSurface`.
    ///
    /// This method updates the compositor's internal record of the focused surface
    /// (`self.keyboard_focus`) and also calls `set_activated(true/false)` on the
    /// corresponding `Window` objects in the `Space` to potentially trigger
    /// changes in server-side decorations or window appearance.
    ///
    /// Note: This method primarily manages the compositor's *view* of focus and associated
    /// window activation states. The actual delivery of `wl_keyboard.enter` and `wl_keyboard.leave`
    /// events to clients is handled by Smithay's `Seat` and `Keyboard` logic, typically when
    /// `Keyboard::set_focus()` is called.
    ///
    /// # Arguments
    ///
    /// * `new_focus`: An `Option<WlSurface>` representing the surface to focus. `None` clears focus.
    pub fn set_keyboard_focus(&mut self, new_focus: Option<WlSurface>) {
        // Defocus old window if focus is changing to a different surface or to None
        if let Some(old_focused_surface) = self.keyboard_focus.as_ref() {
            if new_focus.as_ref() != Some(old_focused_surface) {
                if let Some(window) = self.space.elements().find(|w| w.wl_surface().as_ref() == Some(old_focused_surface)) {
                    window.set_activated(false);
                    slog::trace!(self.logger, "Deactivated old focus: {:?}", old_focused_surface.id());
                }
            }
        }

        self.keyboard_focus = new_focus.clone();

        // Focus new window if a new focus is set
        if let Some(new_focused_surface) = self.keyboard_focus.as_ref() {
             if let Some(window) = self.space.elements().find(|w| w.wl_surface().as_ref() == Some(new_focused_surface)) {
                window.set_activated(true);
                slog::trace!(self.logger, "Activated new focus: {:?}", new_focused_surface.id());
            }
        }
    }
}

impl WindowAutoTune for NovaCompositorState {
    // Documentation for trait impls often goes on the trait definition or here if specific.
    // For now, detailed comments are within the methods.
    fn creation_configure(
        &self,
        surface: &WlSurface,
        role_data: &XdgToplevelSurfaceData,
        configure_constrains: Configure,
    ) -> Configure {
        slog::debug!(self.logger, "WindowAutoTune: creation_configure for surface {:?}", surface.id());
        let mut configure = Configure::default();
        configure.state = role_data.state; // Respect client's initial state request (e.g. maximized)
        // TODO: Apply compositor policies for size, placement if desired.
        // Example: if configure.size.is_none() && role_data.min_size.is_none() { configure.size = Some((800,600)); }
        configure.ensure_constraints(configure_constrains);
        configure
    }

    fn auto_configure(
        &self,
        surface: &WlSurface,
        role_data: &XdgToplevelSurfaceData,
        configure_constrains: Configure,
    ) -> Configure {
        slog::debug!(self.logger, "WindowAutoTune: auto_configure for surface {:?}", surface.id());
        let mut configure = Configure::default();
        configure.state = role_data.state;
        configure.ensure_constraints(configure_constrains);
        configure
    }

    fn get_surface_type(surface: &WlSurface) -> Option<WindowSurfaceType> {
        // Use Smithay's helper to determine surface type from UserData.
        // This is important for correctly handling popups and other surface roles.
        smithay::desktop::get_surface_type(surface)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::wayland_server::Display;

    fn test_logger() -> slog::Logger {
        slog::Logger::root(slog::Discard, slog::o!())
    }

    #[test]
    fn test_nova_compositor_state_new() {
        let logger = test_logger();
        let mut display: Display<NovaCompositorState> = Display::new().unwrap();
        let display_handle = display.handle();

        let state = NovaCompositorState::new(display_handle, logger);

        assert_eq!(state.space.elements().count(), 0, "Initial window count should be 0");
        assert!(state.seat.is_none(), "Initial seat should be None");
        assert!(state.keyboard_focus.is_none(), "Initial keyboard_focus should be None");
        assert_eq!(state.seat_name, "seat-0", "Default seat name should be seat-0");

        assert_eq!(state.output.name(), "winit-1", "Default output name is incorrect");
        let current_mode = state.output.current_mode().expect("Output should have a current mode");
        assert_eq!(current_mode.size, Size::from((800, 600)), "Default output mode size is incorrect");
        assert_eq!(current_mode.refresh, 60_000, "Default output mode refresh rate is incorrect");

        slog::info!(state.logger, "NovaCompositorState::new() test completed successfully.");
    }

    #[test]
    fn test_set_keyboard_focus_updates_internal_state() {
        let logger = test_logger();
        let mut display: Display<NovaCompositorState> = Display::new().unwrap();
        let display_handle = display.handle();
        let mut state = NovaCompositorState::new(display_handle, logger);

        assert!(state.keyboard_focus.is_none());

        // This test is limited as we can't easily create a mock WlSurface
        // and add mock Windows to the space to verify set_activated calls.
        // We primarily check that calling it doesn't panic and that the internal Option is set.
        state.set_keyboard_focus(None);
        assert!(state.keyboard_focus.is_none());

        slog::info!(state.logger, "set_keyboard_focus test (conceptual) completed.");
    }
}
```
