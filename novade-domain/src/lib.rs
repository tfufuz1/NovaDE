// Main public interface for novade_domain
// This file re-exports key entities, services, and errors from the submodules.

// Main Module Declarations
pub mod common_events;
pub mod global_settings;
pub mod notifications_rules;
pub mod shared_types;
pub mod theming;
pub mod user_centric_services;
pub mod window_management_policy;
pub mod workspaces;
pub mod notification_service;
pub mod display_configuration;
pub mod system_health_service;
// pub mod entities; // Example, if you have a top-level entities module
// pub mod error; // Example, for a general DomainError if used

// Public API Re-exports
pub use common_events::{
    UserActivityType, UserActivityDetectedEvent,
    ShutdownReason, SystemShutdownInitiatedEvent,
};

pub use global_settings::{
    DefaultGlobalSettingsService,
    GlobalSettingsError,
    GlobalSettingsService,
    SettingsPersistenceProvider,
    FilesystemSettingsProvider,
    types::{
        GlobalDesktopSettings, AppearanceSettings, 
        ColorScheme as GlobalColorScheme, // Aliased to avoid conflict with theming's ColorSchemeType
        FontSettings, 
        WorkspaceSettings as GlobalWorkspaceSettings, // Aliased for clarity
        InputBehaviorSettings, 
        PowerManagementPolicySettings, DefaultApplicationsSettings
    },
    paths::SettingPath,
    events::{SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent},
};

pub use notifications_rules::{
    DefaultNotificationRulesEngine,
    NotificationRulesEngine,
    NotificationRulesError,
    persistence_iface::NotificationRulesProvider,
    persistence::FilesystemNotificationRulesProvider,
    types::{
        NotificationRule, RuleCondition, RuleAction, RuleProcessingResult, 
        RuleConditionValue, RuleConditionOperator, RuleConditionField, 
        SimpleRuleCondition, NotificationRuleSet
    },
    engine::RuleProcessingResult as EngineRuleProcessingResult, // If RuleProcessingResult is also in engine
};

pub use shared_types::{ApplicationId, UserSessionState, ResourceIdentifier};

pub use theming::{
    ThemingEngine,
    ThemingError,
    types::{
        ThemeDefinition, AppliedThemeState, ThemingConfiguration, TokenIdentifier, 
        TokenValue, RawToken, TokenSet, ThemeIdentifier, 
        ColorSchemeType as ThemingColorSchemeType, // Aliased
        AccentColor, ThemeVariantDefinition, AccentModificationType
    },
    events::ThemeChangedEvent,
};

pub use user_centric_services::{
    ai_interaction::{
        AIInteractionLogicService, DefaultAIInteractionLogicService,
        AIInteractionError,
        AIConsentProvider, AIModelProfileProvider,
        FilesystemAIConsentProvider, FilesystemAIModelProfileProvider,
        types::{
            AIInteractionContext, AIConsent, AIModelProfile, AIDataCategory, 
            AttachmentData, InteractionHistoryEntry, AIConsentStatus, 
            AIConsentScope, AIModelCapability, InteractionParticipant
        },
    },
    notifications_core::{
        NotificationService, DefaultNotificationService,
        NotificationError,
        // persistence_iface::NotificationPersistenceProvider, // If defined & public
        // persistence::FilesystemNotificationPersistenceProvider, // If defined & public
        types::{
            Notification, NotificationInput, NotificationAction, NotificationUrgency, 
            NotificationActionType, NotificationStats, DismissReason, 
            NotificationFilterCriteria, NotificationSortOrder
        },
    },
    events::{UserCentricEvent, AIInteractionEventEnum, NotificationEventEnum},
};

pub use window_management_policy::{
    DefaultWindowManagementPolicyService,
    WindowManagementPolicyService,
    WindowPolicyError,
    types::{
        TilingMode, GapSettings, WindowSnappingPolicy, WindowGroupingPolicy, 
        NewWindowPlacementStrategy, FocusStealingPreventionLevel, FocusPolicy, 
        WindowPolicyOverrides, WorkspaceWindowLayout, WindowLayoutInfo
    },
};

pub use workspaces::{
    DefaultWorkspaceManager,
    WorkspaceManagerService,
    config::{
        WorkspaceConfigProvider, 
        FilesystemConfigProvider as FilesystemWorkspaceConfigProvider, // Aliased for clarity
        WorkspaceSnapshot, WorkspaceSetSnapshot
    },
    core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType as CoreWorkspaceLayoutType},
    core::Workspace,
    core::errors::WorkspaceCoreError, 
    assignment::errors::WindowAssignmentError, 
    manager::errors::WorkspaceManagerError, 
    config::errors::WorkspaceConfigError,
    manager::events::WorkspaceEvent,
};

// --- DomainServices Struct and Initialization ---
use std::path::PathBuf;
use std::sync::Arc;
use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;
use tracing; // For logging

#[derive(Clone)]
pub struct DomainServices {
    pub settings_service: Arc<dyn GlobalSettingsService>,
    pub theming_engine: Arc<ThemingEngine>,
    pub workspace_manager: Arc<dyn WorkspaceManagerService>,
    pub window_management_policy_service: Arc<dyn WindowManagementPolicyService>,
    pub ai_interaction_service: Arc<dyn AIInteractionLogicService>,
    pub notification_rules_engine: Arc<dyn NotificationRulesEngine>,
    pub notification_service: Arc<dyn NotificationService>,
    pub display_configuration_service: Arc<dyn display_configuration::DisplayConfigService>,
}

#[derive(Debug, thiserror::Error)]
pub enum DomainInitializationError {
    #[error("Failed to initialize global settings service: {0}")]
    SettingsInitError(#[from] GlobalSettingsError),
    #[error("Failed to initialize theming engine: {0}")]
    ThemingInitError(#[from] ThemingError),
    #[error("Failed to initialize workspace manager: {0}")]
    WorkspaceInitError(#[from] WorkspaceManagerError),
    #[error("Failed to initialize AI interaction service: {0}")]
    AiInteractionInitError(#[from] AIInteractionError),
    #[error("Failed to initialize notification rules engine: {0}")]
    NotificationRulesInitError(#[from] NotificationRulesError),
    #[error("Failed to initialize notification service: {0}")]
    NotificationServiceInitError(#[from] NotificationError),
    #[error("Failed to initialize display configuration service: {0}")]
    DisplayConfigurationInitError(#[from] display_configuration::DisplayConfigurationError),
    #[error("Core configuration error during domain initialization: {0}")]
    CoreConfigError(#[from] CoreError),
    #[error("User directory not found for default paths: {0}")]
    UserDirectoryNotFound(String),
}

const DOMAIN_CONFIG_BASE_PATH: &str = "novade"; 
const DEFAULT_EVENT_BROADCAST_CAPACITY: usize = 128;

pub async fn initialize_domain_layer(
    core_config_service: Arc<dyn ConfigServiceAsync>,
    current_user_id: String,
    event_broadcast_capacity_override: Option<usize>,
    theme_load_paths_override: Option<Vec<PathBuf>>,
    token_load_paths_override: Option<Vec<PathBuf>>,
) -> Result<DomainServices, DomainInitializationError> {
    tracing::info!("Initializing NovaDE Domain Layer...");
    let capacity = event_broadcast_capacity_override.unwrap_or(DEFAULT_EVENT_BROADCAST_CAPACITY);

    let user_config_dir = dirs::config_dir().ok_or_else(|| DomainInitializationError::UserDirectoryNotFound("User config directory not found.".to_string()))?;
    let user_data_dir = dirs::data_dir().ok_or_else(|| DomainInitializationError::UserDirectoryNotFound("User data directory not found.".to_string()))?;
    
    let domain_config_path = user_config_dir.join(DOMAIN_CONFIG_BASE_PATH);
    let domain_data_path = user_data_dir.join(DOMAIN_CONFIG_BASE_PATH); // For things like consents if they are in data dir

    // Ensure domain config/data directories exist
    core_config_service.ensure_directory_exists(&domain_config_path).await.map_err(DomainInitializationError::CoreConfigError)?;
    core_config_service.ensure_directory_exists(&domain_data_path).await.map_err(DomainInitializationError::CoreConfigError)?;


    // --- Persistence Providers Initialization ---
    let fs_settings_provider = Arc::new(
        global_settings::FilesystemSettingsProvider::new(core_config_service.clone(), domain_config_path.join("global_settings.toml").to_string_lossy().into_owned())
    );
    let fs_workspace_config_provider = Arc::new(
        workspaces::config::FilesystemConfigProvider::new(core_config_service.clone(), domain_config_path.join("workspaces.toml").to_string_lossy().into_owned())
    );
    // Use data_path for consents as it might be more user-specific and less "config"
    let fs_consent_provider = Arc::new(
        user_centric_services::ai_interaction::FilesystemAIConsentProvider::new(core_config_service.clone(), &domain_data_path.to_string_lossy(), current_user_id.clone())
    );
    let fs_profile_provider = Arc::new(
        user_centric_services::ai_interaction::FilesystemAIModelProfileProvider::new(core_config_service.clone(), domain_config_path.join("ai_model_profiles.json").to_string_lossy().into_owned())
    );
    let fs_rules_provider = Arc::new(
        notifications_rules::FilesystemNotificationRulesProvider::new(core_config_service.clone(), domain_config_path.join("notification_rules.json").to_string_lossy().into_owned())
    );
    let fs_display_persistence = Arc::new(
        display_configuration::FileSystemDisplayPersistence::new(domain_config_path.join("display_configuration.json"))
    );

    // --- Services Initialization ---
    let settings_service = Arc::new(
        global_settings::DefaultGlobalSettingsService::new(fs_settings_provider, capacity)
    );
    settings_service.load_settings().await?;
    tracing::info!("GlobalSettingsService initialized and settings loaded.");

    let initial_theming_config = settings_service.get_setting(&SettingPath::AppearanceRoot)
        .ok()
        .and_then(|json_val| serde_json::from_value::<global_settings::types::AppearanceSettings>(json_val).ok())
        .map_or_else(|| {
                tracing::warn!("Could not derive initial ThemingConfiguration from global settings (AppearanceRoot not found or parse error), using default ThemingConfiguration.");
                ThemingConfiguration::default()
            }, 
            |appearance_settings| {
                tracing::info!("Deriving initial ThemingConfiguration from AppearanceSettings. Active theme: '{}', Scheme: {:?}, Accent Token: '{}'",
                    appearance_settings.active_theme_name,
                    appearance_settings.color_scheme,
                    appearance_settings.accent_color_token
                );

                let preferred_color_scheme = match appearance_settings.color_scheme {
                    global_settings::types::ColorScheme::Light => theming::types::ColorSchemeType::Light,
                    global_settings::types::ColorScheme::Dark => theming::types::ColorSchemeType::Dark,
                    global_settings::types::ColorScheme::SystemPreference => {
                        //ANCHOR [SYSTEM_PREFERENCE_COLOR_SCHEME] Placeholder for system preference detection. Defaulting to Light.
                        tracing::info!("SystemPreference color scheme detected, defaulting to Light. Actual system preference detection TBD.");
                        theming::types::ColorSchemeType::Light
                    }
                };

                ThemingConfiguration {
                    selected_theme_id: ThemeIdentifier::new(appearance_settings.active_theme_name.clone()),
                    preferred_color_scheme,
                    selected_accent_color: None, // Per plan, this remains None for now.
                    custom_user_token_overrides: None,
                }
            }
        );
         
    let default_theme_paths = || vec![
        PathBuf::from("/usr/share/novade/themes"), 
        user_config_dir.join(DOMAIN_CONFIG_BASE_PATH).join("themes")
    ];
    let default_token_paths = || vec![
        PathBuf::from("/usr/share/novade/tokens"), 
        user_config_dir.join(DOMAIN_CONFIG_BASE_PATH).join("tokens")
    ];

    let theming_engine = Arc::new(
        theming::ThemingEngine::new(
            initial_theming_config,
            theme_load_paths_override.unwrap_or_else(default_theme_paths),
            token_load_paths_override.unwrap_or_else(default_token_paths),
            core_config_service.clone(),
            capacity
        ).await?
    );
    tracing::info!("ThemingEngine initialized.");

    let workspace_manager = Arc::new(
        workspaces::DefaultWorkspaceManager::new(fs_workspace_config_provider, capacity, true)
    );
    workspace_manager.load_or_initialize_workspaces().await?;
    tracing::info!("WorkspaceManager initialized.");

    let window_management_policy_service = Arc::new(
        window_management_policy::DefaultWindowManagementPolicyService::new(settings_service.clone())
    );
    tracing::info!("WindowManagementPolicyService initialized.");

    let ai_interaction_service = Arc::new(
        user_centric_services::ai_interaction::DefaultAIInteractionLogicService::new(
            fs_consent_provider, fs_profile_provider, current_user_id.clone(), capacity
        ).await?
    );
    tracing::info!("AIInteractionLogicService initialized.");

    let notification_rules_engine = Arc::new(
        notifications_rules::DefaultNotificationRulesEngine::new(fs_rules_provider, settings_service.clone()).await?
    );
    tracing::info!("NotificationRulesEngine initialized.");

    let notification_service = Arc::new(
        user_centric_services::notifications_core::DefaultNotificationService::new(
            notification_rules_engine.clone(), settings_service.clone(), capacity
        ).await?
    );
    tracing::info!("NotificationService initialized.");

    let display_configuration_service = Arc::new(
        display_configuration::DefaultDisplayConfigService::new(fs_display_persistence).await?
    );
    tracing::info!("DisplayConfigurationService initialized.");

    tracing::info!("NovaDE Domain Layer Initialized Successfully.");
    Ok(DomainServices {
        settings_service, theming_engine, workspace_manager, window_management_policy_service,
        ai_interaction_service, notification_rules_engine, notification_service,
        display_configuration_service,
    })
}
