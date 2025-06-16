//! Defines general-purpose, universal data types for NovaDE.
//!
//! This module provides common data types like UUIDs and Timestamps that are
//! used across various parts of the NovaDE system. These types are often
//! simple wrappers or aliases for established Rust crates, ensuring consistency
//! and providing a clear public API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid as ExternalUuid; // Alias to avoid conflict if we wrap it later

// --- Uuid ---

/// A universally unique identifier (UUID).
///
/// This struct is a transparent wrapper around `uuid::Uuid` from the `uuid` crate.
/// It ensures that UUIDs are handled consistently within NovaDE.
///
/// The `uuid` crate should be configured with the "v4" (for `new_v4`) and "serde"
/// (for serialization) features for this struct to function as intended.
///
/// The `#[serde(transparent)]` attribute ensures that this struct is serialized
/// and deserialized as if it were the underlying `ExternalUuid` directly (i.e., as a string).
///
/// # Examples
/// ```
/// use novade_core::types::Uuid;
///
/// let new_id = Uuid::new_v4();
/// println!("Generated UUID: {}", new_id);
///
/// let nil_id = Uuid::nil();
/// assert_eq!(nil_id.to_string(), "00000000-0000-0000-0000-000000000000");
///
/// let parsed_id = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8");
/// assert!(parsed_id.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)] // Serialize/Deserialize as the inner Uuid directly
pub struct Uuid(
    /// The underlying UUID from the `uuid` crate.
    ExternalUuid
);

impl Uuid {
    /// Creates a new random (version 4) UUID.
    ///
    /// This relies on the "v4" feature of the `uuid` crate.
    pub fn new_v4() -> Self {
        Uuid(ExternalUuid::new_v4())
    }

    /// Returns the nil UUID.
    ///
    /// The nil UUID is a special UUID, `00000000-0000-0000-0000-000000000000`.
    pub fn nil() -> Self {
        Uuid(ExternalUuid::nil())
    }

    /// Parses a Uuid from a string slice.
    ///
    /// The string can be in various formats (e.g., "simple", "hyphenated", "urn").
    ///
    /// # Arguments
    /// * `s`: The string slice to parse.
    ///
    /// # Errors
    /// Returns `uuid::Error` if the string is not a valid UUID representation.
    pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
        ExternalUuid::parse_str(s).map(Uuid)
    }

    /// Returns a reference to the underlying `uuid::Uuid` type.
    ///
    /// This can be useful for interoperability with code that expects the original `uuid::Uuid`.
    pub fn as_external(&self) -> &ExternalUuid {
        &self.0
    }
}

impl Default for Uuid {
    /// Returns the nil UUID by default.
    ///
    /// This is consistent with `uuid::Uuid::default()` if the "v5" feature is not enabled
    /// for the `uuid` crate, which often defaults to nil for safety.
    fn default() -> Self {
        Uuid::nil()
    }
}

impl std::fmt::Display for Uuid {
    /// Formats the UUID as a hyphenated string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.hyphenated().fmt(f) // Use hyphenated for consistent string representation
    }
}

// --- Timestamp ---

/// Represents a specific moment in time, in Coordinated Universal Time (UTC).
///
/// This struct is a transparent wrapper around `chrono::DateTime<chrono::Utc>`.
/// It standardizes timestamp handling within NovaDE.
///
/// The `chrono` crate should be configured with the "serde" feature for this
/// struct to function as intended with serialization.
///
/// The `#[serde(transparent)]` attribute ensures that this struct is serialized
/// and deserialized as if it were the underlying `DateTime<Utc>` directly
/// (i.e., typically as an RFC 3339 string).
///
/// # Examples
/// ```
/// use novade_core::types::Timestamp;
///
/// let now = Timestamp::now();
/// println!("Current time: {}", now);
///
/// let parsed_time = Timestamp::parse_from_rfc3339("2023-01-01T12:00:00Z");
/// assert!(parsed_time.is_ok());
///
/// let epoch_time = Timestamp::default();
/// assert_eq!(epoch_time.to_string(), "1970-01-01 00:00:00 UTC");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)] // Serialize/Deserialize as the inner DateTime<Utc> directly
pub struct Timestamp(
    /// The underlying `DateTime<Utc>` from the `chrono` crate.
    DateTime<Utc>
);

impl Timestamp {
    /// Returns the current system time in UTC.
    pub fn now() -> Self {
        Timestamp(Utc::now())
    }

    /// Parses a Timestamp from an RFC 3339 formatted string.
    ///
    /// RFC 3339 is a common standard for representing timestamps as strings,
    /// e.g., "2023-10-26T10:30:00Z" or "2023-10-26T12:30:00+02:00".
    /// The parsed time will be converted to UTC.
    ///
    /// # Arguments
    /// * `s`: The string slice to parse.
    ///
    /// # Errors
    /// Returns `chrono::ParseError` if the string is not a valid RFC 3339 timestamp.
    pub fn parse_from_rfc3339(s: &str) -> Result<Self, chrono::ParseError> {
        DateTime::parse_from_rfc3339(s).map(|dt| Timestamp(dt.with_timezone(&Utc)))
    }

    /// Returns a reference to the underlying `chrono::DateTime<Utc>` type.
    ///
    /// This can be useful for interoperability with code that expects the original `chrono` type.
    pub fn as_external(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Default for Timestamp {
    /// Returns a default timestamp, which is the Unix epoch: "1970-01-01 00:00:00 UTC".
    /// This is consistent with `chrono::DateTime::<Utc>::default()`.
    fn default() -> Self {
        Timestamp(DateTime::<Utc>::default())
    }
}

impl std::fmt::Display for Timestamp {
    /// Formats the timestamp according to RFC 3339 (e.g., "YYYY-MM-DD HH:MM:SS.fffffffff UTC").
    /// Note: `chrono`'s default `Display` for `DateTime<Utc>` includes " UTC" suffix.
    /// For strict RFC3339 "Z" notation, use `timestamp.as_external().to_rfc3339()`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    // Ensure common traits are implemented
    assert_impl_all!(Uuid: Send, Sync, std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Serialize, PartialOrd, Ord, Default, std::fmt::Display);
    assert_impl_all!(Timestamp: Send, Sync, std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, PartialOrd, Ord, Serialize, Default, std::fmt::Display);

    // --- Uuid Tests ---
    #[test]
    fn uuid_creation() {
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        assert_ne!(u1, u2, "Two new v4 UUIDs should not be equal");
        assert_ne!(u1, Uuid::nil(), "A new v4 UUID should not be nil");
    }

    #[test]
    fn uuid_nil() {
        let nil_uuid = Uuid::nil();
        assert!(nil_uuid.as_external().is_nil());
        assert_eq!(Uuid::default(), nil_uuid, "Default Uuid should be nil");
    }

    #[test]
    fn uuid_parse_str_valid() {
        let s = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";
        let u = Uuid::parse_str(s).expect("Failed to parse valid UUID string");
        assert_eq!(u.to_string(), s);
    }

    #[test]
    fn uuid_parse_str_invalid() {
        assert!(Uuid::parse_str("invalid-uuid-string").is_err());
    }

    #[test]
    fn uuid_display() {
        let s = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8";
        let u = Uuid::parse_str(s).unwrap();
        assert_eq!(format!("{}", u), s);
    }

    #[test]
    fn uuid_serde_transparent() {
        let u = Uuid::new_v4();
        let serialized = serde_json::to_string(&u).expect("Failed to serialize Uuid");
        // Expected: a simple string like ""a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8""
        assert!(serialized.starts_with("\"") && serialized.ends_with("\"") && serialized.len() == 36 + 2);

        let deserialized: Uuid = serde_json::from_str(&serialized).expect("Failed to deserialize Uuid");
        assert_eq!(u, deserialized);

        // Test with nil UUID
        let nil_uuid = Uuid::nil();
        let serialized_nil = serde_json::to_string(&nil_uuid).expect("Failed to serialize nil Uuid");
        assert_eq!(serialized_nil, "\"00000000-0000-0000-0000-000000000000\"");
        let deserialized_nil: Uuid = serde_json::from_str(&serialized_nil).expect("Failed to deserialize nil Uuid");
        assert_eq!(nil_uuid, deserialized_nil);
    }

    // --- Timestamp Tests ---
    #[test]
    fn timestamp_now() {
        let ts1 = Timestamp::now();
        // Small delay to ensure a different timestamp if the clock ticks
        std::thread::sleep(std::time::Duration::from_micros(10));
        let ts2 = Timestamp::now();
        assert!(ts1.as_external() <= ts2.as_external(), "Timestamp::now() should produce increasing or equal times");
    }

    #[test]
    fn timestamp_default() {
        let ts_default = Timestamp::default();
        let chrono_default = DateTime::<Utc>::default();
        assert_eq!(ts_default.as_external(), &chrono_default, "Default Timestamp should match chrono's default");
        // Should be "1970-01-01T00:00:00Z"
        assert_eq!(ts_default.to_string(), "1970-01-01 00:00:00 UTC");
    }

    #[test]
    fn timestamp_parse_from_rfc3339_valid() {
        let s = "2023-10-26T10:30:00Z";
        let ts = Timestamp::parse_from_rfc3339(s).expect("Failed to parse valid RFC3339 string");
        // Note: chrono's default to_string for DateTime<Utc> includes " UTC"
        assert_eq!(ts.to_string(), "2023-10-26 10:30:00 UTC");

        let s_with_offset = "2023-10-26T12:30:00+02:00"; // Same as 10:30:00Z
        let ts_offset = Timestamp::parse_from_rfc3339(s_with_offset).expect("Failed to parse valid RFC3339 string with offset");
        assert_eq!(ts_offset.to_string(), "2023-10-26 10:30:00 UTC");
        assert_eq!(ts, ts_offset);
    }

    #[test]
    fn timestamp_parse_from_rfc3339_invalid() {
        assert!(Timestamp::parse_from_rfc3339("invalid-timestamp-string").is_err());
    }

    #[test]
    fn timestamp_display() {
        let ts = Timestamp(Utc::now());
        // Display format is like "YYYY-MM-DD HH:MM:SS.fffffffff UTC"
        // We check it's not empty and contains "UTC"
        let formatted = format!("{}", ts);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn timestamp_serde_transparent() {
        let ts = Timestamp::now();
        let serialized = serde_json::to_string(&ts).expect("Failed to serialize Timestamp");
        // Expected: RFC 3339 string like ""2023-10-26T10:30:00.123456789Z""
        assert!(serialized.starts_with("\"") && serialized.ends_with("\"") && serialized.contains("T") && serialized.contains("Z"));

        let deserialized: Timestamp = serde_json::from_str(&serialized).expect("Failed to deserialize Timestamp");
        // Direct comparison of DateTime<Utc> can have issues with nanosecond precision
        // if not careful, but here it should be fine as it's the same object serialized/deserialized.
        assert_eq!(ts, deserialized);

        // Test with default (epoch)
        let ts_default = Timestamp::default();
        let serialized_default = serde_json::to_string(&ts_default).expect("Failed to serialize default Timestamp");
        assert_eq!(serialized_default, "\"1970-01-01T00:00:00Z\"");
        let deserialized_default: Timestamp = serde_json::from_str(&serialized_default).expect("Failed to deserialize default Timestamp");
        assert_eq!(ts_default, deserialized_default);
    }
}
