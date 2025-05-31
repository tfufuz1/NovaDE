use adw::prelude::*;
use adw::{Application, ApplicationWindow, Flap, HeaderBar, MessageDialog, Toast, ToastOverlay};
use adw::Breakpoint;
use gtk4_layer_shell::LayerShell;
use gtk::{Box as GtkBox, CssProvider, Label, Orientation, StyleContext, Align, Button, Image as GtkImage};
use gtk::gdk::Display;
use gio;
use glib; // Required for glib::clone!
use tracing;
use std::path::Path;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell; // For shared mutable state
use std::rc::Rc;

mod widgets;
use widgets::basic_widget::BasicWidget;
use tokio::sync::{mpsc, oneshot};

// Internationalization
use gettextrs::{gettext, LocaleCategory};

// Declare and use ui_state module (now a GObject)
mod ui_state;
use ui_state::UIState;

// Declare components module and import SimpleTaskbar
mod components;
use components::simple_taskbar::SimpleTaskbar;

// Declare dbus_utils module
mod dbus_utils;

// Declare settings_ui module
mod settings_ui;
use settings_ui::NovaSettingsWindow;

mod wayland_integration;

const APP_ID: &str = "org.novade.UIShellTest";
static CSS_LOAD_SUCCESSFUL: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup localization
    // Set the locale from the environment
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    // Specify the location of the .mo files (translations)
    // For development, this could be a local "./po" or "./locales" directory.
    // For installed applications, this is often /usr/share/locale or similar.
    gettextrs::bindtextdomain("novade-ui", "./po")?; // Using "?" to propagate potential errors
    // Specify the text domain for future gettext calls
    gettextrs::textdomain("novade-ui")?;
    // To test if it's working, you might force a language if messages are compiled:
    // std::env::set_var("LANGUAGE", "de_DE.UTF-8"); // Example for testing German

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

    // Define a simple event type for simulation
    #[derive(Debug, Clone, Copy)]
    enum DomainEvent {
        WindowAdded,
        WindowRemoved,
        OtherEvent, // Example of an event that might not directly affect window_count
    }

    // 1. Set up Tokio MPSC channel
    let (domain_event_sender, mut domain_event_receiver) = mpsc::channel::<DomainEvent>(32);

    let app = Application::builder().application_id(APP_ID).build();

    // The UIState GObject will be created within build_adw_ui.
    // We need to pass the receiver to build_adw_ui.
    // To do this, we can't directly use `app.connect_activate(build_adw_ui);`
    // We need to use a closure that captures the receiver.
    // However, `domain_event_receiver` needs to be 'static for spawn_future_local if not careful.
    // A common way is to pass it into the closure and then move it into the spawned future.
    // Or, if `build_adw_ui` is only called once, we can make it `Option<Receiver>` and `take()` it.

    app.connect_activate(move |app_instance| {
        // Take the receiver. This closure is called once.
        // If connect_activate could be called multiple times for an app instance (it's not typical),
        // this would need a more robust way to handle receiver ownership, e.g. Rc<RefCell<Option<Receiver>>>
        // or ensure build_adw_ui is idempotent regarding the receiver.
        // For simplicity, assume it's fine to move `domain_event_receiver` here.
        // Actually, domain_event_receiver might not be Send, so it must stay on the main thread.
        // glib::spawn_future_local will keep it on the main thread.
        // We pass the receiver by moving it into this closure, then into build_adw_ui.
        build_adw_ui(app_instance, domain_event_receiver, domain_event_sender.clone());
        // Clear the receiver from this scope to ensure it's used only once if needed,
        // but since we moved it, it's fine.
        // domain_event_receiver = tokio::sync::mpsc::channel(1).1; // Effectively drop and replace if needed for multiple calls
    });
    
    // Note: The original `domain_event_receiver` is moved into the closure above.
    // The `domain_event_sender` is cloned for `build_adw_ui` to create simulation buttons.
    // If other parts of `main` needed to send events after this, they'd use another clone of `domain_event_sender`.

    app.connect_startup(|_app_weak| {
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

    // app.run() is below. We don't need to change it.

    // Initialize Wayland Toplevel Integration
    match wayland_integration::WaylandToplevelIntegration::new_and_start_thread() {
        Ok(_integration_handle) => { // Store the handle if needed (e.g. to join thread on app exit)
            tracing::info!("Wayland Toplevel Integration initialized and event loop started.");
            // The Arc<Mutex<Vec<ToplevelInfo>>> is in integration_handle.toplevels_data_access
            // The UI would need to periodically check this or use a glib channel for updates.
            // For this subtask, we are just starting it and logging.
            // To prevent the handle from being dropped immediately if not stored:
            // std::mem::forget(integration_handle); // Or store it in the App state.
            // For now, let it be dropped as the thread is detached (thread::spawn).
        }
        Err(e) => {
            tracing::error!("Failed to initialize Wayland Toplevel Integration: {:?}", e);
            // Decide if this is fatal. For now, just log and continue.
        }
    }

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

fn build_adw_ui(
    app: &Application,
    mut domain_event_receiver: mpsc::Receiver<DomainEvent>, // Take ownership of the receiver
    domain_event_sender: mpsc::Sender<DomainEvent> // Clone of sender for simulation buttons
) {
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

    let main_title_label = Label::new(Some("Main Content Area - Reactive Bindings Demo"));
    main_title_label.set_halign(Align::Center);
    main_content_box.append(&main_title_label);

    // Instantiate the UIState GObject
    let ui_state_instance = UIState::new();
    ui_state_instance.set_other_data("Initial data for UIState GObject"); // Example of using other methods

    // Create a label for window_count
    let window_count_label = Label::new(None);
    window_count_label.set_halign(Align::Center);
    window_count_label.set_margin_top(10);
    main_content_box.append(&window_count_label);

    // Create PropertyExpression for binding: "Window Count: X"
    let count_expression = ui_state_instance.property_expression("window-count");
    
    // Chain a closure to transform the u32 count to a formatted String.
    // The closure takes Option<T> where T is the type of the source property.
    // It must return an Option<U> where U is the type expected by the target property (String for label).
    let formatted_string_expression = count_expression.chain_closure(glib::closure_local!(|value: Option<u32>| {
        Some(format!("Window Count: {}", value.unwrap_or(0)))
    }));

    // Bind the expression to the label's "label" property.
    formatted_string_expression.bind(&window_count_label, "label")
        .sync_create();


    // Button to increment window_count in UIState
    let increment_button = Button::with_label(&gettext("Increment Window Count")); 
    increment_button.set_halign(Align::Center);
    let ui_state_clone_for_button = ui_state_instance.clone(); 
    increment_button.connect_clicked(move |_| {
        let current_count = ui_state_clone_for_button.window_count();
        ui_state_clone_for_button.set_window_count(current_count + 1);
        tracing::info!("Increment button clicked. New window_count: {}", ui_state_clone_for_button.window_count());
    });
    main_content_box.append(&increment_button);

    // Separator before event simulation buttons
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    let event_sim_label = Label::new(Some(&gettext("Simulate Domain Events:"))); 
    event_sim_label.set_halign(Align::Center);
    event_sim_label.set_margin_top(10);
    main_content_box.append(&event_sim_label);

    let sim_buttons_box = GtkBox::new(Orientation::Horizontal, 6);
    sim_buttons_box.set_halign(Align::Center);

    let add_event_button = Button::with_label(&gettext("Send WindowAdded Event")); 
    let sender_clone_add = domain_event_sender.clone();
    add_event_button.connect_clicked(move |_| {
        let sender = sender_clone_add.clone();
        glib::spawn_future_local(async move {
            if let Err(e) = sender.send(DomainEvent::WindowAdded).await {
                tracing::error!("Failed to send WindowAdded event: {}", e);
            } else {
                tracing::info!("Simulated WindowAdded event sent.");
            }
        });
    });
    sim_buttons_box.append(&add_event_button);

    let remove_event_button = Button::with_label(&gettext("Send WindowRemoved Event")); 
    let sender_clone_remove = domain_event_sender.clone(); 
    remove_event_button.connect_clicked(move |_| {
        let sender = sender_clone_remove.clone();
        glib::spawn_future_local(async move {
            if let Err(e) = sender.send(DomainEvent::WindowRemoved).await {
                tracing::error!("Failed to send WindowRemoved event: {}", e);
            } else {
                tracing::info!("Simulated WindowRemoved event sent.");
            }
        });
    });
    sim_buttons_box.append(&remove_event_button);
    main_content_box.append(&sim_buttons_box);

    // Separator for Long Task Simulation
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    let long_task_title_label = Label::new(Some(&gettext("Async Task Simulation:"))); 
    long_task_title_label.set_halign(Align::Center);
    long_task_title_label.set_margin_top(10);
    main_content_box.append(&long_task_title_label);

    let long_task_status_label = Label::new(Some(&gettext("Status: Idle"))); 
    long_task_status_label.set_halign(Align::Center);
    long_task_status_label.set_margin_top(5);
    main_content_box.append(&long_task_status_label);

    let long_task_button = Button::with_label(&gettext("Simulate Long Task & Update UI")); 
    long_task_button.set_halign(Align::Center);
    let long_task_status_label_clone = long_task_status_label.clone(); 
    long_task_button.connect_clicked(move |_| {
        let label_for_update = long_task_status_label_clone.clone();
        
        // --- UI-Thread Optimization Example ---
        // This entire block demonstrates best practices for offloading work from the UI thread
        // to prevent UI freezes, and then safely updating the UI with the result.

        // 1. `glib::spawn_future_local`:
        //    Schedules this whole async block to run on the main GTK UI thread (GLib main context).
        //    This is crucial because all direct UI widget manipulations *must* happen on this thread.
        //    It allows using `await` within this block without blocking the entire UI, as it yields
        //    control back to the main loop while waiting for futures.
        glib::spawn_future_local(async move {
            // Initial UI update: Inform the user that the task has started.
            // This is safe as we are on the main thread here.
            label_for_update.set_text(&gettext("Long task started... (will take 3s)")); 
            tracing::info!("Long task button clicked. Starting simulation.");

            // 2. `tokio::sync::oneshot::channel`:
            //    Creates a single-use channel to communicate a result from a background task (Tokio task)
            //    back to this UI-thread task. `tx` (sender) is moved to the background task,
            //    and `rx` (receiver) is awaited here.
            let (tx, rx) = oneshot::channel();

            // 3. `tokio::spawn`:
            //    Offloads the actual potentially blocking work (here, `tokio::time::sleep`)
            //    to a separate thread managed by Tokio's multi-thread runtime.
            //    This is the key to preventing the UI thread from freezing.
            //    If `std::thread::sleep` or any synchronous I/O (file reading, network request)
            //    or heavy computation were done directly in the `connect_clicked` handler
            //    (even within `glib::spawn_future_local` without `tokio::spawn` for the blocking part),
            //    the UI would become unresponsive for the duration of that operation.
            tokio::spawn(async move { // This async block runs on a Tokio worker thread.
                tracing::info!("Long task executing on Tokio worker thread...");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Simulate non-CPU-intensive I/O-bound work
                
                // The result of the background task. This could be any data that needs to be sent back.
                // If it's a translatable string, it should be `gettext`ed here or before sending.
                // For this example, it's a dynamic result string.
                let result = "Long task complete! Result from background.";
                tracing::info!("Long task finished on Tokio worker. Sending result back to UI thread.");
                
                // Send the result back to the main thread task via the oneshot channel's sender (`tx`).
                // If `tx.send` fails, it typically means the `rx` (receiver) on the main thread task
                // was dropped, which might happen if the UI/widget waiting for the result was destroyed.
                if tx.send(result.to_string()).is_err() {
                    tracing::error!("Failed to send result from long task: oneshot receiver was dropped.");
                }
            });

            // 4. Awaiting the result and updating UI:
            //    `rx.await` suspends execution of *this specific glib::spawn_future_local task*,
            //    allowing other GTK events and UI updates to be processed on the main thread.
            //    It does NOT block the entire main UI thread thanks to `glib::spawn_future_local`'s
            //    integration with the GLib main event loop.
            match rx.await {
                Ok(result) => {
                    // Once the result is received (i.e., the background task has sent it),
                    // update the UI label. This is safe because this part of the code
                    // (after `rx.await`) is still running within the `glib::spawn_future_local`
                    // context, hence on the main GTK thread.
                    label_for_update.set_text(&result); 
                    tracing::info!("UI updated with long task result: {}", result);
                }
                Err(e) => {
                    // Handle potential error if the sender (background task) fails or the channel closes prematurely.
                    let error_msg = format!("{}: {}", gettext("Error receiving result from long task"), e); 
                    label_for_update.set_text(&error_msg);
                    tracing::error!("{}", error_msg);
                }
            }
        });
    });
    main_content_box.append(&long_task_button);

    // Separator for D-Bus Notification Test
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    let dbus_title_label = Label::new(Some(&gettext("D-Bus Desktop Notification Test:"))); 
    dbus_title_label.set_halign(Align::Center);
    dbus_title_label.set_margin_top(10);
    main_content_box.append(&dbus_title_label);

    let send_notification_button = Button::with_label(&gettext("Send Test Notification")); 
    send_notification_button.set_halign(Align::Center);
    // For this simple call, we are using static strings or direct values.
    send_notification_button.connect_clicked(move |_| {
        glib::spawn_future_local(async move {
            tracing::info!("'Send Test Notification' button clicked.");
            match dbus_utils::send_desktop_notification(
                "NovaDE-UI", 
                &gettext("Test Notification"), 
                &gettext("This is a test notification sent from the NovaDE UI application via zbus."), 
                "dialog-information-symbolic", 
                5000 
            ).await {
                Ok(id) => {
                    tracing::info!("Desktop notification successfully sent with ID: {}", id);
                }
                Err(e) => {
                    tracing::error!("Failed to send desktop notification: {}", e);
                }
            }
        });
    });
    main_content_box.append(&send_notification_button);

    // Separator for XDG Portal File Chooser Test
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    let portal_title_label = Label::new(Some(&gettext("XDG Portal File Chooser Test:"))); 
    portal_title_label.set_halign(Align::Center);
    portal_title_label.set_margin_top(10);
    main_content_box.append(&portal_title_label);

    let open_file_portal_button = Button::with_label(&gettext("Open File (Portal)")); 
    open_file_portal_button.set_halign(Align::Center);
    // We get the window when `build_adw_ui` is called, but the ApplicationWindow
    // is typically built towards the end of this function.
    // We can connect the signal *after* the window is built, or pass the window via Rc<RefCell<Option<Window>>>
    // or use glib::clone! with @strong_allow_none if we are sure window will exist.
    // For simplicity, let's connect it here, but the `window` variable is not yet defined.
    // This means we need to defer the connection or ensure `window` is available.
    // A common pattern: define button, add to layout, then later connect signals that need `window`.
    // Or, pass the application `app` and get the active window from it if available.
    // Let's assume we will connect it after `window` is built.
    // For now, this is a placeholder for where the button is added.
    // The actual connect_clicked will be done after `window` is available.
    main_content_box.append(&open_file_portal_button);


    // Separator after XDG Portal test
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    
    // Existing status label (from previous task, for BasicWidget interaction)
    let status_label = Rc::new(RefCell::new(Label::new(Some(&gettext("BasicWidget status: Waiting..."))))); 
    status_label.borrow().set_halign(Align::Center);
    status_label.borrow().set_margin_top(10);
    status_label.borrow().add_css_class("status-label");
    main_content_box.append(&*status_label.borrow());

    let basic_widget_instance = BasicWidget::new();
    basic_widget_instance.set_label_text(&gettext("Interactive BasicWidget")); 
    basic_widget_instance.set_main_image_from_icon_name("document-open-symbolic"); 
    let resource_path = "/org/novade/ui/icons/my-custom-icon.svg";
    basic_widget_instance.set_status_image_from_resource(resource_path); 
    tracing::info!("Attempting to set status image from GResource: {}", resource_path);
    
    let status_label_clone = status_label.clone();
    let click_count = Rc::new(RefCell::new(0));

    basic_widget_instance.connect_button_clicked(move |_widget| {
        let mut count = click_count.borrow_mut();
        *count += 1;
        let message = format!("BasicWidget button clicked {} time(s)!", *count);
        tracing::info!("{}", message);
        status_label_clone.borrow_mut().set_text(&format!("BasicWidget Status: Clicked {} time(s).", *count));
        // Also, let's show a toast for this interaction
        // Need ToastOverlay here. If it's not easily available, we might skip toast here or pass it.
        // For simplicity, we'll just log and update label.
    });
    basic_widget_instance.set_margin_top(15); // Adjusted margin
    main_content_box.append(&basic_widget_instance);

    let standalone_image = GtkImage::from_icon_name("face-smile-symbolic");
    standalone_image.set_pixel_size(48);
    standalone_image.set_halign(Align::Center);
    standalone_image.set_margin_top(15);
    let image_title_label = Label::new(Some(&gettext("Standalone GtkImage:"))); 
    image_title_label.set_halign(Align::Center);
    main_content_box.append(&image_title_label);
    main_content_box.append(&standalone_image);
    
    flap.set_content(Some(&main_content_box));

    // Spawn the task to listen for domain events (as before)
    let ui_state_clone_for_receiver = ui_state_instance.clone();
    glib::spawn_future_local(async move {
        // ... (event listener logic remains the same) ...
        tracing::info!("Event listener task started on UI thread.");
        while let Some(event) = domain_event_receiver.recv().await {
            tracing::info!("Received domain event: {:?}", event);
            match event {
                DomainEvent::WindowAdded => {
                    let current_count = ui_state_clone_for_receiver.window_count();
                    ui_state_clone_for_receiver.set_window_count(current_count + 1);
                }
                DomainEvent::WindowRemoved => {
                    let current_count = ui_state_clone_for_receiver.window_count();
                    if current_count > 0 {
                        ui_state_clone_for_receiver.set_window_count(current_count - 1);
                    }
                }
                DomainEvent::OtherEvent => {}
            }
        }
        tracing::info!("Event listener task finished.");
    });

    // --- Sidebar (Flap) Content (as before) ---
    let sidebar_box = GtkBox::new(Orientation::Vertical, 10);
    // ... (sidebar content setup remains the same) ...
    sidebar_box.add_css_class("sidebar-box");
    // sidebar_box.set_width_request(220); // Width might be better controlled by flap itself or content
    let sidebar_label = Label::new(Some(&gettext("Sidebar Controls"))); 
    sidebar_label.set_halign(Align::Center);
    sidebar_box.append(&sidebar_label);
    let error_button = Button::with_label(&gettext("Trigger Sample Error Toast")); 
    let toast_overlay_clone_for_error_button = toast_overlay.clone();
    error_button.connect_clicked(move |_| {
        toast_overlay_clone_for_error_button.add_toast(Toast::new(&gettext("This is a sample error toast from sidebar!"))); 
    });
    sidebar_box.append(&error_button);
    flap.set_flap(Some(&sidebar_box));
    flap.set_flap_position(gtk::PackType::Start);

    // --- Breakpoint for Flap (as before) ---
    let condition = adw::BreakpointCondition::new_length(adw::BreakpointConditionLengthType::MaxWidth, 600.0, adw::LengthUnit::Px);
    let breakpoint = Breakpoint::new(condition);
    breakpoint.add_setter(&flap, "folded", &true.to_value());
    flap.set_folded(false);


    // --- ToolbarView Setup ---
    let toolbar_view = adw::ToolbarView::new();
    
    // Create and add HeaderBar as top bar
    let header = HeaderBar::new();
    let flap_toggle = gtk::ToggleButton::new(); // Moved flap_toggle here
    flap_toggle.set_icon_name("sidebar-show-left-symbolic");
    flap.bind_property("folded", &flap_toggle, "active")
        .bidirectional()
        .build();
    header.pack_start(&flap_toggle);
    // Add other header content if needed, e.g., window title
    // let window_title_adw = adw::WindowTitle::new("NovaDE UI - Taskbar", "");
    // header.set_title_widget(Some(&window_title_adw));
    toolbar_view.add_top_bar(&header);

    // Set existing content (ToastOverlay with Flap) as the main content of ToolbarView
    toolbar_view.set_content(Some(&toast_overlay));

    // Create and add SimpleTaskbar as bottom bar
    let taskbar = SimpleTaskbar::new();
    taskbar.set_clock_text("00:00:00"); // Set initial static text
    toolbar_view.add_bottom_bar(&taskbar);


    // --- Window Setup ---
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(850)
        .default_height(900) // Adjusted for new button
        .content(&toolbar_view) 
        .build();

    // Stress Test Button
    // main_content_box.append(&gtk::Separator::new(Orientation::Horizontal)); // Visually busy, consider removing some separators
    let stress_test_label = Label::new(Some(&gettext("UI Responsiveness Stress Test:")));
    stress_test_label.set_halign(Align::Center);
    stress_test_label.set_margin_top(10);
    main_content_box.append(&stress_test_label);

    let stress_test_button = Button::with_label(&gettext("Start Stress Test (Rapid State Changes)"));
    stress_test_button.set_halign(Align::Center);
    let stress_sender_clone = domain_event_sender.clone(); // Clone sender for the button
    stress_test_button.connect_clicked(move |_| {
        let sender = stress_sender_clone.clone();
        tracing::info!("Stress Test button clicked. Starting rapid event simulation...");

        // Offload the event sending loop to a Tokio worker thread
        // to avoid blocking the UI thread while sending many messages.
        tokio::spawn(async move {
            let num_events_to_send = 1000; // Send 1000 add and 1000 remove events
            for i in 0..num_events_to_send {
                if sender.send(DomainEvent::WindowAdded).await.is_err() {
                    tracing::error!("StressTest: Failed to send WindowAdded event {}.", i);
                    break; // Stop if channel is closed (e.g., app shutting down)
                }
                // Optional: Small delay to allow UI thread some breathing room,
                // though with a buffered channel, this might not be strictly necessary
                // for the sender not to block. The receiver on the main thread will process
                // these as fast as it can.
                // tokio::time::sleep(std::time::Duration::from_micros(50)).await; 

                if sender.send(DomainEvent::WindowRemoved).await.is_err() {
                    tracing::error!("StressTest: Failed to send WindowRemoved event {}.", i);
                    break;
                }
                // if i % 100 == 0 { // Log progress
                //    tracing::debug!("StressTest: Sent {} add/remove event pairs.", i);
                // }
            }
            tracing::info!("Stress Test: Finished sending {} event pairs.", num_events_to_send);
        });
    });
    main_content_box.append(&stress_test_button);


    // Add AboutDialog and MessageDialog test buttons (as before)
    main_content_box.append(&gtk::Separator::new(Orientation::Horizontal));
    let dialogs_title_label = Label::new(Some(&gettext("Standard Dialogs Test (Adwaita):"))); 
    dialogs_title_label.set_halign(Align::Center);
    dialogs_title_label.set_margin_top(10);
    main_content_box.append(&dialogs_title_label);
    let dialog_buttons_box = GtkBox::new(Orientation::Horizontal, 6);
    dialog_buttons_box.set_halign(Align::Center);

    // About Dialog Button
    let about_button = Button::with_label(&gettext("About NovaDE")); 
    let window_clone_for_about = window.clone();
    about_button.connect_clicked(move |_| {
        let parent_window = &window_clone_for_about;
        let about_dialog = adw::AboutWindow::builder()
            .transient_for(parent_window)
            .modal(true)
            .application_name(&gettext("NovaDE")) 
            .version("0.1.0-dev") // Version typically not translated
            .developer_name(&gettext("NovaDE Development Team")) 
            .license_type(gtk::License::MitX11) // License type usually not translated
            .website("https://github.com/systeminmation/NovaDE") // URLs not translated
            .application_icon("application-x-executable-symbolic") // Icon names not translated
            .comments(&gettext("A DEveloper-first Desktop Environment.")) 
            .developers(vec![&gettext("Jules (AI Agent)"), &gettext("And You!")]) 
            .designers(vec![&gettext("Inspired by many")]) 
            .artists(vec![&gettext("Various icon artists")]) 
            .issue_url("https://github.com/systeminmation/NovaDE/issues") // URLs not translated
            .build();
        about_dialog.present();
    });
    dialog_buttons_box.append(&about_button);

    // Message Dialog Button
    let message_button = Button::with_label(&gettext("Show Message Dialog")); 
    let window_clone_for_message = window.clone();
    message_button.connect_clicked(move |_| {
        let parent_window = &window_clone_for_message;
        let message_dialog = adw::MessageDialog::builder()
            .transient_for(parent_window)
            .modal(true)
            .heading(&gettext("A Friendly Message")) 
            .body(&gettext("This is an example of an Adwaita Message Dialog shown from NovaDE.")) 
            .build();
        message_dialog.add_response("ok", &gettext("Got it!")); 
        message_dialog.set_default_response(Some("ok"));
        message_dialog.set_close_response("ok");
        message_dialog.connect_response(None, |dialog, response| {
            tracing::info!("MessageDialog response: {}", response);
            dialog.destroy();
        });
        message_dialog.present();
    });
    dialog_buttons_box.append(&message_button);
    main_content_box.append(&dialog_buttons_box);

    // Settings Window Button
    let settings_button = Button::with_label(&gettext("Open Settings")); 
    let window_clone_for_settings = window.clone();
    settings_button.connect_clicked(move |_| {
        let settings_window = NovaSettingsWindow::new(&window_clone_for_settings);
        settings_window.present();
    });
    let settings_button_box = GtkBox::new(Orientation::Horizontal, 0); // Centering box
    settings_button_box.set_halign(Align::Center);
    settings_button_box.set_margin_top(10);
    settings_button_box.append(&settings_button);
    main_content_box.append(settings_button_box);


    // Now that `window` is available, connect the portal button's clicked signal
    let window_clone_for_portal = window.clone(); // Clone for the closure
    open_file_portal_button.connect_clicked(move |_| {
        let main_window = window_clone_for_portal.clone();
        glib::spawn_future_local(async move {
            tracing::info!("'Open File (Portal)' button clicked.");
            // Pass the main window as parent
            match dbus_utils::open_file_chooser(Some(&main_window)).await {
                Ok(Some(paths)) => {
                    tracing::info!("File(s) selected via portal: {:?}", paths);
                    if let Some(first_path) = paths.first() {
                        // For demonstration, show a toast with the first selected path
                        // Requires access to toast_overlay, or pass it around.
                        // For simplicity, just log.
                        tracing::info!("First selected path: {}", first_path.display());
                    }
                }
                Ok(None) => {
                    tracing::info!("File selection cancelled by user (portal).");
                }
                Err(e) => {
                    tracing::error!("Error using file chooser portal: {}", e);
                }
            }
        });
    });

    window.add_breakpoint(breakpoint); // Breakpoint still applies to the window

    // Initialize gtk4-layer-shell for the main window
    tracing::info!("Initializing layer shell for the main application window...");
    gtk4_layer_shell::LayerShell::init_layer_shell(&window);

    // Set the layer to Bottom (typical for a panel/taskbar)
    // Other options: Layer::Top, Layer::Overlay, Layer::Background
    gtk4_layer_shell::LayerShell::set_layer(&window, gtk4_layer_shell::Layer::Bottom);
    tracing::info!("Layer shell layer set to Bottom.");

    // Set anchors: Stick to left, right, and bottom. Not top.
    gtk4_layer_shell::LayerShell::set_anchor(&window, gtk4_layer_shell::Edge::Left, true);
    gtk4_layer_shell::LayerShell::set_anchor(&window, gtk4_layer_shell::Edge::Right, true);
    gtk4_layer_shell::LayerShell::set_anchor(&window, gtk4_layer_shell::Edge::Bottom, true);
    gtk4_layer_shell::LayerShell::set_anchor(&window, gtk4_layer_shell::Edge::Top, false);
    tracing::info!("Layer shell anchors set (Left, Right, Bottom: true; Top: false).");

    // Enable auto exclusive zone to reserve space (e.g., for a panel)
    // The window's height will determine the zone size if not set explicitly.
    // If the taskbar (bottom bar of ToolbarView) has a fixed height,
    // that should ideally be the exclusive zone.
    // For now, let's set an explicit zone of 48px as a common taskbar height.
    // The ApplicationWindow itself will still try to take default_height (900px),
    // but the layer shell manager might override or clamp this.
    // We might need to adjust window.set_default_height() if we want the window itself
    // to be only 48px high.
    // window.set_default_height(48); // If the window itself should be the panel height
    gtk4_layer_shell::LayerShell::auto_exclusive_zone_enabled(&window); // Let layer shell manage it based on window size
    // OR, for explicit size:
    // gtk4_layer_shell::LayerShell::set_exclusive_zone(&window, 48);
    // Using auto_exclusive_zone_enabled is often simpler if the window's requested height is correct for the panel.
    // If the ToolbarView's bottom bar (taskbar) is, say, 48px, and the rest of the window is transparent
    // or not there, this works. But ApplicationWindow usually has a background.
    // Let's assume the taskbar's height is what we want for the exclusive zone.
    // A more robust way would be to get the taskbar's allocated height after realization
    // and set the exclusive zone then, but for init, a fixed value is common.
    gtk4_layer_shell::LayerShell::set_exclusive_zone(&window, 48); // Explicitly request 48px for the zone
    tracing::info!("Layer shell exclusive zone set to 48px (or auto-enabled).");

    // Simulated critical UI error (kept for illustration, as before)
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
