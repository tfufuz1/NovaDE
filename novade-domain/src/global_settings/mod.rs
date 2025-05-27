// Declare submodules
pub mod types;
pub mod errors;
pub mod paths;
pub mod persistence_iface;
pub mod providers;
pub mod events;
pub mod service;

// Re-export main public types for easier access
pub use self::types::{
    GlobalDesktopSettings,
    AppearanceSettings,
    InputBehaviorSettings,
    ColorScheme,
    FontSettings,
    FontHinting,
    FontAntialiasing,
    WorkspaceSettings,
    WorkspaceSwitchingBehavior,
    PowerManagementPolicySettings,
    LidCloseAction,
    DefaultApplicationsSettings,
};
pub use self::paths::{
    SettingPath,
    AppearanceSettingPath,
    InputBehaviorSettingPath,
    FontSettingPath,
    WorkspaceSettingPath,
    PowerManagementSettingPath,
    DefaultApplicationSettingPath,
};
pub use self::errors::GlobalSettingsError;
pub use self::persistence_iface::SettingsPersistenceProvider;
pub use self::providers::filesystem_provider::FilesystemSettingsProvider; // Example provider
pub use self::events::{
    SettingChangedEvent,
    SettingsLoadedEvent,
    SettingsSavedEvent,
};
pub use self::service::{
    GlobalSettingsService,
    DefaultGlobalSettingsService,
};
