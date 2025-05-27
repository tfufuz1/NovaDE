use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, Label, Button, Switch, Scale, Spinner, CompositeTemplate, Orientation, Adjustment, Align, prelude::*};

#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct QuickSettingsPanelWidget {
    // Struct remains empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for QuickSettingsPanelWidget {
    const NAME: &'static str = "NovaDEQuickSettingsPanelWidget";
    type Type = super::QuickSettingsPanelWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("quicksettingspanelwidget");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for QuickSettingsPanelWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); // This is the QuickSettingsPanelWidget (gtk::Box)

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(12); // Increased spacing for sections
        
        // Margins for the overall popover content
        obj.set_margin_top(10);
        obj.set_margin_bottom(10);
        obj.set_margin_start(10);
        obj.set_margin_end(10);

        // --- Dark Mode Section ---
        let dark_mode_box = Box::new(Orientation::Horizontal, 6);
        dark_mode_box.set_halign(Align::Fill); // Ensure horizontal box fills width
        let dark_mode_label = Label::new(Some("Dark Mode"));
        dark_mode_label.set_halign(Align::Start); // Align label to the start
        dark_mode_label.set_hexpand(true); // Allow label to expand
        let dark_mode_switch = Switch::new();
        dark_mode_switch.set_halign(Align::End); // Align switch to the end
        
        dark_mode_box.append(&dark_mode_label);
        dark_mode_box.append(&dark_mode_switch);
        obj.append(&dark_mode_box);

        // --- Volume Section ---
        let volume_box = Box::new(Orientation::Horizontal, 6);
        volume_box.set_halign(Align::Fill);
        let volume_label = Label::new(Some("Volume"));
        volume_label.set_halign(Align::Start);
        // volume_label.set_hexpand(true); // Don't let label expand too much if scale is present
        let volume_adjustment = Adjustment::new(50.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let volume_scale = Scale::new(Orientation::Horizontal, Some(&volume_adjustment));
        volume_scale.set_hexpand(true); // Scale should take available space
        volume_scale.set_halign(Align::Fill);
        volume_scale.set_draw_value(false); // Common for volume sliders to not show numeric value on slider

        volume_box.append(&volume_label);
        volume_box.append(&volume_scale);
        obj.append(&volume_box);

        // --- WiFi Section ---
        let wifi_box = Box::new(Orientation::Horizontal, 6);
        wifi_box.set_halign(Align::Fill);
        let wifi_label = Label::new(Some("WiFi"));
        wifi_label.set_halign(Align::Start);
        wifi_label.set_hexpand(true); // Allow label to expand
        
        let wifi_status_box = Box::new(Orientation::Horizontal, 6); // To group spinner and button
        let wifi_spinner = Spinner::new();
        // wifi_spinner.start(); // Start spinner for visual effect if desired
        let wifi_button = Button::with_label("Select Network...");
        
        wifi_status_box.append(&wifi_spinner); // Spinner might be hidden/shown based on state later
        wifi_status_box.append(&wifi_button);
        wifi_status_box.set_halign(Align::End);

        wifi_box.append(&wifi_label);
        wifi_box.append(&wifi_status_box);
        obj.append(&wifi_box);
        
        // Set a minimum width for the popover content to ensure it's not too cramped
        obj.set_width_request(280); 
    }
}

impl WidgetImpl for QuickSettingsPanelWidget {}
impl BoxImpl for QuickSettingsPanelWidget {}
