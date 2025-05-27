use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Box, Label}; // Added Label for type hint if needed

mod imp;

glib::wrapper! {
    pub struct NotificationWidgetStub(ObjectSubclass<imp::NotificationWidgetStub>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NotificationWidgetStub {
    pub fn new(app_name: &str, summary: &str) -> Self {
        let obj: Self = glib::Object::new(&[]);
        
        // Access the TemplateChild fields directly to set their text
        // These labels are created and added to the hierarchy in imp.rs's constructed method
        // and then bound to these TemplateChild fields.
        obj.imp().app_name_label.set_text(app_name);
        obj.imp().summary_label.set_text(summary);
        
        obj
    }
}
