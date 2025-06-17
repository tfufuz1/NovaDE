use gtk4 as gtk;
use gtk::{prelude::*, Box, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, TextView, TextBuffer};
use glib::{clone, Variant}; // Added Variant
use crate::system_health_dashboard::view_model::SystemHealthViewModel; // Changed import
use novade_core::types::system_health::{DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticStatus};
use log::debug; // For logging

pub struct DiagnosticsPanel {
    container: Box,
    // service: Arc<dyn SystemHealthService>, // Removed
    view_model: SystemHealthViewModel, // Added
    tests_listbox: ListBox,
    results_textview: TextView,
    run_button: Button,
}

impl DiagnosticsPanel {
    // Constructor now takes SystemHealthViewModel
    pub fn new(view_model: SystemHealthViewModel) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(5).margin_bottom(5).margin_start(5).margin_end(5)
            .build();

        let top_bar = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        let title_label = Label::builder().label("Available Diagnostic Tests").halign(gtk::Align::Start).hexpand(true).build();
        top_bar.append(&title_label);

        let run_button = Button::with_label("Run Selected Test");
        run_button.set_sensitive(false);
        top_bar.append(&run_button);
        container.append(&top_bar);

        let scrolled_window_list = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .min_content_height(150)
            .build();

        let tests_listbox = ListBox::new();
        scrolled_window_list.set_child(Some(&tests_listbox));
        container.append(&scrolled_window_list);

        let results_label = Label::builder().label("Test Results:").halign(gtk::Align::Start).build();
        container.append(&results_label);

        let scrolled_window_results = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();
        let results_textview = TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .monospace(true)
            .build();
        scrolled_window_results.set_child(Some(&results_textview));
        container.append(&scrolled_window_results);

        let panel = Self {
            container,
            view_model, // Store view_model
            tests_listbox,
            results_textview,
            run_button,
        };

        // panel.load_available_tests(); // Removed, will be triggered by MainView via ViewModel

        // Signal connections to ViewModel
        panel.view_model.connect_available_tests_changed(clone!(@weak panel => move |_vm, tests_variant: &Variant| {
            match tests_variant.get::<Vec<DiagnosticTestInfo>>() {
                Some(tests) => panel.populate_available_tests(tests),
                None => {  debug!("DiagnosticsPanel: Failed to get Vec<DiagnosticTestInfo> from Variant for available_tests_changed"); }
            }
        }));

        panel.view_model.connect_test_result_updated(clone!(@weak panel => move |_vm, result: &DiagnosticTestResult| {
            panel.display_test_result(result);
            // Re-enable run button based on current selection, not just any result update.
            // The status_changed signal is better for managing run_button sensitivity during test execution.
            // If no test is running, button sensitivity depends on selection.
            if panel.view_model.property::<Option<DiagnosticTestId>>("running-test-id").is_none() { // Assuming running-test-id is a property
                 panel.run_button.set_sensitive(panel.tests_listbox.selected_row().is_some());
            }
        }));

        panel.view_model.connect_test_status_changed(clone!(@weak panel => move |_vm, test_id: &DiagnosticTestId, status: &DiagnosticStatus| {
            panel.update_test_status_display(test_id, status);
            match status {
                DiagnosticStatus::Running => panel.run_button.set_sensitive(false),
                _ => panel.run_button.set_sensitive(panel.tests_listbox.selected_row().is_some()), // Re-enable based on selection
            }
        }));


        // UI Signal Handlers
        panel.tests_listbox.connect_row_selected(clone!(@weak panel => move |_, row_opt| {
            // Enable run_button only if a test is selected AND no test is currently running
             let is_running = panel.view_model.imp().running_test_id.borrow().is_some(); // Access through imp().field
            panel.run_button.set_sensitive(row_opt.is_some() && !is_running);
        }));

        panel.run_button.connect_clicked(clone!(@weak panel => move |_| {
            if let Some(selected_row) = panel.tests_listbox.selected_row() {
                let test_id_str = selected_row.widget_name().to_string(); // ID stored in widget name
                if !test_id_str.is_empty() {
                    let test_id = DiagnosticTestId(test_id_str);
                    debug!("DiagnosticsPanel: Run button clicked for test ID: {:?}", test_id);
                    panel.results_textview.buffer().set_text(&format!("Running test: {}...\n", test_id.0));
                    // panel.run_button.set_sensitive(false); // ViewModel's status_changed will handle this
                    panel.view_model.run_diagnostic_test(&test_id);
                } else {
                    panel.append_result_text("Error: Could not get Test ID from selected row.\n");
                }
            }
        }));

        panel
    }

    // Method to populate the listbox with available tests
    fn populate_available_tests(&self, tests: Vec<DiagnosticTestInfo>) {
        debug!("DiagnosticsPanel: Populating available tests list with {} tests.", tests.len());
        // Clear existing rows
        while let Some(child) = self.tests_listbox.first_child() {
            self.tests_listbox.remove(&child);
        }

        if tests.is_empty() {
            let row = ListBoxRow::new();
            row.set_child(Some(&Label::new(Some("No diagnostic tests available."))));
            self.tests_listbox.append(&row);
        } else {
            for test_info in tests {
                let row = ListBoxRow::new();
                row.set_widget_name(&test_info.id.0); // Store ID in widget name

                let row_box = Box::new(Orientation::Vertical, 3);
                let name_label = Label::builder().label(&test_info.name).halign(gtk::Align::Start).build();
                let desc_label = Label::builder()
                    .label(&test_info.description)
                    .halign(gtk::Align::Start)
                    .wrap(true)
                    .css_classes(vec!["caption"])
                    .opacity(0.8)
                    .build();

                row_box.append(&name_label);
                row_box.append(&desc_label);
                row.set_child(Some(&row_box));
                self.tests_listbox.append(&row);
            }
        }
        // After populating, if no test is running, set button sensitivity based on selection
        let is_running = self.view_model.imp().running_test_id.borrow().is_some();
        if !is_running {
            self.run_button.set_sensitive(self.tests_listbox.selected_row().is_some());
        }
    }

    // Method to display the result of a single test
    fn display_test_result(&self, result: &DiagnosticTestResult) {
        debug!("DiagnosticsPanel: Displaying test result for ID: {:?}, Status: {:?}", result.id, result.status);
        let mut text = format!("Test: {}\nStatus: {:?}\n", result.id.0, result.status);
        if !result.summary.is_empty() { // Changed from message to summary to match DiagnosticTestResult
            text.push_str(&format!("Summary: {}\n", result.summary));
        }
        if let Some(details) = &result.details {
            text.push_str(&format!("Details:\n{}\n", details));
        }
        // Assuming DiagnosticTestResult might have start_time and end_time in future.
        // For now, they are Option<DateTime<Utc>>.
        if let (Some(start), Some(end)) = (result.start_time, result.end_time) {
             let duration = end.signed_duration_since(start);
             text.push_str(&format!("Duration: {}.{:03}s\n", duration.num_seconds(), duration.num_milliseconds() % 1000));
        }
        self.results_textview.buffer().set_text(&text);
    }

    // Method to update display based on test status (e.g., visual cues in listbox)
    #[allow(unused_variables)] // test_id might be used later for specific row styling
    fn update_test_status_display(&self, test_id: &DiagnosticTestId, status: &DiagnosticStatus) {
        debug!("DiagnosticsPanel: Updating test status display for ID: {:?}, Status: {:?}", test_id, status);
        // This could be used to update the specific row in tests_listbox, e.g., by adding an icon or changing style.
        // For now, it primarily drives button sensitivity via the signal connection.
        if status == &DiagnosticStatus::Running {
            // Optionally clear results or show a global "Running..." message if not per-test.
            // self.results_textview.buffer().set_text(&format!("Test {} is now {:?}...\n", test_id.0, status));
        }
        // If status is Error, Passed, Failed, etc., the display_test_result will provide full details.
    }

    // Helper to append text to results (not directly used by ViewModel signals now)
    fn append_result_text(&self, text: &str) {
        let buffer = self.results_textview.buffer();
        let mut iter = buffer.end_iter();
        buffer.insert(&mut iter, text);
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

// Manual Clone implementation (remains the same as it's not a GObject)
impl Clone for DiagnosticsPanel {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            view_model: self.view_model.clone(), // ViewModel is a GObject, so this is a ref-count clone
            tests_listbox: self.tests_listbox.clone(),
            results_textview: self.results_textview.clone(),
            run_button: self.run_button.clone(),
        }
    }
}
