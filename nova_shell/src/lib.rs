// nova_shell/src/lib.rs
use gtk4::{self as gtk, glib}; // Use gtk's re-export of glib
use tracing::info;

pub mod panel;
pub mod launcher;
pub mod notifications;
pub mod workspaces;
pub mod background;
pub mod application; // Declare the new application module

// This function is now primarily for library users or tests needing GTK initialized.
pub fn ensure_gtk_initialized() -> Result<(), gtk::glib::BoolError> {
    info!("Ensuring GTK is initialized for NovaDE Shell library context...");
    gtk::init()
}

#[cfg(test)]
mod tests {
    use super::*;
    // It's good practice to initialize GTK for any test that might use its features,
    // even if it's just creating widgets without showing them.
    fn ensure_gtk_for_tests() {
        match ensure_gtk_initialized() {
            Ok(_) => {}, // GTK initialized
            Err(_) => {
                // In some test environments (like CI without a display server),
                // GTK might not initialize. This is often okay for basic widget creation tests.
                // For more complex UI tests, a virtual framebuffer (e.g., xvfb) might be needed.
                info!("GTK initialization failed in test context (may be expected in headless env).");
            }
        }
    }

    #[test]
    fn test_ensure_gtk_initialized_for_lib() {
        ensure_gtk_for_tests();
    }

    #[test]
    fn test_panel_widget_creation() {
        ensure_gtk_for_tests();
        let panel_widget = panel::create_panel_widget();
        // Basic check: Does it have the expected CSS class?
        assert!(panel_widget.has_css_class("nova-panel"));
        // Can check for height request if needed, e.g. assert_eq!(panel_widget.height_request(), 36);
    }

    // Tests for other modules (launcher, notifications, etc.) testing their logic, not UI.
    // These structs are still defined in their respective modules.
    #[test]
    fn test_launcher_logic() { // Renamed to avoid confusion with UI creation
        let launcher = launcher::Launcher::new();
        launcher.show(); // This is a placeholder method, not showing UI
        launcher.launch_app("test.desktop");
    }

    #[test]
    fn test_notification_manager_logic() { // Renamed
        let manager = notifications::NotificationManager::new();
        manager.show_notification("Test Summary", "Test body message.");
    }

    #[test]
    fn test_workspace_manager_logic() { // Renamed
        let mut manager = workspaces::WorkspaceManager::new();
        manager.switch_to(2);
    }

    #[test]
    fn test_background_manager_logic() { // Renamed
        let manager = background::BackgroundManager::new();
        manager.set_wallpaper("/path/to/wallpaper.jpg");
    }
}
