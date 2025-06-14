use novade_system::system_health_collectors::{SystemMetricCollector, SystemLogHarvester, SystemDiagnosticRunner};
use novade_domain::system_health_service::service::{SystemHealthService, SystemHealthServiceTrait}; // Corrected path
use novade_core::types::system_health::SystemHealthDashboardConfig;
use novade_ui::system_health_dashboard::{SystemHealthViewModel, metrics_panel::MetricsPanel}; // Assuming metrics_panel is pub
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
// use log::LevelFilter; // Example for env_logger

#[tokio::main]
async fn main() {
    // Initialize logging (e.g., simple_logger or env_logger)
    // env_logger::Builder::new().filter_level(LevelFilter::Debug).init();
    // For this test, direct println might be okay, but for robust debugging, proper logging is better.
    println!("UI Prototype: Initializing...");

    let metric_collector = Arc::new(SystemMetricCollector::new());
    // Create dummy instances for other adapters as they are required by SystemHealthService::new
    let log_harvester = Arc::new(SystemLogHarvester::new());
    let diagnostic_runner = Arc::new(SystemDiagnosticRunner::new());

    let config = SystemHealthDashboardConfig::default();
    // SystemHealthService::new now returns Arc<SystemHealthService> and starts its own polling
    let service = SystemHealthService::new(config, metric_collector, log_harvester, diagnostic_runner);
    
    // SystemHealthViewModel::new now returns Arc<Mutex<SystemHealthViewModel>> and starts its own subscriptions
    let view_model = SystemHealthViewModel::new(service.clone());
    
    // Instantiate the panel that has the display method
    let metrics_panel = MetricsPanel::new(view_model.clone());

    println!("UI Prototype: Starting display loop. Memory metrics should update via println from MetricsPanel.");
    
    for i in 0..20 { // Run for a few iterations (e.g., 20 iterations * 2 secs = 40 secs)
        println!("\n--- Iteration {} ---", i + 1);
        metrics_panel.display_memory_metrics().await;
        sleep(Duration::from_secs(2)).await;
    }

    println!("UI Prototype: Finished display loop.");

    // Keep the main thread alive for a bit longer if background tasks in VM need to print more.
    // sleep(Duration::from_secs(5)).await;
    // println!("UI Prototype: Terminating.");
}
