//! # Temperature Metrics Collector
//!
//! This module collects system temperature readings from thermal zones exposed
//! by the Linux kernel in `/sys/class/thermal/`.

use async_trait::async_trait;
use tokio::fs;
use novade_core::types::system_health::TemperatureMetric;
use crate::error::SystemError;
use crate::system_health_collectors::TemperatureMetricsCollector;
use glob::glob; // For finding thermal zones. Ensure `glob` is in Cargo.toml.

/// A collector for system temperature metrics on Linux systems.
///
/// Implements the `TemperatureMetricsCollector` trait by scanning `/sys/class/thermal/thermal_zone*`
/// directories. For each thermal zone, it reads the sensor type (name) and current temperature.
/// Temperatures are typically provided in millidegrees Celsius and are converted to Celsius.
/// It also attempts to read "high" and "critical" trip point temperatures if available.
pub struct LinuxTemperatureMetricsCollector;

/// Helper function to read the content of a file to a string.
/// Used for reading sensor type and temperature values.
async fn read_file_to_string(path: &std::path::Path) -> Result<String, SystemError> {
    fs::read_to_string(path).await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to read {:?}: {}", path, e)))
}

/// Helper function to read and parse a temperature value from a file.
/// Temperatures are expected to be integers representing millidegrees Celsius.
async fn read_temp_value(path: &std::path::Path) -> Result<f32, SystemError> {
    let content = read_file_to_string(path).await?;
    content.trim().parse::<i32>()
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse temperature from {:?}: {}", path, e)))
        .map(|temp_milli_c| temp_milli_c as f32 / 1000.0) // Convert millidegrees to degrees
}


#[async_trait::async_trait]
impl TemperatureMetricsCollector for LinuxTemperatureMetricsCollector {
    /// Asynchronously collects temperature metrics from all available thermal zones.
    ///
    /// Iterates through `/sys/class/thermal/thermal_zone*` directories. For each zone,
    /// it reads the sensor type (e.g., "acpitz", "x86_pkg_temp") from the `type` file
    /// and the current temperature from the `temp` file (converting from millidegrees Celsius).
    /// It also attempts to find associated "high" and "critical" trip point temperatures.
    ///
    /// Returns a `Result` containing a `Vec<TemperatureMetric>` on success,
    /// or a `SystemError` if directory scanning or file reading/parsing fails.
    async fn collect_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, SystemError> {
        // TODO: Unit test parsing of /sys/class/thermal/thermal_zone*/temp mock data.
        let mut temperature_metrics = Vec::new();

        // Using glob to find all thermal zones
        // This path requires the glob crate. Ensure it's added to Cargo.toml.
        let thermal_zone_pattern = "/sys/class/thermal/thermal_zone*";
        let entries = glob(thermal_zone_pattern)
            .map_err(|e| SystemError::MetricCollectorError(format!("Failed to read glob pattern {}: {}", thermal_zone_pattern, e)))?;

        for entry_result in entries {
            match entry_result {
                Ok(path_buf) => {
                    if path_buf.is_dir() {
                        let type_path = path_buf.join("type");
                        let temp_path = path_buf.join("temp");

                        if type_path.exists() && temp_path.exists() {
                            let sensor_name = read_file_to_string(&type_path).await?.trim().to_string();
                            let current_temp_celsius = read_temp_value(&temp_path).await?;

                            // Optionally, try to read high and critical thresholds
                            // These might not exist for all sensors.
                            let mut high_threshold_celsius = None;
                            let mut critical_threshold_celsius = None;

                            // Look for trip points. This can be complex as there can be multiple.
                            // For simplicity, let's try to find 'high' and 'critical' types.
                            // A more robust implementation might iterate all trip_point_*_type and _temp.
                            for i in 0..5 { // Check a few trip points
                                let trip_type_path = path_buf.join(format!("trip_point_{}_type", i));
                                let trip_temp_path = path_buf.join(format!("trip_point_{}_temp", i));

                                if trip_type_path.exists() && trip_temp_path.exists() {
                                    let trip_type = read_file_to_string(&trip_type_path).await?.trim().to_lowercase();
                                    let trip_temp = read_temp_value(&trip_temp_path).await?;

                                    if trip_type == "high" || trip_type == "hot" { // "hot" is sometimes used
                                        high_threshold_celsius = Some(trip_temp);
                                    } else if trip_type == "critical" {
                                        critical_threshold_celsius = Some(trip_temp);
                                    }
                                }
                            }

                            temperature_metrics.push(TemperatureMetric {
                                sensor_name,
                                current_temp_celsius,
                                high_threshold_celsius,
                                critical_threshold_celsius,
                            });
                        }
                    }
                }
                Err(e) => {
                    // Log or handle individual glob entry errors
                    eprintln!("Error processing thermal zone entry: {}", e);
                }
            }
        }

        Ok(temperature_metrics)
    }
}
