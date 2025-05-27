use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, Label, Button, CompositeTemplate, Orientation, Align, prelude::*};
use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now
pub struct NotificationWidgetStub {
    // Store labels to set their text in new() via a method
    #[template_child(id = "app_name_label")] // Example if using CompositeTemplate string
    pub app_name_label: TemplateChild<Label>,
    #[template_child(id = "summary_label")]
    pub summary_label: TemplateChild<Label>,

    // If not using CompositeTemplate string for children, define them as RefCell<Option<Label>>
    // pub app_name_label_manual: RefCell<Option<Label>>,
    // pub summary_label_manual: RefCell<Option<Label>>,
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationWidgetStub {
    const NAME: &'static str = "NovaDENotificationWidgetStub";
    type Type = super::NotificationWidgetStub;
    type ParentType = gtk::Box;

    // If not using CompositeTemplate string for children, need a new() for RefCells
    // fn new() -> Self {
    //     Self {
    //         app_name_label_manual: RefCell::new(None),
    //         summary_label_manual: RefCell::new(None),
    //     }
    // }

    fn class_init(klass: &mut Self::Class) {
        // NotificationWidgetStub::bind_template(klass); // If using CompositeTemplate string
        klass.set_css_name("notificationwidgetstub");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NotificationWidgetStub {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); // This is the NotificationWidgetStub (gtk::Box)

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(6);
        obj.set_margin_top(6);
        obj.set_margin_bottom(6);
        obj.set_margin_start(6);
        obj.set_margin_end(6);
        
        // Add a CSS class for styling the whole notification item
        obj.add_css_class("notification-item");


        // If not using CompositeTemplate string for children, create them manually:
        let app_name_label_manual = Label::new(Some("App Name")); // Default text
        app_name_label_manual.set_halign(Align::Start);
        app_name_label_manual.add_css_class("notification-app-name"); // For styling
        // self.app_name_label_manual.replace(Some(app_name_label_manual.clone()));
        // self.app_name_label.set_inner(&app_name_label_manual); // If TemplateChild is used for manual setup

        let summary_label_manual = Label::new(Some("Notification summary...")); // Default text
        summary_label_manual.set_halign(Align::Start);
        summary_label_manual.set_wrap(true); // Allow summary to wrap
        summary_label_manual.add_css_class("notification-summary");
        // self.summary_label_manual.replace(Some(summary_label_manual.clone()));
        // self.summary_label.set_inner(&summary_label_manual);

        // For this stub, we will use TemplateChild to refer to children defined in `mod.rs`'s `new` method.
        // So, the labels are expected to be set up by the TemplateChild mechanism
        // by virtue of being fields in the struct.
        // However, since we are NOT using a template string, we must manually assign them.
        // The TemplateChild fields will be bound to manually created children.
        
        let app_name_label = self.app_name_label.get();
        app_name_label.set_halign(Align::Start);
        app_name_label.add_css_class("notification-app-name");
        
        let summary_label = self.summary_label.get();
        summary_label.set_halign(Align::Start);
        summary_label.set_wrap(true);
        summary_label.add_css_class("notification-summary");

        let close_button = Button::with_label("Close");
        close_button.set_halign(Align::End); // Align button to the right
        close_button.add_css_class("notification-close-button");
        close_button.connect_clicked(|_btn| {
            // In a real app, this would emit a signal or call a method to remove the notification
            tracing::info!("Notification Close button clicked (stub).");
        });
        
        obj.append(&app_name_label);
        obj.append(&summary_label);
        obj.append(&close_button);
    }
}

impl WidgetImpl for NotificationWidgetStub {}
impl BoxImpl for NotificationWidgetStub {}

// If app_name_label and summary_label were RefCell<Option<Label>>, add methods like this:
// impl NotificationWidgetStub {
//     pub fn set_app_name(&self, name: &str) {
//         if let Some(label) = self.app_name_label_manual.borrow().as_ref() {
//             label.set_text(name);
//         }
//     }
//     pub fn set_summary(&self, summary: &str) {
//         if let Some(label) = self.summary_label_manual.borrow().as_ref() {
//             label.set_text(summary);
//         }
//     }
// }
