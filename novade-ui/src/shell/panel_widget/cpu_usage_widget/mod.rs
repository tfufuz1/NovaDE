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

    // Getter for label-format-string
    pub fn label_format_string(&self) -> String {
        self.property("label-format-string")
    }

    // Setter for label-format-string
    pub fn set_label_format_string(&self, format_string: &str) {
        self.set_property("label-format-string", format_string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk::glib; // For property functions

    fn init_gtk() {
        if !gtk::is_initialized() {
            gtk::test_init();
        }
    }

    #[test]
    fn test_cpu_usage_widget_label_format_property() {
        init_gtk();
        let widget = CpuUsageWidget::new();

        // Test default value (though default is set in imp, it's accessible via property)
        let default_format: String = widget.property("label-format-string");
        assert_eq!(default_format, "CPU: {usage}%");
        assert_eq!(widget.label_format_string(), "CPU: {usage}%");

        // Test setting a new value
        let new_format = "Usage: {usage}%%";
        widget.set_label_format_string(new_format);

        let retrieved_format: String = widget.property("label-format-string");
        assert_eq!(retrieved_format, new_format);
        assert_eq!(widget.label_format_string(), new_format);

        // Test setting via set_property
        let another_format = "CPU Load: {usage}";
        widget.set_property("label-format-string", another_format);
        assert_eq!(widget.label_format_string(), another_format);
    }

    // As per subtask, full label update logic test is complex due to async service.
    // A simplified test would require refactoring CpuUsageWidgetImp to expose an update method,
    // or a more complex test setup with a mock service and event loop processing.
    // For now, focusing on property tests as requested.
    #[test]
    fn test_cpu_usage_widget_label_update_simplified() {
        init_gtk();
        let widget = CpuUsageWidget::new();
        let widget_imp = widget.imp();

        // Set a custom format
        let test_format = "Test CPU: {usage}";
        widget.set_label_format_string(test_format);

        // Simulate receiving a CPU update by directly updating `last_known_percentage`
        // and then manually triggering a label refresh if such a method existed.
        // Since `set_property` for "label-format-string" itself triggers a refresh
        // if `last_known_percentage` is Some, let's set that first.

        widget_imp.last_known_percentage.replace(Some(55.5));

        // Now, setting the format string again should trigger the update with the new percentage
        widget.set_label_format_string("CPU is {usage}% now");

        let expected_text = "CPU is 55.5% now";
        assert_eq!(widget_imp.label.borrow().as_ref().unwrap().text(), expected_text);

        // Simulate another update by directly calling parts of the polling logic (conceptually)
        // This part is a bit of a workaround as we are not running the full async machinery.
        let new_percentage = 75.0;
        widget_imp.last_known_percentage.replace(Some(new_percentage));
        // To refresh label with current format and new percentage:
        // We need a method like `refresh_label_from_percentage(percentage)` or rely on set_property.
        // Let's set a different format string to trigger refresh with new percentage.
        widget.set_label_format_string("Load: {usage}%");
        let expected_text_2 = "Load: 75.0%";
         assert_eq!(widget_imp.label.borrow().as_ref().unwrap().text(), expected_text_2);
    }
}
