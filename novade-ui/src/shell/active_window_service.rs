use std::cell::Cell;
use std::sync::Arc;
use gtk::gio;
use std::sync::Arc;
// Removed gtk::gio and gtk::glib as menu_model is no longer handled here
use novade_system::window_info_provider::{SystemWindowInfoProvider, FocusedWindowDetails};

#[derive(Clone, Debug)]
pub struct ActiveWindowData {
    pub app_id: Option<String>,      // From FocusedWindowDetails
    pub window_title: Option<String>,// From FocusedWindowDetails
    pub icon_name: Option<String>,   // From FocusedWindowDetails
    // menu_model field is removed
}

// Helper functions create_stub_menu1 and create_stub_menu2 are removed as they are no longer needed here.

// PredefinedMenuData struct is removed.

pub struct ActiveWindowService {
    system_info_provider: Arc<dyn SystemWindowInfoProvider>,
    // The counter is now only for the SystemWindowInfoProvider's cycling behavior,
    // which is internal to StubSystemWindowInfoProvider.
    // ActiveWindowService itself doesn't need to cycle anything anymore.
}

impl ActiveWindowService {
    pub fn new(system_info_provider: Arc<dyn SystemWindowInfoProvider>) -> Self {
        Self {
            system_info_provider,
            // No predefined_menu_associations or menu_cycle_counter needed
        }
    }

    pub fn get_current_active_window(&self) -> ActiveWindowData {
        // Get all details from the system provider
        let focused_details = self.system_info_provider.get_focused_window_details();

        // Construct ActiveWindowData directly from FocusedWindowDetails
        // menu_model is no longer part of this struct.
        ActiveWindowData {
            app_id: focused_details.app_id,
            window_title: focused_details.title,
            icon_name: focused_details.icon_name,
        }
    }
}
// Default impl can be removed if it's not straightforward or necessary.
// For this service, it requires a SystemWindowInfoProvider, so a Default is not trivial.
