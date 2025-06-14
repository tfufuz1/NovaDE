// novade-ui/src/system_health_dashboard/mod.rs

pub mod main_view; // The main container for the dashboard
pub mod overview_panel;
pub mod metrics_panel;
pub mod log_viewer_panel;
pub mod diagnostics_panel;
pub mod alerts_panel;
pub mod view_model; // Handles state and communication with the domain layer
pub mod widgets; // Common widgets for the dashboard

// Re-export key components, particularly the main view that would be integrated into the app
pub use main_view::SystemHealthDashboardMainView;
pub use view_model::SystemHealthViewModel;
