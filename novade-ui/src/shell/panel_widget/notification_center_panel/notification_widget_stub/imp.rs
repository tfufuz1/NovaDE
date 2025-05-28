use gtk::glib::{self, subclass::Signal}; // Added subclass::Signal
use gtk::subclass::prelude::*;
use gtk::{Box, Label, Button, CompositeTemplate, Orientation, Align, prelude::*};
use std::cell::RefCell;
use once_cell::sync::Lazy; // Added once_cell for static SIGNALS

static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
    vec![Signal::builder("closed")
        .action() // Indicates it's an action signal
        .build(),
    ]
});

use novade_domain::notifications::Notification; // For type hint, though not directly used in struct fields

#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct NotificationWidgetStub {
    #[template_child] 
    pub app_name_label: TemplateChild<Label>,
    #[template_child]
    pub summary_label: TemplateChild<Label>,
    pub notification_id: RefCell<String>, // Added field
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationWidgetStub {
    const NAME: &'static str = "NovaDENotificationWidgetStub";
    type Type = super::NotificationWidgetStub;
    type ParentType = gtk::Box;

    fn new() -> Self { // Added new for initializing notification_id
        Self {
            app_name_label: TemplateChild::default(),
            summary_label: TemplateChild::default(),
            notification_id: RefCell::new(String::new()),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("notificationwidgetstub");
        klass.install_signals(&SIGNALS); 
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NotificationWidgetStub {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); 

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(6);
        obj.set_margin_top(6);
        obj.set_margin_bottom(6);
        obj.set_margin_start(6);
        obj.set_margin_end(6);
        obj.add_css_class("notification-item");
        
        let app_name_label = self.app_name_label.get();
        app_name_label.set_halign(Align::Start);
        app_name_label.add_css_class("notification-app-name");
        
        let summary_label = self.summary_label.get();
        summary_label.set_halign(Align::Start);
        summary_label.set_wrap(true);
        summary_label.add_css_class("notification-summary");

        let close_button = Button::with_label("Close");
        close_button.set_halign(Align::End); 
        close_button.add_css_class("notification-close-button");
        
        // Emit the "closed" signal when the button is clicked
        let self_obj = obj.clone(); // Clone for the closure
        close_button.connect_clicked(move |_btn| {
            tracing::info!("Close button clicked on a notification stub, emitting 'closed' signal.");
            self_obj.emit_by_name::<()>("closed", &[]);
        });
        
        obj.append(&app_name_label);
        obj.append(&summary_label);
        obj.append(&close_button);
    }
}

impl WidgetImpl for NotificationWidgetStub {}
impl BoxImpl for NotificationWidgetStub {}
