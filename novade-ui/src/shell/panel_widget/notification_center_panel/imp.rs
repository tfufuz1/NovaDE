use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, CompositeTemplate, Orientation, prelude::*}; // Removed Label, added prelude
use super::notification_widget_stub::NotificationWidgetStub; // Import the stub

#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct NotificationCenterPanelWidget {
    // Struct can be empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationCenterPanelWidget {
    const NAME: &'static str = "NovaDENotificationCenterPanelWidget";
    type Type = super::NotificationCenterPanelWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
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
        obj.set_spacing(6); 
        
        // Margins for the overall popover content
        obj.set_margin_top(10);
        obj.set_margin_bottom(10);
        obj.set_margin_start(10);
        obj.set_margin_end(10);
        
        // Set a minimum width for the popover content
        obj.set_width_request(300); // Slightly wider for notifications

        // Container for notification items
        // The NotificationCenterPanelWidget itself is already a Box, so we can append directly to it.
        // If we wanted a scrollable area, we'd add a ScrolledWindow here, then a Box inside that.
        // For now, direct appending is fine.

        // Add dummy NotificationWidgetStub instances
        let notification1 = NotificationWidgetStub::new(
            "System Update",
            "Updates are available. Click to install.",
        );
        obj.append(&notification1);

        let notification2 = NotificationWidgetStub::new(
            "Email Client",
            "You have 3 new messages from John Doe.",
        );
        obj.append(&notification2);
        
        let notification3 = NotificationWidgetStub::new(
            "Calendar",
            "Reminder: Team Meeting in 15 minutes.",
        );
        obj.append(&notification3);

        // If no notifications, a label should ideally be shown.
        // For now, we always show these stubs. A more advanced implementation
        // would clear these and show "No Notifications" label if the list is empty.
    }
}

impl WidgetImpl for NotificationCenterPanelWidget {}
impl BoxImpl for NotificationCenterPanelWidget {}
