use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate}; // Added Button

use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, Popover, prelude::*}; // Added Popover and prelude
use std::cell::RefCell;
// Assuming QuickSettingsPanelWidget is in a sibling module `quick_settings_panel`
use super::quick_settings_panel::QuickSettingsPanelWidget;


#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct QuickSettingsButtonWidget {
    pub popover: RefCell<Option<Popover>>,
}

#[glib::object_subclass]
impl ObjectSubclass for QuickSettingsButtonWidget {
    const NAME: &'static str = "NovaDEQuickSettingsButtonWidget";
    type Type = super::QuickSettingsButtonWidget;
    type ParentType = gtk::Button;

    fn new() -> Self { // Added new for initialization
        Self {
            popover: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("quicksettingsbuttonwidget");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for QuickSettingsButtonWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.set_icon_name("preferences-system-symbolic");
        obj.set_tooltip_text(Some("Quick Settings"));

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
                let panel_content = QuickSettingsPanelWidget::new();
                let new_popover = Popover::builder()
                    .child(&panel_content)
                    .autohide(true) // Common for popovers
                    .has_arrow(true) // Common for popovers attached to buttons
                    .build();
                
                new_popover.set_parent(button_instance_ref);
                *popover_borrow = Some(new_popover.clone());
                new_popover.popup();
            }
        });
    }
}

impl WidgetImpl for QuickSettingsButtonWidget {}
impl ButtonImpl for QuickSettingsButtonWidget {}
