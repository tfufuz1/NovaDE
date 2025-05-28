use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, Image, Popover, Orientation, Box as GtkBox, Label, Button as GtkButton, ScrolledWindow, PolicyType};
use std::cell::RefCell;
use std::sync::Arc;
use futures_util::StreamExt; // For event_receiver.next()

// Real NetworkManagerIntegration components
use novade_system::network_management::{
    NetworkManager, NetworkManagerIntegration, NetworkEvent, NetworkConnection,
    NetworkConnectionType, NetworkConnectionState, NetworkEventType,
};
// Error type (adjust if NovaError is not the one returned by NetworkManagerIntegration)
// use novade_core::errors::NovaError; // Assuming this is the error type

// #[derive(Default)] // Default derive won't work with MainContext easily
pub struct NetworkManagementWidget {
    network_manager: RefCell<Option<Arc<dyn NetworkManager>>>,
    icon: RefCell<Option<Image>>,
    popover: RefCell<Option<Popover>>,
    event_receiver: RefCell<Option<tokio::sync::mpsc::Receiver<NetworkEvent>>>,
    main_context: glib::MainContext, // For spawning UI updates from async tasks
}

impl Default for NetworkManagementWidget {
    fn default() -> Self {
        Self {
            network_manager: RefCell::new(None),
            icon: RefCell::new(None),
            popover: RefCell::new(None),
            event_receiver: RefCell::new(None),
            main_context: glib::MainContext::default(), // Get default main context
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for NetworkManagementWidget {
    const NAME: &'static str = "NovaDENetworkManagementWidget";
    type Type = super::NetworkManagementWidget;
    type ParentType = gtk::Button; // It's a button that shows a popover

    fn new() -> Self {
        Self::default()
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("networkmanagementwidget");
    }
}

impl ObjectImpl for NetworkManagementWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Create and set up the icon (default: offline)
        let icon_image = Image::from_icon_name(Some("network-offline-symbolic"));
        self.icon.replace(Some(icon_image.clone()));
        obj.set_child(Some(&icon_image));
        
        // Create the popover
        let popover = Popover::new();
        self.popover.replace(Some(popover.clone()));
        popover.set_parent(&*obj);

        // Initialize NetworkManagerIntegration
        match NetworkManagerIntegration::new() {
            Ok(nm_instance) => {
                let nm_arc: Arc<dyn NetworkManager> = Arc::new(nm_instance);
                self.network_manager.replace(Some(nm_arc.clone()));
                
                // Subscribe to events and start listening
                let main_context_clone = self.main_context.clone();
                let obj_weak = obj.downgrade(); // Use weak ref for async task

                main_context_clone.spawn_local(glib::clone!(@strong nm_arc, @strong main_context_clone as event_loop_main_context => async move {
                    match nm_arc.subscribe().await {
                        Ok(mut receiver) => {
                            // Store receiver if needed, or directly use it in a loop here
                            // For simplicity, let's loop here
                            event_loop_main_context.spawn_local(async move {
                                while let Some(event) = receiver.recv().await {
                                    if let Some(obj_strong) = obj_weak.upgrade() {
                                        println!("Network Event Received: {:?}", event.event_type);
                                        // Trigger UI update based on event
                                        obj_strong.imp().request_ui_update(obj_strong.clone());
                                    } else {
                                        // Object is gone, stop listening
                                        break;
                                    }
                                }
                                println!("Network event stream ended.");
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to subscribe to NetworkManager events: {}", e);
                            // Update UI to show error state
                            if let Some(obj_strong) = obj_weak.upgrade() {
                                obj_strong.imp().set_error_state(obj_strong.clone(), &format!("Subscription failed: {}", e));
                            }
                        }
                    }
                }));

            }
            Err(e) => {
                eprintln!("Failed to initialize NetworkManagerIntegration: {}", e);
                // Update UI to show error state (e.g., specific icon, popover message)
                self.set_error_state(obj.clone(), &format!("Init failed: {}", e));
            }
        }
        
        // Initial UI update request
        self.request_ui_update(obj.clone());

        // Show popover on click
        obj.connect_clicked(glib::clone!(@weak self as widget_imp, @weak obj => move |_button| {
            if let Some(popover_ref) = widget_imp.popover.borrow().as_ref() {
                 // Request UI update for popover content before showing, ensures it's fresh
                widget_imp.request_ui_update(obj.clone());
                popover_ref.popup();
            }
        }));
    }

    fn dispose(&self) {
        // Stop any running tasks, disconnect signals etc.
        // For mpsc, if the sender (in NetworkManagerIntegration) is dropped, receivers will eventually stop.
        // If event_receiver was stored and had a close method, call it here.
        if let Some(icon) = self.icon.borrow_mut().take() {
            icon.unparent();
        }
        if let Some(popover) = self.popover.borrow_mut().take() {
            popover.unparent();
        }
        // Explicitly drop network_manager to release Arc, potentially stopping its tasks if designed so.
        self.network_manager.replace(None);
        self.event_receiver.replace(None);
    }
}

impl WidgetImpl for NetworkManagementWidget {}
impl ButtonImpl for NetworkManagementWidget {}

impl NetworkManagementWidget {
    pub fn request_ui_update(&self, widget_obj: super::NetworkManagementWidget) {
        let main_context_clone = self.main_context.clone();
        main_context_clone.spawn_local(glib::clone!(@weak widget_obj => async move {
            widget_obj.imp().update_network_icon_impl().await;
            widget_obj.imp().update_popover_content_impl(widget_obj.clone()).await; // Pass widget_obj for connect/disconnect
        }));
    }
    
    fn set_error_state(&self, widget_obj: super::NetworkManagementWidget, error_msg: &str) {
        if let Some(icon) = self.icon.borrow().as_ref() {
            icon.set_from_icon_name(Some("network-offline-symbolic")); // Or a specific error icon
        }
        if let Some(popover) = self.popover.borrow().as_ref() {
            let vbox = GtkBox::new(Orientation::Vertical, 5);
            vbox.append(&Label::new(Some("Error:")));
            vbox.append(&Label::new(Some(error_msg)));
            popover.set_child(Some(&vbox));
        }
         eprintln!("Network widget error state set: {}", error_msg);
    }

    async fn update_network_icon_impl(&self) {
        let icon_widget_opt = self.icon.borrow();
        let icon_widget = icon_widget_opt.as_ref().unwrap(); // Assume icon is always there after constructed

        if let Some(nm) = self.network_manager.borrow().as_ref() {
            match nm.get_connections().await {
                Ok(connections) => {
                    let mut primary_connection: Option<&NetworkConnection> = None;
                    for conn in &connections {
                        if conn.is_default() && conn.state() == NetworkConnectionState::Connected {
                            primary_connection = Some(conn);
                            break;
                        }
                    }
                    // If no default connected, check for any connected
                    if primary_connection.is_none() {
                         primary_connection = connections.iter().find(|c| c.state() == NetworkConnectionState::Connected);
                    }
                    // If still none, check for any connecting
                     if primary_connection.is_none() {
                         primary_connection = connections.iter().find(|c| c.state() == NetworkConnectionState::Connecting);
                    }


                    if let Some(conn) = primary_connection {
                        let icon_name = match conn.connection_type() {
                            NetworkConnectionType::Wired => "network-wired-symbolic",
                            NetworkConnectionType::Wireless => {
                                if conn.state() == NetworkConnectionState::Connecting {
                                    "network-wireless-acquiring-symbolic"
                                } else {
                                    match conn.strength() {
                                        Some(s) if s > 0.8 => "network-wireless-signal-excellent-symbolic",
                                        Some(s) if s > 0.6 => "network-wireless-signal-good-symbolic",
                                        Some(s) if s > 0.4 => "network-wireless-signal-ok-symbolic",
                                        Some(s) if s > 0.1 => "network-wireless-signal-weak-symbolic",
                                        _ => "network-wireless-signal-none-symbolic",
                                    }
                                }
                            }
                            NetworkConnectionType::Mobile => "network-cellular-signal-good-symbolic", // Placeholder
                            NetworkConnectionType::VPN => "network-vpn-symbolic",
                            _ => "network-wired-symbolic", // Default for "Other"
                        };
                        icon_widget.set_from_icon_name(Some(icon_name));
                    } else {
                        // No active or connecting connection
                        icon_widget.set_from_icon_name(Some("network-offline-symbolic"));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get connections for icon update: {}", e);
                    icon_widget.set_from_icon_name(Some("network-offline-symbolic")); // Error state
                }
            }
        } else {
            icon_widget.set_from_icon_name(Some("network-offline-symbolic")); // NM not initialized
        }
    }

    async fn update_popover_content_impl(&self, widget_obj: super::NetworkManagementWidget) {
        let popover_opt = self.popover.borrow();
        let popover = popover_opt.as_ref().unwrap(); // Assume popover is always there

        if let Some(child) = popover.child() {
            popover.set_child(Option::<&gtk::Widget>::None); // Clear old content
        }
        
        let vbox = GtkBox::new(Orientation::Vertical, 5);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        if let Some(nm) = self.network_manager.borrow().as_ref() {
            match nm.get_connections().await {
                Ok(connections) => {
                    if connections.is_empty() {
                        vbox.append(&Label::new(Some("No network connections available.")));
                    } else {
                        for conn in connections {
                            let conn_box = GtkBox::new(Orientation::Horizontal, 5);
                            let name_label = Label::new(Some(conn.name()));
                            name_label.set_hexpand(true);
                            name_label.set_xalign(0.0); // Align left
                            conn_box.append(&name_label);

                            let status_label = Label::new(Some(&format!("{:?}", conn.state())));
                            conn_box.append(&status_label);

                            // Use connection ID (UUID) for connect/disconnect actions
                            let conn_id = conn.id().to_string(); 

                            match conn.state() {
                                NetworkConnectionState::Disconnected | NetworkConnectionState::Unknown => {
                                    let connect_button = GtkButton::with_label("Connect");
                                    let widget_obj_clone = widget_obj.clone();
                                    connect_button.connect_clicked(move |_| {
                                        let conn_id_clone = conn_id.clone();
                                        widget_obj_clone.imp().main_context.spawn_local(glib::clone!(@weak widget_obj_clone => async move {
                                            widget_obj_clone.imp().connect_to_network(conn_id_clone).await;
                                            // Request UI update after action
                                            widget_obj_clone.imp().request_ui_update(widget_obj_clone.clone());
                                        }));
                                    });
                                    conn_box.append(&connect_button);
                                }
                                NetworkConnectionState::Connected | NetworkConnectionState::Connecting => {
                                    let disconnect_button = GtkButton::with_label("Disconnect");
                                     let widget_obj_clone = widget_obj.clone();
                                    disconnect_button.connect_clicked(move |_| {
                                        let conn_id_clone = conn_id.clone();
                                         widget_obj_clone.imp().main_context.spawn_local(glib::clone!(@weak widget_obj_clone => async move {
                                            widget_obj_clone.imp().disconnect_from_network(conn_id_clone).await;
                                            widget_obj_clone.imp().request_ui_update(widget_obj_clone.clone());
                                        }));
                                    });
                                    conn_box.append(&disconnect_button);
                                }
                                _ => {} // Disconnecting, etc. - no action button for now
                            }
                            vbox.append(&conn_box);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get connections for popover: {}", e);
                    vbox.append(&Label::new(Some(&format!("Error loading connections: {}", e))));
                }
            }
        } else {
            vbox.append(&Label::new(Some("NetworkManager not available.")));
        }
        
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic); // Horizontal never, Vertical auto
        scrolled_window.set_child(Some(&vbox));
        scrolled_window.set_max_content_height(300); // Max height for the popover content
        scrolled_window.set_min_content_width(250);


        popover.set_child(Some(&scrolled_window));
    }

    async fn connect_to_network(&self, connection_id: String) {
        if let Some(nm) = self.network_manager.borrow().as_ref() {
            println!("Attempting to connect to: {}", connection_id);
            if let Err(e) = nm.connect(&connection_id).await {
                eprintln!("Failed to connect to network {}: {}", connection_id, e);
                // TODO: Show error to user (e.g., notification or popover message)
            } else {
                println!("Connection attempt initiated for: {}", connection_id);
            }
        }
    }

    async fn disconnect_from_network(&self, connection_id: String) {
        if let Some(nm) = self.network_manager.borrow().as_ref() {
            println!("Attempting to disconnect from: {}", connection_id);
            if let Err(e) = nm.disconnect(&connection_id).await {
                eprintln!("Failed to disconnect from network {}: {}", connection_id, e);
                // TODO: Show error to user
            } else {
                 println!("Disconnection attempt initiated for: {}", connection_id);
            }
        }
    }
}
