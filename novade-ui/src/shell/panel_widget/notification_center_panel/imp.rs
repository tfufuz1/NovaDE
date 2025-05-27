use gtk::glib; // Ensure glib is imported for closure_local!
use gtk::subclass::prelude::*;
use gtk::{Box, CompositeTemplate, Orientation, prelude::*};
use super::notification_widget_stub::NotificationWidgetStub; 
use tracing; // Ensure tracing is imported for logging

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
        
        obj.set_margin_top(10);
        obj.set_margin_bottom(10);
        obj.set_margin_start(10);
        obj.set_margin_end(10);
        
        obj.set_width_request(300); 

        // Add dummy NotificationWidgetStub instances and connect to their "closed" signal
        let notifications_data = vec![
            ("System Update", "Updates are available. Click to install."),
            ("Email Client", "You have 3 new messages from John Doe."),
            ("Calendar", "Reminder: Team Meeting in 15 minutes."),
        ];

        for (app_name, summary) in notifications_data {
            let stub_instance = NotificationWidgetStub::new(app_name, summary);
            
            let container_box = obj.clone(); // Clone obj (NotificationCenterPanelWidget) for the closure
            stub_instance.connect_closure(
                "closed",
                false, // after = false
                glib::closure_local!(move |stub: super::notification_widget_stub::NotificationWidgetStub| {
                    tracing::info!("'closed' signal received for notification: '{}'", stub.imp().app_name_label.text());
                    container_box.remove(&stub);
                })
            );
            obj.append(&stub_instance);
        }

        // Future: Logic to show "No Notifications" label if the container_box becomes empty.
    }
}

impl WidgetImpl for NotificationCenterPanelWidget {}
impl BoxImpl for NotificationCenterPanelWidget {}
