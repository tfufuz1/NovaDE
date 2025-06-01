// src/input/config.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{error, info};

// Define a simple error type for configuration loading.
#[deriveDebug, Serialize, Deserialize)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub repeat_rate: i32,    // Characters per second
    pub repeat_delay: i32,   // Milliseconds
    // pub layout: String, // Example: "us", "gb", etc. Might be handled by xkbcommon directly.
    // pub model: Option<String>,
    // pub variant: Option<String>,
    // pub options: Option<String>,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            repeat_rate: 25,
            repeat_delay: 600,
            // layout: "us".to_string(),
            // model: None,
            // variant: None,
            // options: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerConfig {
    pub acceleration_factor: f64, // Simple linear factor for now. 0.0 means 1x speed, >0 increases, <0 decreases.
    pub scroll_factor: f64,       // How much to multiply scroll values by
    pub natural_scrolling: bool,
    // pub button_mapping: Option<HashMap<u32, u32>>, // e.g. {272: 273, 273: 272} to swap left/right

    // Conceptual fields for future advanced acceleration and pointer behavior
    // pub acceleration_profile: String, // e.g., "adaptive", "linear", "flat"
    // pub acceleration_custom_curve_points: Option<Vec<(f64, f64)>>, // For a truly custom curve
    // pub pointer_sensitivity: f64, // General sensitivity setting, often from DE.
}

impl Default for PointerConfig {
    fn default() -> Self {
        Self {
            acceleration_factor: 0.0, // Default to a flat profile (effective multiplier 1.0)
            scroll_factor: 1.0,
            natural_scrolling: false,
            // button_mapping: None,
            // acceleration_profile: "linear".to_string(), // Default profile
            // acceleration_custom_curve_points: None,
            // pointer_sensitivity: 0.0, // Neutral sensitivity
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchConfig {
    pub pressure_threshold: Option<f64>, // Example: For distinguishing tap from drag
    pub disable_while_typing: bool,

    // Conceptual fields for future advanced touch features
    // pub enable_gestures: bool,
    // pub gesture_config_path: Option<String>, // Path to a gesture specific config if needed
    // pub calibration_file_path: Option<String>, // Path to a touch calibration file
    // pub enable_palm_rejection: bool,
}

impl Default for TouchConfig {
    fn default() -> Self {
        Self {
            pressure_threshold: None,
            disable_while_typing: true,
            // enable_gestures: true,
            // gesture_config_path: None,
            // calibration_file_path: None,
            // enable_palm_rejection: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputConfig {
    #[serde(default)]
    pub keyboard: KeyboardConfig,
    #[serde(default)]
    pub pointer: PointerConfig,
    #[serde(default)]
    pub touch: TouchConfig,
    #[serde(default)]
    pub enable_tap_to_click: bool,
    #[serde(default)]
    pub enable_natural_scrolling_pointer: bool, // Specific to pointer, separate from touch
}

impl InputConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        info!("InputConfig: Attempting to load configuration from '{}' (stubbed).", path.display());
        // For now, this is a stub that returns default configuration.
        // In a real implementation:
        // 1. Read the file content.
        // 2. Deserialize from a format like TOML or JSON.
        // Example using std::fs and toml (if toml crate is added):
        /*
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                match toml::from_str(&contents) {
                    Ok(config) => {
                        info!("InputConfig: Successfully loaded and parsed from '{}'.", path.display());
                        Ok(config)
                    }
                    Err(e) => {
                        error!("InputConfig: Failed to parse config file '{}': {}", path.display(), e);
                        Err(ConfigError::ParseError(e.to_string()))
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("InputConfig: Config file '{}' not found. Using default configuration.", path.display());
                Ok(InputConfig::default()) // Or return ConfigError::NotFound if preferred
            }
            Err(e) => {
                error!("InputConfig: Failed to read config file '{}': {}", path.display(), e);
                Err(ConfigError::IoError(e.to_string()))
            }
        }
        */
        // Stub implementation:
        if !path.exists() {
             info!("InputConfig: Config file '{}' not found (stub check). Returning default.", path.display());
        }
        Ok(InputConfig::default())
    }
}

// Example usage (not part of the library code itself, maybe in main.rs or tests)
/*
fn main() {
    let config_path = Path::new("config/input.toml"); // Example path
    match InputConfig::load_from_file(config_path) {
        Ok(config) => {
            println!("Loaded input configuration: {:?}", config);
            // Apply this config to input manager and devices
        }
        Err(e) => {
            eprintln!("Error loading input configuration: {:?}", e);
            // Fallback to default or handle error
        }
    }
}
*/
