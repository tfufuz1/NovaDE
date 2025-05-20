//! Workspace management module for the NovaDE domain layer.
//!
//! This module provides functionality for managing workspaces,
//! including workspace creation, configuration, and window assignment.

pub mod core;
pub mod assignment;
pub mod manager;
pub mod config;

// Re-export key types for convenience
pub use core::{Workspace, WorkspaceId, WorkspaceType};
pub use assignment::{WindowAssignment, AssignmentRule};
pub use manager::{WorkspaceManagerService, DefaultWorkspaceManager};
pub use config::{WorkspaceConfig, WorkspaceConfigProvider};
