use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use super::workspace_item_widget::WorkspaceItemWidget;

// No CompositeTemplate for now, will create Box manually.
#[derive(Default)]
pub struct WorkspaceIndicatorWidget {
    // If not using TemplateChild, this would be an Option<gtk::Box> initialized in constructed
    // For simplicity and since we are building UI in code:
    pub workspace_items_container: RefCell<Option<Box>>, // Container to hold WorkspaceItemWidgets
    pub workspace_item_widgets: RefCell<HashMap<String, WorkspaceItemWidget>>,
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceIndicatorWidget {
    const NAME: &'static str = "NovaDEWorkspaceIndicatorWidget";
    type Type = super::WorkspaceIndicatorWidget;
    type ParentType = gtk::Box; // Changed from gtk::Widget to gtk::Box as it's a container

    fn new() -> Self {
        Self {
            workspace_items_container: RefCell::new(None),
            workspace_item_widgets: RefCell::new(HashMap::new()),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("workspaceindicator");
    }
}

impl ObjectImpl for WorkspaceIndicatorWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); // This is the WorkspaceIndicatorWidget (which is a Box)

        // Configure the WorkspaceIndicatorWidget itself (it's a Box)
        obj.set_orientation(gtk::Orientation::Horizontal);
        obj.set_spacing(4);
        
        // The WorkspaceIndicatorWidget itself is the container.
        // So, we don't need a separate workspace_items_container TemplateChild or field
        // if WorkspaceIndicatorWidget *is* the Box.
        // If it were a gtk::Widget that *contains* a Box, then we'd need it.
        // Given ParentType = gtk::Box, obj is already the box.

        // Store a reference to self (as a Box) in workspace_items_container for consistency if needed by other methods
        // but it's not strictly necessary if methods always use self.obj()
        self.workspace_items_container.replace(Some(obj.clone().upcast::<gtk::Box>()));


        // Call the method to display placeholder workspaces
        // obj.display_placeholder_workspaces(4); // Remove this, will be driven by update_workspaces
    }
}

impl WidgetImpl for WorkspaceIndicatorWidget {}
impl BoxImpl for WorkspaceIndicatorWidget {}

// Private methods for WorkspaceIndicatorWidget
impl WorkspaceIndicatorWidget {
    pub(super) fn update_workspaces_impl(
        &self,
        workspaces_info: Vec<super::types::WorkspaceInfo>,
    ) {
        let obj = self.obj(); // This is the WorkspaceIndicatorWidget (gtk::Box)
        
        // Remove all existing children
        while let Some(child) = obj.first_child() {
            obj.remove(&child);
        }
        self.workspace_item_widgets.borrow_mut().clear();

        for info in workspaces_info {
            let item_widget = WorkspaceItemWidget::new(&info);
            
            // Clone item_widget for the closure
            let item_widget_clone = item_widget.clone();
            let widget_indicator_clone = obj.clone(); // Clone WorkspaceIndicatorWidget for the closure

            item_widget.connect_clicked(move |_btn| {
                widget_indicator_clone.on_workspace_item_clicked_priv(&item_widget_clone);
            });

            obj.append(&item_widget);
            self.workspace_item_widgets.borrow_mut().insert(info.id.clone(), item_widget);
        }
    }

    pub(super) fn on_workspace_item_clicked_priv_impl(&self, item_widget: &WorkspaceItemWidget) {
        let id = item_widget.workspace_id();
        let num = item_widget.workspace_number();
        tracing::info!(
            "Workspace item clicked: ID: {:?}, Number: {:?}",
            id,
            num
        );

        // Simple visual feedback for testing: make the clicked item active
        if let Some(clicked_id) = &id {
            let mut items_map = self.workspace_item_widgets.borrow_mut();
            for (current_id, widget_in_map) in items_map.iter_mut() {
                let mut temp_info = super::types::WorkspaceInfo {
                    id: widget_in_map.workspace_id().unwrap_or_default(),
                    name: widget_in_map.tooltip_text().map(|s| s.to_string()).unwrap_or_default(),
                    icon_name: None, // Not stored in item, retrieve if needed from a central source
                    number: widget_in_map.workspace_number().unwrap_or_default(),
                    is_active: current_id == clicked_id,
                    is_occupied: false, // Not currently tracking
                };
                widget_in_map.update_content(&temp_info);
            }
        }
    }
}
