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

    // Renamed from set_shell_workspace_service
    pub fn set_domain_workspace_connector(
        &self,
        connector: std::rc::Rc<crate::shell::domain_workspace_connector::DomainWorkspaceConnector>,
        // Receiver for UI updates, created in main.rs and its sender is passed to DomainWorkspaceConnector
        ui_event_receiver: glib::Receiver<Vec<WorkspaceInfo>>, 
    ) {
        self.imp().domain_connector.replace(Some(connector.clone()));

        // Attach the receiver to the main context to update UI
        let widget_weak = self.downgrade(); // Use weak reference to avoid cycles
        ui_event_receiver.attach(None, move |infos| {
            if let Some(widget) = widget_weak.upgrade() {
                tracing::info!("WorkspaceIndicatorWidget: Received workspace update via glib channel.");
                widget.update_workspaces(infos);
            } else {
                tracing::warn!("WorkspaceIndicatorWidget: Weak reference upgrade failed in UI event receiver.");
            }
            glib::ControlFlow::Continue
        });
        
        // Perform initial fetch of workspaces
        // This needs to be done carefully to avoid blocking and to ensure it runs on the UI thread
        // for the final update.
        let widget_clone = self.clone(); // Clone for the async block
        // Spawn on the main context to ensure UI updates happen on the correct thread
        glib::MainContext::default().spawn_local(async move {
            tracing::info!("WorkspaceIndicatorWidget: Performing initial workspace fetch.");
            let initial_infos = connector.get_all_workspaces_for_ui().await;
            widget_clone.update_workspaces(initial_infos);
        });
    }

    // The old refresh_workspaces and set_shell_workspace_service are effectively replaced.
    // If a manual refresh is needed, it should be triggered via the connector which then
    // sends an event through the existing channel.
}
