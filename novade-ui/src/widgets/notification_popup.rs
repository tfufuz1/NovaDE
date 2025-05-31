// novade-ui/src/widgets/notification_popup.rs
use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Label, Orientation};

glib::wrapper! {
    pub struct NotificationPopupWidget(ObjectSubclass<imp::NotificationPopupWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NotificationPopupWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_content(&self, id: u32, app_name: &str, summary: &str, body: &str) {
        let imp = self.imp();
        imp.app_name_label.set_text(app_name);
        imp.summary_label.set_text(summary);
        imp.body_label.set_text(body);
        imp.notification_id.replace(id); // Store the ID
        println!("[UI NOTIFICATION POPUP Widget] Set content for ID: {}, App: {}, Summary: '{}'", id, app_name, summary);
    }

    pub fn get_id(&self) -> u32 {
        *self.imp().notification_id.borrow()
    }
}

mod imp {
    use super::*;
    // No need for InitializingObject with current glib versions for ObjectImpl::constructed
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    // This defines the internal state of our widget.
    #[derive(Default)]
    pub struct NotificationPopupWidget {
        pub app_name_label: Label,
        pub summary_label: Label,
        pub body_label: Label,
        pub notification_id: RefCell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationPopupWidget {
        const NAME: &'static str = "NovaNotificationPopupWidget"; // Changed to avoid potential conflict
        type Type = super::NotificationPopupWidget;
        type ParentType = gtk::Box;

        //implicitly calls parent's default new, then this.
        fn new() -> Self {
            let app_name_label = Label::builder().halign(gtk::Align::Start).wrap(true).build();
            let summary_label = Label::builder().halign(gtk::Align::Start).wrap(true).build();
            let body_label = Label::builder().halign(gtk::Align::Start).wrap(true).build();
            Self {
                app_name_label,
                summary_label,
                body_label,
                notification_id: RefCell::new(0),
            }
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for NotificationPopupWidget {
        fn constructed(&self) {
            // Call parent constructor first
            self.parent_constructed();

            // Get the GObject instance (the widget itself)
            let obj = self.obj();
            obj.set_orientation(Orientation::Vertical);
            obj.set_spacing(5); // Set some spacing between elements

            // Add the labels to the Box container
            obj.append(&self.app_name_label);
            obj.append(&self.summary_label);
            obj.append(&self.body_label);
        }
    }

    // Trait shared by all GtkWidgets
    impl WidgetImpl for NotificationPopupWidget {}

    // Trait shared by all GtkBox widgets
    impl BoxImpl for NotificationPopupWidget {}

    // Trait shared by all GtkOrientable widgets (Box is one)
    impl OrientableImpl for NotificationPopupWidget {}
}
