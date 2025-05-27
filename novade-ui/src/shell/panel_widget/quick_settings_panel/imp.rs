use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, Label, Button, CompositeTemplate, Orientation}; // Added necessary imports

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now
pub struct QuickSettingsPanelWidget {
    // Struct can be empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for QuickSettingsPanelWidget {
    const NAME: &'static str = "NovaDEQuickSettingsPanelWidget";
    type Type = super::QuickSettingsPanelWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        // QuickSettingsPanelWidget::bind_template(klass); // No template for now
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
        obj.set_spacing(6); // Add some spacing

        // Add placeholder content
        let wifi_label = Label::new(Some("WiFi Placeholder"));
        obj.append(&wifi_label);

        let bluetooth_button = Button::with_label("Bluetooth Placeholder");
        obj.append(&bluetooth_button);

        let volume_label = Label::new(Some("Volume Control Placeholder"));
        obj.append(&volume_label);
        
        // Add some padding to the box itself to make the popover look a bit nicer
        obj.set_margin_top(6);
        obj.set_margin_bottom(6);
        obj.set_margin_start(6);
        obj.set_margin_end(6);
    }
}

impl WidgetImpl for QuickSettingsPanelWidget {}
impl BoxImpl for QuickSettingsPanelWidget {}
