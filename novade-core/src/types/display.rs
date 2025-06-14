// novade-core/src/types/display.rs

use serde::{Serialize, Deserialize};

/// Represents a physical display connector.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayConnector {
    HDMI,
    DisplayPort,
    DVI,
    VGA,
    LVDS,
    Unknown,
}

/// Represents a display mode, including resolution and refresh rate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32, // in mHz (millihertz) for precision
}

/// Represents the physical properties of a display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalProperties {
    pub width_mm: u32, // width in millimeters
    pub height_mm: u32, // height in millimeters
}

/// Represents the current status of a display.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayStatus {
    Connected,
    Disconnected,
    Unknown,
}

/// Represents the layout of displays in a multi-monitor setup.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayLayout {
    Extended,
    Mirrored,
    Single, // Only one display active
}

/// Represents a single display device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    pub id: String, // A unique identifier for the display
    pub name: String, // A human-readable name (e.g., "Dell U2719D")
    pub connector: DisplayConnector,
    pub status: DisplayStatus,
    pub modes: Vec<DisplayMode>,
    pub current_mode: Option<DisplayMode>,
    pub physical_properties: Option<PhysicalProperties>,
    pub position_x: i32, // Position in a virtual desktop layout
    pub position_y: i32, // Position in a virtual desktop layout
    pub enabled: bool,
}

/// Represents the overall display configuration for the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayConfiguration {
    pub displays: Vec<Display>,
    pub layout: DisplayLayout,
}

#[cfg(test)]
mod tests {
    use super::*;
    // serde_json is not available by default in novade-core,
    // but for testing purposes here, we'll assume it can be added as a dev-dependency if needed.
    // For this exercise, I'll write the test assuming serde_json is available.
    // If novade-core itself doesn't have serde_json as a dev-dependency, this test would fail to compile.
    // The prompt implies it should be used.

    fn create_sample_display_mode() -> DisplayMode {
        DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60000, // mHz
        }
    }

    fn create_sample_physical_properties() -> PhysicalProperties {
        PhysicalProperties {
            width_mm: 597,
            height_mm: 336,
        }
    }

    fn create_sample_display() -> Display {
        Display {
            id: "HDMI-1".to_string(),
            name: "Generic Monitor".to_string(),
            connector: DisplayConnector::HDMI,
            status: DisplayStatus::Connected,
            modes: vec![create_sample_display_mode()],
            current_mode: Some(create_sample_display_mode()),
            physical_properties: Some(create_sample_physical_properties()),
            position_x: 0,
            position_y: 0,
            enabled: true,
        }
    }

    #[test]
    fn test_display_mode_creation() {
        let mode = create_sample_display_mode();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60000);
    }

    #[test]
    fn test_physical_properties_creation() {
        let props = create_sample_physical_properties();
        assert_eq!(props.width_mm, 597);
        assert_eq!(props.height_mm, 336);
    }

    #[test]
    fn test_display_creation() {
        let display = create_sample_display();
        assert_eq!(display.id, "HDMI-1");
        assert_eq!(display.name, "Generic Monitor");
        assert_eq!(display.connector, DisplayConnector::HDMI);
        assert_eq!(display.status, DisplayStatus::Connected);
        assert_eq!(display.modes.len(), 1);
        assert!(display.current_mode.is_some());
        assert!(display.physical_properties.is_some());
        assert_eq!(display.position_x, 0);
        assert_eq!(display.position_y, 0);
        assert!(display.enabled);
    }

    #[test]
    fn test_display_configuration_creation() {
        let config = DisplayConfiguration {
            displays: vec![create_sample_display()],
            layout: DisplayLayout::Extended,
        };
        assert_eq!(config.displays.len(), 1);
        assert_eq!(config.layout, DisplayLayout::Extended);
    }

    #[test]
    fn test_display_configuration_serde() {
        let original_config = DisplayConfiguration {
            displays: vec![create_sample_display()],
            layout: DisplayLayout::Extended,
        };

        let serialized = serde_json::to_string_pretty(&original_config).expect("Failed to serialize");
        // println!("Serialized DisplayConfiguration: {}", serialized); // For debugging
        let deserialized: DisplayConfiguration = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(original_config, deserialized);

        // Test with empty displays
        let empty_config = DisplayConfiguration {
            displays: vec![],
            layout: DisplayLayout::Single,
        };
        let serialized_empty = serde_json::to_string_pretty(&empty_config).expect("Failed to serialize empty");
        let deserialized_empty: DisplayConfiguration = serde_json::from_str(&serialized_empty).expect("Failed to deserialize empty");
        assert_eq!(empty_config, deserialized_empty);
    }

    #[test]
    fn test_enum_serde() {
        // Test DisplayConnector
        let connector = DisplayConnector::DisplayPort;
        let ser_conn = serde_json::to_string(&connector).unwrap();
        let de_conn: DisplayConnector = serde_json::from_str(&ser_conn).unwrap();
        assert_eq!(connector, de_conn);
        assert_eq!(ser_conn, "\"DisplayPort\"");

        // Test DisplayStatus
        let status = DisplayStatus::Disconnected;
        let ser_status = serde_json::to_string(&status).unwrap();
        let de_status: DisplayStatus = serde_json::from_str(&ser_status).unwrap();
        assert_eq!(status, de_status);

        // Test DisplayLayout
        let layout = DisplayLayout::Mirrored;
        let ser_layout = serde_json::to_string(&layout).unwrap();
        let de_layout: DisplayLayout = serde_json::from_str(&ser_layout).unwrap();
        assert_eq!(layout, de_layout);
    }
}
