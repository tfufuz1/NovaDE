// novade-system/src/application_manager.rs
use crate::error::SystemError;
use std::process::Command;

/// Information about an installed application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppInfo {
    pub id: String, // e.g., "firefox.desktop" or "org.mozilla.firefox"
    pub name: String, // Human-readable name, e.g., "Firefox"
    pub icon_path: Option<String>, // Path to an icon file
}

pub trait ApplicationManager: Send + Sync {
    /// Launches an application by its ID or name.
    fn launch_application(&self, app_id: &str) -> Result<(), SystemError>;

    /// Lists installed applications (placeholder).
    fn list_applications(&self) -> Result<Vec<AppInfo>, SystemError>;
}

pub struct DefaultApplicationManager;

impl DefaultApplicationManager {
    pub fn new() -> Self { DefaultApplicationManager }
}

impl Default for DefaultApplicationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationManager for DefaultApplicationManager {
    fn launch_application(&self, app_id: &str) -> Result<(), SystemError> {
        println!("[System Layer] Attempting to launch application: {}", app_id);
        // Basic cross-platform attempt to open something.
        // This is a placeholder and not robust.
        // On Linux, this might try to use xdg-open or a similar utility.
        // On Windows, `start`. On macOS, `open`.
        // For non-desktop specific app_ids, this will likely fail or do something unexpected.
        // A real implementation would use desktop standards (e.g., .desktop files on Linux).
        let command_to_run;
        let mut args: Vec<&str> = Vec::new();

        if cfg!(target_os = "windows") {
            command_to_run = "cmd";
            args.push("/C");
            args.push("start");
            args.push(""); // Title for start command, can be empty
            args.push(app_id);
        } else if cfg!(target_os = "macos") {
            command_to_run = "open";
            args.push("-a"); // Specify application name
            args.push(app_id);
        } else { // Assuming Linux/Unix-like
            command_to_run = "xdg-open";
            args.push(app_id);
        }

        let mut cmd = Command::new(command_to_run);
        cmd.args(&args);

        println!("[System Layer] Executing: {} {:?}", command_to_run, args);

        match cmd.spawn() {
            Ok(mut child) => {
                // Optionally, wait for the child process to exit if it's quick,
                // or detach it. For xdg-open, it usually detaches by itself.
                // For this prototype, let's try to wait for a short duration to see if it errors out immediately.
                // This is not ideal for long-running apps.
                // A better approach for GUI apps is to simply spawn and not wait.
                // For now, we'll detach.
                // child.wait().map_err(|e| SystemError::ApplicationLaunchFailed(app_id.to_string(), e.to_string()))?;
                println!("[System Layer] Launched or attempted to launch '{}' with command '{} {:?}'.", app_id, command_to_run, args);
                Ok(())
            }
            Err(e) => {
                eprintln!("[System Layer] Failed to launch '{}' with command '{} {:?}': {}", app_id, command_to_run, args, e);
                Err(SystemError::ApplicationLaunchFailed(app_id.to_string(), e.to_string()))
            }
        }
    }

    fn list_applications(&self) -> Result<Vec<AppInfo>, SystemError> {
        // Placeholder implementation
        println!("[System Layer] list_applications called (placeholder).");
        Ok(vec![
            AppInfo { id: "firefox.desktop".to_string(), name: "Firefox (Placeholder)".to_string(), icon_path: None },
            AppInfo { id: "gedit.desktop".to_string(), name: "Text Editor (Placeholder)".to_string(), icon_path: None },
        ])
    }
}

// TODO: Assistant Integration: This service will be called by the Smart Assistant
// to handle requests like "Open Firefox" or "List installed browsers".
// TODO: Implementation could involve parsing .desktop files, querying package managers,
// or using desktop environment-specific protocols (e.g., D-Bus services like those for KDE/GNOME).
// TODO: Define SystemError variants for application-specific errors (e.g., AppNotFound, LaunchFailed). (Done for LaunchFailed)
