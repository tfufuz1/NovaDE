use gtk::glib;
use gtk::subclass::prelude::*;
use std::sync::Arc;
// Adjust use path if novade_domain is not directly accessible or is part of a workspace
// This might require: use novade_domain::cpu_usage_service::ICpuUsageService;
// For now, assuming direct path works or will be resolved by Rust compiler with proper Cargo.toml
use novade_domain::cpu_usage_service::ICpuUsageService;


mod imp;

glib::wrapper! {
    pub struct CpuUsageWidget(ObjectSubclass<imp::CpuUsageWidgetImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CpuUsageWidget {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    // Method to inject the service
    pub fn set_cpu_usage_service(&self, service: Arc<dyn ICpuUsageService>) {
        self.imp().set_cpu_usage_service(service);
    }

    // Method to initiate the subscription process
    // This should be called after the widget is part of a realized UI
    // and the service has been set.
    pub fn start_subscription(&self) {
         self.imp().start_subscription_task();
    }
}
