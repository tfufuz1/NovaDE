// novade-ui/src/system_health_dashboard/metrics_panel.rs
// Displays detailed graphs and figures for various system metrics (CPU, Mem, Disk, Net).
// Subscribes to metric updates from SystemHealthViewModel.

// #[cfg(feature = "gtk")]
// use gtk::prelude::*;
// #[cfg(feature = "gtk")]
// use gtk::{Box as GtkBox, Label, Orientation};

// use std::sync::Arc;
// use tokio::sync::Mutex;
use super::view_model::SystemHealthViewModel;
use std::sync::Arc;
use tokio::sync::Mutex;


pub struct MetricsPanel {
    view_model: Arc<Mutex<SystemHealthViewModel>>, // Added field
    // Placeholder for UI elements like charts or detailed metric displays
    // cpu_chart: Option<MyCustomChartWidget>, // Example
    // memory_display: Option<Label>,
    // container: Option<GtkBox>,
}

impl MetricsPanel {
    pub fn new(view_model: Arc<Mutex<SystemHealthViewModel>>) -> Self { // Now takes view_model
        println!("UI: MetricsPanel created (placeholder).");
        Self { view_model }
    }

    pub async fn display_memory_metrics(&self) {
        let vm_guard = self.view_model.lock().await;
        if let Some(metrics) = &vm_guard.latest_memory_metrics {
            println!("UI MetricsPanel Display: Memory Usage: {}/{} bytes (Available: {} bytes)",
                metrics.used_bytes, metrics.total_bytes, metrics.available_bytes);
        } else {
            println!("UI MetricsPanel Display: Memory Metrics not available yet.");
        }
    }

    // #[cfg(feature = "gtk")]
    // pub fn get_widget(&mut self) -> &GtkBox {
    //     if self.container.is_none() {
    //         let container = GtkBox::new(Orientation::Vertical, 5);
    //         let title_label = Label::new(Some("Metrics Panel (Placeholder)"));
    //         container.append(&title_label);
    //         // TODO: Add charts and detailed metric views
    //         self.container = Some(container);
    //         // TODO: Add logic to subscribe to ViewModel updates and refresh displays
    //     }
    //     self.container.as_ref().expect("Container should be initialized")
    // }
}
