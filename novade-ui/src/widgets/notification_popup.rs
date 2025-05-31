// novade-ui/src/widgets/notification_popup.rs
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use std::sync::Arc;
use tracing::debug;

// Define the data structure for an action
#[derive(Clone, Debug, PartialEq)] // Added PartialEq
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
#[derive(Debug, PartialEq)] // Added PartialEq for easier assertion
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
    // Keep handles to buttons for testing if needed, or use gtk::test utilities
    close_button_for_test: Option<gtk::Button>,
    action_buttons_for_test: Option<Vec<gtk::Button>>,
}

impl NotificationPopup {
    pub fn new(data: NotificationPopupData, callback: Arc<PopupCallback>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.add_css_class("notification-popup-widget");

        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

        if let Some(icon_name) = &data.icon_name {
            if !icon_name.is_empty() {
                let icon = gtk::Image::from_icon_name(icon_name);
                icon.set_pixel_size(32);
                header_box.append(&icon);
            }
        }

        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let title_label = gtk::Label::new(Some(&data.title));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_wrap(true);
        title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        title_label.add_css_class("notification-title");
        text_box.append(&title_label);

        let body_label = gtk::Label::new(Some(&data.body));
        body_label.set_halign(gtk::Align::Start);
        body_label.set_wrap(true);
        body_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        body_label.add_css_class("notification-body");
        text_box.append(&body_label);
        header_box.append(&text_box);

        let close_button = gtk::Button::from_icon_name("window-close-symbolic");
        close_button.set_valign(gtk::Align::Start);
        close_button.add_css_class("notification-close");

        let callback_clone_close = callback.clone();
        let dbus_id_clone_close = data.notification_dbus_id;
        close_button.connect_clicked(move |_| {
            debug!("Popup close button clicked for D-Bus ID: {}", dbus_id_clone_close);
            (callback_clone_close)(NotificationPopupEvent::Closed(dbus_id_clone_close));
        });
        header_box.append(&close_button);
        container.append(&header_box);

        let mut action_buttons_for_test_vec = Vec::new();

        if !data.actions.is_empty() {
            let actions_box = gtk::FlowBox::new();
            actions_box.set_valign(gtk::Align::Start);
            actions_box.set_halign(gtk::Align::Fill);
            actions_box.set_selection_mode(gtk::SelectionMode::None);
            actions_box.set_max_children_per_line(3);
            actions_box.add_css_class("notification-actions-box");

            for action_data in data.actions {
                let action_button = gtk::Button::with_label(&action_data.label);
                action_button.add_css_class("notification-action");
                action_button.set_hexpand(true);

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
                action_buttons_for_test_vec.push(action_button); // Store for testing
            }
            container.append(&actions_box);
        }

        Self {
            container,
            dbus_id: data.notification_dbus_id,
            close_button_for_test: Some(close_button),
            action_buttons_for_test: Some(action_buttons_for_test_vec),
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    pub fn dbus_id(&self) -> u32 {
        self.dbus_id
    }

    // Helper for tests to simulate close button click
    #[cfg(test)]
    fn click_close_button(&self) {
        if let Some(btn) = &self.close_button_for_test {
            btn.emit_clicked();
        }
    }

    // Helper for tests to simulate action button click
    #[cfg(test)]
    fn click_action_button(&self, action_index: usize) {
        if let Some(btns) = &self.action_buttons_for_test {
            if let Some(btn) = btns.get(action_index) {
                btn.emit_clicked();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk::glib;
    use std::sync::mpsc;
    use std::time::Duration;

    fn ensure_gtk_init() {
        if !gtk::is_initialized() {
            gtk::init().expect("Failed to initialize GTK for test.");
        }
        // Process GTK events once to allow widget realization if needed by some operations
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }

    #[test]
    fn test_popup_creation_basic() {
        ensure_gtk_init();
        let (tx, _rx) = mpsc::channel::<NotificationPopupEvent>();
        let callback = Arc::new(Box::new(move |event| { tx.send(event).unwrap(); }) as PopupCallback);

        let test_data_no_icon_no_actions = NotificationPopupData {
            notification_dbus_id: 1,
            title: "No Icon No Actions".to_string(),
            body: "Body text.".to_string(),
            icon_name: None,
            actions: vec![],
        };
        let popup1 = NotificationPopup::new(test_data_no_icon_no_actions, callback.clone());
        assert_eq!(popup1.dbus_id(), 1);
        // Check title label (simplified check)
        let main_box = popup1.widget();
        let header_box = main_box.first_child().unwrap().downcast::<gtk::Box>().unwrap();
        let text_box = header_box.last_child().unwrap().prev_sibling().unwrap().downcast::<gtk::Box>().unwrap();
        let title_widget = text_box.first_child().unwrap().downcast::<gtk::Label>().unwrap();
        assert_eq!(title_widget.label().as_str(), "No Icon No Actions");


        let test_data_with_icon_and_actions = NotificationPopupData {
            notification_dbus_id: 2,
            title: "With Icon & Actions".to_string(),
            body: "Another body.".to_string(),
            icon_name: Some("dialog-information-symbolic".to_string()),
            actions: vec![PopupAction { id: "confirm".to_string(), label: "Confirm".to_string() }],
        };
        let popup2 = NotificationPopup::new(test_data_with_icon_and_actions, callback);
        assert_eq!(popup2.dbus_id(), 2);
        let main_box_2 = popup2.widget();
        let header_box_2 = main_box_2.first_child().unwrap().downcast::<gtk::Box>().unwrap();
        // Icon would be header_box_2.first_child() if present
        assert!(header_box_2.first_child().is_some());
        // Actions box would be main_box_2.last_child() if actions are present
        assert!(main_box_2.last_child().is_some());
        let actions_flowbox = main_box_2.last_child().unwrap().downcast::<gtk::FlowBox>().unwrap();
        assert_eq!(actions_flowbox.children().len(), 1);
        let action_button = actions_flowbox.child_at_index(0).unwrap().child().unwrap().downcast::<gtk::Button>().unwrap();
        assert_eq!(action_button.label().unwrap().as_str(), "Confirm");
    }

    #[test]
    fn test_popup_close_button_callback() {
        ensure_gtk_init();
        let (tx, rx) = mpsc::channel::<NotificationPopupEvent>();
        let callback = Arc::new(Box::new(move |event| { tx.send(event).unwrap(); }) as PopupCallback);

        let test_data = NotificationPopupData {
            notification_dbus_id: 10,
            title: "Test Close".to_string(), body: "Body".to_string(),
            icon_name: None, actions: vec![],
        };
        let popup = NotificationPopup::new(test_data, callback);

        popup.click_close_button(); // Simulate click

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(NotificationPopupEvent::Closed(id)) => assert_eq!(id, 10),
            Ok(other) => panic!("Expected Closed event, got {:?}", other),
            Err(e) => panic!("Did not receive event in time: {}", e),
        }
    }

    #[test]
    fn test_popup_action_button_callback() {
        ensure_gtk_init();
        let (tx, rx) = mpsc::channel::<NotificationPopupEvent>();
        let callback = Arc::new(Box::new(move |event| { tx.send(event).unwrap(); }) as PopupCallback);

        let actions = vec![
            PopupAction { id: "action_yes".to_string(), label: "Yes".to_string() },
            PopupAction { id: "action_no".to_string(), label: "No".to_string() },
        ];
        let test_data = NotificationPopupData {
            notification_dbus_id: 20,
            title: "Test Actions".to_string(), body: "Choose an action".to_string(),
            icon_name: None, actions,
        };
        let popup = NotificationPopup::new(test_data, callback);

        popup.click_action_button(0); // Click "Yes"
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(NotificationPopupEvent::ActionInvoked(id, action_id)) => {
                assert_eq!(id, 20);
                assert_eq!(action_id, "action_yes");
            }
            Ok(other) => panic!("Expected ActionInvoked event, got {:?}", other),
            Err(e) => panic!("Did not receive 'action_yes' event in time: {}", e),
        }

        popup.click_action_button(1); // Click "No"
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(NotificationPopupEvent::ActionInvoked(id, action_id)) => {
                assert_eq!(id, 20);
                assert_eq!(action_id, "action_no");
            }
            Ok(other) => panic!("Expected ActionInvoked event, got {:?}", other),
            Err(e) => panic!("Did not receive 'action_no' event in time: {}", e),
        }
    }
}
