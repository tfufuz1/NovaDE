use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate};

use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, Popover, prelude::*}; // Added Popover and prelude
use std::cell::RefCell;
// Assuming NotificationCenterPanelWidget is in a sibling module `notification_center_panel`
use super::notification_center_panel::NotificationCenterPanelWidget;


#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct NotificationCenterButtonWidget {
    pub popover: RefCell<Option<Popover>>,
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationCenterButtonWidget {
    const NAME: &'static str = "NovaDENotificationCenterButtonWidget";
    type Type = super::NotificationCenterButtonWidget;
    type ParentType = gtk::Button;

    fn new() -> Self { // Added new for initialization
        Self {
            popover: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
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

        obj.set_icon_name("notification-symbolic"); 
        obj.set_tooltip_text(Some("Notifications"));

        obj.connect_clicked(move |button_instance_ref| {
            let imp = button_instance_ref.imp();
            let mut popover_borrow = imp.popover.borrow_mut();

            if let Some(popover) = popover_borrow.as_ref() {
                if popover.is_visible() {
                    popover.popdown();
                } else {
                    popover.popup();
                }
            } else {
                let panel_content = NotificationCenterPanelWidget::new();
                let new_popover = Popover::builder()
                    .child(&panel_content)
                    .autohide(true) 
                    .has_arrow(true) 
                    .build();
                
                new_popover.set_parent(button_instance_ref);
                *popover_borrow = Some(new_popover.clone());
                new_popover.popup();
            }
        });
    }
}

impl WidgetImpl for NotificationCenterButtonWidget {}
impl ButtonImpl for NotificationCenterButtonWidget {}
