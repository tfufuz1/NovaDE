use gtk::glib;
use gtk::{prelude::*, Box as GtkBox, Label, Button, Entry};
use gio::{ListStore, prelude::ListStoreExtManual};
use crate::toplevel_gobject::ToplevelListItemGObject;
use crate::components::workspace_switcher::WorkspaceSwitcher; // Import WorkspaceSwitcher
use chrono::Local;
use adw::MessageDialog;

// Define the GObject wrapper for our widget.
glib::wrapper! {
    pub struct SimpleTaskbar(ObjectSubclass<SimpleTaskbarPriv>)
        @extends gtk::Box, gtk::Widget, // SimpleTaskbar IS a GtkBox
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

// "Private" implementation details of our GObject.
#[derive(Default)]
pub struct SimpleTaskbarPriv {
    // We can store child widgets here if we need to access them frequently
    // after initialization, though for this setup, direct packing and
    // capturing clones in closures is also fine.
    // clock_label: RefCell<Option<Label>>,
    // toplevel_container: RefCell<Option<GtkBox>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SimpleTaskbarPriv {
    const NAME: &'static str = "SimpleTaskbar";
    type Type = SimpleTaskbar;
    type ParentType = GtkBox;

    // fn class_init(klass: &mut Self::Class) {}
    // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {}
}

// Implementation of SimpleTaskbar.
impl SimpleTaskbar {
    pub fn new(list_store: ListStore<ToplevelListItemGObject>) -> Self {
        let taskbar: Self = glib::Object::new();
        taskbar.set_orientation(gtk::Orientation::Horizontal);
        taskbar.add_css_class("taskbar");

        // --- Launch Button ---
        let launch_button = Button::with_label("Launch"); // Or use an icon
        // let launch_button = Button::from_icon_name("application-x-executable-symbolic");
        launch_button.add_css_class("taskbar-button"); // Consistent styling
        launch_button.set_tooltip_text(Some("Launch a new application"));
        
        let taskbar_weak = taskbar.downgrade(); // Weak reference for the closure
        launch_button.connect_clicked(move |_btn| {
            if let Some(taskbar_instance) = taskbar_weak.upgrade() {
                if let Some(root) = taskbar_instance.root() {
                    if let Some(parent_window) = root.downcast_ref::<adw::ApplicationWindow>() {
                        let dialog = MessageDialog::new(Some(parent_window), "Launch Application", None);
                        let content_area = GtkBox::new(gtk::Orientation::Vertical, 6);

                        let command_entry = Entry::new();
                        command_entry.set_placeholder_text(Some("Enter command..."));

                        let label_for_entry = Label::new(Some("Command:"));
                        label_for_entry.set_halign(gtk::Align::Start); // Align label to the start

                        content_area.append(&label_for_entry);
                        content_area.append(&command_entry);

                        // MessageDialog in Adwaita uses set_extra_child for custom content below the body.
                        // If we want a more structured dialog, gtk::Dialog would be better.
                        // For this, let's put the entry in extra_child.
                        // dialog.set_body("Enter the command to launch below."); // Optional body
                        dialog.set_extra_child(Some(&content_area));

                        dialog.add_response("launch", "Launch");
                        dialog.add_response("cancel", "Cancel");
                        dialog.set_default_response(Some("launch"));
                        dialog.set_close_response("cancel");

                        let d_command_entry = command_entry.clone();
                        dialog.connect_response(None, move |d, response_id| {
                            if response_id == "launch" {
                                let command_text = d_command_entry.text().to_string();
                                let trimmed_command = command_text.trim();
                                if !trimmed_command.is_empty() {
                                    tracing::info!("Attempting to launch command: '{}'", trimmed_command);
                                    let mut parts = trimmed_command.split_whitespace();
                                    if let Some(executable) = parts.next() {
                                        let args: Vec<&str> = parts.collect();
                                        match std::process::Command::new(executable)
                                            .args(&args)
                                            .spawn() {
                                            Ok(child) => {
                                                tracing::info!("Successfully spawned process for '{}', PID: {}", trimmed_command, child.id());
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to spawn command '{}': {}", trimmed_command, e);
                                                // TODO: Show an error toast/dialog to the user
                                                let error_toast = adw::Toast::new(&format!("Failed: {}", e));
                                                if let Some(toast_overlay) = taskbar_instance.ancestor(adw::ToastOverlay::static_type())
                                                    .and_then(|w| w.downcast::<adw::ToastOverlay>().ok()){
                                                    toast_overlay.add_toast(&error_toast);
                                                }
                                            }
                                        }
                                    } else {
                                        tracing::warn!("Empty executable name after parsing: '{}'", trimmed_command);
                                    }
                                } else {
                                    tracing::info!("Launch command was empty.");
                                }
                            }
                            d.destroy();
                        });
                        dialog.present();
                    } else {
                        tracing::error!("Could not get ApplicationWindow as root for launch dialog.");
                    }
                } else {
                    tracing::error!("Launch button has no root widget when clicked (taskbar not in window yet?).");
                }
            } else {
                 tracing::error!("SimpleTaskbar instance for launch button was dropped.");
            }
        });
        taskbar.pack_start(&launch_button, false, false, 5);

        // --- Workspace Switcher ---
        let workspace_switcher = WorkspaceSwitcher::new(4); // Default to 4 workspaces
        taskbar.pack_start(&workspace_switcher.widget, false, false, 5);

        // --- Toplevel Container ---
        let toplevel_container = GtkBox::new(gtk::Orientation::Horizontal, 5);
        toplevel_container.add_css_class("taskbar-toplevels");
        taskbar.pack_start(&toplevel_container, true, true, 0); // Toplevels take most space

        // --- Clock Label ---
        let clock_label = Label::new(None);
        clock_label.add_css_class("taskbar-clock");
        taskbar.pack_end(&clock_label, false, false, 10);

        // Initial population of taskbar items
        Self::update_widgets_from_store(&toplevel_container, &list_store);

        // Connect to ListStore "items-changed" signal
        let tc_clone = toplevel_container.clone();
        list_store.connect_items_changed(move |store, _position, _removed, _added| {
            tracing::debug!("Taskbar ListStore items_changed triggered.");
            Self::update_widgets_from_store(&tc_clone, store);
        });

        // Setup clock timer
        let cl_clone = clock_label.clone();
        glib::timeout_add_seconds_local(1, move || {
            let now = Local::now();
            cl_clone.set_text(&now.format("%H:%M:%S").to_string());
            glib::ControlFlow::Continue
        });

        taskbar
    }

    fn update_widgets_from_store(container: &GtkBox, store: &ListStore<ToplevelListItemGObject>) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        for i in 0..store.n_items() {
            if let Some(gobject) = store.item(i) {
                if let Ok(item) = gobject.downcast::<ToplevelListItemGObject>() {
                    let title = item.property::<String>("title");
                    let app_id = item.property::<String>("app-id");
                    let wayland_id = item.property::<u32>("wayland-id");

                    let display_text = if !title.is_empty() {
                        title.clone()
                    } else if !app_id.is_empty() {
                        app_id.clone()
                    } else {
                        format!("Window {}", wayland_id)
                    };

                    let button = gtk::Button::with_label(&display_text);
                    button.add_css_class("taskbar-button");
                    let app_id_tooltip = item.property::<String>("app-id");
                    button.set_tooltip_text(Some(&format!("ID: {}, AppID: {}", wayland_id, app_id_tooltip)));

                    let wayland_id_clone = wayland_id;
                    button.connect_clicked(move |_btn| {
                        tracing::info!("Taskbar button clicked for toplevel Wayland ID: {}", wayland_id_clone);
                        // Future: send request to activate this window.
                    });
                    container.append(&button);
                }
            }
        }
        container.show();
    }
}

impl ObjectImpl for SimpleTaskbarPriv {}
impl WidgetImpl for SimpleTaskbarPriv {}
impl BoxImpl for SimpleTaskbarPriv {}
impl OrientableImpl for SimpleTaskbarPriv {}
