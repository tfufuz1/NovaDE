use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{Box, Label, Button, Switch, Scale, Spinner, CompositeTemplate, Orientation, Adjustment, Align, prelude::*};
use tracing;
use std::cell::RefCell;
use std::rc::Rc;
use crate::shell::ui_settings_service::UISettingsService;

// Using RefCell<Option<WidgetType>> for widgets stored in the struct,
// as they are initialized in `constructed`.
#[derive(CompositeTemplate, Default)]
#[template(string = "")] 
pub struct QuickSettingsPanelWidget {
    // Store widgets to interact with them for setting initial state and connecting signals
    pub dark_mode_switch: RefCell<Option<Switch>>,
    pub volume_scale: RefCell<Option<Scale>>,
    // wifi_button is not managed by UISettingsService in this phase
    
    pub ui_settings_service: RefCell<Option<Rc<UISettingsService>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for QuickSettingsPanelWidget {
    const NAME: &'static str = "NovaDEQuickSettingsPanelWidget";
    type Type = super::QuickSettingsPanelWidget;
    type ParentType = gtk::Box;

    fn new() -> Self {
        Self {
            dark_mode_switch: RefCell::new(None),
            volume_scale: RefCell::new(None),
            ui_settings_service: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("quicksettingspanelwidget");
        // Note: If using TemplateChild with IDs, you'd bind them here or use a UI file.
        // Since we are using RefCell<Option<Widget>> and manually creating/storing,
        // no binding of TemplateChild is done here.
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for QuickSettingsPanelWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); 

        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(12); 
        
        obj.set_margin_top(10);
        obj.set_margin_bottom(10);
        obj.set_margin_start(10);
        obj.set_margin_end(10);

        // --- Dark Mode Section ---
        let dark_mode_box = Box::new(Orientation::Horizontal, 6);
        dark_mode_box.set_halign(Align::Fill); 
        let dark_mode_label = Label::new(Some("Dark Mode"));
        dark_mode_label.set_halign(Align::Start); 
        dark_mode_label.set_hexpand(true); 
        
        let dark_mode_switch_widget = Switch::new();
        dark_mode_switch_widget.set_halign(Align::End); 
        // Store the widget in the RefCell
        self.dark_mode_switch.replace(Some(dark_mode_switch_widget.clone()));
        
        let service_ref_dm = self.ui_settings_service.clone(); 
        dark_mode_switch_widget.connect_state_set(move |_switch, active| {
            tracing::info!("Dark Mode Switch toggled by UI: {}", active);
            if let Some(service_rc) = service_ref_dm.borrow().as_ref() {
                let service_clone = service_rc.clone();
                service_clone.tokio_handle().spawn(async move {
                    service_clone.set_dark_mode(active).await;
                });
            }
            glib::Propagation::Stop 
        });
        
        dark_mode_box.append(&dark_mode_label);
        dark_mode_box.append(&dark_mode_switch_widget); 
        obj.append(&dark_mode_box);

        // --- Volume Section ---
        let volume_box = Box::new(Orientation::Horizontal, 6);
        volume_box.set_halign(Align::Fill);
        let volume_label = Label::new(Some("Volume"));
        volume_label.set_halign(Align::Start);
        
        let volume_adjustment = Adjustment::new(50.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let volume_scale_widget = Scale::new(Orientation::Horizontal, Some(&volume_adjustment)); 
        volume_scale_widget.set_hexpand(true); 
        volume_scale_widget.set_halign(Align::Fill);
        volume_scale_widget.set_draw_value(false); 
        // Store the widget in the RefCell
        self.volume_scale.replace(Some(volume_scale_widget.clone()));
        
        let service_ref_vol = self.ui_settings_service.clone();
        volume_scale_widget.connect_value_changed(move |scale| {
            let value = scale.value();
            tracing::info!("Volume Scale changed by UI: {}", value);
            if let Some(service_rc) = service_ref_vol.borrow().as_ref() {
                let service_clone = service_rc.clone();
                service_clone.tokio_handle().spawn(async move {
                    service_clone.set_volume(value).await;
                });
            }
        });

        volume_box.append(&volume_label);
        volume_box.append(&volume_scale_widget); 
        obj.append(&volume_box);

        // --- WiFi Section (unchanged functionality for this task) ---
        let wifi_box = Box::new(Orientation::Horizontal, 6);
        wifi_box.set_halign(Align::Fill);
        let wifi_label = Label::new(Some("WiFi"));
        wifi_label.set_halign(Align::Start);
        wifi_label.set_hexpand(true); 
        
        let wifi_status_box = Box::new(Orientation::Horizontal, 6); 
        let wifi_spinner = Spinner::new();
        let wifi_button = Button::with_label("Select Network...");
        wifi_button.connect_clicked(|_button| {
            tracing::info!("WiFi 'Select Network...' button clicked");
        });
        
        wifi_status_box.append(&wifi_spinner); 
        wifi_status_box.append(&wifi_button);
        wifi_status_box.set_halign(Align::End);

        wifi_box.append(&wifi_label);
        wifi_box.append(&wifi_status_box);
        obj.append(&wifi_box);
        
        obj.set_width_request(280); 
    }
}

impl WidgetImpl for QuickSettingsPanelWidget {}
impl BoxImpl for QuickSettingsPanelWidget {}
