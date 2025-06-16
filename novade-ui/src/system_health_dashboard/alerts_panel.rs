use gtk4 as gtk;
use gtk::{prelude::*, Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Button, CheckButton, ComboBoxText};
use glib::{clone, MainContext};
use std::sync::Arc;
use std::time::Duration;
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{Alert, AlertSeverity, AlertId};

pub struct AlertsPanel {
    container: Box,
    service: Arc<dyn SystemHealthService>,
    alerts_listbox: ListBox,
    show_acknowledged_button: CheckButton,
    sort_combo: ComboBoxText,
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

        // Controls Bar (Filtering and Sorting)
        let controls_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_bottom(5) // Add some space below controls
            .build();

        let show_acknowledged_button = CheckButton::with_label("Show Acknowledged");
        show_acknowledged_button.set_active(true); // Default to showing acknowledged
        controls_box.append(&show_acknowledged_button);

        let sort_combo = ComboBoxText::new();
        sort_combo.append(Some("time_newest"), "Time (Newest First)");
        sort_combo.append(Some("severity_high"), "Severity (High First)");
        sort_combo.append(Some("name_asc"), "Name (A-Z)");
        sort_combo.set_active_id(Some("time_newest")); // Default sort
        controls_box.append(&sort_combo);
        container.append(&controls_box);

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let alerts_listbox = ListBox::new();
        alerts_listbox.set_selection_mode(gtk::SelectionMode::None);
        scrolled_window.set_child(Some(&alerts_listbox));
        container.append(&scrolled_window);

        let panel = Self {
            container,
            service,
            alerts_listbox,
            show_acknowledged_button: show_acknowledged_button.clone(), // Clone for struct field
            sort_combo: sort_combo.clone(), // Clone for struct field
        };

        panel.start_updates();

        // Connect signals for filtering and sorting
        show_acknowledged_button.connect_toggled(clone!(@weak panel => move |_| {
            panel.update_alerts_display();
        }));
        sort_combo.connect_changed(clone!(@weak panel => move |_| {
            panel.update_alerts_display();
        }));

        panel
    }

    fn start_updates(&self) {
        self.update_alerts_display(); // Initial update
        glib::timeout_add_local_ बार(Duration::from_secs(5), clone!(@weak self as panel => @default-return glib::ControlFlow::Break, move || {
            panel.update_alerts_display();
            glib::ControlFlow::Continue
        }));
    }

    fn update_alerts_display(&self) {
        let service = self.service.clone();
        // let listbox_clone = self.alerts_listbox.clone(); // Not needed if using panel_ref
        let show_acknowledged = self.show_acknowledged_button.is_active();
        let sort_active_id = self.sort_combo.active_id().map(|id| id.to_string());

        MainContext::default().spawn_local(clone!(@weak self as panel_ref => async move {
            while let Some(child) = panel_ref.alerts_listbox.first_child() {
                panel_ref.alerts_listbox.remove(&child);
            }

            match panel_ref.service.get_active_alerts().await {
                Ok(mut alerts) => {
                    if !show_acknowledged {
                        alerts.retain(|alert| !alert.acknowledged);
                    }

                    if let Some(sort_id) = sort_active_id {
                        match sort_id.as_str() {
                            "time_newest" => alerts.sort_by(|a, b| b.last_triggered_timestamp.cmp(&a.last_triggered_timestamp)),
                            "severity_high" => alerts.sort_by_key(|a| std::cmp::Reverse(a.severity.clone())),
                            "name_asc" => alerts.sort_by(|a, b| a.name.cmp(&b.name)),
                            _ => {}
                        }
                    }

                    if alerts.is_empty() {
                        let row = ListBoxRow::new();
                        let message = if !show_acknowledged || sort_active_id.is_some() {
                            "No alerts match current criteria."
                        } else {
                            "No active alerts."
                        };
                        row.set_child(Some(&Label::new(Some(message))));
                        panel_ref.alerts_listbox.append(&row);
                    } else {
                        for alert in alerts {
                            let row = ListBoxRow::new();
                            let row_box = Box::new(Orientation::Vertical, 3);
                            row_box.set_margin_top(5);
                            row_box.set_margin_bottom(5);
                            row_box.set_margin_start(5);
                            row_box.set_margin_end(5);

                            let mut name_markup = format!(
                                "<span weight='bold' color='{}'>{} ({})</span>",
                                match alert.severity {
                                    AlertSeverity::Critical | AlertSeverity::High => "red",
                                    AlertSeverity::Medium => "orange",
                                    AlertSeverity::Low => "blue",
                                },
                                alert.name,
                                alert.severity.to_string().to_uppercase()
                            );
                            if alert.acknowledged {
                                name_markup = format!("{} [Acknowledged]", name_markup);
                            }

                            let name_label = Label::builder().use_markup(true).label(&name_markup).halign(gtk::Align::Start).build();
                            let message_label = Label::builder().label(&alert.message).halign(gtk::Align::Start).wrap(true).selectable(true).build();
                            let source_label = Label::builder().label(&format!("Source: {}", alert.source_metric_or_log)).halign(gtk::Align::Start).css_classes(vec!["caption"]).build();
                            source_label.set_opacity(0.7);
                            let time_label_text = format!("Last: {} (First: {})",
                                alert.last_triggered_timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                                alert.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                            );
                            let time_label = Label::builder().label(&time_label_text).halign(gtk::Align::End).css_classes(vec!["caption"]).build();
                            time_label.set_opacity(0.7);

                            row_box.append(&name_label);
                            row_box.append(&message_label);
                            row_box.append(&source_label);
                            row_box.append(&time_label);

                            if !alert.acknowledged {
                                let ack_button = Button::with_label("Acknowledge");
                                let panel_weak_for_button = glib::ብዙ<AlertsPanel>::downgrade(&panel_ref);
                                let service_for_button = panel_ref.service.clone();
                                let alert_id_for_button = alert.id.clone();

                                ack_button.connect_clicked(move |_| {
                                    if let Some(panel_strong_for_button) = panel_weak_for_button.upgrade() {
                                        let srv = service_for_button.clone();
                                        let id_val = alert_id_for_button.0.clone();
                                        MainContext::default().spawn_local(async move {
                                            match srv.acknowledge_alert(id_val).await {
                                                Ok(_) => panel_strong_for_button.update_alerts_display(),
                                                Err(e) => eprintln!("Failed to acknowledge alert: {}", e),
                                            }
                                        });
                                    }
                                });
                                row_box.append(&ack_button);
                            }
                            row.set_child(Some(&row_box));
                            panel_ref.alerts_listbox.append(&row);
                        }
                    }
                }
                Err(e) => {
                    let row = ListBoxRow::new();
                    let error_label = Label::new(Some(&format!("Error fetching alerts: {}", e)));
                    error_label.set_halign(gtk::Align::Start);
                    row.set_child(Some(&error_label));
                    panel_ref.alerts_listbox.append(&row);
                }
            }
        }));
    }

    pub fn get_widget(&self) -> &Box {
        &self.container
    }
}

// TODO: Add UI integration tests for alert display, acknowledgement, filtering, and sorting.
// TODO: If sorting/filtering logic in `update_alerts_display` becomes very complex,
//       consider extracting it into testable helper functions.
