use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate};

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now, as it's a simple button
pub struct NotificationCenterButtonWidget {
    // Struct can be empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationCenterButtonWidget {
    const NAME: &'static str = "NovaDENotificationCenterButtonWidget";
    type Type = super::NotificationCenterButtonWidget;
    type ParentType = gtk::Button;

    fn class_init(klass: &mut Self::Class) {
        // NotificationCenterButtonWidget::bind_template(klass); // No template for now
        klass.set_css_name("notificationcenterbuttonwidget");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NotificationCenterButtonWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.set_icon_name("notification-symbolic"); // Or "emblem-synchronizing-symbolic"
        obj.set_tooltip_text(Some("Notifications"));
    }

    // No properties defined yet
    // fn properties() -> &'static [glib::ParamSpec] { &[] }
    // fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) { unimplemented!() }
    // fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value { unimplemented!() }

    // No signals defined yet
    // fn signals() -> &'static [Signal] { &[] }
}

impl WidgetImpl for NotificationCenterButtonWidget {}
impl ButtonImpl for NotificationCenterButtonWidget {}
