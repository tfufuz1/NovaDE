//! Value objects for the NovaDE domain layer.
//!
//! This module provides immutable value objects that represent descriptive
//! aspects of the domain without conceptual identity.

use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Represents an email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress {
    address: String,
}

impl EmailAddress {
    /// Creates a new email address.
    ///
    /// # Arguments
    ///
    /// * `address` - The email address string
    ///
    /// # Returns
    ///
    /// A `Result` containing the `EmailAddress` if valid, or an error if invalid.
    pub fn new<S: Into<String>>(address: S) -> Result<Self, String> {
        let address = address.into();
        
        // Basic validation: check for @ symbol and non-empty parts
        if !address.contains('@') {
            return Err("Email address must contain '@' symbol".to_string());
        }
        
        let parts: Vec<&str> = address.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Email address must have non-empty local and domain parts".to_string());
        }
        
        // Check for at least one dot in domain part
        if !parts[1].contains('.') {
            return Err("Domain part must contain at least one dot".to_string());
        }
        
        Ok(Self { address })
    }
    
    /// Returns the email address as a string.
    pub fn as_str(&self) -> &str {
        &self.address
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

/// Represents a specific point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    datetime: DateTime<Utc>,
}

impl Timestamp {
    /// Creates a new timestamp from a UTC datetime.
    pub fn new(datetime: DateTime<Utc>) -> Self {
        Self { datetime }
    }
    
    /// Creates a timestamp representing the current time.
    pub fn now() -> Self {
        Self { datetime: Utc::now() }
    }
    
    /// Returns the underlying datetime.
    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
    
    /// Returns true if this timestamp is in the past.
    pub fn is_past(&self) -> bool {
        self.datetime < Utc::now()
    }
    
    /// Returns true if this timestamp is in the future.
    pub fn is_future(&self) -> bool {
        self.datetime > Utc::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.datetime.format("%Y-%m-%d %H:%M:%S UTC"))
    }
}

/// Represents the status of a task or project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    /// Not yet started
    Open,
    /// Currently being worked on
    InProgress,
    /// Temporarily paused
    OnHold,
    /// Completed successfully
    Completed,
    /// Abandoned or cancelled
    Cancelled,
}

impl Status {
    /// Returns true if the status is considered active (Open or InProgress).
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Open | Status::InProgress)
    }
    
    /// Returns true if the status is considered completed (Completed or Cancelled).
    pub fn is_completed(&self) -> bool {
        matches!(self, Status::Completed | Status::Cancelled)
    }
    
    /// Returns a string representation of the status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "Open",
            Status::InProgress => "In Progress",
            Status::OnHold => "On Hold",
            Status::Completed => "Completed",
            Status::Cancelled => "Cancelled",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents the priority of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    /// Lowest priority
    Low,
    /// Normal priority
    Medium,
    /// High priority
    High,
    /// Urgent priority
    Critical,
}

impl Priority {
    /// Returns a numeric value representing the priority (higher = more important).
    pub fn value(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }
    
    /// Returns a string representation of the priority.
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_address_valid() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }
    
    #[test]
    fn test_email_address_invalid_no_at() {
        let result = EmailAddress::new("userexample.com");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_email_address_invalid_empty_parts() {
        let result = EmailAddress::new("@example.com");
        assert!(result.is_err());
        
        let result = EmailAddress::new("user@");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_email_address_invalid_no_dot() {
        let result = EmailAddress::new("user@examplecom");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_timestamp_now() {
        let now = Timestamp::now();
        assert!(!now.is_future());
    }
    
    #[test]
    fn test_timestamp_past_future() {
        use chrono::Duration;
        
        let past = Timestamp::new(Utc::now() - Duration::hours(1));
        assert!(past.is_past());
        assert!(!past.is_future());
        
        let future = Timestamp::new(Utc::now() + Duration::hours(1));
        assert!(!future.is_past());
        assert!(future.is_future());
    }
    
    #[test]
    fn test_status_active_completed() {
        assert!(Status::Open.is_active());
        assert!(Status::InProgress.is_active());
        assert!(!Status::OnHold.is_active());
        assert!(!Status::Completed.is_active());
        assert!(!Status::Cancelled.is_active());
        
        assert!(!Status::Open.is_completed());
        assert!(!Status::InProgress.is_completed());
        assert!(!Status::OnHold.is_completed());
        assert!(Status::Completed.is_completed());
        assert!(Status::Cancelled.is_completed());
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }
    
    #[test]
    fn test_priority_value() {
        assert_eq!(Priority::Low.value(), 0);
        assert_eq!(Priority::Medium.value(), 1);
        assert_eq!(Priority::High.value(), 2);
        assert_eq!(Priority::Critical.value(), 3);
    }
}
