// nova_shell/src/panel.rs
use gtk4::{self as gtk, prelude::*};
use tracing::info;

pub fn create_panel_widget() -> gtk::Box {
    info!("Creating Top Panel widget");

    let panel_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .height_request(36)
        .margin_start(5)
        .margin_end(5)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    // Left-aligned items
    let workspace_indicator = gtk::Label::new(Some("WS: 1"));
    panel_box.append(&workspace_indicator);

    // Center-ish item (will take up available space)
    let app_menu_placeholder = gtk::Label::new(Some("AppMenu"));
    app_menu_placeholder.set_hexpand(true); // Make this widget expand
    // To truly center text within the label if it expands:
    app_menu_placeholder.set_halign(gtk::Align::Center);
    panel_box.append(&app_menu_placeholder);

    // Right-aligned items
    // These will appear to the right of the expanding app_menu_placeholder
    let systray_placeholder = gtk::Label::new(Some("[Tray]"));
    panel_box.append(&systray_placeholder);

    let clock_label = gtk::Label::new(Some("12:00 PM"));
    panel_box.append(&clock_label);

    panel_box.add_css_class("nova-panel");

    info!("Top Panel widget created with placeholders.");
    panel_box
}
