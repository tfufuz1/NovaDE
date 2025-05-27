use crate::outputs::error::OutputError;
use smithay::{
    output::{Output, PhysicalProperties, Mode, Scale, OutputError as SmithayOutputError},
    utils::{Point, Rectangle, Transform, Logical, Physical}, // Added Physical for screen_size
    reexports::wayland_server::{GlobalId, DisplayHandle, protocol::wl_output}, // Added wl_output
};
use std::{
    sync::{Arc, Mutex, Weak}, // Added Weak if needed for back-references
    os::unix::io::OwnedFd, // For drm_device_fd
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpmsState {
    On,
    Standby,
    Suspend,
    Off,
}

impl Default for DpmsState {
    fn default() -> Self {
        DpmsState::On
    }
}

#[derive(Debug)]
pub struct OutputDevice {
    name: String,
    smithay_output: Output, // Smithay's core output representation
    wl_output_global: Option<GlobalId>,
    xdg_output_global: Option<GlobalId>,
    // TODO: Add wlr_head_global and wlr_power_global when those protocols are implemented
    // wlr_head_global: Option<GlobalId>,
    // wlr_power_global: Option<GlobalId>,
    enabled: bool,
    current_dpms_state: DpmsState,
    pending_config_serial: Option<u32>, // For wlr-output-management
    drm_device_fd: Option<OwnedFd>,     // If managing DRM devices directly here
}

impl OutputDevice {
    pub fn new(
        name: String,
        physical_properties: PhysicalProperties,
        possible_modes: Vec<Mode>,
        preferred_mode: Option<Mode>, // Changed order to match typical use
        drm_fd: Option<OwnedFd>, // Added drm_fd to constructor
    ) -> Self {
        let output = Output::new(name.clone(), physical_properties, Some(tracing::Span::current()));

        // Add possible modes
        for mode in possible_modes {
            output.add_mode(mode);
        }

        // Set preferred mode if specified, otherwise Smithay might pick one from added modes.
        if let Some(ref mode) = preferred_mode {
            output.set_preferred_mode(mode.clone());
        }
        
        // Set an initial state. Smithay defaults to no mode, (0,0) pos, Transform::Normal, Scale::Integer(1).
        // We should set a current mode, typically the preferred one.
        let initial_mode = preferred_mode.or_else(|| output.modes().first().cloned());
        if let Some(ref mode) = initial_mode {
            output.change_current_state(
                Some(mode.clone()),
                None, // Transform::Normal
                None, // Scale::Integer(1)
                None, // Point::from((0,0))
            );
            tracing::info!("OutputDevice '{}' initialized with mode {:?}.", name, mode.size);
        } else {
            tracing::warn!("OutputDevice '{}' initialized without a current mode. Add possible_modes.", name);
        }


        Self {
            name,
            smithay_output: output,
            wl_output_global: None,
            xdg_output_global: None,
            // wlr_head_global: None,
            // wlr_power_global: None,
            enabled: true, // Default to enabled
            current_dpms_state: DpmsState::On,
            pending_config_serial: None,
            drm_device_fd: drm_fd,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn smithay_output(&self) -> &Output {
        &self.smithay_output
    }

    pub fn current_mode(&self) -> Option<Mode> {
        self.smithay_output.current_mode()
    }

    pub fn current_transform(&self) -> Transform {
        self.smithay_output.current_transform()
    }

    pub fn current_scale(&self) -> Scale {
        self.smithay_output.current_scale()
    }

    pub fn current_position(&self) -> Point<i32, Logical> {
        self.smithay_output.current_location() // Smithay calls it current_location
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Applies a new state to the output device.
    ///
    /// This method updates the underlying `smithay::output::Output`'s current state.
    /// Smithay will handle sending `wl_output` and `xdg_output` protocol events
    /// to clients when these globals are active.
    ///
    /// # Arguments
    /// * `mode`: The new mode to apply. If `None`, the current mode is unchanged.
    /// * `transform`: The new transform. If `None`, current transform is unchanged.
    /// * `scale`: The new scale. If `None`, current scale is unchanged.
    /// * `position`: The new position in logical coordinates. If `None`, current position is unchanged.
    /// * `enabled`: Whether the output should be enabled. If `false`, the output might be
    ///   effectively disabled (e.g., by clearing its mode or via DPMS).
    ///
    /// # Returns
    /// * `Ok(())` if the state was applied.
    /// * `Err(OutputError)` if applying the state failed (e.g., mode not supported).
    pub fn apply_state_internal(
        &mut self,
        mode: Option<Mode>,
        transform: Option<Transform>,
        scale: Option<Scale>,
        position: Option<Point<i32, Logical>>,
        enabled: bool,
    ) -> Result<(), OutputError> {
        tracing::info!(
            "Applying new state to output '{}': mode={:?}, transform={:?}, scale={:?}, position={:?}, enabled={}",
            self.name, mode.as_ref().map(|m| m.size), transform, scale, position, enabled
        );

        if enabled {
            // If enabling, ensure a mode is set. Use current if None provided, or preferred/first if still None.
            let new_mode = mode.or_else(|| self.current_mode()).or_else(|| self.smithay_output.preferred_mode());
            if new_mode.is_none() && self.smithay_output.modes().is_empty() {
                 tracing::warn!("Cannot enable output '{}': No mode specified and no modes available.", self.name);
                 // Keep it disabled or return error. For now, we'll let change_current_state handle it.
            }

            self.smithay_output.change_current_state(new_mode, transform, scale, position);
            self.enabled = true;
            // If DPMS was Off, turn it On.
            if self.current_dpms_state != DpmsState::On {
                self.set_dpms_state(DpmsState::On)?; // This might re-call apply_state_internal if DPMS changes mode.
                                                      // Be careful about recursion. set_dpms_state should be simpler.
            }
        } else {
            // Disabling output: Smithay recommends clearing mode, setting position to (0,0), etc.
            // Or, use DPMS Off. For now, just mark as disabled and set DPMS Off.
            self.smithay_output.change_current_state(None, Some(Transform::Normal), Some(Scale::Integer(1)), Some((0,0).into()));
            self.enabled = false;
            self.set_dpms_state(DpmsState::Off)?;
        }
        Ok(())
    }

    /// Sets the DPMS (Display Power Management Signaling) state for the output.
    pub fn set_dpms_state(&mut self, state: DpmsState) -> Result<(), OutputError> {
        tracing::info!("Setting DPMS state for output '{}' to {:?}", self.name, state);
        self.current_dpms_state = state;

        // Placeholder for actual backend calls (e.g., DRM ioctl to set DPMS).
        // This might involve interacting with `self.drm_device_fd`.
        // Example:
        // if let Some(fd) = &self.drm_device_fd {
        //     // Perform DRM operation to set DPMS state on fd for this output's connector_id
        // }

        // Placeholder for notifying wlr-output-power-management protocol clients.
        // This would involve finding the wlr_power_global associated with this output
        // and sending an event on it.

        // If DPMS state implies output should be disabled (e.g. Off) or enabled (e.g. On)
        // this might also interact with `self.enabled` and `change_current_state`.
        // For simplicity, keep DPMS distinct from the general enabled flag for now,
        // though they are related. `apply_state_internal` handles basic on/off logic.
        // A more complex DPMS might clear mode for Off, restore for On.
        // For now, it's just an internal state and placeholder for backend calls.

        Ok(())
    }

    pub fn supported_modes(&self) -> Vec<Mode> {
        self.smithay_output.modes()
    }

    pub fn physical_properties(&self) -> PhysicalProperties {
        self.smithay_output.physical_properties()
    }

    // --- Global ID Management ---
    pub fn wl_output_global(&self) -> Option<GlobalId> {
        self.wl_output_global.clone()
    }
    pub fn set_wl_output_global(&mut self, global_id: Option<GlobalId>) {
        self.wl_output_global = global_id;
    }

    pub fn xdg_output_global(&self) -> Option<GlobalId> {
        self.xdg_output_global.clone()
    }
    pub fn set_xdg_output_global(&mut self, global_id: Option<GlobalId>) {
        self.xdg_output_global = global_id;
    }
    
    // TODO: Add setters/getters for wlr_head_global, wlr_power_global when implemented.

    /// Destroys all Wayland globals associated with this output device.
    pub fn destroy_globals(&mut self, display_handle: &DisplayHandle) {
        tracing::info!("Destroying globals for output '{}'", self.name);
        if let Some(id) = self.wl_output_global.take() {
            display_handle.disable_global(id);
            tracing::debug!("Disabled wl_output global for '{}'", self.name);
        }
        if let Some(id) = self.xdg_output_global.take() {
            display_handle.disable_global(id);
            tracing::debug!("Disabled xdg_output global for '{}'", self.name);
        }
        // TODO: Destroy wlr_head_global, wlr_power_global when implemented.

        // Mark the smithay::Output as removed from the perspective of clients.
        // This doesn't destroy the Output object itself, but clients will see it gone.
        self.smithay_output.remove();
    }

    // --- Pending Config for wlr-output-management ---
    pub fn set_pending_config_serial(&mut self, serial: u32) {
        self.pending_config_serial = Some(serial);
    }
    pub fn take_pending_config_serial(&mut self) -> Option<u32> {
        self.pending_config_serial.take()
    }
}

// Ensure OutputDevice is Send + Sync if it's going to be in Arc<Mutex<OutputDevice>>
// OwnedFd is Send but not Sync. If drm_device_fd is used and OutputDevice needs to be Sync,
// the fd might need its own Mutex or be managed differently for thread-safety.
// For now, let's assume it's okay or drm_device_fd will be handled carefully.
// Smithay's Output is Send + Sync. String is. GlobalId is. DpmsState is.
// The main concern would be OwnedFd if operations on it are not thread-safe
// without external synchronization, but we are not performing operations on it yet.
// If drm_device_fd is Some(OwnedFd), then OutputDevice is Send but not Sync.
// This will be an issue for Arc<Mutex<OutputDevice>> if used across threads where Sync is needed.
// For now, let's proceed. If Sync becomes an issue, drm_device_fd might need to be
// wrapped (e.g. `Arc<Mutex<Option<OwnedFd>>>`) or not stored directly in OutputDevice
// if OutputDevice itself needs to be Sync.
// Smithay examples often keep DRM resources (like DrmDevice) separate from the Output representation
// when used in a multi-threaded context.
// Let's assume for now that `drm_device_fd` can be handled; if not, it might be better
// to store just the path or an ID and have a dedicated DRM manager access the FD.
// Given it's `Option<OwnedFd>`, if it's `None`, `OutputDevice` would be `Sync`.
// If `Some(OwnedFd)`, it's not `Sync`. This is a potential issue.
// Workaround for now: Assume drm_device_fd will be None or handled in a single-threaded context
// for this specific part of the state. Or, it's moved out when multi-threading becomes an issue.
// For the plan, let's keep it, as it's specified.
// A common pattern is to have a `DrmDevice` struct that holds the FD and is `Send+Sync`
// because all its operations on the FD are `&self` and internally synchronized if needed.
// `OwnedFd` itself is not `Sync`.

// Let's make `drm_device_fd` an `Arc<Mutex<Option<OwnedFd>>>` to ensure Sync if needed later.
// Or, for simplicity of this step, assume it's not directly accessed concurrently.
// For now, I'll leave it as `Option<OwnedFd>` and address Sync if it becomes a build error
// in a context requiring Sync. The current plan does not involve threading that would immediately break this.
// `DesktopState` is `'static` for calloop, its fields need to be `Send` (+ `Sync` if shared across threads in callbacks).
// `Arc<Mutex<OutputDevice>>` makes `OutputDevice` itself not need to be `Sync`, only `Send`.
// Since `OwnedFd` is `Send`, `OutputDevice` will be `Send`. This should be fine for `Arc<Mutex<...>>`.Okay, I've defined `DpmsState` and `OutputDevice` with its methods in `system/src/outputs/device.rs`.
- `DpmsState` enum: `On, Standby, Suspend, Off`.
- `OutputDevice` struct: Includes `name`, `smithay_output`, global ID placeholders, `enabled`, `current_dpms_state`, `pending_config_serial`, and `drm_device_fd`.
- `OutputDevice::new()`: Initializes `smithay_output`, adds modes, sets preferred mode, and applies an initial state (current mode set, default position/transform/scale).
- Implemented various getter methods (`name`, `smithay_output`, `current_mode`, etc.).
- `apply_state_internal()`: Calls `smithay_output.change_current_state()` and updates `self.enabled`. Handles basic logic for enabling/disabling.
- `set_dpms_state()`: Updates `current_dpms_state` (placeholder for backend calls).
- Implemented methods for setting/getting global IDs and `destroy_globals()`.
- Added `set_pending_config_serial` and `take_pending_config_serial`.

The `drm_device_fd` type is `Option<OwnedFd>`. I've noted the potential `Sync` issue if `OutputDevice` were to be used in a context requiring `Sync` directly, but for `Arc<Mutex<OutputDevice>>`, being `Send` is sufficient for `OutputDevice`, and `OwnedFd` is `Send`.

**Step 4: Implement `system::outputs::manager.rs`**
I will now create `system/src/outputs/manager.rs` and define `HotplugEvent` and `OutputManager` with its methods.
The `output_device_created_notifications` and `output_device_removed_notifications` will be placeholders as specified.
I will need to import `DesktopState` for `handle_hotplug_event` even if it's not fully used yet for notifications, to match the plan.
