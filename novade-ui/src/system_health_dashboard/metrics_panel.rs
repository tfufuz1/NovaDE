use gtk4 as gtk;
use gtk::{prelude::*, Grid, Label, Orientation};
use novade_core::types::system_health::{
    CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric
};
use crate::system_health_dashboard::view_model::SystemHealthViewModel;
use glib::Variant;
use log::debug;

pub struct MetricsPanel {
    container: Grid,
    #[allow(dead_code)]
    view_model: SystemHealthViewModel,
    cpu_total_usage_label: Label,
    cpu_core_grid: Grid,
    memory_ram_label: Label,
    memory_swap_label: Label,

    disk_activity_grid: Grid,
    disk_space_grid: Grid,
    network_activity_grid: Grid,
    temperature_grid: Grid,
}

impl MetricsPanel {
    pub fn new(view_model: SystemHealthViewModel) -> Self {
        let container = Grid::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(10).column_spacing(6)
            .margin_top(10).margin_bottom(10).margin_start(10).margin_end(10)
            .build();

        // --- CPU Metrics ---
        let cpu_frame = create_section_frame("CPU Metrics");
        let cpu_grid_layout = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        let cpu_total_usage_label = Label::new(Some("Total Usage: N/A"));
        cpu_grid_layout.attach(&Label::new(Some("Overall:")), 0, 0, 1, 1);
        cpu_grid_layout.attach(&cpu_total_usage_label, 1, 0, 1, 1);
        let cpu_core_grid_label = Label::new(Some("Per Core:"));
        cpu_grid_layout.attach(&cpu_core_grid_label, 0, 1, 1, 1);
        let cpu_core_grid = Grid::builder().row_spacing(2).column_spacing(5).build(); // This is the grid for core data
        cpu_grid_layout.attach(&cpu_core_grid, 1, 1, 1, 1);
        cpu_frame.set_child(Some(&cpu_grid_layout));
        container.attach(&cpu_frame, 0, 0, 1, 1);

        // --- Memory Metrics ---
        let mem_frame = create_section_frame("Memory Metrics");
        let mem_grid_layout = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        let memory_ram_label = Label::new(Some("RAM: N/A"));
        let memory_swap_label = Label::new(Some("Swap: N/A"));
        mem_grid_layout.attach(&Label::new(Some("RAM:")), 0, 0, 1, 1);
        mem_grid_layout.attach(&memory_ram_label, 1, 0, 1, 1);
        mem_grid_layout.attach(&Label::new(Some("Swap:")), 0, 1, 1, 1);
        mem_grid_layout.attach(&memory_swap_label, 1, 1, 1, 1);
        mem_frame.set_child(Some(&mem_grid_layout));
        container.attach(&mem_frame, 0, 1, 1, 1);

        // --- Other Grids ---
        let disk_act_frame = create_section_frame("Disk Activity");
        let disk_activity_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&disk_activity_grid, &["Read", "Write"]);
        disk_act_frame.set_child(Some(&disk_activity_grid));
        container.attach(&disk_act_frame, 0, 2, 1, 1);

        let disk_space_frame = create_section_frame("Disk Space");
        let disk_space_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&disk_space_grid, &["Mount", "Device", "FS", "Total", "Used", "Free"]);
        disk_space_frame.set_child(Some(&disk_space_grid));
        container.attach(&disk_space_frame, 0, 3, 1, 1);

        let net_frame = create_section_frame("Network Activity");
        let network_activity_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&network_activity_grid, &["Interface", "Rcv B/s", "Sent B/s", "Total Rcv", "Total Sent"]);
        net_frame.set_child(Some(&network_activity_grid));
        container.attach(&net_frame, 0, 4, 1, 1);

        let temp_frame = create_section_frame("Temperatures");
        let temperature_grid = Grid::builder().row_spacing(5).column_spacing(10).margin_start(10).margin_end(10).margin_top(5).margin_bottom(5).build();
        add_grid_headers(&temperature_grid, &["Sensor", "Temp °C", "High °C", "Crit °C"]);
        temp_frame.set_child(Some(&temperature_grid));
        container.attach(&temp_frame, 0, 5, 1, 1);

        Self {
            container, view_model, cpu_total_usage_label, cpu_core_grid,
            memory_ram_label, memory_swap_label, disk_activity_grid,
            disk_space_grid, network_activity_grid, temperature_grid,
        }
    }

    pub fn update_cpu_metrics_display(&self, metrics: &CpuMetrics) {
        debug!("MetricsPanel: Updating CPU display with: {:?}", metrics);
        self.cpu_total_usage_label.set_text(&format!("{:.2}%", metrics.total_usage_percent));

        let num_new_cores = metrics.per_core_usage_percent.len();
        let mut current_display_rows = 0;
        let mut i = 0;
        while self.cpu_core_grid.child_at(0, i).is_some() {
            current_display_rows += 1;
            i += 1;
        }

        for i in 0..num_new_cores {
            let usage_text = format!("{:.2}%", metrics.per_core_usage_percent[i]);
            if i < current_display_rows { // Update existing row
                if let Some(label) = self.cpu_core_grid.child_at(0, i as i32).and_then(|w| w.downcast::<Label>().ok()) {
                    label.set_text(&format!("Core {}:", i));
                }
                if let Some(label) = self.cpu_core_grid.child_at(1, i as i32).and_then(|w| w.downcast::<Label>().ok()) {
                    label.set_text(&usage_text);
                }
            } else { // Add new row
                let core_desc_label = Label::new(Some(&format!("Core {}:", i)));
                core_desc_label.set_halign(gtk::Align::Start);
                self.cpu_core_grid.attach(&core_desc_label, 0, i as i32, 1, 1);

                let usage_value_label = Label::new(Some(&usage_text));
                usage_value_label.set_halign(gtk::Align::Start);
                self.cpu_core_grid.attach(&usage_value_label, 1, i as i32, 1, 1);
            }
        }
        // Remove stale rows if new data has fewer cores than displayed
        if num_new_cores < current_display_rows {
            for i in num_new_cores..current_display_rows {
                if let Some(child) = self.cpu_core_grid.child_at(0, i as i32) { self.cpu_core_grid.remove(&child); }
                if let Some(child) = self.cpu_core_grid.child_at(1, i as i32) { self.cpu_core_grid.remove(&child); }
            }
        }
    }

    pub fn update_memory_metrics_display(&self, metrics: &MemoryMetrics) {
        debug!("MetricsPanel: Updating Memory display with: {:?}", metrics);
        self.memory_ram_label.set_text(&format!(
            "Used: {}MiB / Total: {}MiB (Avail: {}MiB)",
            metrics.used_bytes / (1024 * 1024), metrics.total_bytes / (1024 * 1024), metrics.available_bytes / (1024 * 1024)
        ));
        self.memory_swap_label.set_text(&format!(
            "Used: {}MiB / Total: {}MiB",
            metrics.swap_used_bytes / (1024 * 1024), metrics.swap_total_bytes / (1024 * 1024)
        ));
    }

    pub fn update_disk_activity_display(&self, metrics_variant: &Variant) {
        debug!("MetricsPanel: Updating Disk Activity display.");
        if let Some(metrics_vec) = metrics_variant.get::<Vec<DiskActivityMetrics>>() {
            update_dynamic_grid_content(&self.disk_activity_grid, &metrics_vec, 2, |metric_item| {
                vec![
                    format_bytes(metric_item.read_bytes_per_sec, "/s"),
                    format_bytes(metric_item.write_bytes_per_sec, "/s"),
                ]
            });
        } else { debug!("MetricsPanel: Failed to get Vec<DiskActivityMetrics> from Variant."); }
    }

    pub fn update_disk_space_display(&self, metrics_variant: &Variant) {
        debug!("MetricsPanel: Updating Disk Space display.");
        if let Some(metrics_vec) = metrics_variant.get::<Vec<DiskSpaceMetrics>>() {
            update_dynamic_grid_content(&self.disk_space_grid, &metrics_vec, 6, |metric_item| {
                vec![
                    metric_item.mount_point.clone(),
                    metric_item.device_name.clone(),
                    metric_item.file_system_type.clone(),
                    format_bytes(metric_item.total_bytes, ""),
                    format_bytes(metric_item.used_bytes, ""),
                    format_bytes(metric_item.free_bytes, ""),
                ]
            });
        } else { debug!("MetricsPanel: Failed to get Vec<DiskSpaceMetrics> from Variant."); }
    }

    pub fn update_network_activity_display(&self, metrics_variant: &Variant) {
        debug!("MetricsPanel: Updating Network Activity display.");
        if let Some(metrics_vec) = metrics_variant.get::<Vec<NetworkActivityMetrics>>() {
            update_dynamic_grid_content(&self.network_activity_grid, &metrics_vec, 5, |metric_item| {
                vec![
                    metric_item.interface_name.clone(),
                    format_bytes(metric_item.received_bytes_per_sec, "/s"),
                    format_bytes(metric_item.sent_bytes_per_sec, "/s"),
                    format_bytes(metric_item.total_received_bytes, ""),
                    format_bytes(metric_item.total_sent_bytes, ""),
                ]
            });
        } else { debug!("MetricsPanel: Failed to get Vec<NetworkActivityMetrics> from Variant."); }
    }

    pub fn update_temperature_metrics_display(&self, metrics_variant: &Variant) {
        debug!("MetricsPanel: Updating Temperature display.");
        if let Some(metrics_vec) = metrics_variant.get::<Vec<TemperatureMetric>>() {
            update_dynamic_grid_content(&self.temperature_grid, &metrics_vec, 4, |metric_item| {
                vec![
                    metric_item.sensor_name.clone(),
                    format!("{:.1}°C", metric_item.current_temp_celsius),
                    metric_item.high_threshold_celsius.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}°C", t)),
                    metric_item.critical_threshold_celsius.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}°C", t)),
                ]
            });
        } else { debug!("MetricsPanel: Failed to get Vec<TemperatureMetric> from Variant."); }
    }

    pub fn get_widget(&self) -> &Grid { &self.container }
}

fn create_section_frame(title: &str) -> gtk::Frame {
    let frame = gtk::Frame::new(Some(title));
    frame.set_label_xalign(0.05);
    frame.set_margin_top(5);
    frame.set_margin_bottom(5);
    frame
}

fn add_grid_headers(grid: &Grid, headers: &[&str]) {
    // Clear existing headers to prevent duplication if this function were ever called multiple times on same grid.
    let mut current_child = grid.first_child();
    while let Some(child) = current_child {
        current_child = child.next_sibling();
        grid.remove(&child);
    }
    for (i, header_text) in headers.iter().enumerate() {
        let label = Label::new(Some(header_text));
        label.add_css_class("header");
        label.set_markup(&format!("<span weight='bold'>{}</span>", glib::markup_escape_text(header_text)));
        label.set_halign(gtk::Align::Start);
        grid.attach(&label, i as i32, 0, 1, 1);
    }
}

// Generic function to update dynamic grid content by reusing/adding/removing labels
fn update_dynamic_grid_content<T, F>(
    grid: &Grid,
    new_data: &[T],
    num_columns: i32,
    row_data_extractor: F,
) where
    F: Fn(&T) -> Vec<String>, // Closure extracts Vec<String> for a row from a &T item
    T: std::fmt::Debug + glib::prelude::StaticType + Clone + Send + Sync + 'static,
{
    let num_new_rows = new_data.len();
    let mut num_current_data_rows = 0;
    let mut i = 1; // Start checking from grid row 1 (assuming row 0 is header)
    while grid.child_at(0, i).is_some() { // Check if first cell of row `i` has a widget
        num_current_data_rows += 1;
        i += 1;
    }

    // Update existing rows / Add new rows
    for (item_idx, item_data) in new_data.iter().enumerate() {
        let grid_row_idx = (item_idx + 1) as i32; // Grid data rows start at 1
        let texts_for_row = row_data_extractor(item_data);

        if item_idx < num_current_data_rows { // This data item corresponds to an existing UI row
            for (col_idx, cell_text) in texts_for_row.iter().enumerate() {
                if let Some(widget) = grid.child_at(col_idx as i32, grid_row_idx) {
                    if let Some(label) = widget.downcast_ref::<Label>() {
                        label.set_text(cell_text);
                    } else { // Should not happen if grid is consistent
                        debug!("Widget at [{},{}] is not a Label.", grid_row_idx, col_idx);
                    }
                } else { // Should not happen if row existed
                    debug!("No widget at [{},{}], creating new Label.", grid_row_idx, col_idx);
                    let label = Label::new(Some(cell_text));
                    label.set_halign(gtk::Align::Start);
                    grid.attach(&label, col_idx as i32, grid_row_idx, 1, 1);
                }
            }
        } else { // This data item needs a new UI row
            for (col_idx, cell_text) in texts_for_row.iter().enumerate() {
                let label = Label::new(Some(cell_text));
                label.set_halign(gtk::Align::Start);
                grid.attach(&label, col_idx as i32, grid_row_idx, 1, 1);
            }
        }
    }

    // Remove stale rows if new data has fewer items than current UI rows
    if num_new_rows < num_current_data_rows {
        for stale_row_grid_idx in (num_new_rows + 1)..=(num_current_data_rows) {
            for col_idx in 0..num_columns {
                if let Some(widget) = grid.child_at(col_idx, stale_row_grid_idx as i32) {
                    grid.remove(&widget);
                }
            }
        }
    } else if num_new_rows == 0 && num_current_data_rows == 0 { // Handle empty placeholder
         // Check if placeholder already exists
        if grid.child_at(0,1).is_none() { // if no data rows and no placeholder yet
            let label = Label::new(Some("No data available."));
            label.set_halign(gtk::Align::Center);
            grid.attach(&label, 0, 1, num_columns, 1);
        }
    } else if num_new_rows > 0 { // If there is data, remove placeholder if it exists
        if let Some(child) = grid.child_at(0,1) { // Check first cell of first data row
            if let Some(label) = child.downcast_ref::<Label>() {
                 if label.text() == "No data available." { // This is a heuristic
                    // Remove the placeholder row (all its cells)
                    for col_idx in 0..num_columns { // Assuming it spans num_columns
                         if let Some(placeholder_cell_widget) = grid.child_at(col_idx,1) {
                            grid.remove(&placeholder_cell_widget);
                         } else { break; } // If it didn't span all columns
                    }
                 }
            }
        }
    }
}


fn format_bytes(bytes: u64, suffix: &str) -> String {
    const KB: u64 = 1024; const MB: u64 = KB * 1024; const GB: u64 = MB * 1024; const TB: u64 = GB * 1024;
    if bytes >= TB { format!("{:.2} TiB{}", bytes as f64 / TB as f64, suffix) }
    else if bytes >= GB { format!("{:.2} GiB{}", bytes as f64 / GB as f64, suffix) }
    else if bytes >= MB { format!("{:.2} MiB{}", bytes as f64 / MB as f64, suffix) }
    else if bytes >= KB { format!("{:.2} KiB{}", bytes as f64 / KB as f64, suffix) }
    else { format!("{} B{}", bytes, suffix) }
}
