//! Represents the status of a component or operation.

use serde::{Deserialize, Serialize};

/// Represents the operational status of a component or process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    /// The component is active and functioning normally.
    Enabled,
    /// The component is not active or has been intentionally deactivated.
    Disabled,
    /// The component is in a transitional or indeterminate state (e.g., initializing).
    Pending,
    /// The component has encountered an error, with an associated error code or state.
    Error(i32),
}

impl Default for Status {
    /// Returns `Status::Disabled` by default.
    fn default() -> Self {
        Status::Disabled
    }
}

impl Status {
    /// Checks if the status represents an active state.
    ///
    /// `Enabled` and `Pending` are considered active states.
    /// `Disabled` and `Error` are considered inactive.
    ///
    /// # Examples
    /// ```
    /// # use novade_core::types::status::Status;
    /// assert!(Status::Enabled.is_active());
    /// assert!(Status::Pending.is_active());
    /// assert!(!Status::Disabled.is_active());
    /// assert!(!Status::Error(1).is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        match self {
            Status::Enabled | Status::Pending => true,
            Status::Disabled | Status::Error(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;
    use std::fmt; // Required for fmt::Debug
    use std::hash::Hash; // Required for std::hash::Hash

    assert_impl_all!(Status: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Deserialize<'static>, Send, Sync);

    #[test]
    fn status_default_is_disabled() {
        assert_eq!(Status::default(), Status::Disabled);
    }

    #[test]
    fn status_is_active() {
        assert!(Status::Enabled.is_active());
        assert!(Status::Pending.is_active());
        assert!(!Status::Disabled.is_active());
        assert!(!Status::Error(0).is_active());
        assert!(!Status::Error(-1).is_active());
        assert!(!Status::Error(123).is_active());
    }

    #[test]
    fn status_derives_work() {
        let s1 = Status::Enabled;
        let s2 = Status::Enabled;
        let s3 = Status::Error(10);

        assert_eq!(s1, s2); // PartialEq
        assert_ne!(s1, s3);

        let s_clone = s1.clone(); // Clone
        assert_eq!(s1, s_clone);

        // Copy is implicit if clone works and it's Copy trait
        let s_copied = s1;
        assert_eq!(s1, s_copied);


        println!("{:?}", s1); // Debug
    }
    
    #[test]
    fn status_serde_serialization_deserialization() {
        let statuses = [
            Status::Enabled,
            Status::Disabled,
            Status::Pending,
            Status::Error(404),
            Status::Error(-1),
        ];

        for original_status in &statuses {
            let serialized = serde_json::to_string(original_status).unwrap();
            
            // Expected JSON format depends on the enum variant
            let expected_json = match original_status {
                Status::Enabled => "\"Enabled\"".to_string(),
                Status::Disabled => "\"Disabled\"".to_string(),
                Status::Pending => "\"Pending\"".to_string(),
                Status::Error(code) => format!("{{\"Error\":{}}}", code),
            };
            assert_eq!(serialized, expected_json);

            let deserialized: Status = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, *original_status);
        }
    }

    #[test]
    fn status_error_variant_with_value() {
        let status_err = Status::Error(123);
        match status_err {
            Status::Error(code) => assert_eq!(code, 123),
            _ => panic!("Expected Status::Error variant"),
        }
        assert!(!status_err.is_active());
    }
}
