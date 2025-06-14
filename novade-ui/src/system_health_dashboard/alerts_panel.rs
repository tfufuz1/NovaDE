// novade-ui/src/system_health_dashboard/alerts_panel.rs
// Displays active and historical alerts. Allows acknowledging alerts.
// Subscribes to alert updates from SystemHealthViewModel.

// #[cfg(feature = "gtk")]
// use gtk::prelude::*;
// #[cfg(feature = "gtk")]
// use gtk::{Box as GtkBox, Label, ListBox, ListBoxRow, Button, Orientation};

// use std::sync::Arc;
// use tokio::sync::Mutex;
// use super::view_model::SystemHealthViewModel;
// use novade_core::types::system_health::Alert; // For displaying alert details

pub struct AlertsPanel {
    // view_model: Arc<Mutex<SystemHealthViewModel>>,
    // Placeholder for UI elements
    // active_alerts_list_box: Option<ListBox>,
    // container: Option<GtkBox>,
}

impl AlertsPanel {
    // pub fn new(view_model: Arc<Mutex<SystemHealthViewModel>>) -> Self {
    pub fn new() -> Self { // Simplified for now
        println!("UI: AlertsPanel created (placeholder).");
        // Self { view_model, container: None /*, initialize other fields */ }
        Self {}
    }

    // #[cfg(feature = "gtk")]
    // pub fn get_widget(&mut self) -> &GtkBox {
    //     if self.container.is_none() {
    //         let container = GtkBox::new(Orientation::Vertical, 5);
    //         let title_label = Label::new(Some("Alerts Panel (Placeholder)"));
    //         container.append(&title_label);

    //         let list_box = ListBox::new();
    //         // In a real app, you'd populate this from view_model.get_active_alerts()
    //         // and subscribe to updates.
    //         // For now, a placeholder item:
    //         let placeholder_row = ListBoxRow::new();
    //         let row_box = GtkBox::new(Orientation::Horizontal, 5);
    //         row_box.append(&Label::new(Some("Placeholder Alert: System critical!")));
    //         let ack_button = Button::with_label("Acknowledge");
    //         // ack_button.connect_clicked(...);
    //         row_box.append(&ack_button);
    //         placeholder_row.set_child(Some(&row_box));
    //         list_box.append(&placeholder_row);

    //         self.active_alerts_list_box = Some(list_box.clone());
    //         container.append(&list_box);

    //         self.container = Some(container);
    //         // TODO: Load active alerts and subscribe to updates
    //         // TODO: Implement acknowledge logic
    //     }
    //     self.container.as_ref().expect("Container should be initialized")
    // }
}
