use super::*; 
use gtk::prelude::*;
use gtk::{Image, Popover, ScrolledWindow, Label};

// Helper function to initialize GTK if not already initialized by #[gtk::test]
fn ensure_gtk_init() {
    if !gtk::is_initialized() {
        gtk::init().expect("Failed to initialize GTK for testing.");
    }
    // Allow events to be processed for async tasks spawned via glib::MainContext to run
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}

/// Test basic instantiation and the initial icon state.
/// This test depends on a running D-Bus session but not necessarily a responsive NetworkManager,
/// as failure to connect to NM should result in a defined error state.
#[gtk::test]
#[ignore] // Depends on D-Bus and NetworkManagerIntegration init behavior
fn test_network_widget_instantiation_and_initial_icon() {
    ensure_gtk_init();
    let widget = NetworkManagementWidget::new();

    assert!(widget.is::<gtk::Button>(), "Widget should be a gtk::Button");

    let icon_image = widget.child().and_then(|child| child.downcast::<Image>())
        .expect("Widget should have an Image child");
    
    // Initial icon set in `constructed` before any async ops
    let icon_name_prop = icon_image.property::<Option<String>>("icon-name");
    assert_eq!(
        icon_name_prop.as_deref(), 
        Some("network-offline-symbolic"), 
        "Initial icon name should be network-offline-symbolic."
    );
    
    // After construction, request_ui_update is called.
    // If NM integration fails, set_error_state should be called, keeping the icon offline.
    // If NM integration succeeds, icon might change based on actual network state.
    // This part is hard to test deterministically without mocking.
    // We primarily test the immediate synchronous state here.
    
    // Process GLib main loop a bit to allow async tasks from `constructed` to run
    for _ in 0..5 { // Iterate a few times
        ensure_gtk_init(); // Process events
        std::thread::sleep(std::time::Duration::from_millis(50)); // Short sleep
    }

    // Re-check icon: it might have changed if NM is active, or stayed offline if NM init failed.
    // This is non-deterministic for a simple test. A full integration test would need a known NM state.
    // For now, we just ensure it doesn't crash and the initial state was correct.
    println!("Icon name after async init: {:?}", icon_image.property::<Option<String>>("icon-name"));
}

/// Test popover creation, visibility on click, and basic structure.
#[gtk::test]
#[ignore] // Depends on D-Bus for NM integration during construction
fn test_network_widget_popover_visibility_and_initial_structure() {
    ensure_gtk_init();
    let widget = NetworkManagementWidget::new();
    
    let popover = widget.imp().popover.borrow();
    let popover_ref = popover.as_ref().expect("Popover should be initialized in widget's imp struct");
    
    assert!(!popover_ref.is_visible(), "Popover should initially be hidden.");

    // Simulate a click on the widget. This should also trigger request_ui_update.
    widget.emit_clicked();
    
    // Wait for GTK events to process (e.g., popover showing, async UI updates)
    for _ in 0..5 {
        ensure_gtk_init();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    assert!(popover_ref.is_visible(), "Popover should be visible after click.");

    // Check if the popover has a ScrolledWindow as its child,
    // which indicates update_popover_content_impl has at least partially run.
    let popover_child = popover_ref.child();
    assert!(popover_child.is_some(), "Popover should have a child after UI update.");
    assert!(popover_child.unwrap().is::<ScrolledWindow>(), "Popover child should be a ScrolledWindow.");

    // Further checks could inspect the ScrolledWindow's child (the GtkBox)
    // but specific content depends on live network data or NM init state (error or success).
    // If NM init failed, it should contain an error message.
    if let Some(scrolled_window) = popover_ref.child().and_then(|c| c.downcast::<ScrolledWindow>()) {
        if let Some(vbox) = scrolled_window.child().and_then(|c| c.downcast::<gtk::Box>()) {
            if let Some(first_label) = vbox.first_child().and_then(|c| c.downcast::<Label>()) {
                 println!("Popover first label content: {}", first_label.label());
                 // If NM init failed, this might be "Error:" or "NetworkManager not available."
                 // This is still system-dependent.
            }
        }
    }
}

/// Test that calling `request_ui_update` attempts to run the async UI updates.
/// This test checks if the widget correctly handles the case where NetworkManagerIntegration
/// might fail to initialize (e.g., D-Bus not available).
#[gtk::test]
#[ignore] // This test's outcome heavily depends on the D-Bus environment.
fn test_request_ui_update_handling() {
    ensure_gtk_init();
    let widget = NetworkManagementWidget::new();

    // Manually call request_ui_update (it's also called in `constructed` and on click)
    widget.imp().request_ui_update(widget.clone());

    // Process GLib main loop to allow async tasks to run
    for _ in 0..10 { // Iterate a few times to allow async tasks to proceed
        ensure_gtk_init();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // Check the icon state. If NM init failed, it should be 'network-offline-symbolic'.
    // If NM is running, it will be something else. This part is environment-dependent.
    let icon_image = widget.child().and_then(|child| child.downcast::<Image>())
        .expect("Widget should have an Image child");
    let current_icon_name = icon_image.property::<Option<String>>("icon-name");
    println!("Icon name after request_ui_update: {:?}", current_icon_name);

    // Check popover content for error message if NM likely failed.
    // This is a heuristic. A more robust test would mock NM.
    let popover = widget.imp().popover.borrow();
    let popover_ref = popover.as_ref().expect("Popover should be initialized.");
    if let Some(scrolled_window) = popover_ref.child().and_then(|c| c.downcast::<ScrolledWindow>()) {
        if let Some(vbox) = scrolled_window.child().and_then(|c| c.downcast::<gtk::Box>()) {
            if let Some(first_label) = vbox.first_child().and_then(|c| c.downcast::<Label>()) {
                let label_text = first_label.label().to_string();
                println!("Popover first label after request_ui_update: {}", label_text);
                // If `NetworkManagerIntegration::new()` failed, `set_error_state` would be called.
                // This might set the first label to "Error:" or "NetworkManager not available."
                // This assertion is highly dependent on the error messages in imp.rs
                // and whether NM is actually running or not.
                // For a system without D-Bus/NM, we'd expect an error state.
                // assert!(label_text.starts_with("Error:") || label_text.contains("NetworkManager not available"));
            }
        }
    }
    // This test primarily ensures that calling request_ui_update doesn't panic and attempts
    // the UI update flow. Verifying the outcome precisely requires mocking or a controlled NM environment.
}
