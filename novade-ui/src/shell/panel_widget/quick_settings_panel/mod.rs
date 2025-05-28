use gtk::glib;
use gtk::glib::subclass::prelude::*;
use gtk::{prelude::*, Box}; 
use std::rc::Rc;
use crate::shell::ui_settings_service::UISettingsService;
use tracing; // For logging

mod imp;

glib::wrapper! {
    pub struct QuickSettingsPanelWidget(ObjectSubclass<imp::QuickSettingsPanelWidget>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl QuickSettingsPanelWidget {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_ui_settings_service(&self, service: Rc<UISettingsService>) {
        self.imp().ui_settings_service.replace(Some(service.clone()));

        // Fetch and set initial dark_mode state
        if let Some(dark_mode_switch_opt) = self.imp().dark_mode_switch.borrow().as_ref() {
            let dark_mode_switch_clone = dark_mode_switch_opt.clone(); // Clone the Switch widget
            let service_clone_dark = service.clone();
            let tokio_handle_dark = service.tokio_handle().clone(); 

            tokio_handle_dark.spawn(glib::clone!(@weak dark_mode_switch_clone => async move {
                let is_dark = service_clone_dark.get_dark_mode().await;
                glib::MainContext::default().invoke(move || {
                    // Block signal to prevent re-triggering the service call
                    let handler_id = dark_mode_switch_clone.block_signal_by_name("state-set");
                    dark_mode_switch_clone.set_active(is_dark);
                    if let Some(id) = handler_id {
                        dark_mode_switch_clone.unblock_signal(id);
                    }
                    tracing::info!("QuickSettingsPanel: Initial dark mode switch set to: {}", is_dark);
                });
            }));
        }


        // Fetch and set initial volume state
        if let Some(volume_scale_opt) = self.imp().volume_scale.borrow().as_ref() {
            let volume_scale_clone = volume_scale_opt.clone(); // Clone the Scale widget
            let service_clone_vol = service.clone();
            let tokio_handle_vol = service.tokio_handle().clone();

            tokio_handle_vol.spawn(glib::clone!(@weak volume_scale_clone => async move {
                let vol = service_clone_vol.get_volume().await;
                glib::MainContext::default().invoke(move || {
                    // Block signal to prevent re-triggering the service call
                    let handler_id = volume_scale_clone.block_signal_by_name("value-changed");
                    volume_scale_clone.set_value(vol);
                     if let Some(id) = handler_id {
                        volume_scale_clone.unblock_signal(id);
                    }
                    tracing::info!("QuickSettingsPanel: Initial volume scale set to: {}", vol);
                });
            }));
        }
    }
}
