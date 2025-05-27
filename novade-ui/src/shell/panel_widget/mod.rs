use glib;
use gtk::glib::subclass::prelude::*;
use gtk::{glib, Application, prelude::*};
use gtk4_layer_shell;

// Re-export enums from imp
pub use self::imp::{ModulePosition, PanelPosition};

pub mod app_menu_button;
pub use app_menu_button::AppMenuButton; // Added for easier access

pub mod workspace_indicator_widget;
pub use workspace_indicator_widget::WorkspaceIndicatorWidget; // Added for easier access

pub mod clock_datetime_widget;
pub use clock_datetime_widget::ClockDateTimeWidget; // Added for easier access

pub mod quick_settings_button; // Added QuickSettingsButtonWidget module
pub use quick_settings_button::QuickSettingsButtonWidget; // Added for easier access

pub mod notification_center_button; // Added NotificationCenterButtonWidget module
pub use notification_center_button::NotificationCenterButtonWidget; // Added for easier access

pub mod quick_settings_panel; // Added QuickSettingsPanelWidget module
pub use quick_settings_panel::QuickSettingsPanelWidget; // Added for easier access

pub mod notification_center_panel; // Added NotificationCenterPanelWidget module
pub use notification_center_panel::NotificationCenterPanelWidget; // Added for easier access

pub mod network_management_widget;
pub use network_management_widget::NetworkManagementWidget;

mod imp;

glib::wrapper! {
    pub struct PanelWidget(ObjectSubclass<imp::PanelWidget>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PanelWidget {
    pub fn new(app: &Application) -> Self {
        let obj: Self = glib::Object::new(&[("application", app)]);
        obj
    }

    pub fn add_module(&self, module: &impl glib::IsA<gtk::Widget>, position: ModulePosition, order: i32) {
        self.imp().add_module_priv(module, position, order);
    }

    pub fn remove_module(&self, module: &impl glib::IsA<gtk::Widget>) {
        self.imp().remove_module_priv(module);
    }
    
    // Internal methods, called from imp.rs or by property changes
    pub(super) fn setup_layer_shell(&self) {
        self.imp().setup_layer_shell_priv();
    }

    pub(super) fn update_layout(&self) {
        self.imp().update_layout_priv();
    }

    pub(super) fn update_transparency(&self) {
        self.imp().update_transparency_priv();
    }

    // Property getters (if needed publicly)
    pub fn position(&self) -> PanelPosition {
        *self.imp().position.borrow()
    }

    pub fn panel_height(&self) -> i32 {
        self.imp().panel_height.get()
    }

    pub fn transparency_enabled(&self) -> bool {
        self.imp().transparency_enabled.get()
    }

    pub fn leuchtakzent_color(&self) -> Option<gdk::RGBA> {
        self.property("leuchtakzent-color")
            .expect("leuchtakzent-color property not found")
            .get()
            .expect("leuchtakzent-color value could not be retrieved")
    }

    pub fn leuchtakzent_intensity(&self) -> f64 {
        self.property("leuchtakzent-intensity")
            .expect("leuchtakzent-intensity property not found")
            .get()
            .expect("leuchtakzent-intensity value could not be retrieved")
    }

    // Property setters (if needed publicly, otherwise they are set via glib::Object::set_property)
    pub fn set_position(&self, position: PanelPosition) {
        self.set_property("position", position.to_value()).unwrap();
    }

    pub fn set_panel_height(&self, height: i32) {
        self.set_property("panel-height", height).unwrap();
    }

    pub fn set_transparency_enabled(&self, enabled: bool) {
        self.set_property("transparency-enabled", enabled).unwrap();
    }

    pub fn set_leuchtakzent_color(&self, color: Option<gdk::RGBA>) {
        self.set_property("leuchtakzent-color", color.to_value()).unwrap();
    }

    pub fn set_leuchtakzent_intensity(&self, intensity: f64) {
        self.set_property("leuchtakzent-intensity", intensity).unwrap();
    }
}
