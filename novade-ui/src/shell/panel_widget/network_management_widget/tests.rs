// tests.rs
#![allow(unused_imports)] // Allow unused imports for now, will add more tests

use super::*; // Imports NetworkManagementWidget
use gtk::prelude::*;

// Helper function to initialize GTK if not already initialized by #[gtk::test]
// macro or if running tests that don't use it directly.
fn ensure_gtk_initialized() {
    if !gtk::is_initialized() {
        gtk::init().expect("Failed to initialize GTK for testing.");
    }
}

#[gtk::test]
fn test_network_widget_instantiation_and_default_icon() {
    ensure_gtk_initialized();
    let widget = NetworkManagementWidget::new();

    // Check if the widget is a button
    assert!(widget.is::<gtk::Button>(), "Widget should be a gtk::Button");

    // Access the internal icon Image widget.
    // This requires NetworkManagementWidget::imp() to be accessible or a public getter for the icon.
    // Assuming imp is accessible for testing or there's a way to get the child.
    let button_child = widget.child().expect("Button should have a child (the icon image)");
    let icon_image = button_child.downcast::<gtk::Image>().expect("Child should be a gtk::Image");

    // Check default icon name (based on placeholder in imp.rs)
    // The actual icon name might be set via icon_theme during set_from_icon_name,
    // so we check the "icon-name" property.
    let icon_name_prop = icon_image.property::<Option<String>>("icon-name");
    assert_eq!(
        icon_name_prop.as_deref(), 
        Some("network-wireless-signal-none-symbolic"), 
        "Default icon name is not as expected."
    );
}

#[gtk::test]
fn test_network_widget_popover_creation_and_visibility() {
    ensure_gtk_initialized();
    let widget = NetworkManagementWidget::new();
    
    // Access the popover. This requires a way to get it, e.g., from imp() or a getter.
    // For now, we assume it's created and attached. We'll test its visibility on click.
    // We can't directly access widget.imp().popover here in an external test module easily
    // without specific `pub(crate)` or helper methods in the main widget code.
    // However, we can test the click behavior.

    let popover = widget.imp().popover.borrow(); // Accessing through imp() for test purposes.
    let popover_ref = popover.as_ref().expect("Popover should be initialized");
    
    assert!(!popover_ref.is_visible(), "Popover should initially be hidden.");

    // Simulate a click on the widget
    widget.emit_clicked();
    
    // Wait for GTK events to process (e.g., popover showing)
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }

    assert!(popover_ref.is_visible(), "Popover should be visible after click.");
}

#[gtk::test]
fn test_network_widget_popover_default_content() {
    ensure_gtk_initialized();
    let widget = NetworkManagementWidget::new();

    let popover_cell = widget.imp().popover.borrow();
    let popover = popover_cell.as_ref().expect("Popover should be initialized.");

    // To check content, the popover needs to be built (e.g. by showing it or calling update explicitly)
    // The content is built in update_popover_content_impl, which is called in constructed.
    // So the content should be there.

    let popover_child = popover.child().expect("Popover should have a child (the GtkBox)");
    let vbox = popover_child.downcast::<gtk::Box>().expect("Popover child should be a GtkBox");

    let mut labels_text = Vec::new();
    let mut current_child = vbox.first_child();
    while let Some(child) = current_child {
        if let Some(label) = child.downcast_ref::<gtk::Label>() {
            labels_text.push(label.label().to_string());
        }
        current_child = child.next_sibling();
    }

    // Based on placeholder content in imp.rs:
    // Title: "Available Networks:"
    // Network 1: "WiFi Network 1"
    // Network 2: "Ethernet"
    assert!(labels_text.contains(&"Available Networks:".to_string()), "Popover should contain title.");
    assert!(labels_text.contains(&"WiFi Network 1".to_string()), "Popover should list 'WiFi Network 1'.");
    assert!(labels_text.contains(&"Ethernet".to_string()), "Popover should list 'Ethernet'.");
}

// Test that internal update methods can be called without panic (basic check)
#[gtk::test]
fn test_internal_update_methods_callable() {
    ensure_gtk_initialized();
    let widget = NetworkManagementWidget::new();

    // These methods are private to the imp module but called by the public object.
    // We are calling the public facing methods from `mod.rs` if they existed,
    // or directly the `_impl` methods on `imp` if we had access.
    // Since `NetworkManagementWidget` (the struct in mod.rs) doesn't have public wrappers for these,
    // we test by ensuring the widget construction (which calls them) doesn't panic,
    // and we can simulate a state change that might trigger them if signals were fully connected.
    // For now, just checking they were called during construction is implicitly done by previous tests.

    // We can directly call the imp methods here for more direct testing, as we are in the same crate.
    widget.imp().update_network_icon_impl();
    widget.imp().update_popover_content_impl();

    // No panic means the methods are callable with the current placeholder setup.
    // More detailed tests would require mocking NetworkManagerIntegration.
}
