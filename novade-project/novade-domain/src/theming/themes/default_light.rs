//! Default light theme for the NovaDE desktop environment.
//!
//! This module provides the default light theme for the NovaDE desktop environment.

use std::collections::HashMap;
use crate::types::color::Color;
use crate::entities::value_objects::Timestamp;
use super::super::{Theme, ThemeVariant, ThemeComponentType};

/// Default light theme implementation.
pub struct DefaultLightTheme;

impl DefaultLightTheme {
    /// Creates the default light theme.
    pub fn create() -> Theme {
        let mut components = HashMap::new();
        
        // Base colors
        components.insert(ThemeComponentType::Background, Color::rgb(248, 248, 248));
        components.insert(ThemeComponentType::Foreground, Color::rgb(33, 33, 33));
        components.insert(ThemeComponentType::Primary, Color::rgb(0, 120, 215));
        components.insert(ThemeComponentType::Secondary, Color::rgb(0, 99, 177));
        
        // Status colors
        components.insert(ThemeComponentType::Success, Color::rgb(16, 124, 16));
        components.insert(ThemeComponentType::Warning, Color::rgb(197, 134, 7));
        components.insert(ThemeComponentType::Error, Color::rgb(232, 17, 35));
        components.insert(ThemeComponentType::Info, Color::rgb(0, 120, 215));
        
        // UI elements
        components.insert(ThemeComponentType::Border, Color::rgb(213, 213, 213));
        components.insert(ThemeComponentType::Shadow, Color::rgba(0, 0, 0, 77));
        components.insert(ThemeComponentType::Hover, Color::rgb(229, 241, 251));
        components.insert(ThemeComponentType::Active, Color::rgb(204, 228, 247));
        components.insert(ThemeComponentType::Disabled, Color::rgb(204, 204, 204));
        
        // Create properties
        let mut properties = HashMap::new();
        properties.insert("font-family".to_string(), "Segoe UI, sans-serif".to_string());
        properties.insert("font-size".to_string(), "14px".to_string());
        properties.insert("border-radius".to_string(), "4px".to_string());
        properties.insert("animation-duration".to_string(), "0.2s".to_string());
        
        Theme {
            theme_id: "default-light".to_string(),
            name: "Default Light".to_string(),
            description: "Default light theme for NovaDE".to_string(),
            variant: ThemeVariant::Light,
            author: "NovaDE Team".to_string(),
            version: "1.0.0".to_string(),
            components,
            properties,
            created_at: Timestamp::now(),
            modified_at: Timestamp::now(),
        }
    }
}
