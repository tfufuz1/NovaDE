// Main module for window management policy definitions.

pub mod types;
pub mod errors;
pub mod service; // For the WindowManagementPolicyService trait and its impl

// Re-exports for easier access by consumers of the crate.
// These will be populated as the types and service trait are defined.
// Example:
pub use types::{TilingMode, GapSettings, WorkspaceWindowLayout, WindowPolicyOverrides, FocusPolicy, NewWindowPlacementStrategy, WindowSnappingPolicy, WindowGroupingPolicy, FocusStealingPreventionLevel, WindowLayoutInfo};
pub use errors::WindowPolicyError;
pub use service::{WindowManagementPolicyService, DefaultWindowManagementPolicyService}; // Updated
