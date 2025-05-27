pub mod workspaces;

// Re-export key public types from the workspaces module (Iteration 1)
pub use workspaces::{
    // core types
    Workspace,
    WorkspaceId, // This is uuid::Uuid
    WindowIdentifier,
    WorkspaceLayoutType,
    WorkspaceCoreError,
    WorkspaceRenamedData,
    WorkspaceLayoutChangedData,
    // config types
    WorkspaceSnapshot,
    WorkspaceSetSnapshot,
    WorkspaceConfigError,
    WorkspaceConfigProvider,
    FilesystemConfigProvider, // Re-exporting the concrete provider for convenience
    // assignment types
    assign_window_to_workspace,
    remove_window_from_workspace,
    find_workspace_for_window,
    WindowAssignmentError,
    // manager types
    WorkspaceManagerService,
    DefaultWorkspaceManager,
    WorkspaceManagerError,
    WorkspaceEvent,
};

// Declare the theming module
pub mod theming;

// Re-export main public types from the theming module
pub use theming::{
    ThemingEngine,
    ThemeChangedEvent,
    ThemingConfiguration,
    AppliedThemeState,
    ThemeDefinition,
    ThemingError,
    // Specific token types if they are commonly used directly by consumers,
    // otherwise, they might be kept within the theming module's scope.
    // For example, TokenIdentifier and TokenValue might be too granular for top-level re-export
    // unless a consumer is expected to build themes programmatically frequently.
    // For now, keeping them out of top-level re-export unless a clear need arises.
    // Re-exporting TokenIdentifier as it's part of some public event/state structures indirectly.
    TokenIdentifier,
};

// Declare the global_settings module
pub mod global_settings;

// Re-export main public types from the global_settings module
pub use global_settings::{
    GlobalDesktopSettings,
    AppearanceSettings,
    InputBehaviorSettings,
    ColorScheme,
    SettingPath,
    AppearanceSettingPath, // Added for completeness if needed by consumers
    InputBehaviorSettingPath, // Added for completeness
    FontSettingPath, // New
    WorkspaceSettingPath, // New
    PowerManagementSettingPath, // New
    DefaultApplicationSettingPath, // New
    GlobalSettingsError,
    SettingsPersistenceProvider,
    FilesystemSettingsProvider, // Example provider
    SettingChangedEvent,
    SettingsLoadedEvent,
    SettingsSavedEvent, // New
    GlobalSettingsService,
    DefaultGlobalSettingsService,
    // New specific types from global_settings::types
    FontSettings,
    FontHinting,
    FontAntialiasing,
    WorkspaceSettings,
    WorkspaceSwitchingBehavior,
    PowerManagementPolicySettings,
    LidCloseAction,
    DefaultApplicationsSettings,
};

// Declare the window_management_policy module
pub mod window_management_policy;

// Re-export main public types from the window_management_policy module
pub use window_management_policy::{
    TilingMode,
    NewWindowPlacementStrategy,
    GapSettings,
    WindowSnappingPolicy,
    WindowLayoutInfo,
    WorkspaceWindowLayout,
    WindowPolicyError,
    WindowManagementPolicyService,
    DefaultWindowManagementPolicyService,
    // New types for Iteration 3
    WindowPolicyOverrides,
    FocusPolicy,
    FocusStealingPreventionLevel,
    WindowGroupingPolicy,
};


// Declare the user_centric_services module
pub mod user_centric_services;

// Re-export main public types from the user_centric_services module
pub use user_centric_services::{
    // ai_interaction types
    AIDataCategory,
    AIConsentStatus,
    AIModelCapability,
    AIModelProfile,
    AIConsentScope,
    AIConsent,
    AIInteractionError,
    AIConsentProvider,
    AIModelProfileProvider,
    AIInteractionLogicService,
    DefaultAIInteractionLogicService,
    // New concrete provider types if they are intended for direct use by consumers
    // For now, assuming consumers primarily use the traits (AIConsentProvider, etc.)
    // and the DefaultAIInteractionLogicService would be configured with concrete providers
    // by the application setup layer, not necessarily needing direct crate-level re-export
    // of FilesystemAIConsentProvider unless explicitly stated.
    // The prompt for ai_interaction/mod.rs re-exports them, so let's include them here for consistency.
    FilesystemAIConsentProvider,
    FilesystemAIModelProfileProvider,
    // New iteration 2 types for AI
    AttachmentData,
    InteractionParticipant,
    InteractionHistoryEntry,
    AIInteractionContext,
    // events (AIInteractionEvent is already here, NotificationEvent and DismissReason are new)
    AIInteractionEvent, // This enum now includes new variants
    NotificationEvent, // New from user_centric_services::events
    NotificationDismissReason, // New from user_centric_services::events
    // notifications_core types
    NotificationUrgency,
    NotificationActionType,
    NotificationAction,
    NotificationInput,
    Notification,
    NotificationError,
    NotificationService,
    DefaultNotificationService,
    NotificationHistoryProvider, 
    FilesystemNotificationHistoryProvider, // Added in Iteration 3 for notifications_core
};

// Declare the notifications_rules module
pub mod notifications_rules;

// Re-export main public types from the notifications_rules module
pub use notifications_rules::{
    // types
    RuleConditionValue,
    RuleConditionOperator,
    RuleConditionField,
    SimpleRuleCondition,
    RuleCondition,
    RuleAction,
    NotificationRule,
    NotificationRuleSet,
    // errors
    NotificationRulesError,
    // persistence_iface
    NotificationRulesProvider,
    FilesystemNotificationRulesProvider, // Added in Iteration 2 for notifications_rules
    // engine
    RuleProcessingResult,
    NotificationRulesEngine,
    DefaultNotificationRulesEngine,
};


#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::UserSessionState; // Ensure UserSessionState is in scope for tests
    // Temporarily comment out workspace-dependent tests until WorkspaceId is fully integrated from the new module
    // use crate::workspaces::WorkspaceId as ActualWorkspaceId; // Assuming this path after re-export

    #[test]
    fn application_id_creation_and_display() {
        let app_id_str = "test_app_id";
        let app_id = ApplicationId::new(app_id_str.to_string());
        assert_eq!(app_id.as_str(), app_id_str);
        assert_eq!(format!("{}", app_id), app_id_str);

        let app_id_from_string = ApplicationId::from(app_id_str.to_string());
        assert_eq!(app_id_from_string, app_id);

        let app_id_from_str_slice = ApplicationId::from(app_id_str);
        assert_eq!(app_id_from_str_slice, app_id);
    }

    #[test]
    #[should_panic]
    fn application_id_empty_new_panics() {
        ApplicationId::new("".to_string());
    }

    #[test]
    #[should_panic]
    fn application_id_empty_from_string_panics() {
        ApplicationId::from("".to_string());
    }

    #[test]
    #[should_panic]
    fn application_id_empty_from_str_panics() {
        ApplicationId::from("");
    }

    #[test]
    fn user_session_state_default() {
        assert_eq!(UserSessionState::default(), UserSessionState::Active);
    }

    #[test]
    fn resource_identifier_creation() {
        let r_id = ResourceIdentifier::new("type1".to_string(), "id1".to_string(), Some("label1".to_string()));
        assert_eq!(r_id.r#type, "type1");
        assert_eq!(r_id.id, "id1");
        assert_eq!(r_id.label, Some("label1".to_string()));

        let file_id = ResourceIdentifier::file("/path/to/file".to_string(), None);
        assert_eq!(file_id.r#type, "file");
        assert_eq!(file_id.id, "/path/to/file");

        let url_id = ResourceIdentifier::url("http://example.com".to_string(), Some("Example".to_string()));
        assert_eq!(url_id.r#type, "url");
        assert_eq!(url_id.id, "http://example.com");

        let uuid_id = ResourceIdentifier::new_uuid("user".to_string(), None);
        assert_eq!(uuid_id.r#type, "user");
        assert!(!uuid_id.id.is_empty()); // Check that a UUID was generated
    }

    #[test]
    #[should_panic]
    fn resource_identifier_empty_type_panics() {
        ResourceIdentifier::new("".to_string(), "id1".to_string(), None);
    }

    #[test]
    #[should_panic]
    fn resource_identifier_empty_id_panics() {
        ResourceIdentifier::new("type1".to_string(), "".to_string(), None);
    }

    // This test needs to be updated to use the actual WorkspaceId from the new module.
    // For now, the `active_workspace_id` field in `UserActivityDetectedEvent` might cause a type mismatch
    // if it still expects the placeholder `WorkspaceId(String)`.
    // The `common_events.rs` file needs to be updated to use `crate::workspaces::WorkspaceId`.
    // This change is outside the scope of the current subtask but is a necessary follow-up.
    // For now, this test might fail or might need to be commented out if `UserActivityDetectedEvent` hasn't been updated.
    // Assuming `common_events.rs` gets updated to use `crate::workspaces::WorkspaceId` (which is `uuid::Uuid`)
    #[test]
    fn user_activity_event_creation() {
        // Placeholder: actual WorkspaceId is uuid::Uuid.
        // The `UserActivityDetectedEvent` struct needs to be updated to use the proper WorkspaceId type.
        // For now, let's assume it's updated or this test is illustrative.
        // If WorkspaceId is now uuid::Uuid, we need a Uuid here.
        let example_workspace_id = uuid::Uuid::new_v4(); // Actual WorkspaceId from the new module

        let event = UserActivityDetectedEvent::new(
            UserActivityType::MouseClicked,
            UserSessionState::Active,
            Some(ApplicationId::new("app1".to_string())),
            Some(example_workspace_id), // Use the actual WorkspaceId type
        );
        assert_eq!(event.activity_type, UserActivityType::MouseClicked);
        assert!(event.active_application_id.is_some());
        assert!(event.active_workspace_id.is_some());
        assert_eq!(event.active_workspace_id.unwrap(), example_workspace_id);
    }


    #[test]
    fn system_shutdown_event_creation() {
        let event = SystemShutdownInitiatedEvent::new(
            ShutdownReason::UserRequest,
            false,
            Some(30),
            Some("Shutting down for maintenance".to_string()),
        );
        assert_eq!(event.reason, ShutdownReason::UserRequest);
        assert_eq!(event.is_reboot, false);
        assert_eq!(event.delay_seconds, Some(30));
        assert_eq!(event.message, Some("Shutting down for maintenance".to_string()));
    }

     #[test]
    fn shutdown_reason_default() {
        assert_eq!(ShutdownReason::default(), ShutdownReason::Other);
    }
}

