//! Repository interfaces for the NovaDE domain layer.
//!
//! This module provides repository interfaces for accessing and persisting
//! domain entities in the NovaDE desktop environment.

use async_trait::async_trait;
use crate::error::DomainError;
use crate::entities::{UserProfile, Task, Project, Configuration};

pub mod user_profile;
pub mod task;
pub mod project;
pub mod configuration;

// Re-export repository traits for convenience
pub use user_profile::UserProfileRepository;
pub use task::TaskRepository;
pub use project::ProjectRepository;
pub use configuration::ConfigurationRepository;
