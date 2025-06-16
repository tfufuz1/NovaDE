use gtk4 as gtk;
use gtk::{prelude::*, Box, Button, ComboBoxText, Entry, Orientation, ScrolledWindow, TextView, TextBuffer, TextTagTable, TextTag};
use glib::{clone, MainContext};
use std::sync::{Arc, Mutex}; // Added Mutex for JoinHandle
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{LogFilter, LogPriority, LogEntry}; // Updated LogLevel to LogPriority
use futures_util::StreamExt; // For stream.next()

pub struct LogViewerPanel {
    container: Box,
    service: Arc<dyn SystemHealthService>,
    log_textview: TextView,
    keyword_filter_entry: Entry,
    level_filter_combo: ComboBoxText,
    stream_start_button: Button,
    stream_stop_button: Button,
    streaming_task_handle: Arc<Mutex<Option<glib::JoinHandle<()>>>>,
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
        level_filter_combo.append(Some(LogPriority::Error.to_string().as_str()), "Error+");
        level_filter_combo.append(Some(LogPriority::Warning.to_string().as_str()), "Warning+");
        level_filter_combo.append(Some(LogPriority::Info.to_string().as_str()), "Info+");
        level_filter_combo.append(Some(LogPriority::Debug.to_string().as_str()), "Debug+");
        level_filter_combo.set_active_id(Some("ALL")); // Default to All
        filter_bar.append(&level_filter_combo);

        let query_button = Button::with_label("Query Logs");
        filter_bar.append(&query_button);
        container.append(&filter_bar);

        // Streaming controls
        let stream_controls_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        let stream_start_button = Button::with_label("Start Streaming");
        stream_controls_box.append(&stream_start_button);

        let stream_stop_button = Button::builder()
            .label("Stop Streaming")
            .sensitive(false) // Initially disabled
            .build();
        stream_controls_box.append(&stream_stop_button);
        container.append(&stream_controls_box);

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
            stream_start_button: stream_start_button.clone(), // Clone for struct storage
            stream_stop_button: stream_stop_button.clone(),   // Clone for struct storage
            streaming_task_handle: Arc::new(Mutex::new(None)),
        };

        query_button.connect_clicked(clone!(@weak panel => move |_| {
            panel.query_and_display_logs();
        }));

        stream_start_button.connect_clicked(clone!(@weak panel => move |_| {
            panel.start_log_streaming();
        }));

        stream_stop_button.connect_clicked(clone!(@weak panel => move |_| {
            panel.stop_log_streaming();
        }));

        panel
    }

    fn get_current_filter(&self) -> LogFilter {
        let keywords_text = self.keyword_filter_entry.text().to_string();
        let keywords = if keywords_text.trim().is_empty() {
            None
        } else {
            Some(keywords_text.split_whitespace().map(String::from).collect::<Vec<String>>())
        };

        let min_priority = self.level_filter_combo.active_id().and_then(|id_str| {
            match id_str.as_str() {
                "ALL" => None,
                // Assuming LogPriority::Error.to_string() matches these IDs
                _ => LogPriority::try_from(id_str.as_ref()).ok(),
            }
        });

        LogFilter {
            sources: None, // Not implemented in UI yet
            min_priority,
            time_range: None, // Not implemented in UI for streaming/querying yet
            keywords,
            field_filters: None, // Not implemented in UI yet
        }
    }

    fn append_log_entry_to_buffer(&self, entry: &LogEntry, buffer: &TextBuffer) {
        let mut iter = buffer.end_iter();
        let tag_name = match entry.priority {
            LogPriority::Critical | LogPriority::Error => Some("error"),
            LogPriority::Warning => Some("warning"),
            _ => None,
        };
        // Format: Timestamp [LEVEL] Component: Message
        let line = format!(
            "{} [{}] {}: {}\n",
            entry.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            entry.priority.to_string().to_uppercase(),
            entry.source_component, // Updated field name
            entry.message
        );
        if let Some(tag) = tag_name {
            buffer.insert_with_tags_by_name(&mut iter, &line, &[tag]);
        } else {
            buffer.insert(&mut iter, &line);
        }
        // Auto-scroll
        if let Some(adj) = self.log_textview.parent().and_then(|p| p.downcast::<ScrolledWindow>().ok()).map(|sw| sw.vadjustment()) {
            adj.set_value(adj.upper() - adj.page_size());
        }
    }


    fn query_and_display_logs(&self) {
        let filter = self.get_current_filter();
        let service = self.service.clone();
        let log_buffer = self.log_textview.buffer();
        let panel_clone = self.clone_for_callback(); // Helper to clone Arcs for async

        MainContext::default().spawn_local(async move {
            log_buffer.set_text(""); // Clear previous logs
            match service.query_logs(Some(filter), None, Some(1000)).await {
                Ok(log_entries) => {
                    if log_entries.is_empty() {
                         let mut iter = log_buffer.end_iter();
                        log_buffer.insert(&mut iter, "No log entries found matching criteria.\n");
                    } else {
                        for entry in log_entries {
                           panel_clone.append_log_entry_to_buffer(&entry, &log_buffer);
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

    fn start_log_streaming(&self) {
        self.stream_start_button.set_sensitive(false);
        self.stream_stop_button.set_sensitive(true);

        let filter = self.get_current_filter();
        let service = self.service.clone();
        let log_buffer = self.log_textview.buffer();
        log_buffer.set_text("Starting log stream...\n"); // Clear previous and indicate streaming

        let panel_clone = self.clone_for_callback();
        let streaming_task_handle_clone = self.streaming_task_handle.clone();

        let stream_future = async move {
            match service.stream_logs(Some(filter)).await {
                Ok(mut stream) => {
                    while let Some(log_result) = stream.next().await {
                        match log_result {
                            Ok(entry) => {
                                panel_clone.append_log_entry_to_buffer(&entry, &log_buffer);
                            }
                            Err(e) => {
                                let mut iter = log_buffer.end_iter();
                                log_buffer.insert_with_tags_by_name(&mut iter, &format!("Error in log stream: {}\n", e), &["error"]);
                            }
                        }
                    }
                    // Stream ended normally
                    MainContext::default().invoke(move || {
                         let mut iter = log_buffer.end_iter();
                         log_buffer.insert(&mut iter, "\nLog stream ended.\n");
                         panel_clone.stream_start_button.set_sensitive(true);
                         panel_clone.stream_stop_button.set_sensitive(false);
                         if let Ok(mut handle_opt) = panel_clone.streaming_task_handle.lock() {
                            *handle_opt = None;
                         }
                    });
                }
                Err(e) => {
                     MainContext::default().invoke(move || {
                        let mut iter = log_buffer.end_iter();
                        log_buffer.insert_with_tags_by_name(&mut iter, &format!("Failed to start log stream: {}\n", e), &["error"]);
                        panel_clone.stream_start_button.set_sensitive(true);
                        panel_clone.stream_stop_button.set_sensitive(false);
                        if let Ok(mut handle_opt) = panel_clone.streaming_task_handle.lock() {
                            *handle_opt = None;
                        }
                    });
                }
            }
        };

        let join_handle = MainContext::default().spawn_local_with_priority(glib::Priority::DEFAULT_IDLE, stream_future);

        if let Ok(mut handle_opt) = streaming_task_handle_clone.lock() {
            // If there was an old handle, it should have been cleared or aborted.
            // For safety, ensure it's cleared before setting a new one.
            if let Some(old_handle) = handle_opt.take() {
                old_handle.abort();
            }
            *handle_opt = Some(join_handle);
        }
    }

    fn stop_log_streaming(&self) {
        if let Ok(mut handle_opt) = self.streaming_task_handle.lock() {
            if let Some(handle) = handle_opt.take() {
                handle.abort();
                // The task itself should handle UI updates upon abortion/completion.
                // For immediate feedback:
                self.log_textview.buffer().insert_at_cursor("\nLog stream stopped by user.\n");
            }
        }
        self.stream_start_button.set_sensitive(true);
        self.stream_stop_button.set_sensitive(false);
    }

    // Helper to clone necessary Arcs for async callbacks
    // This is a bit manual; a macro or a more structured approach could be used in larger apps.
    fn clone_for_callback(&self) -> Self {
        // This clone is only for passing to async blocks where panel's lifetime might be an issue.
        // It clones the Arcs and essential GTK widgets that are cheap to clone.
        Self {
            container: self.container.clone(), // Might not be needed in callback
            service: self.service.clone(),
            log_textview: self.log_textview.clone(),
            keyword_filter_entry: self.keyword_filter_entry.clone(), // For filters if needed
            level_filter_combo: self.level_filter_combo.clone(),   // For filters if needed
            stream_start_button: self.stream_start_button.clone(),
            stream_stop_button: self.stream_stop_button.clone(),
            streaming_task_handle: self.streaming_task_handle.clone(),
        }
    }


    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

// TODO: Add UI integration tests for log streaming start/stop and log display updates.
// TODO: Consider testing filter logic in `get_current_filter` if it becomes more complex.
