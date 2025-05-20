//! High contrast theme for the NovaDE desktop environment.
//!
//! This module provides a high contrast theme for the NovaDE desktop environment,
//! designed for accessibility and improved readability.

use std::collections::HashMap;
use crate::types::color::Color;
use crate::entities::value_objects::Timestamp;
use super::super::{Theme, ThemeVariant, ThemeComponentType};

/// High contrast theme implementation.
pub struct HighContrastTheme;

impl HighContrastTheme {
    /// Creates the high contrast theme.
    pub fn create() -> Theme {
        let mut components = HashMap::new();
        
        // Base colors - high contrast black and white
        components.insert(ThemeComponentType::Background, Color::rgb(0, 0, 0));
        components.insert(ThemeComponentType::Foreground, Color::rgb(255, 255, 255));
        components.insert(ThemeComponentType::Primary, Color::rgb(255, 255, 0));
        components.insert(ThemeComponentType::Secondary, Color::rgb(0, 255, 255));
        
        // Status colors - highly distinguishable
        components.insert(ThemeComponentType::Success, Color::rgb(0, 255, 0));
        components.insert(ThemeComponentType::Warning, Color::rgb(255, 255, 0));
        components.insert(ThemeComponentType::Error, Color::rgb(255, 0, 0));
        components.insert(ThemeComponentType::Info, Color::rgb(0, 255, 255));
        
        // UI elements - strong contrast
        components.insert(ThemeComponentType::Border, Color::rgb(255, 255, 255));
        components.insert(ThemeComponentType::Shadow, Color::rgba(255, 255, 255, 255));
        components.insert(ThemeComponentType::Hover, Color::rgb(128, 128, 0));
        components.insert(ThemeComponentType::Active, Color::rgb(255, 255, 0));
        components.insert(ThemeComponentType::Disabled, Color::rgb(128, 128, 128));
        
        // Create properties
        let mut properties = HashMap::new();
        properties.insert("font-family".to_string(), "Segoe UI, sans-serif".to_string());
        properties.insert("font-size".to_string(), "16px".to_string()); // Larger font for readability
        properties.insert("border-radius".to_string(), "0px".to_string()); // No rounded corners for clearer boundaries
        properties.insert("animation-duration".to_string(), "0s".to_string()); // No animations
        properties.insert("border-width".to_string(), "2px".to_string()); // Thicker borders
        properties.insert("focus-outline-width".to_string(), "3px".to_string()); // Very visible focus indicators
        
        Theme {
            theme_id: "high-contrast".to_string(),
            name: "High Contrast".to_string(),
            description: "High contrast theme for improved accessibility and readability".to_string(),
            variant: ThemeVariant::HighContrast,
            author: "NovaDE Team".to_string(),
            version: "1.0.0".to_string(),
            components,
            properties,
            created_at: Timestamp::now(),
            modified_at: Timestamp::now(),
        }
    }
}
