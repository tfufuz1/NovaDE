use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Box}; // Added Box and prelude

mod imp;

glib::wrapper! {
    pub struct NotificationCenterPanelWidget(ObjectSubclass<imp::NotificationCenterPanelWidget>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NotificationCenterPanelWidget {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }
}
