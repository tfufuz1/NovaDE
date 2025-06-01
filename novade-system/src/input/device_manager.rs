// src/input/device_manager.rs
use crate::input::libinput_handler::LibinputUdevHandler;
use crate::input::config::InputConfig;
use crate::input::keyboard;
use crate::input::pointer;
use crate::input::touch; // Import the touch module
use input::{Device, capability::Capability, event::device::DeviceEvent, event::Event as LibinputEvent};
use tracing::{info, warn, debug};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DeviceWrapper {
    device: Device,
    name: String,
    capabilities: Vec<Capability>,
    pub keyboard_handler: Option<keyboard::Keyboard>,
    pub pointer_handler: Option<pointer::Pointer>,
    pub touch_handler: Option<touch::Touch>, // Added touch handler
}

impl DeviceWrapper {
    pub fn new(device: Device) -> Self {
        let name = device.name().to_string();
        let mut capabilities = Vec::new();
        if device.has_capability(Capability::Keyboard) { capabilities.push(Capability::Keyboard); }
        if device.has_capability(Capability::Pointer) { capabilities.push(Capability::Pointer); }
        if device.has_capability(Capability::Touch) { capabilities.push(Capability::Touch); }
        if device.has_capability(Capability::TabletTool) { capabilities.push(Capability::TabletTool); }
        if device.has_capability(Capability::Gesture) { capabilities.push(Capability::Gesture); }

        debug!("DeviceWrapper: Found device '{}' with capabilities: {:?}", name, capabilities);
        Self {
            device,
            name,
            capabilities,
            keyboard_handler: None,
            pointer_handler: None,
            touch_handler: None, // Initialize to None
        }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn capabilities(&self) -> &[Capability] { &self.capabilities }
    pub fn has_capability(&self, capability: Capability) -> bool { self.capabilities.contains(&capability) }
    pub fn get_libinput_device(&self) -> &Device { &self.device }
}

pub struct InputDeviceManager {
    input_config: InputConfig,
    devices: HashMap<String, DeviceWrapper>,
}

impl InputDeviceManager {
    pub fn new(libinput_handler: &mut LibinputUdevHandler, config_path: &Path) -> Self {
        info!("InputDeviceManager: Initializing...");

        let input_config = InputConfig::load_from_file(config_path).unwrap_or_else(|err| {
            warn!("InputDeviceManager: Failed to load input configuration from '{}': {:?}. Using default config.",
                  config_path.display(), err);
            InputConfig::default()
        });
        info!("InputDeviceManager: Loaded configuration (or default): {:?}", input_config);

        let mut manager = Self {
            input_config,
            devices: HashMap::new(),
        };

        manager.update_devices(libinput_handler);

        info!("InputDeviceManager: Initialization complete. Currently managing {} devices.", manager.devices.len());
        for device_wrapper in manager.devices.values() {
            info!("  - Initial Device: {}, Name: {}, Caps: {:?}, HasKbdH: {}, HasPtrH: {}, HasTouchH: {}",
                  device_wrapper.get_libinput_device().sysname().unwrap_or("N/A"),
                  device_wrapper.name(),
                  device_wrapper.capabilities(),
                  device_wrapper.keyboard_handler.is_some(),
                  device_wrapper.pointer_handler.is_some(),
                  device_wrapper.touch_handler.is_some());
        }

        manager
    }

    pub fn update_devices(&mut self, libinput_handler: &mut LibinputUdevHandler) {
        debug!("InputDeviceManager: Updating device list from libinput events...");

        if libinput_handler.dispatch_events().is_err() {
            warn!("InputDeviceManager: Error during libinput dispatch while updating devices.");
        }

        let libinput_context = libinput_handler.context_mut();
        let mut new_devices_to_add = Vec::new();
        let mut devices_removed_count = 0;
        let initial_device_count = self.devices.len();

        while let Some(event) = libinput_context.next() {
            match event {
                LibinputEvent::Device(device_event) => match device_event {
                    DeviceEvent::Added(added_event) => {
                        let device = added_event.device();
                        let mut wrapper = DeviceWrapper::new(device.clone());
                        let device_key = wrapper.name().to_string();

                        if wrapper.has_capability(Capability::Keyboard) {
                            info!("InputDeviceManager: Device '{}' has keyboard capability. Initializing Keyboard handler.", wrapper.name());
                            match keyboard::Keyboard::new(&self.input_config.keyboard) {
                                Ok(kb_handler) => {
                                    wrapper.keyboard_handler = Some(kb_handler);
                                    info!("InputDeviceManager: Successfully initialized Keyboard handler for '{}'.", wrapper.name());
                                }
                                Err(e) => {
                                    error!("InputDeviceManager: Failed to initialize Keyboard handler for '{}': {}", wrapper.name(), e);
                                }
                            }
                        }
                        if wrapper.has_capability(Capability::Pointer) {
                            info!("InputDeviceManager: Device '{}' has pointer capability. Initializing Pointer handler.", wrapper.name());
                            let ptr_handler = pointer::Pointer::new(&self.input_config.pointer);
                            wrapper.pointer_handler = Some(ptr_handler);
                            info!("InputDeviceManager: Successfully initialized Pointer handler for '{}'.", wrapper.name());
                        }
                        if wrapper.has_capability(Capability::Touch) {
                            info!("InputDeviceManager: Device '{}' has touch capability. Initializing Touch handler.", wrapper.name());
                            let touch_h = touch::Touch::new(&self.input_config.touch);
                            wrapper.touch_handler = Some(touch_h);
                            info!("InputDeviceManager: Successfully initialized Touch handler for '{}'.", wrapper.name());
                        }
                        new_devices_to_add.push((device_key, wrapper));
                    }
                    DeviceEvent::Removed(removed_event) => {
                        let device = removed_event.device();
                        let device_name_key = device.name().to_string();

                        if self.devices.remove(&device_name_key).is_some() {
                            info!("InputDeviceManager: Device removed: {} (Key: {})", device_name_key, device_name_key);
                            devices_removed_count += 1;
                        } else {
                            warn!("InputDeviceManager: Received Removed event for device '{}', but it was not found.", device_name_key);
                        }
                    }
                },
                _ => {}
            }
        }

        let mut new_devices_added_count = 0;
        for (key, wrapper) in new_devices_to_add {
            if self.devices.contains_key(&key) {
                warn!("InputDeviceManager: Device '{}' (Key: {}) was already present. Overwriting.", wrapper.name(), key);
            } else {
                new_devices_added_count +=1;
            }
            info!("InputDeviceManager: Adding device to map: {} (Key: {}) - Caps: {:?}, Kbd: {}, Ptr: {}, Touch: {}",
                  wrapper.name(), key, wrapper.capabilities(),
                  wrapper.keyboard_handler.is_some(),
                  wrapper.pointer_handler.is_some(),
                  wrapper.touch_handler.is_some());
            self.devices.insert(key, wrapper);
        }

        // After processing additions/removals, if there was any change in device count,
        // the seat capabilities might need to be re-advertised.
        // A more robust check would compare the actual capabilities (keyboard, pointer, touch) before and after.
        if new_devices_added_count > 0 || devices_removed_count > 0 {
             debug!("InputDeviceManager: Device list changed (added: {}, removed: {}). Seat capabilities might need update. (Intent logged)",
                    new_devices_added_count, devices_removed_count);
        } else if initial_device_count != self.devices.len() && (new_devices_added_count == 0 && devices_removed_count == 0) {
            // This case handles if devices were overwritten but counts didn't change, less likely to change caps.
            debug!("InputDeviceManager: Device list potentially changed (e.g. overwritten device). Total count {}. Seat capabilities might need update. (Intent logged)", self.devices.len());
        }


        debug!("InputDeviceManager: Device list update complete. Total devices: {}", self.devices.len());
    }

    pub fn get_device(&self, key: &str) -> Option<&DeviceWrapper> {
        self.devices.get(key)
    }

    pub fn all_devices(&self) -> Vec<&DeviceWrapper> {
        self.devices.values().collect()
    }

    pub fn get_input_config(&self) -> &InputConfig {
        &self.input_config
    }

    // Helper to determine current capabilities based on managed devices
    pub fn current_capabilities(&self) -> (bool, bool, bool) { // has_keyboard, has_pointer, has_touch
        let mut has_keyboard = false;
        let mut has_pointer = false;
        let mut has_touch = false;
        for wrapper in self.devices.values() {
            if wrapper.keyboard_handler.is_some() { has_keyboard = true; }
            if wrapper.pointer_handler.is_some() { has_pointer = true; }
            if wrapper.touch_handler.is_some() { has_touch = true; }
        }
        (has_keyboard, has_pointer, has_touch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::libinput_handler::{LibinputHandler, StubbedInputDevice as LibinputStubbedDevice, DeviceCapability as LibinputCapabilityEnum};
    use crate::input::config::{InputConfig, PointerConfig, KeyboardConfig, DeviceSpecificConfigEntry};
    use crate::wayland_server_module_placeholder::WaylandServerHandle; // For notify_seat
    use std::collections::HashMap;

    fn create_test_input_config() -> InputConfig {
        let mut specific_configs = HashMap::new();
        specific_configs.insert(
            "Stubbed Mouse".to_string(),
            DeviceSpecificConfigEntry {
                pointer: Some(PointerConfig {
                    sensitivity: Some(2.0), acceleration_factor: Some(0.1), acceleration_curve: None, button_mapping: None,
                }),
                keyboard: None,
            }
        );
        InputConfig {
            default_pointer_config: Some(PointerConfig {
                sensitivity: Some(1.0), acceleration_factor: Some(0.0), acceleration_curve: None, button_mapping: None,
            }),
            default_keyboard_config: Some(KeyboardConfig { repeat_rate: Some(25), repeat_delay: Some(600) }),
            device_specific: Some(specific_configs),
        }
    }

    // Mock LibinputHandler that returns a predefined set of devices
    struct MockLibinputHandler {
        devices_to_return: Vec<LibinputStubbedDevice>,
    }
    impl MockLibinputHandler {
        fn new(devices: Vec<LibinputStubbedDevice>) -> Self { Self { devices_to_return: devices } }
        // Implement methods that LibinputHandler has, which DeviceManager uses
        #[allow(dead_code)] // Original new is not used in this test setup
        fn new_stub() -> LibinputHandler { LibinputHandler::new() } // Keep the real new for take_libinput_handler
        fn get_devices(&self) -> Vec<LibinputStubbedDevice> { self.devices_to_return.clone() }
    }


    #[test]
    fn test_device_manager_new_with_config_and_device_application() {
        // Create a LibinputHandler that DeviceManager can use (even if it's the real stub)
        let libinput_handler_stub = LibinputHandler::new();
        let config = create_test_input_config();

        let dm = DeviceManager::new_with_config(libinput_handler_stub, config.clone());

        assert_eq!(dm.devices.len(), 3); // LibinputHandler stub provides 3 devices

        let mouse = dm.devices.iter().find(|d| d.name == "Stubbed Mouse").unwrap();
        assert!(mouse.capabilities.contains(&DeviceCapability::Pointer));
        assert_eq!(mouse.pointer_config.as_ref().unwrap().sensitivity, Some(2.0)); // Specific config

        let keyboard = dm.devices.iter().find(|d| d.name == "Stubbed Keyboard").unwrap();
        assert!(keyboard.capabilities.contains(&DeviceCapability::Keyboard));
        assert_eq!(keyboard.keyboard_config.as_ref().unwrap().repeat_rate, Some(25)); // Default

        let touchscreen = dm.devices.iter().find(|d| d.name == "Stubbed Touchscreen").unwrap();
        assert!(touchscreen.capabilities.contains(&DeviceCapability::Touch));
        // No specific touch config, so pointer/keyboard configs should be None
        assert!(touchscreen.pointer_config.is_none());
        assert!(touchscreen.keyboard_config.is_none());
    }

    #[test]
    fn test_refresh_devices_and_notify_seat() {
        let libinput_handler_stub = LibinputHandler::new();
        let config = create_test_input_config();
        let mut dm = DeviceManager::new_with_config(libinput_handler_stub, config);

        let wayland_server_handle = WaylandServerHandle::new(); // Stub
        dm.refresh_devices_and_notify_seat(&wayland_server_handle);
        // Verification here is primarily that it runs and logs correctly.
        // We'd need a mock WaylandServerHandle to check if update_seat_capabilities was called with correct data.
        // For now, rely on logs and no panic.
        // Check that devices are still there:
        assert_eq!(dm.devices.len(), 3);
    }

    #[test]
    fn test_get_primary_configs() {
        let libinput_handler_stub = LibinputHandler::new();
        let config = create_test_input_config();
        let dm = DeviceManager::new_with_config(libinput_handler_stub, config);

        let kbd_cfg = dm.get_primary_keyboard_config().unwrap();
        assert_eq!(kbd_cfg.repeat_rate, Some(25));

        let ptr_cfg = dm.get_primary_pointer_config().unwrap();
        assert_eq!(ptr_cfg.sensitivity, Some(2.0)); // From "Stubbed Mouse" specific config
    }
}
