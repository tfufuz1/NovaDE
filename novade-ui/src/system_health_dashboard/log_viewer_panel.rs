use gtk4 as gtk;
use gtk::{prelude::*, Box, Button, ComboBoxText, Entry, Orientation, ScrolledWindow, TextView, TextBuffer, TextTagTable, TextTag};
use glib::clone;
use std::sync::Arc;
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{LogFilter, LogLevel, TimeRange}; // Assuming TimeRange might be added later

pub struct LogViewerPanel {
    container: Box,
    service: Arc<dyn SystemHealthService>,
    log_textview: TextView,
    keyword_filter_entry: Entry,
    level_filter_combo: ComboBoxText,
    // source_filter_entry: Entry, // Example for another filter
}

impl LogViewerPanel {
    pub fn new(service: Arc<dyn SystemHealthService>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(5).margin_bottom(5).margin_start(5).margin_end(5)
            .build();

        // Filter controls
        let filter_bar = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        let keyword_filter_entry = Entry::builder()
            .placeholder_text("Keywords...")
            .hexpand(true)
            .build();
        filter_bar.append(&keyword_filter_entry);

        let level_filter_combo = ComboBoxText::new();
        level_filter_combo.append(Some("ALL"), "All Levels");
        level_filter_combo.append(Some(LogLevel::Error.to_string().as_str()), "Error+"); // Error, Critical, Alert, Emergency
        level_filter_combo.append(Some(LogLevel::Warning.to_string().as_str()), "Warning+");
        level_filter_combo.append(Some(LogLevel::Info.to_string().as_str()), "Info+");
        level_filter_combo.append(Some(LogLevel::Debug.to_string().as_str()), "Debug+");
        level_filter_combo.set_active_id(Some("ALL")); // Default to All
        filter_bar.append(&level_filter_combo);

        let query_button = Button::with_label("Query Logs");
        filter_bar.append(&query_button);
        container.append(&filter_bar);

        // Log display area
        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let tag_table = TextTagTable::new();
        // Add tags for different log levels if desired for styling
        let error_tag = TextTag::new(Some("error"));
        error_tag.set_foreground(Some("red"));
        tag_table.add(&error_tag);
        let warning_tag = TextTag::new(Some("warning"));
        warning_tag.set_foreground(Some("orange"));
        tag_table.add(&warning_tag);

        let log_buffer = TextBuffer::new(Some(&tag_table));
        let log_textview = TextView::builder()
            .buffer(&log_buffer)
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .build();
        scrolled_window.set_child(Some(&log_textview));
        container.append(&scrolled_window);

        let panel = Self {
            container,
            service,
            log_textview,
            keyword_filter_entry,
            level_filter_combo,
        };

        query_button.connect_clicked(clone!(@weak panel => move |_| {
            panel.query_and_display_logs();
        }));

        // Initial log load (optional)
        // panel.query_and_display_logs();

        panel
    }

    fn query_and_display_logs(&self) {
        // TODO: Unit test LogFilter construction from UI inputs.
        let keywords_text = self.keyword_filter_entry.text().to_string();
        let keywords = if keywords_text.trim().is_empty() {
            None
        } else {
            Some(keywords_text.split_whitespace().map(String::from).collect::<Vec<String>>())
        };

        let min_level = self.level_filter_combo.active_id().and_then(|id_str| {
            match id_str.as_str() {
                "ALL" => None,
                "Error+" => Some(LogLevel::Error),
                "Warning+" => Some(LogLevel::Warning),
                "Info+" => Some(LogLevel::Info),
                "Debug+" => Some(LogLevel::Debug),
                _ => LogLevel::try_from(id_str.as_ref()).ok() // Direct match if ID is a LogLevel string
            }
        });

        let filter = LogFilter {
            level: min_level,
            keywords,
            component_filter: None, // TODO: Add UI for component filter if needed
        };

        let service = self.service.clone();
        let log_buffer = self.log_textview.buffer();

        // TODO: UI Test: Verify correct display of mock LogEntry list.
        glib::MainContext::default().spawn_local(async move {
            log_buffer.set_text(""); // Clear previous logs
            match service.query_logs(Some(filter), None /* TimeRange */, Some(1000) /* Limit */).await {
                Ok(log_entries) => {
                    let mut iter = log_buffer.end_iter();
                    if log_entries.is_empty() {
                        log_buffer.insert(&mut iter, "No log entries found matching criteria.\n");
                    } else {
                        for entry in log_entries {
                            let tag_name = match entry.level {
                                LogLevel::Error | LogLevel::Critical | LogLevel::Alert | LogLevel::Emergency => Some("error"),
                                LogLevel::Warning => Some("warning"),
                                _ => None,
                            };
                            // Format: Timestamp [LEVEL] Component: Message
                            let line = format!(
                                "{} [{}] {}: {}\n",
                                entry.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                                entry.level.to_string().to_uppercase(),
                                entry.component,
                                entry.message
                            );
                            if let Some(tag) = tag_name {
                                log_buffer.insert_with_tags_by_name(&mut iter, &line, &[tag]);
                            } else {
                                log_buffer.insert(&mut iter, &line);
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut iter = log_buffer.end_iter();
                    log_buffer.insert_with_tags_by_name(&mut iter, &format!("Error fetching logs: {}\n", e), &["error"]);
                }
            }
        });
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}
