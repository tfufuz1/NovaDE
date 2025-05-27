use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Button};
use super::types::WorkspaceInfo;

mod imp;

glib::wrapper! {
    pub struct WorkspaceItemWidget(ObjectSubclass<imp::WorkspaceItemWidget>)
        @extends gtk::Widget, gtk::Button, @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl WorkspaceItemWidget {
    pub fn new(info: &WorkspaceInfo) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.imp().workspace_id.replace(Some(info.id.clone()));
        obj.imp().workspace_number.replace(Some(info.number));
        obj.update_content(info);
        obj
    }

    pub fn update_content(&self, info: &WorkspaceInfo) {
        self.set_label(&info.number.to_string());
        if info.is_active {
            self.add_css_class("active-workspace");
        } else {
            self.remove_css_class("active-workspace");
        }
        self.set_tooltip_text(Some(&info.name));

        // Future: Add icon if info.icon_name is Some(_)
        // Future: Add "occupied" class if info.is_occupied is true
    }

    pub fn workspace_id(&self) -> Option<String> {
        self.imp().workspace_id.borrow().clone()
    }

    pub fn workspace_number(&self) -> Option<usize> {
        *self.imp().workspace_number.borrow()
    }
}
