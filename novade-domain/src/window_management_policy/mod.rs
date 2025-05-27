// Declare submodules
pub mod types;
pub mod errors;
pub mod service;

// Re-export main public types for easier access
pub use self::types::{
    TilingMode,
    NewWindowPlacementStrategy,
    GapSettings,
    WindowSnappingPolicy,
    WindowLayoutInfo,
    WorkspaceWindowLayout,
    WindowPolicyOverrides, // New
    FocusPolicy, // New
    FocusStealingPreventionLevel, // New
    WindowGroupingPolicy, // New
};
pub use self::errors::WindowPolicyError;
pub use self::service::{
    WindowManagementPolicyService,
    DefaultWindowManagementPolicyService,
};
