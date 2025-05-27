use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Box}; 

// Declare and use NotificationWidgetStub
pub mod notification_widget_stub;
pub use notification_widget_stub::NotificationWidgetStub;

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
