// novade-ui/src/widgets/notification_popup.rs
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use std::sync::Arc;
use tracing::debug;

// Define the data structure for an action
#[derive(Clone, Debug)]
pub struct PopupAction {
    pub id: String,
    pub label: String,
}

// Define the data for constructing a notification popup
#[derive(Clone, Debug)]
pub struct NotificationPopupData {
    pub notification_dbus_id: u32, // The D-Bus ID, important for actions/closing
    pub title: String,
    pub body: String,
    pub icon_name: Option<String>,
    pub actions: Vec<PopupAction>,
    // pub urgency: ??? // Could be used for styling
    // pub resident: bool, // If it should auto-close or not (timeout)
    // pub created_at: std::time::Instant, // For managing expiration if handled by this widget
}

// Define events that the popup can emit
#[derive(Debug)]
pub enum NotificationPopupEvent {
    Closed(u32), // dbus_id of the notification closed by its own button
    ActionInvoked(u32, String), // dbus_id, action_id
}

// Using a simple function type for callbacks for now
// In a real scenario, this might use glib::closure or other event mechanisms
pub type PopupCallback = Box<dyn Fn(NotificationPopupEvent) + Send + Sync>;

// The main widget struct
// For simplicity, making it a newtype around a GtkBox.
// Could also be a proper composite widget using #[extends(gtk::Box)]
#[derive(Clone)]
pub struct NotificationPopup {
    container: gtk::Box,
    dbus_id: u32, // Store the D-Bus ID
    // callback: Option<Arc<PopupCallback>>, // To send events back to manager
}

impl NotificationPopup {
    pub fn new(data: NotificationPopupData, callback: Arc<PopupCallback>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.add_css_class("notification-popup-widget"); // For styling

        // Header (Icon, Text, Close Button)
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

        // Icon
        if let Some(icon_name) = &data.icon_name {
            if !icon_name.is_empty() {
                let icon = gtk::Image::from_icon_name(icon_name);
                icon.set_pixel_size(32); // Consistent size
                header_box.append(&icon);
            }
        }

        // Text content (Title and Body)
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let title_label = gtk::Label::new(Some(&data.title));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_wrap(true);
        title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        title_label.add_css_class("notification-title"); // From notification_ui.rs
        text_box.append(&title_label);

        let body_label = gtk::Label::new(Some(&data.body));
        body_label.set_halign(gtk::Align::Start);
        body_label.set_wrap(true);
        body_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        // body_label.set_lines(3); // Allow more lines if needed, or manage via CSS
        // body_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        body_label.add_css_class("notification-body"); // From notification_ui.rs
        text_box.append(&body_label);
        header_box.append(&text_box);

        // Close button for this specific notification
        let close_button = gtk::Button::from_icon_name("window-close-symbolic");
        close_button.set_valign(gtk::Align::Start);
        close_button.add_css_class("notification-close"); // From notification_ui.rs
        
        let callback_clone_close = callback.clone();
        let dbus_id_clone_close = data.notification_dbus_id;
        close_button.connect_clicked(move |_| {
            debug!("Popup close button clicked for D-Bus ID: {}", dbus_id_clone_close);
            (callback_clone_close)(NotificationPopupEvent::Closed(dbus_id_clone_close));
        });
        header_box.append(&close_button);
        container.append(&header_box);

        // Actions area
        if !data.actions.is_empty() {
            let actions_box = gtk::FlowBox::new(); // Using FlowBox for better wrapping if many actions
            actions_box.set_valign(gtk::Align::Start);
            actions_box.set_halign(gtk::Align::Fill); // Or Start/End depending on desired layout
            actions_box.set_selection_mode(gtk::SelectionMode::None);
            actions_box.set_max_children_per_line(3); // Example: max 3 actions per line
            actions_box.add_css_class("notification-actions-box");

            for action_data in data.actions {
                let action_button = gtk::Button::with_label(&action_data.label);
                action_button.add_css_class("notification-action"); // From notification_ui.rs
                action_button.set_hexpand(true); // Allow buttons to grow
                
                let callback_clone_action = callback.clone();
                let dbus_id_clone_action = data.notification_dbus_id;
                let action_id_clone = action_data.id.clone();
                action_button.connect_clicked(move |_| {
                    debug!("Popup action '{}' clicked for D-Bus ID: {}", action_id_clone, dbus_id_clone_action);
                    (callback_clone_action)(NotificationPopupEvent::ActionInvoked(
                        dbus_id_clone_action,
                        action_id_clone.clone(),
                    ));
                });
                actions_box.insert(&action_button, -1);
            }
            container.append(&actions_box);
        }
        
        Self { container, dbus_id: data.notification_dbus_id /* callback: Some(callback) */ }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
    
    pub fn dbus_id(&self) -> u32 {
        self.dbus_id
    }
}

// Example of how this widget might be used (for testing or by a manager)
#[cfg(test)]
mod tests {
    use super::*;
    use gtk::glib; // For main loop in test
    use std::sync::mpsc;

    // Helper to initialize GTK for tests if not already done.
    fn ensure_gtk_init() {
        if !gtk::is_initialized() {
            gtk::init().expect("Failed to initialize GTK for test.");
        }
    }

    #[test]
    fn test_notification_popup_creation() {
        ensure_gtk_init();
        let (tx, rx) = mpsc::channel::<NotificationPopupEvent>();

        let test_data = NotificationPopupData {
            notification_dbus_id: 1,
            title: "Test Title".to_string(),
            body: "This is the test body of the notification.".to_string(),
            icon_name: Some("dialog-information-symbolic".to_string()),
            actions: vec![
                PopupAction { id: "action1".to_string(), label: "Confirm".to_string() },
                PopupAction { id: "action2".to_string(), label: "Cancel".to_string() },
            ],
        };

        let callback = Arc::new(Box::new(move |event: NotificationPopupEvent| {
            tx.send(event).unwrap();
        }) as PopupCallback);

        let popup_widget = NotificationPopup::new(test_data.clone(), callback);
        
        // Basic check: widget is created
        assert_eq!(popup_widget.dbus_id(), 1);

        // To visually test or interact, you'd need to add it to a window and run a GTK main loop.
        // For unit tests, we can simulate clicks if we had access to buttons, or check properties.
        // This example doesn't expose buttons directly, interaction is via callback.
        
        // Example: To test callbacks, you would need to run a GTK main context iteration
        // or directly call `clicked` on the buttons if they were accessible.
        // This is complex for simple unit tests. Integration testing is better for UI interaction.
        
        // Here, we just check creation. Further tests would require a running GTK loop.
        // For instance, one might create a window, add the widget, then use `gtk::test::find_widget`
        // and `gtk::test::widget_event` for more in-depth testing.
    }
}
