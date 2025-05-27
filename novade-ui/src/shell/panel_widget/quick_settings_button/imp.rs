use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate}; // Added Button

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now, as it's a simple button
pub struct QuickSettingsButtonWidget {
    // Struct can be empty for this stub
}

#[glib::object_subclass]
impl ObjectSubclass for QuickSettingsButtonWidget {
    const NAME: &'static str = "NovaDEQuickSettingsButtonWidget";
    type Type = super::QuickSettingsButtonWidget;
    type ParentType = gtk::Button;

    fn class_init(klass: &mut Self::Class) {
        // QuickSettingsButtonWidget::bind_template(klass); // No template for now
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
        // obj.set_label(""); // Ensure no label if it's an icon-only button
                            // set_icon_name usually makes it icon-only if label is not set
    }

    // No properties defined yet
    // fn properties() -> &'static [glib::ParamSpec] { &[] }
    // fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) { unimplemented!() }
    // fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value { unimplemented!() }

    // No signals defined yet
    // fn signals() -> &'static [Signal] { &[] }
}

impl WidgetImpl for QuickSettingsButtonWidget {}
impl ButtonImpl for QuickSettingsButtonWidget {}
