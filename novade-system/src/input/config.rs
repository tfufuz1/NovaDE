// src/input/config.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct PointerConfig {
    pub acceleration_factor: Option<f64>,
    pub sensitivity: Option<f64>,
    pub acceleration_curve: Option<String>, // e.g., "linear", "adaptive"
    pub button_mapping: Option<HashMap<u32, u32>>, // Raw button to mapped button
}

#[derive(Deserialize, Debug, Clone)]
pub struct KeyboardConfig {
    pub repeat_rate: Option<u32>, // chars per second
    pub repeat_delay: Option<u32>, // ms
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSpecificConfig {
    pub name_match: String, // Field to match against device name
    pub pointer: Option<PointerConfig>,
    pub keyboard: Option<KeyboardConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InputConfig {
    pub default_pointer_config: Option<PointerConfig>,
    pub default_keyboard_config: Option<KeyboardConfig>,
    // Store device-specific configs in a way that's easy to look up.
    // A Vec is fine if we iterate, or HashMap if names are unique identifiers.
    // The prompt used HashMap<String, DeviceSpecificConfig> for device_specific,
    // implying the key (device name) is part of the map itself.
    // Let's refine DeviceSpecificConfig to not hold its own name_match if it's a key.
    pub device_specific: Option<HashMap<String, DeviceSpecificConfigEntry>>,
}

// Renaming DeviceSpecificConfig to DeviceSpecificConfigEntry to avoid confusion
// when it's used as a value in the HashMap. The key of the HashMap will be the device name.
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSpecificConfigEntry {
    pub pointer: Option<PointerConfig>,
    pub keyboard: Option<KeyboardConfig>,
}


impl InputConfig {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        // For this subtask, we'll return a default/stubbed config.
        // Actual file reading (e.g., from a TOML file) would be:
        // let content = std::fs::read_to_string(path).map_err(|e| crate::input::errors::InputError::ConfigFileError(e.to_string()))?;
        // toml::from_str(&content).map_err(|e| crate::input::errors::InputError::TomlDeserializationError(e))?;
        tracing::info!("InputConfig: Loading stubbed configuration from path: '{}'.", path);

        let mut specific_configs = HashMap::new();
        // Example of adding a specific config for a known stubbed device
        specific_configs.insert(
            "Stubbed Mouse".to_string(),
            DeviceSpecificConfigEntry {
                pointer: Some(PointerConfig {
                    acceleration_factor: Some(0.2), // Override default
                    sensitivity: Some(1.5),         // Override default
                    acceleration_curve: Some("linear".to_string()),
                    button_mapping: None, // No specific button mapping for this device
                }),
                keyboard: None,
            }
        );

        Ok(Self {
            default_pointer_config: Some(PointerConfig {
                acceleration_factor: Some(0.5),
                sensitivity: Some(1.0),
                acceleration_curve: Some("adaptive".to_string()),
                button_mapping: None, // No default button mapping
            }),
            default_keyboard_config: Some(KeyboardConfig {
                repeat_rate: Some(25),
                repeat_delay: Some(600),
            }),
            device_specific: Some(specific_configs),
        })
    }

    // Helper method to get config for a device
    pub fn get_effective_pointer_config(&self, device_name: &str) -> Option<PointerConfig> {
        let mut effective_config = self.default_pointer_config.clone();

        if let Some(specific_map) = &self.device_specific {
            if let Some(specific_device_entry) = specific_map.get(device_name) {
                if let Some(specific_pointer_cfg) = &specific_device_entry.pointer {
                    let mut current_effective = effective_config.unwrap_or_else(|| PointerConfig {
                        acceleration_factor: None,
                        sensitivity: None,
                        acceleration_curve: None,
                        button_mapping: None,
                    });
                    if specific_pointer_cfg.acceleration_factor.is_some() {
                        current_effective.acceleration_factor = specific_pointer_cfg.acceleration_factor;
                    }
                    if specific_pointer_cfg.sensitivity.is_some() {
                        current_effective.sensitivity = specific_pointer_cfg.sensitivity;
                    }
                    if specific_pointer_cfg.acceleration_curve.is_some() {
                        current_effective.acceleration_curve = specific_pointer_cfg.acceleration_curve.clone();
                    }
                    if specific_pointer_cfg.button_mapping.is_some() {
                        // Button mappings usually replace, not merge, unless explicitly designed to.
                        current_effective.button_mapping = specific_pointer_cfg.button_mapping.clone();
                    }
                    effective_config = Some(current_effective);
                }
            }
        }
        effective_config
    }

    // Helper method to get config for a device (already exists, ensure logging if complex decisions made)
    // pub fn get_effective_pointer_config(&self, device_name: &str) -> Option<PointerConfig> {
    //     tracing::debug!("Getting effective pointer config for device: {}", device_name);
    //     ...
    // }

    pub fn get_effective_keyboard_config(&self, device_name: &str) -> Option<KeyboardConfig> {
        let mut effective_config = self.default_keyboard_config.clone();

        if let Some(specific_map) = &self.device_specific {
            if let Some(specific_device_entry) = specific_map.get(device_name) {
                if let Some(specific_keyboard_cfg) = &specific_device_entry.keyboard {
                     let mut current_effective = effective_config.unwrap_or_else(|| KeyboardConfig {
                        repeat_rate: None,
                        repeat_delay: None,
                    });
                    if specific_keyboard_cfg.repeat_rate.is_some() {
                        current_effective.repeat_rate = specific_keyboard_cfg.repeat_rate;
                    }
                    if specific_keyboard_cfg.repeat_delay.is_some() {
                        current_effective.repeat_delay = specific_keyboard_cfg.repeat_delay;
                    }
                    effective_config = Some(current_effective);
                }
            }
        }
        effective_config
    }
}
