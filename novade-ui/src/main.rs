use gtk4 as gtk;
use gtk::{prelude::*, Application, ApplicationWindow};
use libadwaita as adw;
use adw::prelude::*;

use novade_core::config::CoreConfig;
use novade_domain::system_health_service::{DefaultSystemHealthService, SystemHealthService};
use novade_system::system_health_collectors::{
    LinuxCpuMetricsCollector, LinuxMemoryMetricsCollector, LinuxDiskMetricsCollector,
    LinuxNetworkMetricsCollector, LinuxTemperatureMetricsCollector, JournaldLogHarvester,
    BasicDiagnosticsRunner,
};
use novade_ui::system_health_dashboard::main_view::SystemHealthDashboardView; // Adjusted path
use std::sync::Arc;

const APP_ID: &str = "org.novade.SystemHealthDashboard";

fn main() {
    // Initialize CoreConfig (using default for now)
    // In a real app, this would be loaded from a file.
    let core_config = Arc::new(CoreConfig::default());

    // Instantiate system layer collectors
    let cpu_collector = Arc::new(LinuxCpuMetricsCollector);
    let memory_collector = Arc::new(LinuxMemoryMetricsCollector);
    let disk_collector = Arc::new(LinuxDiskMetricsCollector);
    let network_collector = Arc::new(LinuxNetworkMetricsCollector);
    let temperature_collector = Arc::new(LinuxTemperatureMetricsCollector);
    let log_harvester = Arc::new(JournaldLogHarvester);
    // BasicDiagnosticsRunner needs a ping target
    // The diagnostic_ping_target field was added to SystemHealthDashboardConfig in a previous step.
    let diagnostic_runner = Arc::new(BasicDiagnosticsRunner::new(core_config.system_health.diagnostic_ping_target.clone()));

    // Instantiate Domain Layer SystemHealthService
    let system_health_service: Arc<dyn SystemHealthService> = Arc::new(DefaultSystemHealthService::new(
        core_config.clone(), // Pass the config Arc
        cpu_collector,
        memory_collector,
        disk_collector,
        network_collector,
        temperature_collector,
        log_harvester,
        diagnostic_runner,
    ));

    // Create a new GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Clone the service Arc to move into the connect_activate callback
    let service_for_ui = system_health_service.clone();

    // Connect to "activate" signal of the application
    app.connect_activate(move |app| {
        // Create the SystemHealthDashboardView (main UI for the dashboard)
        // Pass the SystemHealthService to it.
        let dashboard_view = SystemHealthDashboardView::new(service_for_ui.clone());

        // Create a new window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("NovaDE System Health Dashboard")
            .default_width(800)
            .default_height(600)
            .child(&dashboard_view) // Add the dashboard view as a child of the window
            .build();

        // Use adw::ApplicationWindow if available and preferred for libadwaita styling
        // let window = adw::ApplicationWindow::builder()
        //     .application(app)
        //     .title("NovaDE System Health Dashboard")
        //     .default_width(800)
        //     .default_height(600)
        //     .content(&dashboard_view)
        //     .build();


        // Present the window
        window.present();
    });

    // Run the application
    std::process::exit(app.run_with_args::<&str>(&[]));
}
