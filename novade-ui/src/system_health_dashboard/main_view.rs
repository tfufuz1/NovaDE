use gtk4 as gtk;
use gtk::{prelude::*, Box, Orientation, Label, Notebook};
use std::sync::Arc;
use novade_domain::system_health_service::SystemHealthService;
use super::metrics_panel::MetricsPanel;
use super::log_viewer_panel::LogViewerPanel;
use super::diagnostics_panel::DiagnosticsPanel;
use super::alerts_panel::AlertsPanel;

#[derive(Clone)]
pub struct SystemHealthDashboardView {
    container: Box,
    _service: Arc<dyn SystemHealthService>,
}

impl SystemHealthDashboardView {
    pub fn new(service: Arc<dyn SystemHealthService>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let notebook = Notebook::new();
        container.append(&notebook);

        // --- Metrics Panel ---
        let metrics_panel = MetricsPanel::new(service.clone());
        notebook.append_page(&metrics_panel.get_widget(), Some(&Label::new(Some("Metrics"))));

        // --- Logs Panel ---
        let logs_panel = LogViewerPanel::new(service.clone());
        notebook.append_page(&logs_panel.get_widget(), Some(&Label::new(Some("Logs"))));

        // --- Diagnostics Panel ---
        let diagnostics_panel = DiagnosticsPanel::new(service.clone());
        notebook.append_page(&diagnostics_panel.get_widget(), Some(&Label::new(Some("Diagnostics"))));

        // --- Alerts Panel ---
        let alerts_panel = AlertsPanel::new(service.clone());
        notebook.append_page(&alerts_panel.get_widget(), Some(&Label::new(Some("Alerts"))));

        Self {
            container,
            _service: service,
        }
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

impl AsRef<gtk::Widget> for SystemHealthDashboardView {
    fn as_ref(&self) -> &gtk::Widget {
        self.container.upcast_ref()
    }
}

impl From<SystemHealthDashboardView> for gtk::Widget {
    fn from(view: SystemHealthDashboardView) -> Self {
        view.container.upcast()
    }
}
