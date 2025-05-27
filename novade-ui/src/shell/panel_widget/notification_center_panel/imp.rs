use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, Label, CompositeTemplate, Orientation}; // Added necessary imports

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now
pub struct NotificationCenterPanelWidget {
    // Struct can be empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationCenterPanelWidget {
    const NAME: &'static str = "NovaDENotificationCenterPanelWidget";
    type Type = super::NotificationCenterPanelWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        // NotificationCenterPanelWidget::bind_template(klass); // No template for now
        klass.set_css_name("notificationcenterpanelwidget");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NotificationCenterPanelWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); // This is the NotificationCenterPanelWidget (gtk::Box)

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(6); // Add some spacing

        // Add placeholder content
        let no_notifications_label = Label::new(Some("No Notifications"));
        obj.append(&no_notifications_label);
        
        // Add some padding to the box itself to make the popover look a bit nicer
        obj.set_margin_top(12); // More top margin for a "title" feel if desired
        obj.set_margin_bottom(12);
        obj.set_margin_start(12);
        obj.set_margin_end(12);
        
        // Set a minimum width for the popover content
        obj.set_width_request(250); 
    }
}

impl WidgetImpl for NotificationCenterPanelWidget {}
impl BoxImpl for NotificationCenterPanelWidget {}
