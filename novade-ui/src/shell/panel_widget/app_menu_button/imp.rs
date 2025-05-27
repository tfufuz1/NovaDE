use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{MenuButton, CompositeTemplate};

use std::cell::RefCell;

use std::cell::RefCell;
use std::rc::Rc; // For Rc<ActiveWindowService>

// Correct the path to ActiveWindowService based on its new location
// Assuming active_window_service.rs is in novade-ui/src/shell/
use crate::shell::active_window_service::ActiveWindowService;


#[derive(CompositeTemplate, Default)]
#[template(string = "")] // No template for now
pub struct AppMenuButton {
    pub active_app_id: RefCell<Option<String>>,
    pub active_window_title: RefCell<Option<String>>,
    pub active_icon_name: RefCell<Option<String>>,
    pub service: RefCell<Option<Rc<ActiveWindowService>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppMenuButton {
    const NAME: &'static str = "NovaDEAppMenuButton";
    type Type = super::AppMenuButton;
    type ParentType = gtk::MenuButton;

    fn new() -> Self {
        Self {
            active_app_id: RefCell::new(None),
            active_window_title: RefCell::new(None),
            active_icon_name: RefCell::new(None),
            service: RefCell::new(None), // Initialize the service field
        }
    }

    fn class_init(klass: &mut Self::Class) {
        // AppMenuButton::bind_template(klass); // No template for now
        klass.set_css_name("appmenubutton");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for AppMenuButton {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Set initial default state
        obj.set_icon_name("application-x-executable-symbolic"); // Default icon
        obj.set_label("Apps"); // Default label
        obj.set_always_show_arrow(false);
        
        // Placeholder for popover/menu
        // let popover = gtk::PopoverMenu::new_from_model(None::<&gio::MenuModel>);
        // obj.set_popover(Some(&popover));
    }

    // No GObject properties defined for these internal states yet
}

impl WidgetImpl for AppMenuButton {}
impl ButtonImpl for AppMenuButton {}
impl MenuButtonImpl for AppMenuButton {}

// Private methods
impl AppMenuButton {
    pub(super) fn update_active_window_info_impl(
        &self,
        app_id: Option<String>,
        window_title: Option<String>,
        icon_name: Option<String>,
    ) {
        let obj = self.obj();

        // Store new state
        self.active_app_id.replace(app_id.clone());
        self.active_window_title.replace(window_title.clone());
        self.active_icon_name.replace(icon_name.clone());

        // Update Icon
        if let Some(icon_name_str) = icon_name.as_ref() {
            obj.set_icon_name(Some(icon_name_str));
        } else {
            obj.set_icon_name("application-x-executable-symbolic"); // Default icon
        }

        // Update Label
        if let Some(title_str) = window_title.as_ref() {
            obj.set_label(title_str);
        } else if let Some(app_id_str) = app_id.as_ref() {
            obj.set_label(app_id_str);
        } else {
            obj.set_label("Apps"); // Default label
        }
    }
}
