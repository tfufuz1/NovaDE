use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Button}; // Added Button and prelude

mod imp;

glib::wrapper! {
    pub struct NotificationCenterButtonWidget(ObjectSubclass<imp::NotificationCenterButtonWidget>)
        @extends gtk::Widget, gtk::Button, @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl NotificationCenterButtonWidget {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new(&[]);
        // Specific click handler for NotificationCenterButton can be added here or in constructed
        // For now, the base Button behavior is sufficient for a stub.
        // obj.connect_clicked(|_btn| {
        //     println!("Notification Center Button Clicked! Popover/panel would appear here.");
        // });
        obj
    }
}
