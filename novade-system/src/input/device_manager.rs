// src/input/device_manager.rs
use super::libinput_handler::{LibinputHandler, StubbedInputDevice, DeviceCapability as LibinputDeviceCapability};
use super::config::{InputConfig, PointerConfig, KeyboardConfig};
use crate::wayland_server_module_placeholder::{WaylandServerHandle, WlSeatCapability}; // For notify_seat
use std::collections::{HashSet, HashMap}; // Added HashMap

#[derive(Debug, Clone)]
pub struct InputDevice {
    pub name: String,
    pub capabilities: HashSet<DeviceCapability>,
    pub pointer_config: Option<PointerConfig>,
    pub keyboard_config: Option<KeyboardConfig>,
    // Add other specific configs if needed, e.g., TouchConfig
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)] // Added Copy
pub enum DeviceCapability {
    Keyboard,
    Pointer,
    Touch,
    Tablet,
    Gesture,
}

fn convert_capability(lib_cap: &LibinputDeviceCapability) -> DeviceCapability {
    match lib_cap {
        LibinputDeviceCapability::Keyboard => DeviceCapability::Keyboard,
        LibinputDeviceCapability::Pointer => DeviceCapability::Pointer,
        LibinputDeviceCapability::Touch => DeviceCapability::Touch,
        LibinputDeviceCapability::Tablet => DeviceCapability::Tablet,
        LibinputDeviceCapability::Gesture => DeviceCapability::Gesture,
    }
}

pub struct DeviceManager {
    libinput_handler: LibinputHandler, // Keep libinput_handler for potential re-scan/refresh
    pub devices: Vec<InputDevice>,
    input_config: InputConfig,
    // We won't store individual Keyboard/Pointer/Touch handlers here anymore.
    // FocusManager will own them. DeviceManager provides the configurations for them.
}

impl DeviceManager {
    // Renamed from `new` and takes full config
    pub fn new_with_config(libinput_handler: LibinputHandler, config: InputConfig) -> Self {
        let mut dm = Self {
            libinput_handler,
            devices: Vec::new(),
            input_config: config,
        };
        tracing::info!("DeviceManager: Initializing with stubbed libinput_handler and provided config.");
        dm.refresh_devices_internal(); // Initial device scan
        tracing::info!("DeviceManager: Initialized with {} stubbed devices. Configurations applied.", dm.devices.len());
        for device in &dm.devices {
            tracing::debug!("  Device: {}, Caps: {:?}, PtrCfg: {:?}, KbdCfg: {:?}",
                device.name, device.capabilities, device.pointer_config, device.keyboard_config);
        }
        dm
    }

    // Internal refresh logic used by new and public refresh
    fn refresh_devices_internal(&mut self) {
        tracing::debug!("DeviceManager: Refreshing devices internally...");
        self.devices.clear();
        // Get devices from the libinput_handler stub
        for stub_dev in self.libinput_handler.get_devices() {
            tracing::trace!("DeviceManager: Processing stub_dev: {:?}", stub_dev);
            let converted_caps: HashSet<DeviceCapability> =
                stub_dev.capabilities.iter().map(|c| convert_capability(c)).collect();

            // Get effective configs for this specific device
            let pointer_cfg = self.input_config.get_effective_pointer_config(&stub_dev.name);
            let keyboard_cfg = self.input_config.get_effective_keyboard_config(&stub_dev.name);

            if converted_caps.contains(&DeviceCapability::Pointer) && pointer_cfg.is_none() {
                 tracing::warn!("DeviceManager: Pointer device '{}' has no pointer configuration.", stub_dev.name);
            }
            if converted_caps.contains(&DeviceCapability::Keyboard) && keyboard_cfg.is_none() {
                 tracing::warn!("DeviceManager: Keyboard device '{}' has no keyboard configuration.", stub_dev.name);
            }

            self.devices.push(InputDevice {
                name: stub_dev.name.clone(),
                capabilities: converted_caps,
                pointer_config: pointer_cfg,
                keyboard_config: keyboard_cfg,
            });
        }
    }

    // Public method to refresh and notify seat capabilities
    pub fn refresh_devices_and_notify_seat(&mut self, wayland_server: &WaylandServerHandle) {
        tracing::info!("DeviceManager: Refreshing devices and notifying seat capabilities.");
        self.refresh_devices_internal();
        tracing::debug!("DeviceManager: Refreshed devices. Found {} stubbed devices. Configurations applied.", self.devices.len());

        let mut seat_capabilities = HashSet::new();
        for device in &self.devices {
            tracing::trace!("DeviceManager: Checking device for seat caps: {}", device.name);
            if device.capabilities.contains(&DeviceCapability::Keyboard) {
                seat_capabilities.insert(WlSeatCapability::Keyboard);
            }
            if device.capabilities.contains(&DeviceCapability::Pointer) {
                seat_capabilities.insert(WlSeatCapability::Pointer);
            }
            if device.capabilities.contains(&DeviceCapability::Touch) {
                seat_capabilities.insert(WlSeatCapability::Touch);
            }
        }
        let cap_vec: Vec<WlSeatCapability> = seat_capabilities.into_iter().collect();
        tracing::info!("DeviceManager: Notifying Wayland server of seat capabilities: {:?}", cap_vec);
        wayland_server.update_seat_capabilities(&cap_vec);
    }

    pub fn get_managed_devices(&self) -> &Vec<InputDevice> {
        tracing::trace!("DeviceManager: get_managed_devices() called.");
        &self.devices
    }

    // Method to pass ownership of libinput_handler if needed by InputManager
    pub fn take_libinput_handler(self) -> LibinputHandler {
        tracing::debug!("DeviceManager: take_libinput_handler() called.");
        self.libinput_handler
    }

    // Method to get specific configurations for FocusManager to initialize its handlers
    // This assumes one primary keyboard and pointer for simplicity in the stub.
    // A real system might have multiple of each.
    pub fn get_primary_keyboard_config(&self) -> Option<KeyboardConfig> {
        tracing::debug!("DeviceManager: get_primary_keyboard_config() called.");
        let config = self.devices.iter()
            .find(|d| d.capabilities.contains(&DeviceCapability::Keyboard))
            .and_then(|d| d.keyboard_config.clone())
            .or_else(|| self.input_config.default_keyboard_config.clone());
        tracing::trace!("DeviceManager: Primary keyboard config: {:?}", config);
        config
    }

    pub fn get_primary_pointer_config(&self) -> Option<PointerConfig> {
        tracing::debug!("DeviceManager: get_primary_pointer_config() called.");
        let config = self.devices.iter()
            .find(|d| d.capabilities.contains(&DeviceCapability::Pointer))
            .and_then(|d| d.pointer_config.clone())
            .or_else(|| self.input_config.default_pointer_config.clone());
        tracing::trace!("DeviceManager: Primary pointer config: {:?}", config);
        config
    }

    // Stubbed method for InputManager simulation
    pub fn get_stubbed_pointer_coords(&self) -> (f64, f64) {
        tracing::trace!("DeviceManager: get_stubbed_pointer_coords() called.");
        // In a real scenario, this might come from the actual pointer device state
        (100.0, 150.0) // Arbitrary coordinates
    }
}
