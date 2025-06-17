// novade-ui/src/ui_feedback.rs
use gtk::{glib, prelude::*, AlertDialog, IsA, Window}; // Added IsA, Window
use libadwaita as adw;
use log::debug; // For logging toast/dialog calls

// UiFeedbackService can be an empty struct if all methods are free functions.
// If it needed to hold state (e.g., a reference to a default window or settings),
// it would have fields. For now, free functions are fine.
#[derive(Debug, Default)]
pub struct UiFeedbackService;

impl UiFeedbackService {
    pub fn new() -> Self {
        Self::default()
    }

    // Shows a toast notification on the provided Adwaita ApplicationWindow.
    pub fn show_toast(window: &adw::ApplicationWindow, message: &str, priority: adw::ToastPriority) {
        debug!("Showing toast: '{}', priority: {:?}", message, priority);
        let toast = adw::Toast::new(message);
        toast.set_priority(priority);
        // toast.set_timeout(3); // Default is usually fine, but can be set
        window.add_toast(toast);
    }

    // Convenience function for informational toasts.
    pub fn show_info_toast(window: &adw::ApplicationWindow, message: &str) {
        Self::show_toast(window, message, adw::ToastPriority::Normal);
    }

    // Convenience function for error toasts.
    pub fn show_error_toast(window: &adw::ApplicationWindow, message: &str) {
        // Adwaita doesn't have explicit "error" styled toasts by default.
        // Priority might influence accessibility or log output, but not visual style without custom CSS.
        // For visual distinction, one might use a custom ToastOverlay and custom styled toasts.
        // For now, using High priority for errors.
        Self::show_toast(window, message, adw::ToastPriority::High);
    }

    // Shows a critical error dialog.
    // `parent` should ideally be the window that the dialog should be transient for.

    // Step 1: Create a separate creation function (made public for testing)
    pub fn create_critical_error_dialog(title: &str, message: &str) -> AlertDialog {
        debug!("Creating critical error dialog: '{}' - '{}'", title, message);
        let dialog = AlertDialog::builder()
            .heading(title)
            .body(message)
            .modal(true)
            .build();

        dialog.add_button("Close", gtk::ResponseType::Close);
        dialog.set_default_response(Some(gtk::ResponseType::Close));
        dialog
    }

    // Step 2: Modify show_critical_error_dialog to use the creation function
    pub fn show_critical_error_dialog<P: IsA<Window>>(
        parent: Option<&P>,
        title: &str,
        message: &str,
    ) {
        let dialog = Self::create_critical_error_dialog(title, message);

        if let Some(p) = parent {
            dialog.present(Some(p));
        } else {
            eprintln!("Attempting to show critical dialog without a parent: {} - {}. Dialog not shown.", title, message);
            // Not calling present() here as it's problematic without a parent.
            // The caller should ensure a parent is available or handle this case.
        }
    }
}

// Make it easier to call module functions if preferred without instantiating UiFeedbackService
// (show_critical_error_dialog helper function updated to match the struct method)
pub fn show_info_toast(window: &adw::ApplicationWindow, message: &str) {
    UiFeedbackService::show_info_toast(window, message);
}

pub fn show_error_toast(window: &adw::ApplicationWindow, message: &str) {
    UiFeedbackService::show_error_toast(window, message);
}

pub fn show_critical_error_dialog<P: IsA<Window>>( // Free function signature matches struct method
    parent: Option<&P>,
    title: &str,
    message: &str,
) {
    UiFeedbackService::show_critical_error_dialog(parent, title, message);
}


#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (ui_feedback)

    fn init_gtk() {
        if !gtk::is_initialized() {
            gtk::test_init();
        }
    }

    #[test]
    fn test_create_critical_error_dialog_properties() {
        init_gtk(); // Ensure GTK is initialized for creating AlertDialog

        let title = "Test Error Title";
        let message = "This is a test error message.";

        let dialog = UiFeedbackService::create_critical_error_dialog(title, message);

        assert_eq!(dialog.heading(), Some(title.to_glib_none().0)); // Compare with GString
        assert_eq!(dialog.body(), Some(message.to_glib_none().0));
        assert_eq!(dialog.is_modal(), true);

        // Check button (more complex, involves action names or response IDs)
        // For simplicity, we know we add one button with ResponseType::Close.
        // This is harder to directly inspect without listing actions/responses.
        // We'll trust the `add_button` and `set_default_response` calls for now.
        // If more detailed button inspection is needed, it would require iterating
        // over dialog actions or a more involved setup.
    }

    // Testing show_error_toast and show_info_toast is difficult because:
    // 1. They require an adw::ApplicationWindow, which is heavy to set up in a unit test
    //    without a running application.
    // 2. Toasts are transient UI elements added to a window's overlay; verifying their
    //    presence and properties programmatically is non-trivial in a unit test.
    // These are better tested via integration or UI tests.
    // The subtask asks to focus on dialog creation for UiFeedbackService tests.
}

// Need to add this file to `novade-ui/src/lib.rs` or `novade-ui/src/mod.rs`
// For now, assuming it will be: `pub mod ui_feedback;` in `novade-ui/src/lib.rs` (if that's the crate root)
// or in `novade-ui/src/system_health_dashboard/mod.rs` if it's specific to the dashboard.
// Given its generic nature, `novade-ui/src/lib.rs` or a general `ui_components` module seems more appropriate.
// For this task, I'll assume it's accessible via `crate::ui_feedback`.
// If `novade-ui/src/lib.rs` exists and is the crate root:
// ```rust
// // In novade-ui/src/lib.rs
// pub mod system_health_dashboard;
// pub mod theming_gtk;
// pub mod ui_feedback; // Add this line
// ```
// If the crate root is `main.rs` (binary crate), then it might be `mod ui_feedback;` in `main.rs`
// or `pub mod ui_feedback;` in `novade-ui/src/lib.rs` if `novade-ui` is a library used by `main.rs`.
// The project structure implies `novade-ui` is a library, so `novade-ui/src/lib.rs` is the place.
// I will assume `novade-ui/src/lib.rs` exists and add `pub mod ui_feedback;` there.
// If `novade-ui/src/lib.rs` does not exist, this module needs to be declared in `main.rs` using `mod ui_feedback;`
// and then accessed via `crate::ui_feedback`.
// For now, I will also try to update `novade-ui/src/lib.rs` if it exists.
// If `novade-ui/src/lib.rs` doesn't exist, the `ui_feedback` module would typically be declared in `main.rs`
// using `mod ui_feedback;` and then accessed from `SystemHealthViewModel` as `crate::ui_feedback::...`.
// Let's check for `novade-ui/src/lib.rs`.
