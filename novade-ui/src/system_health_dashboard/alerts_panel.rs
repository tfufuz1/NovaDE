use gtk4 as gtk;
use gtk::{prelude::*, Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Button};
use glib::clone;
use std::sync::Arc;
use std::time::Duration;
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{Alert, AlertSeverity};

pub struct AlertsPanel {
    container: Box,
    service: Arc<dyn SystemHealthService>,
    alerts_listbox: ListBox,
}

impl AlertsPanel {
    pub fn new(service: Arc<dyn SystemHealthService>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(5).margin_bottom(5).margin_start(5).margin_end(5)
            .build();

        let title_label = Label::builder().label("Active Alerts").halign(gtk::Align::Start).build();
        container.append(&title_label);

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let alerts_listbox = ListBox::new();
        alerts_listbox.set_selection_mode(gtk::SelectionMode::None); // No selection needed for now
        scrolled_window.set_child(Some(&alerts_listbox));
        container.append(&scrolled_window);

        let panel = Self {
            container,
            service,
            alerts_listbox,
        };

        panel.start_updates();
        panel
    }

    fn start_updates(&self) {
        // Update immediately once
        self.update_alerts_display();

        // Periodically update alerts
        glib::timeout_add_local_ बार(Duration::from_secs(5), clone!(@weak self as panel => @default-return glib::ControlFlow::Break, move || {
            panel.update_alerts_display();
            glib::ControlFlow::Continue
        }));
    }

    fn update_alerts_display(&self) {
        // TODO: UI Test: Verify correct display of mock Alert list.
        let service = self.service.clone();
        let listbox_clone = self.alerts_listbox.clone();

        glib::MainContext::default().spawn_local(async move {
            // Clear existing rows
            while let Some(child) = listbox_clone.first_child() {
                listbox_clone.remove(&child);
            }

            match service.get_active_alerts().await {
                Ok(alerts) => {
                    if alerts.is_empty() {
                        let row = ListBoxRow::new();
                        row.set_child(Some(&Label::new(Some("No active alerts."))));
                        listbox_clone.append(&row);
                    } else {
                        for alert in alerts {
                            let row = ListBoxRow::new();
                            let row_box = Box::new(Orientation::Vertical, 3);
                            row_box.set_margin_top(5);
                            row_box.set_margin_bottom(5);
                            row_box.set_margin_start(5);
                            row_box.set_margin_end(5);

                            let name_markup = format!(
                                "<span weight='bold' color='{}'>{} ({})</span>",
                                match alert.severity {
                                    AlertSeverity::Critical => "red",
                                    AlertSeverity::Warning => "orange",
                                    AlertSeverity::Info => "blue",
                                },
                                alert.name,
                                alert.severity.to_string().to_uppercase()
                            );
                            let name_label = Label::builder()
                                .use_markup(true)
                                .label(&name_markup)
                                .halign(gtk::Align::Start)
                                .build();

                            let desc_label = Label::builder()
                                .label(&alert.description)
                                .halign(gtk::Align::Start)
                                .wrap(true)
                                .selectable(true)
                                .build();

                            let time_label = Label::builder()
                                .label(&format!("Time: {}", alert.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)))
                                .halign(gtk::Align::End)
                                .css_classes(vec!["caption"])
                                .build();
                            time_label.set_opacity(0.7);

                            row_box.append(&name_label);
                            row_box.append(&desc_label);
                            row_box.append(&time_label);
                            // TODO: Add Acknowledge button per alert if needed
                            row.set_child(Some(&row_box));
                            listbox_clone.append(&row);
                        }
                    }
                }
                Err(e) => {
                    let row = ListBoxRow::new();
                    let error_label = Label::new(Some(&format!("Error fetching alerts: {}", e)));
                    error_label.set_halign(gtk::Align::Start);
                    row.set_child(Some(&error_label));
                    listbox_clone.append(&row);
                }
            }
        });
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

// Required for clone! macro if we were to use it more complexly,
// but for simple timeout, @weak self is fine.
// If we needed to pass Arc<Self> around more, this might be needed.
// impl Clone for AlertsPanel { ... }
