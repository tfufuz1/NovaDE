use adw::prelude::*;
use adw::{Application, ApplicationWindow, Flap, HeaderBar, MessageDialog, Toast, ToastOverlay};
use adw::Breakpoint;
use gtk::{Box as GtkBox, CssProvider, Label, Orientation, StyleContext, Align, Button, Image as GtkImage};
use gtk::gdk::Display;
use gio;
use glib; // Required for glib::clone!
use tracing;
use std::path::Path;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell; // For shared mutable state
use std::rc::Rc;      // For shared mutable state

mod widgets;
use widgets::basic_widget::BasicWidget;

const APP_ID: &str = "org.novade.UIShellTest";
static CSS_LOAD_SUCCESSFUL: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init(); // Initialize tracing first

    // Load GResources using include_bytes! via OUT_DIR
    // This is a robust way to ensure the gresource file is found.
    // The build script (build.rs) compiles resources.xml into novade_ui.gresource in OUT_DIR.
    // The include_bytes! macro embeds the content of this file directly into the binary.
    // Then, Resource::from_data creates a gio::Resource from these bytes.
    tracing::info!("Attempting to load and register GResource from embedded data...");
    let gresource_bytes = glib::Bytes::from_static(include_bytes!(concat!(env!("OUT_DIR"), "/novade_ui.gresource")));
    let resource = match gio::Resource::from_data(&gresource_bytes) {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Failed to create GResource from data: {}", e);
            return Err(Box::new(e)); // Or handle more gracefully
        }
    };
    gio::resources_register(&resource);
    tracing::info!("GResource 'novade_ui.gresource' loaded and registered from embedded data.");
    
    tracing::info!("NovaDE UI Application starting...");

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_app_weak| {
        // CSS Loading can happen after GResources, especially if CSS might reference gresource paths.
        match load_css() {
            Ok(message) => {
                tracing::info!("{}", message);
                CSS_LOAD_SUCCESSFUL.store(true, Ordering::Relaxed);
            }
            Err(e) => {
                tracing::error!("CSS Loading Error: {}", e);
                CSS_LOAD_SUCCESSFUL.store(false, Ordering::Relaxed);
            }
        }
    });

    app.connect_activate(build_adw_ui);
    app.run();
    Ok(())
}

fn load_css() -> Result<String, Box<dyn Error>> {
    let provider = CssProvider::new();
    let css_path_str = "novade-ui/src/style.css"; 
    let css_path = Path::new(css_path_str);

    if css_path.exists() {
        provider.load_from_path(css_path);
        if let Some(display) = Display::default() {
            StyleContext::add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            Ok(format!("CSS loaded successfully from: {}", css_path.display()))
        } else {
            Err("Could not get default display to add CSS provider.".into())
        }
    } else {
        let alternative_css_path_str = "src/style.css";
        let alternative_css_path = Path::new(alternative_css_path_str);
        if alternative_css_path.exists() {
            provider.load_from_path(alternative_css_path);
             if let Some(display) = Display::default() {
                StyleContext::add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
                Ok(format!("CSS loaded successfully from: {}", alternative_css_path.display()))
            } else {
                Err("Could not get default display to add CSS provider (alternative path).".into())
            }
        } else {
            Err(format!("CSS file not found at {} or {}. Styling will be default.", css_path.display(), alternative_css_path.display()).into())
        }
    }
}

fn build_adw_ui(app: &Application) {
    let toast_overlay = ToastOverlay::new();
    let flap = Flap::new();
    toast_overlay.set_child(Some(&flap));

    if CSS_LOAD_SUCCESSFUL.load(Ordering::Relaxed) {
        toast_overlay.add_toast(Toast::new("CSS loaded successfully!"));
    } else {
        toast_overlay.add_toast(Toast::new("Error loading CSS. Using default styles."));
    }

    let main_content_box = GtkBox::new(Orientation::Vertical, 15); // Adjusted spacing
    main_content_box.set_margin_top(10);
    main_content_box.set_margin_bottom(10);
    main_content_box.set_margin_start(10);
    main_content_box.set_margin_end(10);
    main_content_box.add_css_class("flap-content-box");

    let main_title_label = Label::new(Some("Main Content Area - Signal Handling Demo"));
    main_title_label.set_halign(Align::Center);
    main_content_box.append(&main_title_label);

    // 1. Add a new gtk4::Label to the main UI (wrapped for shared access)
    let status_label = Rc::new(RefCell::new(Label::new(Some("Initial status: Waiting for button press..."))));
    status_label.borrow().set_halign(Align::Center);
    status_label.borrow().set_margin_top(10);
    status_label.borrow().add_css_class("status-label"); // For potential styling
    main_content_box.append(&*status_label.borrow());


    let basic_widget_instance = BasicWidget::new();
    basic_widget_instance.set_label_text("Interactive BasicWidget");
    // Set the main image from a themed icon name
    basic_widget_instance.set_main_image_from_icon_name("document-open-symbolic"); 
    // Set the status image from our GResource
    let resource_path = "/org/novade/ui/icons/my-custom-icon.svg";
    basic_widget_instance.set_status_image_from_resource(resource_path); 
    tracing::info!("Attempting to set status image from GResource: {}", resource_path);
    
    // 2. Clone Rc<RefCell<gtk::Label>> for the callback
    let status_label_clone = status_label.clone();
    // Counter for button clicks
    let click_count = Rc::new(RefCell::new(0));

    basic_widget_instance.connect_button_clicked(move |_widget| {
        let mut count = click_count.borrow_mut();
        *count += 1;
        
        let message = format!("BasicWidget button clicked {} time(s)!", *count);
        tracing::info!("{}", message);
        
        // Update the shared label's text
        status_label_clone.borrow_mut().set_text(&format!("Status: BasicWidget clicked {} time(s).", *count));
        
        // Also, let's show a toast for this interaction
        // Need ToastOverlay here. If it's not easily available, we might skip toast here or pass it.
        // For simplicity, we'll just log and update label.
        // If toast_overlay were needed here, it would also need to be Rc<RefCell<>> or passed.
    });
    basic_widget_instance.set_margin_top(15);
    main_content_box.append(&basic_widget_instance);

    let standalone_image = GtkImage::from_icon_name("face-smile-symbolic");
    standalone_image.set_pixel_size(48);
    standalone_image.set_halign(Align::Center);
    standalone_image.set_margin_top(15);
    let image_title_label = Label::new(Some("Standalone GtkImage:"));
    image_title_label.set_halign(Align::Center);
    main_content_box.append(&image_title_label);
    main_content_box.append(&standalone_image);
    
    flap.set_content(Some(&main_content_box));

    // --- Sidebar (Flap) Content ---
    let sidebar_box = GtkBox::new(Orientation::Vertical, 10);
    sidebar_box.add_css_class("sidebar-box");
    sidebar_box.set_width_request(220);

    let sidebar_label = Label::new(Some("Sidebar Controls"));
    sidebar_label.set_halign(Align::Center);
    sidebar_box.append(&sidebar_label);
    
    let error_button = Button::with_label("Trigger Sample Error Toast");
    // Clone toast_overlay for its callback
    let toast_overlay_clone_for_error_button = toast_overlay.clone();
    error_button.connect_clicked(move |_| {
        toast_overlay_clone_for_error_button.add_toast(Toast::new("This is a sample error toast from sidebar!"));
        tracing::warn!("Sample error toast triggered by sidebar button.");
    });
    sidebar_box.append(&error_button);
    
    flap.set_flap(Some(&sidebar_box));
    flap.set_flap_position(gtk::PackType::Start);

    // --- Breakpoint for Flap ---
    let condition = adw::BreakpointCondition::new_length(adw::BreakpointConditionLengthType::MaxWidth, 600.0, adw::LengthUnit::Px);
    let breakpoint = Breakpoint::new(condition);
    breakpoint.add_setter(&flap, "folded", &true.to_value());
    flap.set_folded(false);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("NovaDE UI - Signal Handling")
        .default_width(850)
        .default_height(700)
        .content(&toast_overlay) // ToastOverlay is the root content now
        .build();

    window.add_breakpoint(breakpoint);

    let header = HeaderBar::new();
    window.set_title_widget(Some(&header));

    let flap_toggle = gtk::ToggleButton::new();
    flap_toggle.set_icon_name("sidebar-show-left-symbolic");
    flap.bind_property("folded", &flap_toggle, "active")
        .bidirectional()
        .build();
    header.pack_start(&flap_toggle);
    
    if false { 
        let dialog = MessageDialog::builder()
            .heading("Critical UI Setup Error").transient_for(&window).modal(true).build();
        dialog.add_response("ok", "OK");
        dialog.connect_response(None, |d, _| d.close());
        dialog.present();
        return;
    }

    window.present();
}
