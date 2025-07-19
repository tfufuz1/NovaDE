// novade-ui/src/widgets/window_decoration/mod.rs

pub mod imp;

use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct WindowDecoration(ObjectSubclass<imp::WindowDecoration>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl WindowDecoration {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a WindowDecoration")
    }
}

impl Default for WindowDecoration {
    fn default() -> Self {
        Self::new()
    }
}
