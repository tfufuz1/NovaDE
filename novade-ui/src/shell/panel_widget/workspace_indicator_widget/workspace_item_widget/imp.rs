use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

#[derive(Default)]
pub struct WorkspaceItemWidget {
    pub workspace_id: RefCell<Option<String>>,
    pub workspace_number: RefCell<Option<usize>>,
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceItemWidget {
    const NAME: &'static str = "NovaDEWorkspaceItemWidget";
    type Type = super::WorkspaceItemWidget;
    type ParentType = gtk::Button;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("workspaceitemwidget");
    }
}

impl ObjectImpl for WorkspaceItemWidget {
    fn constructed(&self) {
        self.parent_constructed();
        // Additional setup can go here if needed
    }
}

impl WidgetImpl for WorkspaceItemWidget {}
impl ButtonImpl for WorkspaceItemWidget {}
