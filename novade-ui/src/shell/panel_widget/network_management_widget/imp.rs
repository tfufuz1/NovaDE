use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, Image, Popover, Orientation, Box as GtkBox, Label};
use std::cell::RefCell;
use once_cell::sync::Lazy;

// Assuming NetworkManagerIntegration is available from a crate.
// This will likely require adding a dependency to Cargo.toml later.
// use network_manager_integration_system::NetworkManagerIntegration; 
// For now, let's define a placeholder if the actual crate is not yet available
// to allow the rest of the code structure to be outlined.

struct PlaceholderNetworkManagerIntegration;

impl PlaceholderNetworkManagerIntegration {
    fn new() -> Self { PlaceholderNetworkManagerIntegration }
    fn connect_network_state_changed<F: Fn() + 'static>(&self, _callback: F) {}
    fn get_current_icon_name(&self) -> String { "network-wireless-signal-none-symbolic".to_string() } // Placeholder
    fn get_available_connections(&self) -> Vec<String> { vec!["WiFi Network 1".to_string(), "Ethernet".to_string()] } // Placeholder
}


#[derive(Default)]
pub struct NetworkManagementWidget {
    network_manager: RefCell<Option<PlaceholderNetworkManagerIntegration>>, // Placeholder
    icon: RefCell<Option<Image>>,
    popover: RefCell<Option<Popover>>,
}

#[glib::object_subclass]
impl ObjectSubclass for NetworkManagementWidget {
    const NAME: &'static str = "NovaDENetworkManagementWidget";
    type Type = super::NetworkManagementWidget;
    type ParentType = gtk::Button;

    fn new() -> Self {
        Self {
            network_manager: RefCell::new(None),
            icon: RefCell::new(None),
            popover: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("networkmanagementwidget");
        // klass.bind_template(); // If using a UI template
    }
}

impl ObjectImpl for NetworkManagementWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Initialize NetworkManagerIntegration
        let nm_integration = PlaceholderNetworkManagerIntegration::new(); // Replace with actual
        self.network_manager.replace(Some(nm_integration));

        // Create and set up the icon
        let icon_image = Image::new();
        self.icon.replace(Some(icon_image.clone()));
        obj.set_child(Some(self.icon.borrow().as_ref().unwrap()));
        
        // Create the popover
        let popover = Popover::new();
        self.popover.replace(Some(popover.clone()));
        popover.set_parent(&*obj); // Attach popover to the button

        // Initial UI update
        self.update_network_icon_impl();
        self.update_popover_content_impl();

        // Connect to NetworkManager signals (placeholder)
        if let Some(nm) = self.network_manager.borrow().as_ref() {
            nm.connect_network_state_changed(glib::clone!(@weak obj => move || {
                obj.imp().update_network_icon_impl();
                obj.imp().update_popover_content_impl();
            }));
        }

        // Show popover on click
        obj.connect_clicked(glib::clone!(@weak self as widget_imp => move |_button| {
            if let Some(popover) = widget_imp.popover.borrow().as_ref() {
                popover.popup();
            }
        }));
    }

    fn dispose(&self) {
        // Disconnect signals, clean up resources
        // For example, if NetworkManagerIntegration had a disconnect method
        // if let Some(nm) = self.network_manager.borrow_mut().take() {
        //     nm.disconnect_all_signals(); // Assuming such a method exists
        // }
        if let Some(icon) = self.icon.borrow_mut().take() {
            icon.unparent();
        }
        if let Some(popover) = self.popover.borrow_mut().take() {
            popover.unparent();
        }
    }

    // No custom properties for now, so no need for properties(), set_property(), property()
}

impl WidgetImpl for NetworkManagementWidget {}
impl ButtonImpl for NetworkManagementWidget {}

// Private helper methods
impl NetworkManagementWidget {
    fn update_network_icon_impl(&self) {
        if let (Some(icon_widget), Some(nm)) = (self.icon.borrow().as_ref(), self.network_manager.borrow().as_ref()) {
            let icon_name = nm.get_current_icon_name(); // Method from NetworkManagerIntegration
            icon_widget.set_from_icon_name(Some(&icon_name));
        }
    }

    fn update_popover_content_impl(&self) {
        if let (Some(popover), Some(nm)) = (self.popover.borrow().as_ref(), self.network_manager.borrow().as_ref()) {
            // Clear existing content
            if let Some(child) = popover.child() {
                popover.set_child(Option::<&gtk::Widget>::None);
            }

            let vbox = GtkBox::new(Orientation::Vertical, 5);
            
            // Placeholder: Add a label indicating it's the network popover
            vbox.append(&Label::new(Some("Available Networks:")));

            let connections = nm.get_available_connections(); // Method from NetworkManagerIntegration
            for conn_name in connections {
                // In a real scenario, this would be a custom widget for each network,
                // showing more details and allowing interaction (connect/disconnect buttons)
                let label = Label::new(Some(&conn_name));
                vbox.append(&label);
                // TODO: Add button or interaction for each network
            }
            
            // If no connections, show a message
            if vbox.first_child().is_none() { // Check if anything was added besides the title
                 let no_networks_label = Label::new(Some("No networks found or NetworkManager not available."));
                 vbox.append(&no_networks_label);
            }

            popover.set_child(Some(&vbox));
        }
    }
}
