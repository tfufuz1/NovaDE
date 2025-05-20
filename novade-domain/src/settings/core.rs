//! Core settings types for the NovaDE domain layer.
//!
//! This module provides the fundamental types and structures
//! for global settings management in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;
use crate::error::{DomainResult, SettingsError};

/// A setting key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettingKey {
    /// The category of the setting.
    pub category: SettingCategory,
    /// The name of the setting.
    pub name: String,
}

impl SettingKey {
    /// Creates a new setting key.
    ///
    /// # Arguments
    ///
    /// * `category` - The category of the setting
    /// * `name` - The name of the setting
    ///
    /// # Returns
    ///
    /// A new setting key.
    pub fn new(category: SettingCategory, name: impl Into<String>) -> Self {
        SettingKey {
            category,
            name: name.into(),
        }
    }

    /// Parses a setting key from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// The parsed setting key, or an error if parsing failed.
    pub fn parse(s: &str) -> Result<Self, SettingsError> {
        let parts: Vec<&str> = s.split('.').collect();
        
        if parts.len() != 2 {
            return Err(SettingsError::InvalidKey(s.to_string()));
        }
        
        let category = SettingCategory::from_str(parts[0])?;
        let name = parts[1].to_string();
        
        Ok(SettingKey { category, name })
    }
}

impl fmt::Display for SettingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.category, self.name)
    }
}

/// A setting category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingCategory {
    /// General settings.
    General,
    /// Appearance settings.
    Appearance,
    /// Behavior settings.
    Behavior,
    /// Performance settings.
    Performance,
    /// Privacy settings.
    Privacy,
    /// Accessibility settings.
    Accessibility,
    /// Advanced settings.
    Advanced,
}

impl SettingCategory {
    /// Gets the display name of the category.
    ///
    /// # Returns
    ///
    /// The display name of the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingCategory::General => "General",
            SettingCategory::Appearance => "Appearance",
            SettingCategory::Behavior => "Behavior",
            SettingCategory::Performance => "Performance",
            SettingCategory::Privacy => "Privacy",
            SettingCategory::Accessibility => "Accessibility",
            SettingCategory::Advanced => "Advanced",
        }
    }
}

impl FromStr for SettingCategory {
    type Err = SettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "general" => Ok(SettingCategory::General),
            "appearance" => Ok(SettingCategory::Appearance),
            "behavior" => Ok(SettingCategory::Behavior),
            "performance" => Ok(SettingCategory::Performance),
            "privacy" => Ok(SettingCategory::Privacy),
            "accessibility" => Ok(SettingCategory::Accessibility),
            "advanced" => Ok(SettingCategory::Advanced),
            _ => Err(SettingsError::InvalidCategory(s.to_string())),
        }
    }
}

impl fmt::Display for SettingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name().to_lowercase())
    }
}

/// A setting value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    /// A string value.
    String(String),
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// An array of values.
    Array(Vec<SettingValue>),
    /// A map of values.
    Map(HashMap<String, SettingValue>),
}

impl SettingValue {
    /// Checks if the value is a string.
    ///
    /// # Returns
    ///
    /// `true` if the value is a string, `false` otherwise.
    pub fn is_string(&self) -> bool {
        matches!(self, SettingValue::String(_))
    }

    /// Gets the value as a string.
    ///
    /// # Returns
    ///
    /// The value as a string, or `None` if it's not a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SettingValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Checks if the value is an integer.
    ///
    /// # Returns
    ///
    /// `true` if the value is an integer, `false` otherwise.
    pub fn is_integer(&self) -> bool {
        matches!(self, SettingValue::Integer(_))
    }

    /// Gets the value as an integer.
    ///
    /// # Returns
    ///
    /// The value as an integer, or `None` if it's not an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            SettingValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Checks if the value is a floating-point number.
    ///
    /// # Returns
    ///
    /// `true` if the value is a floating-point number, `false` otherwise.
    pub fn is_float(&self) -> bool {
        matches!(self, SettingValue::Float(_))
    }

    /// Gets the value as a floating-point number.
    ///
    /// # Returns
    ///
    /// The value as a floating-point number, or `None` if it's not a floating-point number.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            SettingValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Checks if the value is a boolean.
    ///
    /// # Returns
    ///
    /// `true` if the value is a boolean, `false` otherwise.
    pub fn is_boolean(&self) -> bool {
        matches!(self, SettingValue::Boolean(_))
    }

    /// Gets the value as a boolean.
    ///
    /// # Returns
    ///
    /// The value as a boolean, or `None` if it's not a boolean.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            SettingValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Checks if the value is an array.
    ///
    /// # Returns
    ///
    /// `true` if the value is an array, `false` otherwise.
    pub fn is_array(&self) -> bool {
        matches!(self, SettingValue::Array(_))
    }

    /// Gets the value as an array.
    ///
    /// # Returns
    ///
    /// The value as an array, or `None` if it's not an array.
    pub fn as_array(&self) -> Option<&[SettingValue]> {
        match self {
            SettingValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Checks if the value is a map.
    ///
    /// # Returns
    ///
    /// `true` if the value is a map, `false` otherwise.
    pub fn is_map(&self) -> bool {
        matches!(self, SettingValue::Map(_))
    }

    /// Gets the value as a map.
    ///
    /// # Returns
    ///
    /// The value as a map, or `None` if it's not a map.
    pub fn as_map(&self) -> Option<&HashMap<String, SettingValue>> {
        match self {
            SettingValue::Map(m) => Some(m),
            _ => None,
        }
    }
}

impl From<String> for SettingValue {
    fn from(s: String) -> Self {
        SettingValue::String(s)
    }
}

impl From<&str> for SettingValue {
    fn from(s: &str) -> Self {
        SettingValue::String(s.to_string())
    }
}

impl From<i64> for SettingValue {
    fn from(i: i64) -> Self {
        SettingValue::Integer(i)
    }
}

impl From<i32> for SettingValue {
    fn from(i: i32) -> Self {
        SettingValue::Integer(i as i64)
    }
}

impl From<f64> for SettingValue {
    fn from(f: f64) -> Self {
        SettingValue::Float(f)
    }
}

impl From<f32> for SettingValue {
    fn from(f: f32) -> Self {
        SettingValue::Float(f as f64)
    }
}

impl From<bool> for SettingValue {
    fn from(b: bool) -> Self {
        SettingValue::Boolean(b)
    }
}

impl<T> From<Vec<T>> for SettingValue
where
    T: Into<SettingValue>,
{
    fn from(v: Vec<T>) -> Self {
        SettingValue::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T> From<HashMap<String, T>> for SettingValue
where
    T: Into<SettingValue>,
{
    fn from(m: HashMap<String, T>) -> Self {
        SettingValue::Map(m.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl fmt::Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingValue::String(s) => write!(f, "{}", s),
            SettingValue::Integer(i) => write!(f, "{}", i),
            SettingValue::Float(fl) => write!(f, "{}", fl),
            SettingValue::Boolean(b) => write!(f, "{}", b),
            SettingValue::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            SettingValue::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// A setting in the NovaDE desktop environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    /// The key of the setting.
    pub key: SettingKey,
    /// The value of the setting.
    pub value: SettingValue,
    /// The description of the setting.
    pub description: Option<String>,
    /// Whether the setting is read-only.
    pub read_only: bool,
}

impl Setting {
    /// Creates a new setting.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    /// * `value` - The value of the setting
    ///
    /// # Returns
    ///
    /// A new setting.
    pub fn new(key: SettingKey, value: impl Into<SettingValue>) -> Self {
        Setting {
            key,
            value: value.into(),
            description: None,
            read_only: false,
        }
    }

    /// Creates a new setting with a description.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    /// * `value` - The value of the setting
    /// * `description` - The description of the setting
    ///
    /// # Returns
    ///
    /// A new setting with a description.
    pub fn with_description(
        key: SettingKey,
        value: impl Into<SettingValue>,
        description: impl Into<String>,
    ) -> Self {
        Setting {
            key,
            value: value.into(),
            description: Some(description.into()),
            read_only: false,
        }
    }

    /// Creates a new read-only setting.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    /// * `value` - The value of the setting
    ///
    /// # Returns
    ///
    /// A new read-only setting.
    pub fn read_only(key: SettingKey, value: impl Into<SettingValue>) -> Self {
        Setting {
            key,
            value: value.into(),
            description: None,
            read_only: true,
        }
    }

    /// Gets the key of the setting.
    pub fn key(&self) -> &SettingKey {
        &self.key
    }

    /// Gets the value of the setting.
    pub fn value(&self) -> &SettingValue {
        &self.value
    }

    /// Sets the value of the setting.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value of the setting
    ///
    /// # Returns
    ///
    /// `Ok(())` if the value was set, or an error if the setting is read-only.
    pub fn set_value(&mut self, value: impl Into<SettingValue>) -> DomainResult<()> {
        if self.read_only {
            return Err(SettingsError::ReadOnly(self.key.to_string()).into());
        }
        
        self.value = value.into();
        Ok(())
    }

    /// Gets the description of the setting.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description of the setting.
    ///
    /// # Arguments
    ///
    /// * `description` - The new description of the setting
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Checks if the setting is read-only.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Sets whether the setting is read-only.
    ///
    /// # Arguments
    ///
    /// * `read_only` - Whether the setting is read-only
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Validates the setting.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the setting is valid, or an error if it is invalid.
    pub fn validate(&self) -> DomainResult<()> {
        // For now, all settings are considered valid
        Ok(())
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_setting_key_new() {
        let key = SettingKey::new(SettingCategory::General, "language");
        
        assert_eq!(key.category, SettingCategory::General);
        assert_eq!(key.name, "language");
    }
    
    #[test]
    fn test_setting_key_parse() {
        let key = SettingKey::parse("general.language").unwrap();
        
        assert_eq!(key.category, SettingCategory::General);
        assert_eq!(key.name, "language");
        
        let error = SettingKey::parse("invalid").unwrap_err();
        assert!(matches!(error, SettingsError::InvalidKey(_)));
        
        let error = SettingKey::parse("invalid.key.with.dots").unwrap_err();
        assert!(matches!(error, SettingsError::InvalidKey(_)));
        
        let error = SettingKey::parse("unknown.key").unwrap_err();
        assert!(matches!(error, SettingsError::InvalidCategory(_)));
    }
    
    #[test]
    fn test_setting_key_display() {
        let key = SettingKey::new(SettingCategory::General, "language");
        
        assert_eq!(format!("{}", key), "general.language");
    }
    
    #[test]
    fn test_setting_category_display_name() {
        assert_eq!(SettingCategory::General.display_name(), "General");
        assert_eq!(SettingCategory::Appearance.display_name(), "Appearance");
        assert_eq!(SettingCategory::Behavior.display_name(), "Behavior");
        assert_eq!(SettingCategory::Performance.display_name(), "Performance");
        assert_eq!(SettingCategory::Privacy.display_name(), "Privacy");
        assert_eq!(SettingCategory::Accessibility.display_name(), "Accessibility");
        assert_eq!(SettingCategory::Advanced.display_name(), "Advanced");
    }
    
    #[test]
    fn test_setting_category_from_str() {
        assert_eq!(SettingCategory::from_str("general").unwrap(), SettingCategory::General);
        assert_eq!(SettingCategory::from_str("GENERAL").unwrap(), SettingCategory::General);
        assert_eq!(SettingCategory::from_str("General").unwrap(), SettingCategory::General);
        
        let error = SettingCategory::from_str("unknown").unwrap_err();
        assert!(matches!(error, SettingsError::InvalidCategory(_)));
    }
    
    #[test]
    fn test_setting_category_display() {
        assert_eq!(format!("{}", SettingCategory::General), "general");
        assert_eq!(format!("{}", SettingCategory::Appearance), "appearance");
    }
    
    #[test]
    fn test_setting_value_accessors() {
        let string_value = SettingValue::String("test".to_string());
        assert!(string_value.is_string());
        assert_eq!(string_value.as_string(), Some("test"));
        assert!(!string_value.is_integer());
        assert_eq!(string_value.as_integer(), None);
        
        let int_value = SettingValue::Integer(42);
        assert!(int_value.is_integer());
        assert_eq!(int_value.as_integer(), Some(42));
        assert!(!int_value.is_string());
        assert_eq!(int_value.as_string(), None);
        
        let float_value = SettingValue::Float(3.14);
        assert!(float_value.is_float());
        assert_eq!(float_value.as_float(), Some(3.14));
        
        let bool_value = SettingValue::Boolean(true);
        assert!(bool_value.is_boolean());
        assert_eq!(bool_value.as_boolean(), Some(true));
        
        let array_value = SettingValue::Array(vec![
            SettingValue::String("a".to_string()),
            SettingValue::Integer(1),
        ]);
        assert!(array_value.is_array());
        assert_eq!(array_value.as_array().unwrap().len(), 2);
        
        let mut map = HashMap::new();
        map.insert("key".to_string(), SettingValue::String("value".to_string()));
        let map_value = SettingValue::Map(map);
        assert!(map_value.is_map());
        assert_eq!(map_value.as_map().unwrap().len(), 1);
    }
    
    #[test]
    fn test_setting_value_from() {
        let string_value: SettingValue = "test".into();
        assert_eq!(string_value, SettingValue::String("test".to_string()));
        
        let string_value: SettingValue = "test".to_string().into();
        assert_eq!(string_value, SettingValue::String("test".to_string()));
        
        let int_value: SettingValue = 42i64.into();
        assert_eq!(int_value, SettingValue::Integer(42));
        
        let int_value: SettingValue = 42i32.into();
        assert_eq!(int_value, SettingValue::Integer(42));
        
        let float_value: SettingValue = 3.14f64.into();
        assert_eq!(float_value, SettingValue::Float(3.14));
        
        let float_value: SettingValue = 3.14f32.into();
        assert_eq!(float_value, SettingValue::Float(3.14 as f64));
        
        let bool_value: SettingValue = true.into();
        assert_eq!(bool_value, SettingValue::Boolean(true));
        
        let array_value: SettingValue = vec!["a", "b"].into();
        assert_eq!(
            array_value,
            SettingValue::Array(vec![
                SettingValue::String("a".to_string()),
                SettingValue::String("b".to_string()),
            ])
        );
        
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value");
        let map_value: SettingValue = map.into();
        
        if let SettingValue::Map(m) = map_value {
            assert_eq!(m.len(), 1);
            assert_eq!(m.get("key"), Some(&SettingValue::String("value".to_string())));
        } else {
            panic!("Expected Map value");
        }
    }
    
    #[test]
    fn test_setting_value_display() {
        assert_eq!(format!("{}", SettingValue::String("test".to_string())), "test");
        assert_eq!(format!("{}", SettingValue::Integer(42)), "42");
        assert_eq!(format!("{}", SettingValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", SettingValue::Boolean(true)), "true");
        
        let array_value = SettingValue::Array(vec![
            SettingValue::String("a".to_string()),
            SettingValue::Integer(1),
        ]);
        assert_eq!(format!("{}", array_value), "[a, 1]");
        
        let mut map = HashMap::new();
        map.insert("key".to_string(), SettingValue::String("value".to_string()));
        let map_value = SettingValue::Map(map);
        assert_eq!(format!("{}", map_value), "{key: value}");
    }
    
    #[test]
    fn test_setting_new() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");
        
        assert_eq!(setting.key(), &key);
        assert_eq!(setting.value(), &SettingValue::String("en-US".to_string()));
        assert_eq!(setting.description(), None);
        assert!(!setting.is_read_only());
    }
    
    #[test]
    fn test_setting_with_description() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::with_description(key.clone(), "en-US", "The language to use");
        
        assert_eq!(setting.key(), &key);
        assert_eq!(setting.value(), &SettingValue::String("en-US".to_string()));
        assert_eq!(setting.description(), Some("The language to use"));
        assert!(!setting.is_read_only());
    }
    
    #[test]
    fn test_setting_read_only() {
        let key = SettingKey::new(SettingCategory::General, "version");
        let setting = Setting::read_only(key.clone(), "1.0.0");
        
        assert_eq!(setting.key(), &key);
        assert_eq!(setting.value(), &SettingValue::String("1.0.0".to_string()));
        assert_eq!(setting.description(), None);
        assert!(setting.is_read_only());
    }
    
    #[test]
    fn test_setting_set_value() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let mut setting = Setting::new(key.clone(), "en-US");
        
        setting.set_value("fr-FR").unwrap();
        assert_eq!(setting.value(), &SettingValue::String("fr-FR".to_string()));
        
        let mut read_only = Setting::read_only(key.clone(), "en-US");
        let result = read_only.set_value("fr-FR");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().downcast_ref::<SettingsError>(), Some(SettingsError::ReadOnly(_))));
    }
    
    #[test]
    fn test_setting_set_description() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let mut setting = Setting::new(key.clone(), "en-US");
        
        setting.set_description(Some("The language to use".to_string()));
        assert_eq!(setting.description(), Some("The language to use"));
        
        setting.set_description(None);
        assert_eq!(setting.description(), None);
    }
    
    #[test]
    fn test_setting_set_read_only() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let mut setting = Setting::new(key.clone(), "en-US");
        
        assert!(!setting.is_read_only());
        
        setting.set_read_only(true);
        assert!(setting.is_read_only());
        
        setting.set_read_only(false);
        assert!(!setting.is_read_only());
    }
    
    #[test]
    fn test_setting_validate() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");
        
        assert!(setting.validate().is_ok());
    }
    
    #[test]
    fn test_setting_display() {
        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");
        
        assert_eq!(format!("{}", setting), "general.language = en-US");
    }
}
