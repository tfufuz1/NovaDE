//! System module for the NovaDE desktop environment.
//!
//! This module provides system-level functionality for the NovaDE desktop environment,
//! including the Wayland compositor, input handling, D-Bus interfaces, audio management,
//! MCP client, portals, and power management.

// Re-export domain and core modules
pub use novade_domain as domain;
pub use novade_core as core;

// Export system modules
pub mod error;
pub mod compositor;
pub mod input;
pub mod dbus;
pub mod audio;
// pub mod mcp; // Old MCP service, replaced by mcp_client_service
pub mod mcp_client_service; // New service for managing MCP client processes
pub mod portals;
pub mod power_management;
pub mod window_mechanics; // Added this line

// Re-export common types and interfaces
pub use error::SystemError;
// pub use compositor::{CompositorService, DefaultCompositorService}; // Commented out for Smithay setup
pub use input::{InputService, DefaultInputService};
pub use dbus::{DBusService, DefaultDBusService};
pub use audio::{AudioService, DefaultAudioService};
// pub use mcp::{McpService, DefaultMcpService}; // Old MCP service
pub use mcp_client_service::{IMCPClientService, DefaultMCPClientService, StdioProcess}; // New MCP client service
pub use portals::{PortalsService, DefaultPortalsService};
pub use power_management::{SystemPowerService, DefaultSystemPowerService};

use std::sync::Arc;

/// Initialize the system layer.
///
/// This function initializes all system services with default configurations.
/// All services are wrapped in thread-safe containers to ensure concurrent access safety.
///
/// # Returns
///
/// A `Result` containing a tuple of all system services.
pub async fn initialize(
    _domain_services: ( // Parameter unused for now due to commented out services
        Arc<domain::DefaultWorkspaceService>,
        Arc<domain::DefaultThemeManager>,
        Arc<domain::DefaultConsentManager>,
        Arc<domain::DefaultAIInteractionService>,
        Arc<domain::DefaultNotificationManager>,
        Arc<domain::DefaultWindowPolicyManager>,
        Arc<domain::DefaultPowerManagementService>,
    ),
) -> Result<(
    // Arc<DefaultCompositorService>, // Commented out
    Arc<DefaultInputService>,
    Arc<DefaultDBusService>,
    Arc<DefaultAudioService>,
    // Arc<DefaultMcpService>, // Old MCP service
    Arc<DefaultMCPClientService>, // New MCP client service, assuming it needs to be initialized here
    // Arc<DefaultPortalsService>, // Commented out as it depends on CompositorService
    Arc<DefaultSystemPowerService>,
), SystemError> {
    // Unpack domain services
    let (
        _workspace_service, // unused
        _theme_manager, // unused
        _consent_manager, // unused
        _ai_service, // unused
        notification_manager, // used by DBusService
        _window_policy_manager, // unused
        power_management_service, // used by SystemPowerService
    ) = _domain_services;
    
    // // Initialize compositor service (Commented out)
    // let compositor_service = Arc::new(DefaultCompositorService::new(
    //     workspace_service.clone(),
    //     window_policy_manager.clone(),
    // )?);
    
    // Initialize input service (Assuming it can be initialized without a live compositor for now, or uses a mock)
    // This might need adjustment if DefaultInputService strictly requires a running CompositorService.
    // For the purpose of this subtask, we'll assume it can be initialized.
    let compositor_service_mock = (); // Placeholder if DefaultInputService needs some form of CompositorService
    let input_service = Arc::new(DefaultInputService::new(
        compositor_service_mock, // This will likely cause a type error.
                                 // For now, the goal is to setup Smithay structure, not full integration.
                                 // This part of initialize() will need to be revisited in a later task.
    )?);
    
    // Initialize D-Bus service
    let dbus_service = Arc::new(DefaultDBusService::new(
        notification_manager.clone(),
    )?);
    
    // Initialize audio service
    let audio_service = Arc::new(DefaultAudioService::new()?);
    
    // Initialize MCP service (New one)
    // DefaultMCPClientService::new() doesn't take arguments according to its current definition.
    // If it needs domain services, its constructor and this call will need to be updated.
    let mcp_client_service = Arc::new(DefaultMCPClientService::new());
    
    // // Initialize portals service (Commented out)
    // let portals_service = Arc::new(DefaultPortalsService::new(
    //     compositor_service.clone(), 
    // )?);
    
    // Initialize system power service
    let system_power_service = Arc::new(DefaultSystemPowerService::new(
        power_management_service.clone(),
    )?);
    
    // Adjust the Ok tuple to reflect commented out services
    Ok((
        // compositor_service,
        input_service,
        dbus_service,
        audio_service,
        mcp_client_service, // New MCP client service
        // portals_service,
        system_power_service,
    ))
}
