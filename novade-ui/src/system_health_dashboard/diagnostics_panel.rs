use gtk4 as gtk;
use gtk::{prelude::*, Box, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, TextView, TextBuffer};
use glib::clone;
use std::sync::Arc;
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticTestStatus};

pub struct DiagnosticsPanel {
    container: Box,
    service: Arc<dyn SystemHealthService>,
    tests_listbox: ListBox,
    results_textview: TextView,
    run_button: Button,
}

impl DiagnosticsPanel {
    pub fn new(service: Arc<dyn SystemHealthService>) -> Self {
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
        run_button.set_sensitive(false); // Initially no test is selected
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
            service,
            tests_listbox,
            results_textview,
            run_button,
        };

        panel.load_available_tests();

        let p_clone = panel.clone(); // Clone for closure
        panel.tests_listbox.connect_row_selected(clone!(@weak p_clone as panel_for_selection => move |_, row_opt| {
            panel_for_selection.run_button.set_sensitive(row_opt.is_some());
        }));

        let p_clone2 = panel.clone();
        panel.run_button.connect_clicked(clone!(@weak p_clone2 as panel_for_run => move |_| {
            if let Some(selected_row) = panel_for_run.tests_listbox.selected_row() {
                // Assuming the DiagnosticTestId is stored as the name of the row widget or similar.
                // For simplicity, let's use a child label's text if the ID is simple.
                // A more robust way is to subclass ListBoxRow and store the ID.
                // Here, we'll try to get it from a data attribute if we set it.
                // For this example, we'll assume test_id is stored in row's name.
                let test_id_str = selected_row.widget_name().to_string();
                if !test_id_str.is_empty() {
                     panel_for_run.run_selected_test(DiagnosticTestId(test_id_str));
                } else {
                    panel_for_run.append_result_text("Error: Could not get Test ID from selected row.\n");
                }
            }
        }));

        panel
    }

    fn load_available_tests(&self) {
        // TODO: UI Test: Verify correct display of DiagnosticTestInfo and DiagnosticTestResult.
        match self.service.list_available_diagnostic_tests() {
            Ok(tests) => {
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
                        // Store the test_id as the widget name for retrieval
                        row.set_widget_name(&test_info.id.0);

                        let row_box = Box::new(Orientation::Vertical, 3);
                        let name_label = Label::builder().label(&test_info.name).halign(gtk::Align::Start).build();
                        let desc_label = Label::builder()
                            .label(&test_info.description)
                            .halign(gtk::Align::Start)
                            .wrap(true)
                            .css_classes(vec!["caption"]) // Use CSS for smaller text if theme supports
                            .build();
                        desc_label.set_opacity(0.8);

                        row_box.append(&name_label);
                        row_box.append(&desc_label);
                        row.set_child(Some(&row_box));
                        self.tests_listbox.append(&row);
                    }
                }
            }
            Err(e) => {
                self.append_result_text(&format!("Error loading diagnostic tests: {}\n", e));
            }
        }
    }

    fn run_selected_test(&self, test_id: DiagnosticTestId) {
        // TODO: UI Test: Simulate test execution and result display.
        self.results_textview.buffer().set_text(&format!("Running test: {}...\n", test_id.0));
        self.run_button.set_sensitive(false); // Disable while running

        let service = self.service.clone();
        let results_textview_clone = self.results_textview.clone();
        let run_button_clone = self.run_button.clone();

        glib::MainContext::default().spawn_local(async move {
            match service.run_diagnostic_test(&test_id).await {
                Ok(result) => {
                    let mut text = format!("Test: {}\nStatus: {:?}\n", result.test_id.0, result.status);
                    if !result.message.is_empty() {
                        text.push_str(&format!("Message: {}\n", result.message));
                    }
                    if let Some(details) = result.details {
                        text.push_str(&format!("Details:\n{}\n", details));
                    }
                    if let Some(duration) = result.duration {
                        text.push_str(&format!("Duration: {:.2?}\n", duration));
                    }
                    results_textview_clone.buffer().set_text(&text);
                }
                Err(e) => {
                    results_textview_clone.buffer().set_text(&format!("Error running test {}: {}\n", test_id.0, e));
                }
            }
            run_button_clone.set_sensitive(true); // Re-enable after completion
        });
    }

    fn append_result_text(&self, text: &str) {
        let buffer = self.results_textview.buffer();
        let mut iter = buffer.end_iter();
        buffer.insert(&mut iter, text);
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

// Required for clone! macro
impl Clone for DiagnosticsPanel {
    fn clone(&self) -> Self {
        // This is a simplified clone for the GTK callbacks.
        // It doesn't deep clone the service, but shares the Arc.
        // GTK widgets themselves are reference counted (implicitly Arc-like).
        // This is generally okay for GTK signal handlers as long as the panel itself
        // is alive when callbacks are invoked or @weak is used correctly.
        Self {
            container: self.container.clone(), // Shallow clone (ref count increment)
            service: self.service.clone(),     // Arc clone
            tests_listbox: self.tests_listbox.clone(),
            results_textview: self.results_textview.clone(),
            run_button: self.run_button.clone(),
        }
    }
}
