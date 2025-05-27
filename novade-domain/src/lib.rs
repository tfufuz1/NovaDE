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
pub mod shared_types;
pub mod common_events; // Added common_events module
pub mod repositories;
pub mod workspace;
pub mod theming; // Already added, ensure it's correct
pub mod ai;
pub mod notification;
pub mod window_management;
pub mod power_management;
pub mod global_settings_management; // Add new module

// Re-export common types and interfaces
pub use error::DomainError;
pub use shared_types::{ApplicationId, UserSessionState, ResourceIdentifier};
pub use common_events::{UserActivityType, UserActivityDetectedEvent, ShutdownReason, SystemShutdownInitiatedEvent}; // Added common_events types
pub use workspace::{Workspace, WorkspaceService, DefaultWorkspaceService};
// Update theming re-exports as per the new structure
pub use theming::{
    ThemingEngine, ThemeChangedEvent, AppliedThemeState, ThemingConfiguration, 
    ThemeDefinition, TokenIdentifier, ThemeIdentifier, ColorSchemeType as ThemeColorSchemeType, AccentColor, ThemingError // Renamed ColorSchemeType to avoid conflict
};
// Re-export from global_settings_management
pub use global_settings_management::{
    GlobalDesktopSettings,
    SettingsPersistenceProvider, // Trait
    // FilesystemSettingsProvider, // Concrete type, optional to re-export from lib.rs
    SettingPath,
    GlobalSettingsError,
    AppearanceSettings,
    FontSettings,
    WorkspaceSettings,
    InputBehaviorSettings,
    PowerManagementPolicySettings,
    DefaultApplicationsSettings,
    ColorScheme as GlobalColorScheme, // Renamed to avoid conflict
    FontHinting,
    FontAntialiasing,
    MouseAccelerationProfile,
    LidCloseAction,
    WorkspaceSwitchingBehavior,
};
// Updated re-exports for the AI module to reflect new structure
pub use ai::{
    AIInteractionLogicService, DefaultAIInteractionLogicService, // Core logic service
    MCPConnectionService, MCPConsentManager, // Key MCP services from ai::mcp
    MCPServerConfig, ClientCapabilities, ServerInfo, ServerCapabilities, // Common MCP types from ai::mcp::types
    AIInteractionContext, AIModelProfile, AIDataCategory, AIConsentStatus, AIConsent, AttachmentData, AIInteractionError, // AI specific types from ai::mcp::types
    IMCPTransport // Transport trait from ai::mcp::transport
};
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
    // std::sync::Arc<DefaultConsentManager>, // Old
    // std::sync::Arc<DefaultAIInteractionService>, // Old
    // Replace with new AI services. DefaultAIInteractionLogicService now depends on MCPConsentManager and MCPConnectionService.
    // MCPConsentManager is simple. MCPConnectionService needs SystemIMCPClientService.
    std::sync::Arc<ai::MCPConsentManager>, // New
    std::sync::Arc<ai::DefaultAIInteractionLogicService>, // New
    std::sync::Arc<DefaultNotificationManager>,
    std::sync::Arc<DefaultWindowPolicyManager>,
    std::sync::Arc<DefaultPowerManagementService>,
), DomainError> {
    use std::sync::Arc;
    
    // Initialize workspace service
    let workspace_service = Arc::new(DefaultWorkspaceService::with_default_workspace()?);
    
    // Initialize theme manager
    let theme_manager = Arc::new(DefaultThemeManager::new()?);
    
    // Initialize new AI services
    // MCPConsentManager is straightforward.
    let mcp_consent_manager = Arc::new(ai::MCPConsentManager::new());

    // MCPConnectionService needs a SystemIMCPClientService.
    // Assuming novade_system::mcp_client_service::DefaultMCPClientService is available
    // and can be instantiated here. This implies novade-system is a dependency.
    let system_mcp_service = Arc::new(novade_system::mcp_client_service::DefaultMCPClientService::new());
    
    // Default client capabilities for MCPConnectionService
    let default_client_capabilities = ai::ClientCapabilities { supports_streaming: false }; // Example
    
    let mcp_connection_service = Arc::new(ai::MCPConnectionService::new(
        default_client_capabilities,
        system_mcp_service,
    ));
    
    // DefaultAIInteractionLogicService now takes MCPConnectionService and MCPConsentManager
    let ai_logic_service = Arc::new(ai::DefaultAIInteractionLogicService::new(
        mcp_connection_service,
        mcp_consent_manager.clone(), // Clone the Arc for MCPConsentManager
    ));
    
    // Initialize notification manager
    let notification_manager = Arc::new(DefaultNotificationManager::new());
    
    // Initialize window policy manager
    let window_policy_manager = Arc::new(DefaultWindowPolicyManager::with_default_policies()?);
    
    // Initialize power management service
    let power_management_service = Arc::new(DefaultPowerManagementService::new());
    
    Ok((
        workspace_service,
        theme_manager,
        mcp_consent_manager, // Return the new MCPConsentManager instance
        ai_logic_service,    // Return the new DefaultAIInteractionLogicService instance
        notification_manager,
        window_policy_manager,
        power_management_service,
    ))
}
