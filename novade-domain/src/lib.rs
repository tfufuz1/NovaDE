//! Domain module for the NovaDE desktop environment.
//!
//! This module provides domain-specific functionality for the NovaDE desktop environment,
//! including workspace management, theming, AI interactions, notifications, window management,
//! and power management.

// Re-export core module
pub use novade_core as core;

// Export domain modules
pub mod error;
pub mod entities;
pub mod repositories;
pub mod workspace;
pub mod theming;
pub mod ai;
pub mod notification;
pub mod window_management;
pub mod power_management;

// Re-export common types and interfaces
pub use error::DomainError;
pub use workspace::{Workspace, WorkspaceService, DefaultWorkspaceService};
pub use theming::{Theme, ThemeManager, DefaultThemeManager, ThemeVariant, ThemeComponentType};
pub use ai::{ConsentManager, AIInteractionService, DefaultConsentManager, DefaultAIInteractionService};
pub use notification::{NotificationManager, DefaultNotificationManager, NotificationCategory, NotificationUrgency};
pub use window_management::{WindowPolicyManager, DefaultWindowPolicyManager, WindowAction, WindowType, WindowState};
pub use power_management::{PowerManagementService, DefaultPowerManagementService, PowerState, BatteryState, BatteryInfo};

/// Initialize the domain layer.
///
/// This function initializes all domain services with default configurations.
/// All services are wrapped in thread-safe containers to ensure concurrent access safety.
///
/// # Returns
///
/// A `Result` containing a tuple of all domain services.
pub async fn initialize() -> Result<(
    std::sync::Arc<DefaultWorkspaceService>,
    std::sync::Arc<DefaultThemeManager>,
    std::sync::Arc<DefaultConsentManager>,
    std::sync::Arc<DefaultAIInteractionService>,
    std::sync::Arc<DefaultNotificationManager>,
    std::sync::Arc<DefaultWindowPolicyManager>,
    std::sync::Arc<DefaultPowerManagementService>,
), DomainError> {
    use std::sync::Arc;
    
    // Initialize workspace service
    let workspace_service = Arc::new(DefaultWorkspaceService::with_default_workspace()?);
    
    // Initialize theme manager
    let theme_manager = Arc::new(DefaultThemeManager::new()?);
    
    // Initialize consent manager
    let consent_manager = Arc::new(DefaultConsentManager::new());
    
    // Initialize AI interaction service
    let ai_service = Arc::new(DefaultAIInteractionService::new());
    
    // Initialize notification manager
    let notification_manager = Arc::new(DefaultNotificationManager::new());
    
    // Initialize window policy manager
    let window_policy_manager = Arc::new(DefaultWindowPolicyManager::with_default_policies()?);
    
    // Initialize power management service
    let power_management_service = Arc::new(DefaultPowerManagementService::new());
    
    Ok((
        workspace_service,
        theme_manager,
        consent_manager,
        ai_service,
        notification_manager,
        window_policy_manager,
        power_management_service,
    ))
}
