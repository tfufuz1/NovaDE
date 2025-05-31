// nova_shell/src/application.rs
use gtk4::{self as gtk, prelude::*};
// Use gtk's re-export of glib to ensure version compatibility
use gtk4::glib;
use tracing::info;
use crate::panel; // Import the panel module

const APP_ID: &str = "org.novade.Shell";

pub fn build_ui(app: &gtk::Application) {
    info!("NovaDE Shell UI building...");

    let main_vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // Create and add the panel
    let top_panel = panel::create_panel_widget();
    main_vbox.append(&top_panel); // Panel doesn't expand by default with append

    // Create a content area that expands
    let content_area = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true) // Make this area expand
        .hexpand(true)
        .build();

    let content_label = gtk::Label::new(Some("Desktop Content Area"));
    content_label.set_vexpand(true); // Make label expand within its area
    content_label.set_halign(gtk::Align::Center);
    content_label.set_valign(gtk::Align::Center);
    content_area.append(&content_label);

    main_vbox.append(&content_area); // Content area expands

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("NovaDE Shell")
        .default_width(1024) // Increased default size
        .default_height(768)
        .child(&main_vbox) // Set the main_vbox as the child of the window
        .build();

    window.present();
    info!("Main window with panel structure presented.");
}

// Changed return type to gtk::glib::ExitCode
pub fn launch_shell_application() -> gtk::glib::ExitCode {
    info!("Launching NovaDE Shell Application...");
    let app = gtk::Application::new(Some(APP_ID), Default::default());

    app.connect_activate(|app| {
        build_ui(app);
    });

    // Run the application
    let exit_code = app.run();
    info!("NovaDE Shell Application finished with exit code: {:?}", exit_code);
    exit_code
}
