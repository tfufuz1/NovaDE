//! Defines the core traits and interfaces for NovaDE plugins.
//!
//! These traits allow plugins to integrate with the NovaDE shell and other UI components
//! by providing a standardized way to extend functionality.

use gtk::glib; // For GObject meta types if needed, and error handling
use gtk::prelude::*; // For WidgetExt, BoxExt etc.
use gtk::Widget;

/// Represents a generic error type that can be returned by plugin operations.
/// Plugins should aim to return more specific errors where possible, but this
/// can be used as a general fallback.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Failed to initialize plugin: {0}")]
    InitializationFailed(String),
    #[error("Feature not implemented by this plugin")]
    NotImplemented,
    #[error("Failed to retrieve widget")]
    WidgetRetrievalFailed,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("An underlying operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    GlibError(#[from] glib::Error),
    // TODO: Add more specific error types as needed.
}

/// A provider for core NovaDE APIs that plugins can use.
/// This trait will be implemented by NovaDE's core and passed to plugins
/// during initialization. It acts as a gateway for plugins to interact
/// with the main system in a controlled manner.
pub trait NovaApiProvider: Send + Sync {
    // Example methods:
    // fn get_settings_service(&self) -> Arc<dyn SettingsService>;
    // fn get_notification_manager(&self) -> Arc<dyn NotificationManager>;
    // fn get_theming_tokens(&self) -> Arc<dyn ThemingTokens>;

    /// Returns a unique ID for the host application (NovaDE).
    fn get_host_id(&self) -> &str;

    /// Returns the current version of NovaDE.
    fn get_host_version(&self) -> &str;

    // More methods will be added here as the API surface expands.
    // For example, methods to register different types of extension providers.
}

/// The main trait that all NovaDE plugins must implement.
///
/// This is the entry point for the plugin lifecycle.
pub trait Plugin: Send + Sync {
    /// Called when the plugin is loaded and initialized.
    ///
    /// Plugins should perform any necessary setup here.
    /// The `api_provider` allows the plugin to interact with NovaDE's core services.
    /// The `plugin_id` is the unique ID of this plugin instance, as defined in its manifest.
    fn initialize(&mut self, plugin_id: &str, api_provider: Box<dyn NovaApiProvider>) -> Result<(), PluginError>;

    /// Called when the plugin is about to be unloaded.
    ///
    /// Plugins should perform any necessary cleanup here, such as releasing resources,
    /// saving state, or unregistering UI components.
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Returns a human-readable name for the plugin.
    /// This might be different from the `name` in the manifest if the plugin
    /// supports localization or dynamic naming.
    fn get_display_name(&self) -> String;

    // Optional: Plugins can provide providers for specific extension points.
    // These methods allow the plugin manager to query for specific capabilities.

    /// If the plugin provides panel widgets, it should return an implementation of `PanelWidgetProvider`.
    fn get_panel_widget_provider(&self) -> Option<Box<dyn PanelWidgetProvider>> {
        None
    }

    /// If the plugin provides sidebar widgets, it should return an implementation of `SidebarWidgetProvider`.
    fn get_sidebar_widget_provider(&self) -> Option<Box<dyn SidebarWidgetProvider>> {
        None
    }

    /// If the plugin provides settings pages, it should return an implementation of `SettingsPageProvider`.
    fn get_settings_page_provider(&self) -> Option<Box<dyn SettingsPageProvider>> {
        None
    }

    /// If the plugin provides commands for the command palette, it should return an implementation of `CommandProvider`.
    fn get_command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        None
    }
}

/// Trait for widgets that can be displayed in a NovaDE panel.
///
/// Implementors of this trait define the GTK widget to be shown and how it updates.
pub trait PanelWidget: Send + Sync {
    /// Creates and returns the GTK widget to be displayed in the panel.
    /// The widget should be self-contained and ready to be added to a `gtk::Box` or similar container.
    fn get_widget(&mut self) -> Result<Widget, PluginError>;

    /// Called when the panel requests the widget to update its content.
    /// This could be triggered by new data, configuration changes, or periodic refresh.
    /// `data` is a generic placeholder; specific data types might be passed in the future.
    fn update_data(&mut self, data: Option<&glib::Variant>) -> Result<(), PluginError>;

    /// Optional: Called when the widget is added to the panel.
    fn on_added(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Optional: Called when the widget is about to be removed from the panel.
    fn on_removed(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Optional: Returns a unique identifier for this specific widget instance,
    /// if multiple instances of the same widget type are possible.
    fn get_instance_id(&self) -> Option<String> {
        None
    }
}

/// Provides instances of `PanelWidget`.
///
/// A plugin can implement this trait to offer one or more types of panel widgets.
pub trait PanelWidgetProvider: Send + Sync {
    /// Returns a list of unique identifiers for the panel widget types this provider offers.
    /// These IDs can be used in configuration to specify which widget to load.
    fn get_available_widget_types(&self) -> Vec<String>;

    /// Creates a new instance of a panel widget of the specified `widget_type_id`.
    /// The `config` parameter can be used to pass widget-specific configuration.
    fn create_widget(&self, widget_type_id: &str, config: Option<&glib::Variant>) -> Result<Box<dyn PanelWidget>, PluginError>;
}

/// Trait for widgets that can be displayed in a NovaDE sidebar.
///
/// Similar to `PanelWidget`, but intended for sidebar containers.
pub trait SidebarWidget: Send + Sync {
    /// Creates and returns the GTK widget to be displayed in the sidebar.
    fn get_widget(&mut self) -> Result<Widget, PluginError>;

    /// Called to update the widget's content.
    fn update_data(&mut self, data: Option<&glib::Variant>) -> Result<(), PluginError>;

    /// Optional: Called when the widget is added to the sidebar.
    fn on_added(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Optional: Called when the widget is about to be removed from the sidebar.
    fn on_removed(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Optional: Returns a unique identifier for this specific widget instance.
    fn get_instance_id(&self) -> Option<String> {
        None
    }
}

/// Provides instances of `SidebarWidget`.
pub trait SidebarWidgetProvider: Send + Sync {
    /// Returns a list of unique identifiers for the sidebar widget types this provider offers.
    fn get_available_widget_types(&self) -> Vec<String>;

    /// Creates a new instance of a sidebar widget.
    fn create_widget(&self, widget_type_id: &str, config: Option<&glib::Variant>) -> Result<Box<dyn SidebarWidget>, PluginError>;
}

/// Describes a page to be added to the NovaDE settings application.
pub struct SettingsPageInfo {
    /// Unique identifier for this settings page.
    pub id: String,
    /// Human-readable title of the settings page.
    pub title: String,
    /// Optional: Name of the icon to display in the settings navigation.
    pub icon_name: Option<String>,
    /// The GTK widget that represents the content of this settings page.
    pub widget: Widget,
    // TODO: Add keywords for search, categories, etc.
}

/// Provides custom pages to be integrated into the NovaDE settings application.
///
/// Plugins can implement this to offer configuration UIs for their features.
pub trait SettingsPageProvider: Send + Sync {
    /// Returns a list of `SettingsPageInfo` objects, one for each settings page
    /// this plugin provides.
    /// These pages will be added to the NovaDE settings application.
    fn get_settings_pages(&mut self) -> Result<Vec<SettingsPageInfo>, PluginError>;

    /// Optional: Called when a specific settings page provided by this plugin is shown.
    fn on_page_shown(&mut self, page_id: &str) -> Result<(), PluginError> {
        let _ = page_id; // Avoid unused variable warning
        Ok(())
    }

    /// Optional: Called when a specific settings page provided by this plugin is hidden.
    fn on_page_hidden(&mut self, page_id: &str) -> Result<(), PluginError> {
        let _ = page_id; // Avoid unused variable warning
        Ok(())
    }
}

/// Represents a command that can be invoked, typically through the command palette.
pub struct CommandInfo {
    /// Unique identifier for the command (e.g., "myplugin.do_action").
    pub id: String,
    /// Human-readable name of the command (e.g., "Do Action").
    pub name: String,
    /// Optional: A more detailed description of what the command does.
    pub description: Option<String>,
    /// Optional: Name of the icon associated with this command.
    pub icon_name: Option<String>,
    // TODO: Add categories, keywords for search, etc.
}

/// Provides commands that can be registered with the NovaDE command palette or other action invokers.
pub trait CommandProvider: Send + Sync {
    /// Returns a list of `CommandInfo` objects for all commands this plugin provides.
    fn get_commands(&self) -> Result<Vec<CommandInfo>, PluginError>;

    /// Executes the command identified by `command_id`.
    /// `args` can be used to pass arguments to the command.
    fn execute_command(&mut self, command_id: &str, args: Option<&glib::Variant>) -> Result<(), PluginError>;
}

// It's important to ensure that these traits are object-safe if they are to be used as
// Box<dyn TraitName>. Most of these are, but care should be taken with generics or `Self` in
// method signatures if not used as `Box<Self>`.
// The `Send + Sync` bounds are important for potential multithreaded environments
// where plugins might be managed or called from different threads.
//
// These definitions are a starting point and will likely evolve as the plugin system
// and UI capabilities of NovaDE mature.
//
// Consider adding a `PluginMetadata` struct that could be returned by the `Plugin` trait,
// containing information like name, version, author, etc., which could be an alternative
// to parsing everything from `Plugin.toml` at runtime by the plugin consumers,
// or serve as a verification step. For now, the manifest is the primary source.

// End of novade-ui/src/plugin_api.rs
