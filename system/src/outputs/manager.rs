use crate::{
    compositor::core::state::DesktopState, // For notification methods
    outputs::{
        device::{OutputDevice, DpmsState}, // DpmsState might not be directly used here but good for context
        error::OutputError,
    },
};
use smithay::{
    output::{PhysicalProperties, Mode},
    reexports::wayland_server::{protocol::wl_output, DisplayHandle, GlobalId},
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    os::unix::io::OwnedFd,
};

/// Represents events related to output device hotplugging.
#[derive(Debug)]
pub enum HotplugEvent {
    DeviceAdded {
        name: String,
        path: PathBuf, // Path to the DRM device node or similar identifier
        physical_properties: PhysicalProperties,
        modes: Vec<Mode>,
        preferred_mode: Option<Mode>,
        enabled: bool,
        is_drm: bool, // Indicates if this event is from a DRM backend device
        drm_device_fd: Option<OwnedFd>, // FD if opened by the backend source
    },
    DeviceRemoved {
        name: String,
    },
}

/// Manages all active output devices in the compositor.
#[derive(Debug, Default)]
pub struct OutputManager {
    outputs: HashMap<String, Arc<Mutex<OutputDevice>>>,
}

impl OutputManager {
    /// Creates a new, empty `OutputManager`.
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    /// Adds a pre-configured `OutputDevice` to the manager.
    ///
    /// # Arguments
    /// * `output_device`: An `Arc<Mutex<OutputDevice>>` representing the output to add.
    pub fn add_output(&mut self, output_device_arc: Arc<Mutex<OutputDevice>>) {
        let name = output_device_arc.lock().unwrap().name().to_string();
        tracing::info!("Adding output '{}' to OutputManager.", name);
        self.outputs.insert(name, output_device_arc);
    }

    /// Removes an output device from the manager by its name.
    ///
    /// This also calls `destroy_globals()` on the `OutputDevice` to clean up
    /// its Wayland global resources.
    ///
    /// # Arguments
    /// * `name`: The name of the output device to remove.
    /// * `display_handle`: A handle to the Wayland display for global destruction.
    ///
    /// # Returns
    /// * `Some(Arc<Mutex<OutputDevice>>)` if the output was found and removed.
    /// * `None` if no output with the given name was found.
    pub fn remove_output(
        &mut self,
        name: &str,
        display_handle: &DisplayHandle,
    ) -> Option<Arc<Mutex<OutputDevice>>> {
        tracing::info!("Attempting to remove output '{}' from OutputManager.", name);
        if let Some(output_device_arc) = self.outputs.remove(name) {
            tracing::info!("Output '{}' found, destroying its globals.", name);
            // destroy_globals is now called within output_device_removed_notifications
            // or relies on it being called before this if this function is used externally.
            // For handle_hotplug_event, output_device_removed_notifications is called first.
            // If remove_output is called directly, ensure destroy_globals was handled.
            // To be safe, let's ensure it's called if the output is being removed here.
            // However, the plan states output_device_removed_notifications handles this.
            // For now, let's assume the caller of remove_output (like handle_hotplug_event)
            // coordinates this. If remove_output is a public API that implies full cleanup,
            // it should call destroy_globals.
            // The current plan calls destroy_globals within output_device_removed_notifications,
            // which is called by handle_hotplug_event *before* this remove_output.
            // So, direct call to destroy_globals here might be redundant in that flow.
            // Let's keep it simple: destroy_globals should be part of the removal process.
            // `OutputDevice::destroy_globals` is idempotent due to `take()`.
            output_device_arc.lock().unwrap().destroy_globals(display_handle);
            Some(output_device_arc)
        } else {
            tracing::warn!("Output '{}' not found for removal.", name);
            None
        }
    }

    /// Finds an output device by its name.
    pub fn find_output_by_name(&self, name: &str) -> Option<Arc<Mutex<OutputDevice>>> {
        self.outputs.get(name).cloned()
    }

    /// Finds an output device by its associated `wl_output` global.
    ///
    /// This iterates through managed outputs and compares their stored `wl_output_global` ID
    /// with the ID of the provided `wl_output` object.
    pub fn find_output_by_wl_output(
        &self,
        wl_output: &wl_output::WlOutput,
    ) -> Option<Arc<Mutex<OutputDevice>>> {
        let target_global_id = wl_output.id();
        self.outputs.values().find(|output_arc| {
            output_arc
                .lock()
                .unwrap()
                .wl_output_global()
                .map_or(false, |gid| gid == target_global_id)
        }).cloned()
    }

    /// Returns a list of all currently managed output devices.
    pub fn all_outputs(&self) -> Vec<Arc<Mutex<OutputDevice>>> {
        self.outputs.values().cloned().collect()
    }

    /// Handles a hotplug event, adding or removing an output device.
    ///
    /// # Arguments
    /// * `event`: The `HotplugEvent` to process.
    /// * `display_handle`: A handle to the Wayland display, for global management.
    /// * `desktop_state`: Mutable reference to `DesktopState`, for notifications and context.
    ///
    /// # Returns
    /// * `Ok(())` if the event was handled successfully.
    /// * `Err(OutputError)` if an error occurred during device setup or removal.
    pub fn handle_hotplug_event(
        &mut self,
        event: HotplugEvent,
        display_handle: &DisplayHandle,
        desktop_state: &mut DesktopState, // Used for notifications
    ) -> Result<(), OutputError> {
        match event {
            HotplugEvent::DeviceAdded {
                name,
                path: _path, // path might be used for logging or specific backend interaction
                physical_properties,
                modes,
                preferred_mode,
                enabled: _initial_enabled, // OutputDevice::new sets initial enabled state
                is_drm: _is_drm, // Flag for future use if behavior differs
                drm_device_fd,
            } => {
                tracing::info!("HotplugEvent: DeviceAdded - Name: {}", name);
                if self.outputs.contains_key(&name) {
                    let err_msg = format!("Output with name '{}' already exists.", name);
                    tracing::error!("{}", err_msg);
                    return Err(OutputError::ConfigurationConflict { details: err_msg });
                }

                let new_output_device = OutputDevice::new(
                    name,
                    physical_properties,
                    modes,
                    preferred_mode,
                    drm_device_fd, // Pass the FD to OutputDevice
                );
                let new_output_device_arc = Arc::new(Mutex::new(new_output_device));
                self.add_output(new_output_device_arc.clone()); // Add to map first

                // Create globals and notify
                self.output_device_created_notifications(
                    new_output_device_arc.clone(), // Pass the Arc
                    display_handle,
                    desktop_state,
                )?;

                // TODO: Trigger layout recalculation in DesktopState.space or similar.
                // desktop_state.space.map_output(...) or arrange_outputs()
                // desktop_state.space.refresh(); // Example
                // desktop_state.loop_signal.wakeup(); // To process new layout and render
            }
            HotplugEvent::DeviceRemoved { name } => {
                tracing::info!("HotplugEvent: DeviceRemoved - Name: {}", name);
                if let Some(removed_output_device_arc) = self.find_output_by_name(&name).cloned() {
                    // Notify and destroy globals first
                    self.output_device_removed_notifications(
                        removed_output_device_arc.clone(), // Pass the Arc
                        display_handle,
                        desktop_state,
                    )?;
                    
                    // Then remove from the map. `remove_output` also calls destroy_globals,
                    // which is now redundant if called in notifications. Let's ensure it's called once.
                    // The `destroy_globals` in `output_device_removed_notifications` is sufficient.
                    // So `self.outputs.remove(&name)` is the direct action here.
                    self.outputs.remove(&name);
                    tracing::info!("Output '{}' removed from OutputManager map.", name);


                    // TODO: Trigger layout recalculation.
                    // desktop_state.space.unmap_output(...) or arrange_outputs()
                    // desktop_state.space.refresh();
                    // desktop_state.loop_signal.wakeup();
                } else {
                    tracing::warn!("Attempted to remove non-existent output: {}", name);
                    return Err(OutputError::OutputNotFound { name });
                }
            }
        }
        Ok(())
    }

    /// Creates Wayland globals (`wl_output`, `zxdg_output_v1`) for a new output device.
    ///
    /// This function is called when an `OutputDevice` is first added to the manager.
    /// It uses Smithay's `OutputManagerState` to create and manage these globals.
    fn output_device_created_notifications(
        &self,
        output_device_arc: Arc<Mutex<OutputDevice>>,
        display_handle: &DisplayHandle,
        desktop_state: &mut DesktopState,
    ) -> Result<(), OutputError> {
        let mut device_guard = output_device_arc.lock().unwrap();
        let output_name = device_guard.name().to_string();

        tracing::info!("Creating Wayland globals for output device '{}'.", output_name);

        // Create the wl_output global using Smithay's Output object and OutputManagerState.
        let wl_global = desktop_state
            .output_manager_state
            .create_global(display_handle, device_guard.smithay_output());
        
        device_guard.set_wl_output_global(Some(wl_global.id()));
        tracing::info!("Created wl_output global for '{}' with ID: {:?}", output_name, wl_global.id());

        // Smithay's OutputManagerState, when initialized with `new_with_xdg_output`,
        // automatically creates and manages the zxdg_output_v1 resource when the
        // wl_output global is created. No separate call is needed here for zxdg_output_v1 global creation.
        // The `xdg_output_global` field in `OutputDevice` can be removed if not used for other tracking.
        // For now, it's not set here as Smithay handles the XDG part implicitly.
        // If we needed to track it for some reason:
        // device_guard.set_xdg_output_global(None); // Or some way to get it if Smithay exposed it.

        Ok(())
    }

    /// Handles notifications and actions upon output device removal.
    ///
    /// This function is called *before* the `OutputDevice` is fully removed from the manager's map.
    /// It ensures that `OutputDevice::destroy_globals` is called.
    fn output_device_removed_notifications(
        &self,
        output_device_arc: Arc<Mutex<OutputDevice>>,
        display_handle: &DisplayHandle,
        _desktop_state: &mut DesktopState, // Not strictly needed if not using OutputManagerState here for removal
    ) -> Result<(), OutputError> {
        let mut device_guard = output_device_arc.lock().unwrap();
        let output_name = device_guard.name().to_string();

        tracing::info!(
            "Processing removal notifications for output device '{}'. Destroying globals.",
            output_name
        );

        // Call destroy_globals on the OutputDevice. This disables the wl_output global.
        // Smithay's OutputManagerState will then handle the destruction of the associated zxdg_output_v1.
        device_guard.destroy_globals(display_handle);
        
        tracing::info!("Globals for output device '{}' destroyed.", output_name);
        Ok(())
    }
}
