// src/input/udev_handler.rs
use crate::input::device_manager::InputDeviceManager;
use crate::input::libinput_handler::LibinputUdevHandler;
use udev::{MonitorBuilder, MonitorSocket, EventType};
use tracing::{error, info, warn, debug};
use std::io::ErrorKind;

pub struct UdevMonitorHandler {
    monitor_socket: MonitorSocket,
}

impl UdevMonitorHandler {
    pub fn new() -> Result<Self, String> {
        info!("UdevMonitorHandler: Initializing...");
        match MonitorBuilder::new() {
            Ok(builder) => {
                // Add matches for relevant subsystems.
                // "input" is primary for most direct input devices.
                // "drm" can be relevant for graphics-related hotplugging (e.g., virtual terminals, GPUs)
                // which might indirectly affect input (e.g., seat assignment or graphics tablet context).
                // Other subsystems like "hid" or "usb" could be monitored for more specific device types
                // if "input" isn't sufficient, but "input" is usually the most direct.
                match builder
                    .match_subsystem_devtype("input", None) // All device types in "input"
                    .or_else(|e| {
                        error!("UdevMonitorHandler: Failed to add subsystem match 'input': {}", e);
                        Err(e)
                    })?
                    .match_subsystem_devtype("drm", None) // All device types in "drm"
                    .or_else(|e| {
                        error!("UdevMonitorHandler: Failed to add subsystem match 'drm': {}", e);
                        Err(e)
                    })?
                    // Add more specific matches if needed, e.g.:
                    // .match_subsystem_devtype("input", Some("mouse"))
                    // .match_subsystem_devtype("input", Some("keyboard"))
                    .enable_receiving()
                {
                    Ok(monitor) => {
                        info!("UdevMonitorHandler: Udev monitor created and listening for 'input' and 'drm' subsystems.");
                        Ok(Self { monitor_socket: monitor })
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to build udev monitor socket: {}", e);
                        error!("UdevMonitorHandler: {}", err_msg);
                        Err(err_msg)
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to create udev monitor builder: {}", e);
                error!("UdevMonitorHandler: {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Polls for udev events and notifies relevant managers.
    ///
    /// Note: In a typical event loop, the udev monitor socket's file descriptor
    /// would be added to a poller (like epoll, kqueue, mio, etc.) and events
    /// would be processed when the fd becomes readable. This function simulates
    /// a part of that for demonstration. `InputDeviceManager` and `LibinputUdevHandler`
    /// are passed to allow this handler to trigger updates.
    pub fn poll_events(
        &mut self,
        device_manager: &mut InputDeviceManager,
        libinput_handler: &mut LibinputUdevHandler,
    ) {
        // Iterate over available events from the MonitorSocket.
        // This is non-blocking by default if using `iter()`.
        // For blocking behavior, one would typically use a poll loop on the fd.
        for event in self.monitor_socket.iter() {
            let subsystem = event.subsystem().unwrap_or_default().to_string_lossy();
            let devtype = event.devtype().unwrap_or_default().to_string_lossy();
            let action = event.action().unwrap_or_default().to_string_lossy();
            let syspath = event.syspath().to_string_lossy();

            debug!(
                "UdevMonitorHandler: Event: action='{}', subsystem='{}', devtype='{}', syspath='{}'",
                action, subsystem, devtype, syspath
            );

            match event.event_type() {
                EventType::Add => {
                    info!("UdevMonitorHandler: Device added: {}", syspath);
                    // Let libinput know there might be new devices.
                    // Libinput's udev backend should pick up changes.
                    // A dispatch processes pending events and updates libinput's internal device list.
                    if libinput_handler.dispatch_events().is_err() {
                        warn!("UdevMonitorHandler: Error dispatching libinput events after udev 'add' event for {}.", syspath);
                    }
                    // The DeviceManager will pick up new devices from libinput on its next update_devices() call.
                    // Optionally, we could be more proactive here if needed, e.g., by directly
                    // hinting to DeviceManager about the syspath, but relying on libinput's events
                    // (triggered by the dispatch above) is cleaner.
                    device_manager.update_devices(libinput_handler); // Proactively update
                }
                EventType::Remove => {
                    info!("UdevMonitorHandler: Device removed: {}", syspath);
                    // Similar to 'add', dispatch libinput events to process removals.
                    if libinput_handler.dispatch_events().is_err() {
                        warn!("UdevMonitorHandler: Error dispatching libinput events after udev 'remove' event for {}.", syspath);
                    }
                    // DeviceManager will pick up removed devices from libinput on its next update_devices() call.
                    device_manager.update_devices(libinput_handler); // Proactively update
                }
                EventType::Change => {
                    debug!("UdevMonitorHandler: Device changed: {}", syspath);
                    // For some 'change' events, libinput might also need a dispatch.
                    if libinput_handler.dispatch_events().is_err() {
                        warn!("UdevMonitorHandler: Error dispatching libinput events after udev 'change' event for {}.", syspath);
                    }
                    device_manager.update_devices(libinput_handler);
                }
                _ => {
                    // Other event types like Bind, Unbind, Online, Offline, etc.
                    // These might not always require immediate libinput dispatch or device manager updates,
                    // but logging them can be useful.
                    debug!("UdevMonitorHandler: Other udev event type '{:?}' for syspath '{}'", event.event_type(), syspath);
                }
            }
        }
    }

    // Method to get the file descriptor for polling (e.g., with mio or epoll)
    // This is how a real event loop would integrate udev monitoring.
    // pub fn fd(&self) -> RawFd {
    //     use std::os::unix::io::AsRawFd;
    //     self.monitor_socket.as_raw_fd()
    // }
}
