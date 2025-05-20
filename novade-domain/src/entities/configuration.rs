//! Configuration entity for the NovaDE domain layer.
//!
//! This module provides the Configuration entity, which represents
//! user-specific settings in the NovaDE desktop environment.

use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entities::value_objects::Timestamp;

/// Represents a configuration in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuration {
    /// Unique identifier for the configuration
    config_id: String,
    /// The user ID this configuration belongs to
    user_id: String,
    /// The configuration name
    name: String,
    /// The configuration values as key-value pairs
    values: std::collections::HashMap<String, ConfigValue>,
    /// The configuration creation timestamp
    created_at: Timestamp,
    /// The configuration last modified timestamp
    modified_at: Timestamp,
}

/// Represents a configuration value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigValue {
    /// A string value
    String(String),
    /// An integer value
    Integer(i64),
    /// A floating-point value
    Float(f64),
    /// A boolean value
    Boolean(bool),
    /// An array of configuration values
    Array(Vec<ConfigValue>),
    /// A nested object of configuration values
    Object(std::collections::HashMap<String, ConfigValue>),
}

impl Configuration {
    /// Creates a new configuration for the given user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID this configuration belongs to
    /// * `name` - The configuration name
    ///
    /// # Returns
    ///
    /// A new `Configuration` with a generated UUID and default values.
    pub fn new<S1: Into<String>, S2: Into<String>>(user_id: S1, name: S2) -> Self {
        let now = Timestamp::now();
        Self {
            config_id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            name: name.into(),
            values: std::collections::HashMap::new(),
            created_at: now,
            modified_at: now,
        }
    }
    
    /// Creates a new configuration with the given ID.
    ///
    /// # Arguments
    ///
    /// * `config_id` - The configuration's unique identifier
    /// * `user_id` - The user ID this configuration belongs to
    /// * `name` - The configuration name
    ///
    /// # Returns
    ///
    /// A new `Configuration` with the specified ID and default values.
    pub fn with_id<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        config_id: S1,
        user_id: S2,
        name: S3,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            config_id: config_id.into(),
            user_id: user_id.into(),
            name: name.into(),
            values: std::collections::HashMap::new(),
            created_at: now,
            modified_at: now,
        }
    }
    
    /// Returns the configuration ID.
    pub fn config_id(&self) -> &str {
        &self.config_id
    }
    
    /// Returns the user ID this configuration belongs to.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    
    /// Returns the configuration name.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Sets the configuration name.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
        self.modified_at = Timestamp::now();
    }
    
    /// Returns the configuration creation timestamp.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
    
    /// Returns the configuration last modified timestamp.
    pub fn modified_at(&self) -> Timestamp {
        self.modified_at
    }
    
    /// Returns the configuration values.
    pub fn values(&self) -> &std::collections::HashMap<String, ConfigValue> {
        &self.values
    }
    
    /// Returns a configuration value by key.
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&ConfigValue> {
        self.values.get(key.as_ref())
    }
    
    /// Sets a configuration value.
    pub fn set<K: Into<String>>(&mut self, key: K, value: ConfigValue) {
        self.values.insert(key.into(), value);
        self.modified_at = Timestamp::now();
    }
    
    /// Removes a configuration value.
    pub fn remove<K: AsRef<str>>(&mut self, key: K) -> Option<ConfigValue> {
        let result = self.values.remove(key.as_ref());
        if result.is_some() {
            self.modified_at = Timestamp::now();
        }
        result
    }
    
    /// Clears all configuration values.
    pub fn clear(&mut self) {
        if !self.values.is_empty() {
            self.values.clear();
            self.modified_at = Timestamp::now();
        }
    }
    
    /// Returns the number of configuration values.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Returns whether the configuration has no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration({}, {}, {})", self.config_id, self.user_id, self.name)
    }
}

impl ConfigValue {
    /// Creates a new string value.
    pub fn string<S: Into<String>>(value: S) -> Self {
        ConfigValue::String(value.into())
    }
    
    /// Creates a new integer value.
    pub fn integer(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
    
    /// Creates a new floating-point value.
    pub fn float(value: f64) -> Self {
        ConfigValue::Float(value)
    }
    
    /// Creates a new boolean value.
    pub fn boolean(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
    
    /// Creates a new array value.
    pub fn array(values: Vec<ConfigValue>) -> Self {
        ConfigValue::Array(values)
    }
    
    /// Creates a new object value.
    pub fn object(values: std::collections::HashMap<String, ConfigValue>) -> Self {
        ConfigValue::Object(values)
    }
    
    /// Returns whether this is a string value.
    pub fn is_string(&self) -> bool {
        matches!(self, ConfigValue::String(_))
    }
    
    /// Returns whether this is an integer value.
    pub fn is_integer(&self) -> bool {
        matches!(self, ConfigValue::Integer(_))
    }
    
    /// Returns whether this is a floating-point value.
    pub fn is_float(&self) -> bool {
        matches!(self, ConfigValue::Float(_))
    }
    
    /// Returns whether this is a boolean value.
    pub fn is_boolean(&self) -> bool {
        matches!(self, ConfigValue::Boolean(_))
    }
    
    /// Returns whether this is an array value.
    pub fn is_array(&self) -> bool {
        matches!(self, ConfigValue::Array(_))
    }
    
    /// Returns whether this is an object value.
    pub fn is_object(&self) -> bool {
        matches!(self, ConfigValue::Object(_))
    }
    
    /// Returns the string value, if this is a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Returns the integer value, if this is an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    /// Returns the floating-point value, if this is a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }
    
    /// Returns the boolean value, if this is a boolean.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Returns the array value, if this is an array.
    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            ConfigValue::Array(a) => Some(a),
            _ => None,
        }
    }
    
    /// Returns the object value, if this is an object.
    pub fn as_object(&self) -> Option<&std::collections::HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "\"{}\"", s),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            ConfigValue::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_configuration_new() {
        let config = Configuration::new("user123", "app_settings");
        
        assert!(!config.config_id().is_empty());
        assert_eq!(config.user_id(), "user123");
        assert_eq!(config.name(), "app_settings");
        assert!(config.values().is_empty());
    }
    
    #[test]
    fn test_configuration_with_id() {
        let config = Configuration::with_id("config123", "user123", "app_settings");
        
        assert_eq!(config.config_id(), "config123");
        assert_eq!(config.user_id(), "user123");
        assert_eq!(config.name(), "app_settings");
    }
    
    #[test]
    fn test_configuration_set_name() {
        let mut config = Configuration::new("user123", "app_settings");
        let original_modified = config.modified_at();
        
        // Wait a moment to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        config.set_name("new_settings");
        assert_eq!(config.name(), "new_settings");
        assert!(config.modified_at().datetime() > original_modified.datetime());
    }
    
    #[test]
    fn test_configuration_values() {
        let mut config = Configuration::new("user123", "app_settings");
        
        // Test setting values
        config.set("theme", ConfigValue::string("dark"));
        config.set("font_size", ConfigValue::integer(14));
        config.set("show_notifications", ConfigValue::boolean(true));
        
        // Test getting values
        assert_eq!(config.get("theme").unwrap().as_string(), Some("dark"));
        assert_eq!(config.get("font_size").unwrap().as_integer(), Some(14));
        assert_eq!(config.get("show_notifications").unwrap().as_boolean(), Some(true));
        
        // Test non-existent key
        assert_eq!(config.get("non_existent"), None);
        
        // Test removing a value
        let removed = config.remove("font_size");
        assert_eq!(removed.unwrap().as_integer(), Some(14));
        assert_eq!(config.get("font_size"), None);
        
        // Test length
        assert_eq!(config.len(), 2);
        
        // Test clear
        config.clear();
        assert!(config.is_empty());
    }
    
    #[test]
    fn test_config_value_types() {
        // Test string
        let string_val = ConfigValue::string("test");
        assert!(string_val.is_string());
        assert_eq!(string_val.as_string(), Some("test"));
        
        // Test integer
        let int_val = ConfigValue::integer(42);
        assert!(int_val.is_integer());
        assert_eq!(int_val.as_integer(), Some(42));
        
        // Test float
        let float_val = ConfigValue::float(3.14);
        assert!(float_val.is_float());
        assert_eq!(float_val.as_float(), Some(3.14));
        
        // Test boolean
        let bool_val = ConfigValue::boolean(true);
        assert!(bool_val.is_boolean());
        assert_eq!(bool_val.as_boolean(), Some(true));
        
        // Test array
        let array_val = ConfigValue::array(vec![
            ConfigValue::string("item1"),
            ConfigValue::integer(2),
        ]);
        assert!(array_val.is_array());
        let array = array_val.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0].as_string(), Some("item1"));
        assert_eq!(array[1].as_integer(), Some(2));
        
        // Test object
        let mut obj = std::collections::HashMap::new();
        obj.insert("key1".to_string(), ConfigValue::string("value1"));
        obj.insert("key2".to_string(), ConfigValue::integer(2));
        let obj_val = ConfigValue::object(obj);
        assert!(obj_val.is_object());
        let object = obj_val.as_object().unwrap();
        assert_eq!(object.len(), 2);
        assert_eq!(object.get("key1").unwrap().as_string(), Some("value1"));
        assert_eq!(object.get("key2").unwrap().as_integer(), Some(2));
    }
    
    #[test]
    fn test_config_value_display() {
        assert_eq!(format!("{}", ConfigValue::string("test")), "\"test\"");
        assert_eq!(format!("{}", ConfigValue::integer(42)), "42");
        assert_eq!(format!("{}", ConfigValue::float(3.14)), "3.14");
        assert_eq!(format!("{}", ConfigValue::boolean(true)), "true");
        
        let array_val = ConfigValue::array(vec![
            ConfigValue::string("item1"),
            ConfigValue::integer(2),
        ]);
        assert_eq!(format!("{}", array_val), "[\"item1\", 2]");
        
        let mut obj = std::collections::HashMap::new();
        obj.insert("key1".to_string(), ConfigValue::string("value1"));
        obj.insert("key2".to_string(), ConfigValue::integer(2));
        let obj_val = ConfigValue::object(obj);
        
        // HashMap doesn't guarantee order, so we need to check both possibilities
        let display = format!("{}", obj_val);
        assert!(
            display == "{\"key1\": \"value1\", \"key2\": 2}" ||
            display == "{\"key2\": 2, \"key1\": \"value1\"}"
        );
    }
}
