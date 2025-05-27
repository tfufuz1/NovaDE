// Main public interface for novade_domain
// This file re-exports key entities, services, and errors from the submodules.

// Submodule declarations
pub mod ai_interaction_service;
pub mod common_events;
pub mod entities;
pub mod error;
pub mod global_settings;
pub mod global_settings_management;
pub mod notification;
pub mod notifications;
pub mod notifications_rules;
pub mod power_management;
pub mod repositories;
pub mod settings;
pub mod shared_types;
pub mod theming;
pub mod user_centric_services;
pub mod window_management;
pub mod window_management_policy;
pub mod workspaces;
pub mod ports; // Added for traits like ConfigServiceAsync

// Re-exports for easier access by consumers of the crate

// From error module
pub use error::DomainError;

// From entities module (example re-exports, adjust based on actual entities)
pub use entities::configuration::AppConfiguration;
pub use entities::project::Project;
pub use entities::task::Task;
pub mod task_management { // Example of grouping re-exports
    pub use crate::entities::task::{Task, TaskPriority, TaskStatus};
}


// From power_management module
pub use power_management::{
    PowerManagementService, DefaultPowerManagementService, 
    BatteryInfo, PowerState, PowerCapabilities, PowerError, PowerEvent,
    SystemPowerState // Assuming this is also part of the public API
};

// From workspaces module
pub use workspaces::{
    WorkspaceManagerService, DefaultWorkspaceManager, // Assuming DefaultWorkspaceManager is also for public use
    Workspace, WorkspaceId, WindowIdentifier, WorkspaceError, WorkspaceEvent, WorkspaceLayoutType,
    WorkspaceConfigProvider, FilesystemConfigProvider // Re-exporting provider and its concrete impl
};
pub use workspaces::common_types::*; // Re-export common_types used by workspaces

// From theming module
pub use theming::{ThemeManager, Theme, ThemeMeta, ThemeError, ThemeChangeEvent, ThemeMode, ColorToken, FontToken, ThemeDefinition};

// From ports (newly added)
pub use ports::config_service::ConfigServiceAsync;

// Other important re-exports would follow a similar pattern, for example:
// pub use global_settings::{GlobalSettingsService, GlobalSettingsError, GlobalSettingKey, GlobalSettingValue};
// pub use notifications::{NotificationService, Notification, NotificationLevel, NotificationError};
// pub use ai_interaction_service::{AIInteractionService, UserQuery, AIResponse, AIInteractionError, AIInteractionEvent};

// It's good practice to only re-export the parts of the domain that are meant to be public.
// If some modules are internal to novade-domain, they shouldn't be re-exported here.

// Example for a specific service if its trait and default impl are commonly used:
// pub use crate::some_service_module::{SomeServiceTrait, DefaultSomeServiceImpl};

// Ensure all re-exported types that use other re-exported types are consistent.
// E.g. if Workspace uses WindowIdentifier, both should be accessible.
// The `pub use module_name::*` pattern can be broad; specific re-exports are often better for a controlled API.
// For this pass, I'm re-exporting key service traits, their default implementations if common,
// primary data DTOs, and errors.
// The structure above is a more complete example of what a lib.rs might look like.
// The previous content `pub mod workspaces;` was far too minimal.
// Based on the errors in `novade-domain/src/workspaces/config/provider.rs`, it needs access to `novade_core::CoreError`.
// This file (`novade-domain/src/lib.rs`) doesn't directly help with that, that's a dependency in `novade-domain/Cargo.toml`
// and correct `use` statements in the `provider.rs` file itself.
// The purpose of *this* file is to define the public API of the `novade-domain` crate.
// The critical part for the current error is making `ConfigServiceAsync` available.
// And ensuring other modules like `power_management` are declared if `power_mcp_server` needs them.
// `power_mcp_server` will need `DefaultPowerManagementService`, etc.
// So, declaring `pub mod power_management;` and re-exporting its contents is vital.
