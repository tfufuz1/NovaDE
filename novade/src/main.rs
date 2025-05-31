// NovaDE Main Application Entry Point

use anyhow::{Context, Result};
use std::process::Command;
use tokio::process::Command as TokioCommand;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    info!("Starting NovaDE orchestrator...");

    // Spawn novade-system
    info!("Launching novade-system...");
    let mut system_process = TokioCommand::new("target/debug/novade-system")
        .spawn()
        .context("Failed to spawn novade-system")?;
    let system_pid = system_process.id().context("Failed to get novade-system PID")?;
    info!("novade-system launched with PID: {}", system_pid);

    // Wait for novade-system to initialize (placeholder)
    info!("Waiting 5 seconds for novade-system to initialize...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Spawn novade-ui
    info!("Launching novade-ui...");
    let mut ui_process = TokioCommand::new("target/debug/novade-ui")
        .env("WAYLAND_DISPLAY", "wayland-1")
        .spawn()
        .context("Failed to spawn novade-ui")?;
    let ui_pid = ui_process.id().context("Failed to get novade-ui PID")?;
    info!("novade-ui launched with PID: {}", ui_pid);

    info!("NovaDE orchestrator running with novade-system (PID: {}) and novade-ui (PID: {})...", system_pid, ui_pid);

    // Wait for either process to exit
    tokio::select! {
        system_exit_status = system_process.wait() => {
            match system_exit_status {
                Ok(status) => info!("novade-system exited with status: {}", status),
                Err(e) => error!("Error waiting for novade-system: {}", e),
            }
            info!("Attempting to terminate novade-ui...");
            if let Err(e) = ui_process.start_kill() {
                error!("Failed to terminate novade-ui: {}", e);
            } else {
                match ui_process.wait().await {
                    Ok(status) => info!("novade-ui exited with status: {}", status),
                    Err(e) => error!("Error waiting for novade-ui after termination: {}", e),
                }
            }
        }
        ui_exit_status = ui_process.wait() => {
            match ui_exit_status {
                Ok(status) => info!("novade-ui exited with status: {}", status),
                Err(e) => error!("Error waiting for novade-ui: {}", e),
            }
            info!("Attempting to terminate novade-system...");
            if let Err(e) = system_process.start_kill() {
                error!("Failed to terminate novade-system: {}", e);
            } else {
                match system_process.wait().await {
                    Ok(status) => info!("novade-system exited with status: {}", status),
                    Err(e) => error!("Error waiting for novade-system after termination: {}", e),
                }
            }
        }
    }

    info!("NovaDE orchestrator shutting down.");
    Ok(())
}
