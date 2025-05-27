use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Button}; // Added Button

mod imp;

glib::wrapper! {
    pub struct ClockDateTimeWidget(ObjectSubclass<imp::ClockDateTimeWidget>)
        @extends gtk::Widget, gtk::Button, @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ClockDateTimeWidget {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    // Public getter for format-string
    pub fn format_string(&self) -> String {
        self.property("format-string")
            .expect("format-string property not found")
            .get()
            .expect("format-string value could not be retrieved")
    }

    // Public setter for format-string
    pub fn set_format_string(&self, format_string: &str) {
        self.set_property("format-string", format_string.to_value())
            .unwrap();
    }

    // Public getter for show-calendar-on-click
    pub fn show_calendar_on_click(&self) -> bool {
        self.property("show-calendar-on-click")
            .expect("show-calendar-on-click property not found")
            .get()
            .expect("show-calendar-on-click value could not be retrieved")
    }

    // Public setter for show-calendar-on-click
    pub fn set_show_calendar_on_click(&self, show: bool) {
        self.set_property("show-calendar-on-click", show)
            .unwrap();
    }
    
    // Private method callers (should match names in imp.rs but call via self.imp())
    // These are for internal use by the ObjectImpl or other parts of the widget's own logic
    // For external calls, direct property access or public methods are preferred.
    // However, for consistency with the plan to have `update_display_priv` and `setup_timer_priv`
    // callable from `imp.rs` via `self.obj().method_name()`, they need to be exposed here.
    // The actual implementation for these is in `imp.rs` as `*_impl`.
    
    // Renaming to avoid conflict with imp module's methods when called from imp.rs
    pub(super) fn update_display_priv(&self) {
        self.imp().update_display_priv_impl();
    }

    pub(super) fn setup_timer_priv(&self) {
        self.imp().setup_timer_priv_impl();
    }
}
