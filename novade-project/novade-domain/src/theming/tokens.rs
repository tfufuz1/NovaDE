//! Token module for the NovaDE domain layer.
//!
//! This module provides token types and utilities for theme token
//! management in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

/// A theme token value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// A string value.
    String(String),
    /// A color value.
    Color(String),
    /// A dimension value.
    Dimension(String),
    /// A number value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A reference to another token.
    Reference(String),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenValue::String(s) => write!(f, "{}", s),
            TokenValue::Color(c) => write!(f, "{}", c),
            TokenValue::Dimension(d) => write!(f, "{}", d),
            TokenValue::Number(n) => write!(f, "{}", n),
            TokenValue::Boolean(b) => write!(f, "{}", b),
            TokenValue::Reference(r) => write!(f, "{{{}}}", r),
        }
    }
}

/// A theme token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeToken {
    /// The value of the token.
    pub value: TokenValue,
    /// The description of the token.
    pub description: Option<String>,
}

impl ThemeToken {
    /// Creates a new theme token.
    ///
    /// # Arguments
    ///
    /// * `value` - The value of the token
    ///
    /// # Returns
    ///
    /// A new theme token.
    pub fn new(value: TokenValue) -> Self {
        ThemeToken {
            value,
            description: None,
        }
    }

    /// Creates a new theme token with a description.
    ///
    /// # Arguments
    ///
    /// * `value` - The value of the token
    /// * `description` - The description of the token
    ///
    /// # Returns
    ///
    /// A new theme token with a description.
    pub fn with_description(value: TokenValue, description: impl Into<String>) -> Self {
        ThemeToken {
            value,
            description: Some(description.into()),
        }
    }

    /// Gets the value of the token.
    pub fn value(&self) -> &TokenValue {
        &self.value
    }

    /// Gets the description of the token.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description of the token.
    ///
    /// # Arguments
    ///
    /// * `description` - The new description of the token
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }
}

impl fmt::Display for ThemeToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{} ({})", self.value, description)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

/// A token path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenPath {
    /// The segments of the path.
    segments: Vec<String>,
}

impl TokenPath {
    /// Creates a new token path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path as a string
    ///
    /// # Returns
    ///
    /// A new token path.
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let segments = path.split('.').map(|s| s.to_string()).collect();
        TokenPath { segments }
    }

    /// Gets the segments of the path.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Gets the parent path.
    ///
    /// # Returns
    ///
    /// The parent path, or `None` if this is a root path.
    pub fn parent(&self) -> Option<TokenPath> {
        if self.segments.len() <= 1 {
            None
        } else {
            let parent_segments = self.segments[0..self.segments.len() - 1].to_vec();
            Some(TokenPath {
                segments: parent_segments,
            })
        }
    }

    /// Gets the last segment of the path.
    ///
    /// # Returns
    ///
    /// The last segment, or `None` if the path is empty.
    pub fn last_segment(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }

    /// Joins this path with another segment.
    ///
    /// # Arguments
    ///
    /// * `segment` - The segment to join
    ///
    /// # Returns
    ///
    /// A new path with the segment appended.
    pub fn join(&self, segment: impl Into<String>) -> TokenPath {
        let mut segments = self.segments.clone();
        segments.push(segment.into());
        TokenPath { segments }
    }
}

impl From<&str> for TokenPath {
    fn from(s: &str) -> Self {
        TokenPath::new(s)
    }
}

impl From<String> for TokenPath {
    fn from(s: String) -> Self {
        TokenPath::new(s)
    }
}

impl fmt::Display for TokenPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

/// A context for token resolution.
#[derive(Debug, Clone)]
pub struct TokenResolutionContext {
    /// The tokens available for resolution.
    tokens: HashMap<String, TokenValue>,
    /// The maximum depth for reference resolution.
    max_depth: usize,
}

impl TokenResolutionContext {
    /// Creates a new token resolution context.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens available for resolution
    /// * `max_depth` - The maximum depth for reference resolution
    ///
    /// # Returns
    ///
    /// A new token resolution context.
    pub fn new(tokens: HashMap<String, TokenValue>, max_depth: usize) -> Self {
        TokenResolutionContext { tokens, max_depth }
    }

    /// Gets the tokens available for resolution.
    pub fn tokens(&self) -> &HashMap<String, TokenValue> {
        &self.tokens
    }

    /// Gets the maximum depth for reference resolution.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Resolves a token value.
    ///
    /// # Arguments
    ///
    /// * `value` - The token value to resolve
    ///
    /// # Returns
    ///
    /// The resolved token value, or the original value if it couldn't be resolved.
    pub fn resolve(&self, value: &TokenValue) -> TokenValue {
        self.resolve_with_depth(value, 0)
    }

    /// Resolves a token value with a depth counter.
    ///
    /// # Arguments
    ///
    /// * `value` - The token value to resolve
    /// * `depth` - The current depth of resolution
    ///
    /// # Returns
    ///
    /// The resolved token value, or the original value if it couldn't be resolved.
    fn resolve_with_depth(&self, value: &TokenValue, depth: usize) -> TokenValue {
        if depth >= self.max_depth {
            return value.clone();
        }

        match value {
            TokenValue::Reference(path) => {
                if let Some(referenced_value) = self.tokens.get(path) {
                    self.resolve_with_depth(referenced_value, depth + 1)
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_value_display() {
        assert_eq!(format!("{}", TokenValue::String("test".to_string())), "test");
        assert_eq!(format!("{}", TokenValue::Color("#FF0000".to_string())), "#FF0000");
        assert_eq!(format!("{}", TokenValue::Dimension("10px".to_string())), "10px");
        assert_eq!(format!("{}", TokenValue::Number(42.0)), "42");
        assert_eq!(format!("{}", TokenValue::Boolean(true)), "true");
        assert_eq!(format!("{}", TokenValue::Reference("colors.primary".to_string())), "{colors.primary}");
    }

    #[test]
    fn test_theme_token_new() {
        let token = ThemeToken::new(TokenValue::Color("#FF0000".to_string()));
        
        assert_eq!(token.value(), &TokenValue::Color("#FF0000".to_string()));
        assert_eq!(token.description(), None);
    }

    #[test]
    fn test_theme_token_with_description() {
        let token = ThemeToken::with_description(
            TokenValue::Color("#FF0000".to_string()),
            "Primary color",
        );
        
        assert_eq!(token.value(), &TokenValue::Color("#FF0000".to_string()));
        assert_eq!(token.description(), Some("Primary color"));
    }

    #[test]
    fn test_theme_token_set_description() {
        let mut token = ThemeToken::new(TokenValue::Color("#FF0000".to_string()));
        
        token.set_description(Some("Primary color".to_string()));
        assert_eq!(token.description(), Some("Primary color"));
        
        token.set_description(None);
        assert_eq!(token.description(), None);
    }

    #[test]
    fn test_theme_token_display() {
        let token1 = ThemeToken::new(TokenValue::Color("#FF0000".to_string()));
        assert_eq!(format!("{}", token1), "#FF0000");
        
        let token2 = ThemeToken::with_description(
            TokenValue::Color("#FF0000".to_string()),
            "Primary color",
        );
        assert_eq!(format!("{}", token2), "#FF0000 (Primary color)");
    }

    #[test]
    fn test_token_path_new() {
        let path = TokenPath::new("colors.primary");
        
        assert_eq!(path.segments(), &["colors".to_string(), "primary".to_string()]);
    }

    #[test]
    fn test_token_path_parent() {
        let path1 = TokenPath::new("colors.primary");
        let parent1 = path1.parent();
        
        assert!(parent1.is_some());
        assert_eq!(parent1.unwrap().segments(), &["colors".to_string()]);
        
        let path2 = TokenPath::new("colors");
        let parent2 = path2.parent();
        
        assert!(parent2.is_none());
    }

    #[test]
    fn test_token_path_last_segment() {
        let path1 = TokenPath::new("colors.primary");
        assert_eq!(path1.last_segment(), Some("primary"));
        
        let path2 = TokenPath::new("");
        assert_eq!(path2.last_segment(), Some(""));
    }

    #[test]
    fn test_token_path_join() {
        let path1 = TokenPath::new("colors");
        let path2 = path1.join("primary");
        
        assert_eq!(path2.segments(), &["colors".to_string(), "primary".to_string()]);
    }

    #[test]
    fn test_token_path_from() {
        let path1: TokenPath = "colors.primary".into();
        assert_eq!(path1.segments(), &["colors".to_string(), "primary".to_string()]);
        
        let path2: TokenPath = "colors.primary".to_string().into();
        assert_eq!(path2.segments(), &["colors".to_string(), "primary".to_string()]);
    }

    #[test]
    fn test_token_path_display() {
        let path = TokenPath::new("colors.primary");
        assert_eq!(format!("{}", path), "colors.primary");
    }

    #[test]
    fn test_token_resolution_context() {
        let mut tokens = HashMap::new();
        tokens.insert("colors.primary".to_string(), TokenValue::Color("#FF0000".to_string()));
        tokens.insert("colors.secondary".to_string(), TokenValue::Reference("colors.primary".to_string()));
        tokens.insert("colors.tertiary".to_string(), TokenValue::Reference("colors.secondary".to_string()));
        tokens.insert("colors.circular".to_string(), TokenValue::Reference("colors.circular".to_string()));
        
        let context = TokenResolutionContext::new(tokens, 10);
        
        // Direct value
        let resolved1 = context.resolve(&TokenValue::Color("#00FF00".to_string()));
        assert_eq!(resolved1, TokenValue::Color("#00FF00".to_string()));
        
        // Single reference
        let resolved2 = context.resolve(&TokenValue::Reference("colors.primary".to_string()));
        assert_eq!(resolved2, TokenValue::Color("#FF0000".to_string()));
        
        // Double reference
        let resolved3 = context.resolve(&TokenValue::Reference("colors.secondary".to_string()));
        assert_eq!(resolved3, TokenValue::Color("#FF0000".to_string()));
        
        // Triple reference
        let resolved4 = context.resolve(&TokenValue::Reference("colors.tertiary".to_string()));
        assert_eq!(resolved4, TokenValue::Color("#FF0000".to_string()));
        
        // Circular reference (should stop at max_depth)
        let resolved5 = context.resolve(&TokenValue::Reference("colors.circular".to_string()));
        assert_eq!(resolved5, TokenValue::Reference("colors.circular".to_string()));
        
        // Non-existent reference
        let resolved6 = context.resolve(&TokenValue::Reference("colors.nonexistent".to_string()));
        assert_eq!(resolved6, TokenValue::Reference("colors.nonexistent".to_string()));
    }
}
