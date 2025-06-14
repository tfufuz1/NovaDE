use gtk4 as gtk;
use gtk::{prelude::*, Grid, Label, Orientation, ScrolledWindow};
use glib::clone;
use std::sync::Arc;
use std::time::Duration;
use novade_domain::system_health_service::SystemHealthService;
use novade_core::types::system_health::{CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric};

// Helper struct to hold labels for a specific disk/network interface
struct ResourceLabels {
    name: Label,
    val1: Label, // e.g. reads/s or sent/s or total space
    val2: Label, // e.g. writes/s or received/s or used space
    val3: Label, // e.g. busy time or free space
}

pub struct MetricsPanel {
    container: Grid,
    service: Arc<dyn SystemHealthService>,
    cpu_total_usage_label: Label,
    cpu_core_grid: Grid, // For per-core usage
    memory_ram_label: Label,
    memory_swap_label: Label,
    disk_activity_grid: Grid,
    disk_space_grid: Grid,
    network_activity_grid: Grid,
    temperature_grid: Grid,
    // Store labels for dynamic resources
    // disk_activity_labels: RefCell<HashMap<String, ResourceLabels>>,
    // disk_space_labels: RefCell<HashMap<String, ResourceLabels>>,
    // network_labels: RefCell<HashMap<String, ResourceLabels>>,
    // temperature_labels: RefCell<HashMap<String, ResourceLabels>>,
}

impl MetricsPanel {
    pub fn new(service: Arc<dyn SystemHealthService>) -> Self {
        let container = Grid::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(10)
            .column_spacing(6)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .build();

        // --- CPU Metrics ---
        let cpu_frame = create_section_frame("CPU Metrics");
        let cpu_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        let cpu_total_usage_label = Label::new(Some("Total Usage: N/A"));
        cpu_grid.attach(&Label::new(Some("Overall:")), 0, 0, 1, 1);
        cpu_grid.attach(&cpu_total_usage_label, 1, 0, 1, 1);
        let cpu_core_grid_label = Label::new(Some("Per Core:"));
        cpu_grid.attach(&cpu_core_grid_label, 0, 1, 1, 1);
        let cpu_core_grid = Grid::builder().row_spacing(2).column_spacing(5).build();
        cpu_grid.attach(&cpu_core_grid, 1, 1, 1, 1);
        cpu_frame.set_child(Some(&cpu_grid));
        container.attach(&cpu_frame, 0, 0, 1, 1);

        // --- Memory Metrics ---
        let mem_frame = create_section_frame("Memory Metrics");
        let mem_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        let memory_ram_label = Label::new(Some("RAM: N/A"));
        let memory_swap_label = Label::new(Some("Swap: N/A"));
        mem_grid.attach(&Label::new(Some("RAM:")), 0, 0, 1, 1);
        mem_grid.attach(&memory_ram_label, 1, 0, 1, 1);
        mem_grid.attach(&Label::new(Some("Swap:")), 0, 1, 1, 1);
        mem_grid.attach(&memory_swap_label, 1, 1, 1, 1);
        mem_frame.set_child(Some(&mem_grid));
        container.attach(&mem_frame, 0, 1, 1, 1);

        // --- Disk Activity ---
        let disk_act_frame = create_section_frame("Disk Activity (IOPS, B/s)");
        let disk_activity_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&disk_activity_grid, &["Device", "Reads/s", "Writes/s", "R. Busy%", "W. Busy%"]);
        disk_act_frame.set_child(Some(&disk_activity_grid));
        container.attach(&disk_act_frame, 0, 2, 1, 1);

        // --- Disk Space ---
        let disk_space_frame = create_section_frame("Disk Space (Usage)");
        let disk_space_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&disk_space_grid, &["Mount Point", "Total", "Used", "Free", "Avail."]);
        disk_space_frame.set_child(Some(&disk_space_grid));
        container.attach(&disk_space_frame, 0, 3, 1, 1);

        // --- Network Activity ---
        let net_frame = create_section_frame("Network Activity (B/s, Pkt/s)");
        let network_activity_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&network_activity_grid, &["Interface", "Rcv B/s", "Sent B/s", "Rcv Pkt/s", "Sent Pkt/s"]);
        net_frame.set_child(Some(&network_activity_grid));
        container.attach(&net_frame, 0, 4, 1, 1);

        // --- Temperature ---
        let temp_frame = create_section_frame("Temperatures (°C)");
        let temperature_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&temperature_grid, &["Sensor", "Current", "High", "Critical"]);
        temp_frame.set_child(Some(&temperature_grid));
        container.attach(&temp_frame, 0, 5, 1, 1);

        let panel = Self {
            container,
            service,
            cpu_total_usage_label,
            cpu_core_grid,
            memory_ram_label,
            memory_swap_label,
            disk_activity_grid,
            disk_space_grid,
            network_activity_grid,
            temperature_grid,
        };

        // TODO: Unit test data formatting functions (if any extractable).
        panel.start_updates();
        panel
    }

    fn start_updates(&self) {
        // Update immediately once, then set up periodic updates
        self.update_metrics();

        glib::timeout_add_local_ बार(Duration::from_secs(2), clone!(@weak self as panel => @default-return glib::ControlFlow::Break, move || {
            panel.update_metrics();
            glib::ControlFlow::Continue
        }));
    }

    fn update_metrics(&self) {
        // TODO: UI Test: Verify correct display of mock CpuMetrics, MemoryMetrics, etc.
        let service = self.service.clone();
        let cpu_total_label = self.cpu_total_usage_label.clone();
        let cpu_core_grid = self.cpu_core_grid.clone();
        let mem_ram_label = self.memory_ram_label.clone();
        let mem_swap_label = self.memory_swap_label.clone();
        let disk_activity_grid = self.disk_activity_grid.clone();
        let disk_space_grid = self.disk_space_grid.clone();
        let network_activity_grid = self.network_activity_grid.clone();
        let temperature_grid = self.temperature_grid.clone();

        glib::MainContext::default().spawn_local(async move {
            // CPU
            match service.get_cpu_metrics().await {
                Ok(metrics) => {
                    cpu_total_label.set_text(&format!("{:.2}%", metrics.total_usage_percent));
                    // Clear old core labels
                    while let Some(child) = cpu_core_grid.first_child() { cpu_core_grid.remove(&child); }
                    for (i, usage) in metrics.per_core_usage_percent.iter().enumerate() {
                        cpu_core_grid.attach(&Label::new(Some(&format!("Core {}:", i))), 0, i as i32, 1, 1);
                        cpu_core_grid.attach(&Label::new(Some(&format!("{:.2}%", usage))), 1, i as i32, 1, 1);
                    }
                }
                Err(e) => cpu_total_label.set_text(&format!("Error: {}", e)),
            }

            // Memory
            match service.get_memory_metrics().await {
                Ok(metrics) => {
                    mem_ram_label.set_text(&format!(
                        "Total: {} MB, Used: {} MB, Avail: {} MB",
                        metrics.total_bytes / (1024 * 1024), metrics.used_bytes / (1024 * 1024), metrics.available_bytes / (1024 * 1024)
                    ));
                    mem_swap_label.set_text(&format!(
                        "Total: {} MB, Used: {} MB, Free: {} MB",
                        metrics.swap_total_bytes / (1024 * 1024), metrics.swap_used_bytes / (1024 * 1024), metrics.swap_free_bytes / (1024 * 1024)
                    ));
                }
                Err(e) => mem_ram_label.set_text(&format!("Error: {}", e)),
            }

            // Disk Activity
            update_dynamic_grid(&disk_activity_grid, service.get_disk_activity_metrics().await,
                |metric: &DiskActivityMetrics, row_idx| {
                    vec![
                        Label::new(Some(&metric.device_name)),
                        Label::new(Some(&format!("{:.1}", metric.reads_per_second))),
                        Label::new(Some(&format!("{:.1}", metric.writes_per_second))),
                        Label::new(Some(&format!("{:.1}%", metric.read_busy_time_percent))),
                        Label::new(Some(&format!("{:.1}%", metric.write_busy_time_percent))),
                    ]
                }, 5);

            // Disk Space
            update_dynamic_grid(&disk_space_grid, service.get_disk_space_metrics().await,
                |metric: &DiskSpaceMetrics, row_idx| {
                     vec![
                        Label::new(Some(&metric.mount_point)),
                        Label::new(Some(&format!("{} GB", metric.total_bytes / (1024 * 1024 * 1024)))),
                        Label::new(Some(&format!("{} GB", metric.used_bytes / (1024 * 1024 * 1024)))),
                        Label::new(Some(&format!("{} GB", metric.free_bytes / (1024 * 1024 * 1024)))),
                        Label::new(Some(&format!("{} GB", metric.available_bytes / (1024 * 1024 * 1024)))),
                    ]
                }, 5);

            // Network Activity
            update_dynamic_grid(&network_activity_grid, service.get_network_activity_metrics().await,
                |metric: &NetworkActivityMetrics, row_idx| {
                    vec![
                        Label::new(Some(&metric.interface_name)),
                        Label::new(Some(&format!("{} KB/s", metric.received_bytes_per_second / 1024))),
                        Label::new(Some(&format!("{} KB/s", metric.sent_bytes_per_second / 1024))),
                        Label::new(Some(&format!("{:.1}", metric.received_packets_per_second))),
                        Label::new(Some(&format!("{:.1}", metric.sent_packets_per_second))),
                    ]
                }, 5);

            // Temperature
            update_dynamic_grid(&temperature_grid, service.get_temperature_metrics().await,
                |metric: &TemperatureMetric, row_idx| {
                    vec![
                        Label::new(Some(&metric.sensor_name)),
                        Label::new(Some(&format!("{:.1}°C", metric.current_temp_celsius))),
                        Label::new(Some(&metric.high_threshold_celsius.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}°C", t)))),
                        Label::new(Some(&metric.critical_threshold_celsius.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}°C", t)))),
                    ]
                }, 4);
        });
    }

    pub fn get_widget(&self) -> &Grid {
        &self.container
    }
}

fn create_section_frame(title: &str) -> gtk::Frame {
    let frame = gtk::Frame::new(Some(title));
    frame.set_label_xalign(0.05); // Align title to the left
    frame.set_margin_top(5);
    frame.set_margin_bottom(5);
    frame
}

fn add_grid_headers(grid: &Grid, headers: &[&str]) {
    for (i, header_text) in headers.iter().enumerate() {
        let label = Label::new(Some(header_text));
        label.add_css_class("header"); // For styling if CSS is used
        // Make header bold using Pango markup
        label.set_markup("<span weight='bold'></span>");
        label.set_text(header_text); // Set text after markup to ensure it's not overridden
        grid.attach(&label, i as i32, 0, 1, 1);
    }
}

fn update_dynamic_grid<T, F>(
    grid: &Grid,
    metrics_result: Result<Vec<T>, impl std::fmt::Display>,
    row_builder: F,
    num_columns: i32,
) where
    F: Fn(&T, i32) -> Vec<Label>,
{
    // Clear old rows (skip header row at index 0)
    let mut current_row = 1;
    loop {
        match grid.child_at(0, current_row) {
            Some(_) => { // If there's a child in the first column of this row, remove all children in this row
                for col in 0..num_columns {
                    if let Some(child) = grid.child_at(col, current_row) {
                        grid.remove(&child);
                    }
                }
                current_row += 1;
            }
            None => break, // No more rows with content in the first column
        }
        if current_row > 50 { break; } // Safety break for very large grids / runaway loops
    }

    match metrics_result {
        Ok(metrics_vec) => {
            if metrics_vec.is_empty() {
                let label = Label::new(Some("No data available."));
                grid.attach(&label, 0, 1, num_columns, 1);
            } else {
                for (idx, metric_item) in metrics_vec.iter().enumerate() {
                    let row_idx = (idx + 1) as i32; // Start from row 1 (below headers)
                    let labels = row_builder(metric_item, row_idx);
                    for (col_idx, label_widget) in labels.iter().enumerate() {
                        grid.attach(label_widget, col_idx as i32, row_idx, 1, 1);
                    }
                }
            }
        }
        Err(e) => {
            let error_label = Label::new(Some(&format!("Error: {}", e)));
            grid.attach(&error_label, 0, 1, num_columns, 1);
        }
    }
}
