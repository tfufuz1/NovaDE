pub mod panel_widget;
pub use panel_widget::PanelWidget;
pub use panel_widget::PanelPosition; // Added PanelPosition for main.rs
pub use panel_widget::ModulePosition;
pub use panel_widget::app_menu_button;
pub use panel_widget::workspace_indicator_widget;
pub use panel_widget::clock_datetime_widget;

pub mod active_window_service;
pub use active_window_service::ActiveWindowService;

// pub mod shell_workspace_service; // Old name
// pub use shell_workspace_service::ShellWorkspaceService; // Old name
pub mod domain_workspace_connector; // New name
pub use domain_workspace_connector::DomainWorkspaceConnector; // New name
