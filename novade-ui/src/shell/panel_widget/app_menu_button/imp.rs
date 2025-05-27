use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{MenuButton, CompositeTemplate};

use std::cell::RefCell;

use std::cell::RefCell;
use std::rc::Rc; // For Rc<ActiveWindowService>

// Correct the path to ActiveWindowService 
use crate::shell::active_window_service::ActiveWindowService;
// Import AppMenuService
use crate::shell::app_menu_service::AppMenuService;


#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct AppMenuButton {
    pub active_app_id: RefCell<Option<String>>,
    pub active_window_title: RefCell<Option<String>>,
    pub active_icon_name: RefCell<Option<String>>,
    pub active_window_service: RefCell<Option<Rc<ActiveWindowService>>>, // Renamed from 'service' for clarity
    pub app_menu_service: RefCell<Option<Rc<AppMenuService>>>, // New service field
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
            active_window_service: RefCell::new(None), 
            app_menu_service: RefCell::new(None), // Initialize new service field
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
use gtk::gio; // Ensure gio is imported for MenuModel

// Private methods
impl AppMenuButton {
    pub(super) fn update_active_window_info_impl(
        &self,
        app_id: Option<String>, // app_id is mainly for internal state tracking now
        window_title: Option<String>,
        icon_name: Option<String>,
        // menu_model parameter is removed from here, will be set asynchronously
    ) {
        let obj = self.obj();

        // Store new state for app_id, title, icon
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
            // Fallback to app_id for label if title is None
            obj.set_label(app_id_str);
        } else {
            obj.set_label("Apps"); // Default label
        }
        
        // Menu Model is no longer set directly here.
        // It's set by refresh_display in mod.rs after fetching from AppMenuService.
    }
}
