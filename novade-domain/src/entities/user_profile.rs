//! User profile entity for the NovaDE domain layer.
//!
//! This module provides the UserProfile entity, which represents a user
//! of the NovaDE desktop environment.

use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entities::value_objects::EmailAddress;

/// Represents a user profile in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique identifier for the user profile
    user_id: String,
    /// User's display name
    display_name: String,
    /// User's email address
    email: Option<EmailAddress>,
    /// User's preferred theme
    preferred_theme: Option<String>,
    /// Whether the user has completed initial setup
    setup_completed: bool,
}

impl UserProfile {
    /// Creates a new user profile with the given display name.
    ///
    /// # Arguments
    ///
    /// * `display_name` - The user's display name
    ///
    /// # Returns
    ///
    /// A new `UserProfile` with a generated UUID and default values.
    pub fn new<S: Into<String>>(display_name: S) -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            display_name: display_name.into(),
            email: None,
            preferred_theme: None,
            setup_completed: false,
        }
    }
    
    /// Creates a new user profile with the given ID and display name.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's unique identifier
    /// * `display_name` - The user's display name
    ///
    /// # Returns
    ///
    /// A new `UserProfile` with the specified ID and default values.
    pub fn with_id<S1: Into<String>, S2: Into<String>>(user_id: S1, display_name: S2) -> Self {
        Self {
            user_id: user_id.into(),
            display_name: display_name.into(),
            email: None,
            preferred_theme: None,
            setup_completed: false,
        }
    }
    
    /// Returns the user ID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    
    /// Returns the display name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    
    /// Sets the display name.
    pub fn set_display_name<S: Into<String>>(&mut self, display_name: S) {
        self.display_name = display_name.into();
    }
    
    /// Returns the email address, if set.
    pub fn email(&self) -> Option<&EmailAddress> {
        self.email.as_ref()
    }
    
    /// Sets the email address.
    pub fn set_email(&mut self, email: Option<EmailAddress>) {
        self.email = email;
    }
    
    /// Returns the preferred theme, if set.
    pub fn preferred_theme(&self) -> Option<&str> {
        self.preferred_theme.as_deref()
    }
    
    /// Sets the preferred theme.
    pub fn set_preferred_theme<S: Into<String>>(&mut self, theme: Option<S>) {
        self.preferred_theme = theme.map(|t| t.into());
    }
    
    /// Returns whether setup has been completed.
    pub fn setup_completed(&self) -> bool {
        self.setup_completed
    }
    
    /// Sets whether setup has been completed.
    pub fn set_setup_completed(&mut self, completed: bool) {
        self.setup_completed = completed;
    }
    
    /// Marks setup as completed.
    pub fn complete_setup(&mut self) {
        self.setup_completed = true;
    }
}

impl fmt::Display for UserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserProfile({}, {})", self.user_id, self.display_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_profile_new() {
        let profile = UserProfile::new("Test User");
        
        assert!(!profile.user_id().is_empty());
        assert_eq!(profile.display_name(), "Test User");
        assert_eq!(profile.email(), None);
        assert_eq!(profile.preferred_theme(), None);
        assert_eq!(profile.setup_completed(), false);
    }
    
    #[test]
    fn test_user_profile_with_id() {
        let profile = UserProfile::with_id("user123", "Test User");
        
        assert_eq!(profile.user_id(), "user123");
        assert_eq!(profile.display_name(), "Test User");
    }
    
    #[test]
    fn test_user_profile_setters() {
        let mut profile = UserProfile::new("Test User");
        
        profile.set_display_name("Updated User");
        assert_eq!(profile.display_name(), "Updated User");
        
        let email = EmailAddress::new("user@example.com").unwrap();
        profile.set_email(Some(email.clone()));
        assert_eq!(profile.email(), Some(&email));
        
        profile.set_preferred_theme(Some("dark"));
        assert_eq!(profile.preferred_theme(), Some("dark"));
        
        profile.set_setup_completed(true);
        assert_eq!(profile.setup_completed(), true);
    }
    
    #[test]
    fn test_user_profile_complete_setup() {
        let mut profile = UserProfile::new("Test User");
        assert_eq!(profile.setup_completed(), false);
        
        profile.complete_setup();
        assert_eq!(profile.setup_completed(), true);
    }
}
