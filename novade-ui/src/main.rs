use gtk4 as gtk;
use gtk::{prelude::*, gdk, Application, ApplicationWindow}; // Added gdk
use libadwaita as adw;
use adw::prelude::*;
use std::path::PathBuf; // Added for ThemingEngine paths

use novade_core::config::CoreConfig;
// --- Theming Imports ---
use novade_domain::theming::{
    ThemingEngine,
    types::ThemingConfiguration,
    // No specific events needed here unless main directly handles them
};
use novade_core::errors::CoreError; // For SimpleFileConfigService
use novade_core::config::ConfigServiceAsync; // For SimpleFileConfigService
use crate::theming_gtk::GtkThemeManager; // Local GtkThemeManager
use async_trait::async_trait; // For SimpleFileConfigService

// --- System Health Imports ---
use novade_domain::system_health_service::{DefaultSystemHealthService, SystemHealthService};
use novade_system::system_health_collectors::{
    LinuxCpuMetricsCollector, LinuxMemoryMetricsCollector, LinuxDiskMetricsCollector,
    LinuxNetworkMetricsCollector, LinuxTemperatureMetricsCollector, JournaldLogHarvester,
    BasicDiagnosticsRunner,
};
use novade_ui::system_health_dashboard::main_view::SystemHealthDashboardView;
use std::sync::Arc;

const APP_ID: &str = "org.novade.SystemHealthDashboard";

// --- Dummy ConfigServiceAsync for ThemingEngine ---
// This service will read files directly from the filesystem.
// It's a simplified version for UI integration without needing the full CoreConfig setup for themes.
#[derive(Clone)] // Clone is useful if multiple services might need an instance.
struct SimpleFileConfigService;

#[async_trait]
impl ConfigServiceAsync for SimpleFileConfigService {
    async fn read_file_to_string(&self, path: &std::path::Path) -> Result<String, CoreError> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to read {:?}", path), Some(e)))
    }

    // Implement other methods if the trait requires them, potentially with `unimplemented!()`
    // For ThemingEngine, only read_file_to_string is critical for loading theme/token JSONs.
    async fn list_files_in_dir(&self, _path: &std::path::Path, _extensions: &[String]) -> Result<Vec<PathBuf>, CoreError> {
        unimplemented!("SimpleFileConfigService::list_files_in_dir not implemented")
    }
    async fn file_exists(&self, _path: &std::path::Path) -> bool {
        unimplemented!("SimpleFileConfigService::file_exists not implemented")
    }
    async fn dir_exists(&self, _path: &std::path::Path) -> bool {
        unimplemented!("SimpleFileConfigService::dir_exists not implemented")
    }
     async fn ensure_dir_exists(&self, _path: &std::path::Path) -> Result<(), CoreError> {
        unimplemented!("SimpleFileConfigService::ensure_dir_exists not implemented")
    }
}


fn main() {
    // Initialize logging (optional, but good for debugging)
    // Initialize Adwaita
    adw::init().expect("Failed to initialize Adwaita.");
    tracing_subscriber::fmt::init();

    // Initialize CoreConfig (using default for now)
    let core_config = Arc::new(CoreConfig::default());

    // --- Instantiate System Health Service (as before) ---
    let cpu_collector = Arc::new(LinuxCpuMetricsCollector);
    let memory_collector = Arc::new(LinuxMemoryMetricsCollector);
    let disk_collector = Arc::new(LinuxDiskMetricsCollector);
    let network_collector = Arc::new(LinuxNetworkMetricsCollector);
    let temperature_collector = Arc::new(LinuxTemperatureMetricsCollector);
    let log_harvester = Arc::new(JournaldLogHarvester);
    let diagnostic_runner = Arc::new(BasicDiagnosticsRunner::new(Some("8.8.8.8".to_string())));
    let system_health_service: Arc<dyn SystemHealthService> = Arc::new(DefaultSystemHealthService::new(
        core_config.clone(),
        cpu_collector,
        memory_collector,
        disk_collector,
        network_collector,
        temperature_collector,
        log_harvester,
        diagnostic_runner,
    ));

    // --- Instantiate ThemingEngine ---
    let initial_theming_config = ThemingConfiguration::default(); // Use default config
    let theme_load_paths: Vec<PathBuf> = Vec::new(); // No external themes for now
    let token_load_paths: Vec<PathBuf> = Vec::new(); // No external global tokens for now
    let simple_config_service = Arc::new(SimpleFileConfigService);
    let broadcast_capacity = 16;

    // We need to run the ThemingEngine::new() within a tokio runtime context
    // because it's an async function.
    let theming_engine_arc = glib::MainContext::default().block_on(async {
        Arc::new(
            ThemingEngine::new(
                initial_theming_config,
                theme_load_paths,
                token_load_paths,
                simple_config_service,
                broadcast_capacity,
            )
            .await
            .expect("Failed to create ThemingEngine"),
        )
    });


    // Create a new GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Clone services and managers for connect_activate closure
    let service_for_ui = system_health_service.clone();
    let theming_engine_for_gtk_manager = theming_engine_arc.clone();
    // GtkThemeManager will be created inside connect_activate to access the display property of the window

    app.connect_activate(move |app| {
        // --- Instantiate and Initialize GtkThemeManager ---
        // It's better to initialize GtkThemeManager after the main window is created,
        // so we can get its gdk::Display.

        // Create a new Adwaita window first (without content)
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("NovaDE System Health Dashboard")
            .default_width(800)
            .default_height(600)
            .build();

        // Create the SystemHealthDashboardView, passing the downgraded window reference
        let dashboard_view = SystemHealthDashboardView::new(service_for_ui.clone(), window.downgrade());

        // Set the dashboard_view as the content of the window
        window.set_content(Some(&dashboard_view));

        // Now that we have a window, we can get its display to init GtkThemeManager
        let display = window.display();
        let gtk_theme_manager = Arc::new(GtkThemeManager::new(theming_engine_for_gtk_manager.clone()));
        gtk_theme_manager.initialize_for_display(&display);

        // To keep gtk_theme_manager alive, if it were not Arc'd and moved elsewhere,
        // one might attach it to the window or app, or ensure its lifetime.
        // Since it spawns its own listener task that holds Arcs, it should manage its lifetime
        // as long as the css_provider is attached to the display.
        // For safety, could store `gtk_theme_manager` in a static or app property if needed,
        // but for now, the spawned tasks holding Arcs should be okay.

        window.present();
    });

    // Run the application
    std::process::exit(app.run_with_args::<&str>(&[]));
}
