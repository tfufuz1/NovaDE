//! Entities module for the NovaDE domain layer.
//!
//! This module provides the core domain entities and value objects
//! used throughout the NovaDE desktop environment.

pub mod user_profile;
pub mod task;
pub mod project;
pub mod configuration;
pub mod value_objects;

// Re-export key types for convenience
pub use user_profile::UserProfile;
pub use task::Task;
pub use project::Project;
pub use configuration::Configuration;
pub use value_objects::{EmailAddress, Timestamp, Status, Priority};
