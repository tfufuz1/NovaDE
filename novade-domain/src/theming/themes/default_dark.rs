//! Default dark theme for the NovaDE desktop environment.
//!
//! This module provides the default dark theme for the NovaDE desktop environment.

use std::collections::HashMap;
use crate::types::color::Color;
use crate::entities::value_objects::Timestamp;
use super::super::{Theme, ThemeVariant, ThemeComponentType};

/// Default dark theme implementation.
pub struct DefaultDarkTheme;

impl DefaultDarkTheme {
    /// Creates the default dark theme.
    pub fn create() -> Theme {
        let mut components = HashMap::new();
        
        // Base colors
        components.insert(ThemeComponentType::Background, Color::rgb(32, 32, 32));
        components.insert(ThemeComponentType::Foreground, Color::rgb(255, 255, 255));
        components.insert(ThemeComponentType::Primary, Color::rgb(0, 120, 215));
        components.insert(ThemeComponentType::Secondary, Color::rgb(0, 99, 177));
        
        // Status colors
        components.insert(ThemeComponentType::Success, Color::rgb(92, 184, 92));
        components.insert(ThemeComponentType::Warning, Color::rgb(240, 173, 78));
        components.insert(ThemeComponentType::Error, Color::rgb(217, 83, 79));
        components.insert(ThemeComponentType::Info, Color::rgb(91, 192, 222));
        
        // UI elements
        components.insert(ThemeComponentType::Border, Color::rgb(69, 69, 69));
        components.insert(ThemeComponentType::Shadow, Color::rgba(0, 0, 0, 128));
        components.insert(ThemeComponentType::Hover, Color::rgb(45, 45, 45));
        components.insert(ThemeComponentType::Active, Color::rgb(51, 51, 51));
        components.insert(ThemeComponentType::Disabled, Color::rgb(102, 102, 102));
        
        // Create properties
        let mut properties = HashMap::new();
        properties.insert("font-family".to_string(), "Segoe UI, sans-serif".to_string());
        properties.insert("font-size".to_string(), "14px".to_string());
        properties.insert("border-radius".to_string(), "4px".to_string());
        properties.insert("animation-duration".to_string(), "0.2s".to_string());
        
        Theme {
            theme_id: "default-dark".to_string(),
            name: "Default Dark".to_string(),
            description: "Default dark theme for NovaDE".to_string(),
            variant: ThemeVariant::Dark,
            author: "NovaDE Team".to_string(),
            version: "1.0.0".to_string(),
            components,
            properties,
            created_at: Timestamp::now(),
            modified_at: Timestamp::now(),
        }
    }
}
