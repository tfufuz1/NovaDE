use std::process::{Command, Stdio, Child};
use std::thread;
use std::time::Duration;
use anyhow::{Context, Result};

fn kill_process_tree(pid: u32) -> Result<()> {
    // Use pkill to kill the process and all its descendants.
    // This is a common approach on Linux.
    // Ensure `pkill` is available or adapt for other platforms if needed.
    let status = Command::new("pkill")
        .arg("-P") // Match children of this parent PID
        .arg(pid.to_string())
        .status()
        .context(format!("Failed to execute pkill for PID {}", pid))?;

    if status.success() {
        tracing::info!("Successfully sent SIGTERM to process tree of PID {}", pid);
    } else {
        tracing::warn!("pkill for PID {} exited with status: {}", pid, status);
    }
    Ok(())
}


#[test]
#[ignore] // Ignored by default as it requires built binaries and has side effects (spawns processes)
fn test_novade_orchestrator_starts_and_runs_briefly() -> Result<()> {
    // This test assumes that `cargo build` has already been run for novade,
    // novade-system, and novade-ui, so the binaries are in target/debug/.
    // It also assumes it's run from the workspace root or a context where these paths are valid.

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer() // Configure for tests
        .init(); // Initialize tracing for the test itself

    tracing::info!("Starting NovaDE orchestrator test...");

    let novade_executable = if cfg!(target_os = "windows") {
        "target/debug/novade.exe"
    } else {
        "target/debug/novade"
    };

    tracing::info!("Launching NovaDE executable: {}", novade_executable);
    let mut novade_process = Command::new(novade_executable)
        .stdout(Stdio::inherit()) // Inherit stdio to see logs from novade and its children
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn novade orchestrator process")?;

    let novade_pid = novade_process.id();
    tracing::info!("NovaDE orchestrator process spawned with PID: {}", novade_pid);

    // Let it run for a defined period.
    // This duration needs to be long enough for novade-system and novade-ui to attempt startup.
    // 15 seconds should be ample for initialization and basic Wayland handshake.
    let run_duration = Duration::from_secs(15);
    tracing::info!("Letting NovaDE run for {:?}...", run_duration);
    thread::sleep(run_duration);

    tracing::info!("Attempting to terminate NovaDE orchestrator process (PID: {}) and its children...", novade_pid);

    // Terminate the main `novade` process and its children.
    // Sending SIGTERM to `novade` might not be enough if it doesn't propagate to children well,
    // or if children are in new process groups.
    // A more robust way is to kill the process group or use pkill -P.
    if let Err(e) = kill_process_tree(novade_pid) {
        tracing::error!("Error trying to kill process tree for PID {}: {:?}", novade_pid, e);
        // Fallback: try to kill the main process directly if pkill failed or isn't available
        if let Err(kill_err) = novade_process.kill() {
             tracing::error!("Failed to kill novade orchestrator process directly: {}", kill_err);
             // Continue to try and wait for it anyway
        }
    }

    // Wait for the novade process to exit and check its status
    // This might hang if the process or its children don't terminate.
    // Consider adding a timeout to the wait as well for very robust CI.
    match novade_process.wait() {
        Ok(status) => {
            tracing::info!("NovaDE orchestrator process exited with status: {}", status);
            // A basic success condition for this test is that it ran and then was terminated.
            // More advanced checks could involve parsing logs or checking for specific IPC signals.
            // If `novade` itself has an error during startup, it might exit non-zero here.
            // If it was killed by our signal, the status might reflect that (e.g., on Unix).
            // For now, just logging the status is the main check.
        }
        Err(e) => {
            tracing::error!("Failed to wait for novade orchestrator process: {}", e);
            return Err(e.into());
        }
    }

    tracing::info!("NovaDE orchestrator test finished.");
    Ok(())
}
