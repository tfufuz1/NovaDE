use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Box}; // Added Box
use super::workspace_item_widget::WorkspaceItemWidget;
use super::types::WorkspaceInfo; // Corrected path

pub mod workspace_item_widget;
pub mod types;

mod imp;

glib::wrapper! {
    pub struct WorkspaceIndicatorWidget(ObjectSubclass<imp::WorkspaceIndicatorWidget>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl WorkspaceIndicatorWidget {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    // Remove display_placeholder_workspaces as it's replaced by update_workspaces
    // pub fn display_placeholder_workspaces(&self, count: usize) { ... }


    pub fn update_workspaces(&self, workspaces_info: Vec<WorkspaceInfo>) {
        self.imp().update_workspaces_impl(workspaces_info);
    }
    
    // This method is called from the signal handler in imp.rs
    // It needs to be callable by the imp module.
    pub(super) fn on_workspace_item_clicked_priv(&self, item_widget: &WorkspaceItemWidget) {
        self.imp().on_workspace_item_clicked_priv_impl(item_widget);
    }
}
