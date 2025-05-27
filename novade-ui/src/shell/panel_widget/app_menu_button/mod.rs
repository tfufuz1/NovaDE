use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, gio}; // Added gio for MenuModel if used in future

mod imp;

glib::wrapper! {
    pub struct AppMenuButton(ObjectSubclass<imp::AppMenuButton>)
        @extends gtk::Widget, gtk::Button, gtk::MenuButton, @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl AppMenuButton {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new(&[]);
        // In a real scenario, you might set up a default menu model here,
        // but for a stub, this is sufficient.
        // For example:
        // let menu = gio::Menu::new();
        // menu.append(Some("Preferences"), Some("app.preferences"));
        // menu.append(Some("Quit"), Some("app.quit"));
        // obj.set_menu_model(Some(&menu));
        obj
    }

    pub fn update_active_window_info(
        &self,
        app_id: Option<String>,
        window_title: Option<String>,
        icon_name: Option<String>,
    ) {
        self.imp().update_active_window_info_impl(app_id, window_title, icon_name);
    }
}
