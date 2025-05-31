use gtk::glib;
use gtk::prelude::*;
use gtk::{Button, Box as GtkBox, Orientation}; // Renamed Box to GtkBox
use std::cell::RefCell;
use std::rc::Rc;

// Simple struct for the widget
#[derive(Clone)] // Clone to allow using in closures easily if needed
pub struct WorkspaceSwitcher {
    pub widget: GtkBox, // Changed from gtk::Box to GtkBox
    buttons: Rc<RefCell<Vec<Button>>>,
    active_ws_index: Rc<RefCell<usize>>,
}

impl WorkspaceSwitcher {
    pub fn new(num_workspaces: u32) -> Self {
        let widget = GtkBox::new(Orientation::Horizontal, 5); // Changed from gtk::Box
        widget.set_halign(gtk::Align::Center);
        widget.add_css_class("workspace-switcher-container"); // For styling the container

        let buttons = Rc::new(RefCell::new(Vec::new()));
        let active_ws_index = Rc::new(RefCell::new(0_usize));

        for i in 0..num_workspaces {
            let button_label = format!("{}", i + 1);
            let button = Button::with_label(&button_label);
            button.add_css_class("workspace-button");
            if i == 0 {
                button.add_css_class("active-workspace");
            }

            let buttons_clone = buttons.clone();
            let active_ws_clone = active_ws_index.clone();
            let widget_clone = widget.clone(); // For accessing parent if needed for signals later

            button.connect_clicked(move |clicked_btn| {
                let mut current_active_idx = active_ws_clone.borrow_mut();
                let btns_borrow = buttons_clone.borrow();

                // Remove active class from previously active button
                if let Some(prev_active_btn) = btns_borrow.get(*current_active_idx) {
                    prev_active_btn.remove_css_class("active-workspace");
                }

                // Find index of clicked button
                if let Some(clicked_idx) = btns_borrow.iter().position(|b| b == clicked_btn) {
                    clicked_btn.add_css_class("active-workspace");
                    *current_active_idx = clicked_idx;
                    tracing::info!("Switched to (placeholder) workspace: {}", clicked_idx + 1);
                    // Future: send signal to backend here
                    // Example: widget_clone.emit_by_name::<()>("workspace-changed", &[&(clicked_idx as u32)]);
                }
            });
            widget.append(&button);
            buttons.borrow_mut().push(button);
        }

        Self {
            widget,
            buttons,
            active_ws_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports WorkspaceSwitcher
    use gtk::prelude::*; // For ButtonExt, WidgetExt etc.

    // Helper to get button by label (or index) if needed, and check its CSS classes
    // Not strictly needed for these tests as we can access the buttons Rc directly.
    /*
    fn get_button_by_index(switcher_widget: &GtkBox, index: usize) -> Option<Button> {
        switcher_widget.child() // Get first child
            .and_then(|mut child| {
                for _ in 0..index {
                    child = child.next_sibling()?;
                }
                child.downcast::<Button>().ok()
            })
    }
    */

    #[test]
    fn test_workspace_switcher_creation() {
        gtk::test_init(); // Initialize GTK for testing UI components

        let num_ws = 4;
        let switcher = WorkspaceSwitcher::new(num_ws);
        let buttons_rc_borrow = switcher.buttons.borrow();

        assert_eq!(buttons_rc_borrow.len(), num_ws as usize, "Correct number of buttons should be created.");

        // Check initial active state (first button)
        for (i, btn) in buttons_rc_borrow.iter().enumerate() {
            if i == 0 {
                assert!(btn.has_css_class("active-workspace"), "First button should be active initially.");
            } else {
                assert!(!btn.has_css_class("active-workspace"), "Other buttons should not be active initially.");
            }
            assert_eq!(btn.label().unwrap_or_default(), (i + 1).to_string());
        }
        assert_eq!(*switcher.active_ws_index.borrow(), 0, "Initial active index should be 0.");
    }

    #[test]
    fn test_workspace_switcher_click_behavior() {
        gtk::test_init();

        let num_ws = 3;
        let switcher = WorkspaceSwitcher::new(num_ws);
        // Get direct access to the buttons Vec via the Rc<RefCell<>>
        let buttons_vec_ref = switcher.buttons.borrow();

        // Click the second button (index 1)
        let second_button = buttons_vec_ref.get(1).expect("Second button should exist.");
        second_button.emit_clicked();

        // Check state after clicking second button
        assert_eq!(*switcher.active_ws_index.borrow(), 1, "Active index should be 1 after clicking second button.");
        assert!(!buttons_vec_ref.get(0).unwrap().has_css_class("active-workspace"), "First button should not be active.");
        assert!(buttons_vec_ref.get(1).unwrap().has_css_class("active-workspace"), "Second button should be active.");
        assert!(!buttons_vec_ref.get(2).unwrap().has_css_class("active-workspace"), "Third button should not be active.");

        // Click the third button (index 2)
        let third_button = buttons_vec_ref.get(2).expect("Third button should exist.");
        third_button.emit_clicked();

        assert_eq!(*switcher.active_ws_index.borrow(), 2, "Active index should be 2 after clicking third button.");
        assert!(!buttons_vec_ref.get(0).unwrap().has_css_class("active-workspace"), "First button should not be active.");
        assert!(!buttons_vec_ref.get(1).unwrap().has_css_class("active-workspace"), "Second button should not be active.");
        assert!(buttons_vec_ref.get(2).unwrap().has_css_class("active-workspace"), "Third button should be active.");

        // Click the first button again (index 0)
        let first_button = buttons_vec_ref.get(0).expect("First button should exist.");
        first_button.emit_clicked();

        assert_eq!(*switcher.active_ws_index.borrow(), 0, "Active index should be 0 after clicking first button again.");
        assert!(buttons_vec_ref.get(0).unwrap().has_css_class("active-workspace"), "First button should be active again.");
        assert!(!buttons_vec_ref.get(1).unwrap().has_css_class("active-workspace"), "Second button should not be active.");
        assert!(!buttons_vec_ref.get(2).unwrap().has_css_class("active-workspace"), "Third button should not be active.");
    }
}
