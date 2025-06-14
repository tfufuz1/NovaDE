// novade-core/src/types/system_health.rs
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Represents CPU usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuMetrics {
    /// Total CPU usage across all cores, as a percentage.
    pub total_usage_percent: f32,
    /// CPU usage for each individual core, as a percentage.
    pub per_core_usage_percent: Vec<f32>,
    // Future: Add temperature, frequency if available and relevant
}

/// Represents memory usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryMetrics {
    /// Total physical memory in bytes.
    pub total_bytes: u64,
    /// Used physical memory in bytes.
    pub used_bytes: u64,
    /// Free physical memory in bytes (often less relevant than available_bytes).
    pub free_bytes: u64,
    /// Available memory in bytes (memory that can be readily used by applications).
    pub available_bytes: u64,
    /// Total swap space in bytes.
    pub swap_total_bytes: u64,
    /// Used swap space in bytes.
    pub swap_used_bytes: u64,
}

/// Represents disk I/O activity metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskActivityMetrics {
    /// Bytes read per second.
    pub read_bytes_per_sec: u64,
    /// Bytes written per second.
    pub write_bytes_per_sec: u64,
    // Future: Add IOPS (Input/Output Operations Per Second) if available
}

/// Represents disk space usage metrics for a specific partition/mount point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskSpaceMetrics {
    /// Name of the disk device (e.g., "sda1", "nvme0n1p2").
    pub device_name: String,
    /// Mount point of the file system (e.g., "/", "/home").
    pub mount_point: String,
    /// Type of the file system (e.g., "ext4", "ntfs", "btrfs").
    pub file_system_type: String,
    /// Total space on the disk/partition in bytes.
    pub total_bytes: u64,
    /// Used space on the disk/partition in bytes.
    pub used_bytes: u64,
    /// Free space available to unprivileged users in bytes.
    pub free_bytes: u64,
}

/// Represents network I/O activity metrics for a specific interface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkActivityMetrics {
    /// Name of the network interface (e.g., "eth0", "wlan0").
    pub interface_name: String,
    /// Bytes sent per second over this interface.
    pub sent_bytes_per_sec: u64,
    /// Bytes received per second over this interface.
    pub received_bytes_per_sec: u64,
    /// Total bytes sent over this interface since system/interface startup.
    pub total_sent_bytes: u64,
    /// Total bytes received over this interface since system/interface startup.
    pub total_received_bytes: u64,
    // Future: Add packet counts, error counts
}

/// Represents a temperature reading from a system sensor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemperatureMetric {
    /// Name of the sensor (e.g., "CPU Core 0", "GPU", "Ambient").
    pub sensor_name: String,
    /// Current temperature in Celsius.
    pub current_temp_celsius: f32,
    /// Optional high temperature threshold for warnings.
    pub high_threshold_celsius: Option<f32>,
    /// Optional critical temperature threshold for critical alerts.
    pub critical_threshold_celsius: Option<f32>,
}

/// Defines the priority levels for log entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LogPriority {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Represents a single log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    /// Timestamp of when the log entry was recorded.
    pub timestamp: DateTime<Utc>,
    /// Component or source that generated the log (e.g., "kernel", "app_name").
    pub source_component: String,
    /// Severity level of the log entry.
    pub priority: LogPriority,
    /// The log message itself.
    pub message: String,
    /// Additional structured data associated with the log entry.
    pub fields: HashMap<String, String>,
}

/// A unique identifier for a log source (e.g., a specific log file or service).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LogSourceIdentifier(pub String); // e.g., "journald", "novade_daemon_log"

/// Defines filters for querying or streaming log entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogFilter {
    /// Optional list of specific log sources to include.
    pub sources: Option<Vec<LogSourceIdentifier>>,
    /// Optional minimum priority level for log entries.
    pub min_priority: Option<LogPriority>,
    /// Optional time range to filter log entries by.
    pub time_range: Option<TimeRange>,
    /// Optional list of keywords to search for in the log message.
    pub keywords: Option<Vec<String>>,
    /// Optional filters for specific structured fields in log entries.
    pub field_filters: Option<HashMap<String, String>>,
}

/// Represents a time range with a start and end timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// A unique identifier for a diagnostic test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiagnosticTestId(pub String);

/// Provides information about an available diagnostic test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticTestInfo {
    /// Unique ID of the diagnostic test.
    pub id: DiagnosticTestId,
    /// Human-readable name of the test.
    pub name: String,
    /// Description of what the test does.
    pub description: String,
    // Future: Add estimated duration, required permissions
}

/// Represents the status of a diagnostic test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DiagnosticStatus {
    NotRun,
    Running,
    Passed,
    Failed,
    Warning, // Test completed but found non-critical issues
    Cancelled, // Test was cancelled by the user or system
    Error,   // An error occurred during the execution of the test itself
}

/// Contains the results of a diagnostic test execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticTestResult {
    /// ID of the diagnostic test that was run.
    pub id: DiagnosticTestId,
    /// Current status of the test.
    pub status: DiagnosticStatus,
    /// Brief human-readable summary of the test outcome.
    pub summary: String,
    /// Optional detailed output, logs, or structured data from the test.
    pub details: Option<String>,
    /// Timestamp of when the test started.
    pub start_time: Option<DateTime<Utc>>,
    /// Timestamp of when the test ended.
    pub end_time: Option<DateTime<Utc>>,
}

/// Defines the severity levels for alerts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A unique identifier for an alert. Typically a UUID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AlertId(pub String);

/// Represents an alert triggered by the system health monitor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    /// Unique ID of the alert.
    pub id: AlertId,
    /// Human-readable name or title for the alert.
    pub name: String,
    /// Severity level of the alert.
    pub severity: AlertSeverity,
    /// Detailed message describing the alert condition.
    pub message: String,
    /// Timestamp of when the alert was first triggered.
    pub timestamp: DateTime<Utc>,
    /// Description of the metric or log condition that triggered the alert.
    pub source_metric_or_log: String, // e.g., "CPU Usage > 90%", "Disk space /dev/sda1 < 1GB"
    /// Whether the alert has been acknowledged by a user.
    pub acknowledged: bool,
    /// How many times this specific alert condition has re-triggered.
    pub last_triggered_count: u32,
    /// Optional suggested steps or guidance for resolving the alert condition.
    pub resolution_steps: Option<String>,
}

/// Configuration for the System Health Dashboard feature.
/// This will be part of the main NovaDE configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemHealthDashboardConfig {
    /// How often to refresh metric data, in milliseconds.
    pub metric_refresh_interval_ms: u64,
    /// How often to refresh log data (if polling), in milliseconds.
    pub log_refresh_interval_ms: u64,
    /// Default log sources to display if not overridden by user.
    pub default_log_sources: Vec<String>,
    /// Configuration for various alert thresholds.
    pub alert_thresholds: AlertThresholdsConfig,
    // Future: Per-metric refresh intervals, specific log source configs
}

/// Configuration for specific alert thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertThresholdsConfig {
    /// Configuration for high CPU usage alerts.
    pub high_cpu_usage_percent: Option<CpuAlertConfig>,
    /// Configuration for low available memory alerts.
    pub low_memory_available_percent: Option<MemoryAlertConfig>,
    /// Configuration for low disk space alerts. Can be a list in the future.
    pub low_disk_space_warnings: Option<Vec<DiskSpaceAlertConfig>>, // Changed to Vec
    pub low_disk_space_criticals: Option<Vec<DiskSpaceAlertConfig>>,// Changed to Vec
    // Future: network inactivity, high temperature alerts
}

/// Configuration for CPU usage-based alerts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuAlertConfig {
    /// CPU usage percentage threshold.
    pub threshold_percent: f32,
    /// Duration in seconds CPU must be above threshold to trigger alert.
    pub duration_seconds: u32,
}

/// Configuration for available memory-based alerts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryAlertConfig {
    /// Available memory percentage threshold.
    pub threshold_percent: f32, // Alert if available memory < X%
}

/// Configuration for disk space-based alerts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskSpaceAlertConfig {
    /// Path to the device (e.g., "/dev/sda1") or mount point (e.g., "/home").
    /// Use "*" to apply to all monitored mount points.
    pub device_path_or_mount_point: String,
    /// Free disk space percentage threshold.
    pub threshold_percent: f32, // Alert if free space < X%
}

impl Default for SystemHealthDashboardConfig {
    fn default() -> Self {
        SystemHealthDashboardConfig {
            metric_refresh_interval_ms: 1000,
            log_refresh_interval_ms: 2000,
            default_log_sources: vec!["journald".to_string()],
            alert_thresholds: AlertThresholdsConfig {
                high_cpu_usage_percent: Some(CpuAlertConfig {
                    threshold_percent: 90.0,
                    duration_seconds: 60,
                }),
                low_memory_available_percent: Some(MemoryAlertConfig {
                    threshold_percent: 10.0,
                }),
                low_disk_space_warnings: Some(vec![
                    DiskSpaceAlertConfig {
                        device_path_or_mount_point: "*".to_string(),
                        threshold_percent: 15.0, // Warning at 15%
                    }
                ]),
                low_disk_space_criticals: Some(vec![
                     DiskSpaceAlertConfig {
                        device_path_or_mount_point: "*".to_string(),
                        threshold_percent: 5.0, // Critical at 5%
                    }
                ])
            },
        }
    }
}

/*
    DESIGN NOTES FOR CONFIGURATION INTEGRATION:

    The `SystemHealthDashboardConfig` struct defined above is intended to be part of
    the main `NovaConfiguration` struct in `novade-core/src/config/mod.rs`.

    1. Modify `novade-core/src/config/mod.rs`:
       - Add `pub mod system_health_config;` (if moving struct definitions there) OR ensure types are correctly pathed.
       - In the main `NovaConfiguration` struct (or its equivalent), add a field:
         `pub system_health: crate::types::system_health::SystemHealthDashboardConfig,`
         (Adjust path if `SystemHealthDashboardConfig` is moved, e.g. to `crate::config::system_health_config::SystemHealthDashboardConfig`)

    2. Modify `novade-core/src/config/defaults.rs` (or where `NovaConfiguration::default()` is implemented):
       - When constructing the default `NovaConfiguration`, initialize the new field:
         `system_health: crate::types::system_health::SystemHealthDashboardConfig::default(),`

    3. Modify `novade-core/src/config/loader.rs` (or equivalent config loading logic):
       - The configuration loader (likely using Serde for TOML/JSON deserialization)
         needs to be updated to recognize and parse the `[system_health]` section
         from the configuration file.
       - Ensure the struct used for deserializing the main configuration includes
         the `system_health` field with the correct type.
       - Example TOML structure expected:
         ```toml
         # In config.toml or similar

         [system_health]
         metric_refresh_interval_ms = 1000
         log_refresh_interval_ms = 2000
         default_log_sources = ["journald", "syslog"] # Example

         [system_health.alert_thresholds]
           [system_health.alert_thresholds.high_cpu_usage_percent]
           threshold_percent = 90.0
           duration_seconds = 60

           [system_health.alert_thresholds.low_memory_available_percent]
           threshold_percent = 10.0

           # Example for multiple disk space alert configurations
           # Users can define different thresholds for different mount points or devices.
           # A "warning" level and a "critical" level can be defined.
           low_disk_space_warnings = [
             { device_path_or_mount_point = "/home", threshold_percent = 20.0 },
             { device_path_or_mount_point = "*", threshold_percent = 15.0 } # Default for others
           ]
           low_disk_space_criticals = [
             { device_path_or_mount_point = "/var/log", threshold_percent = 10.0 },
             { device_path_or_mount_point = "*", threshold_percent = 5.0 }  # Default for others
           ]
         ```
       - The `DiskSpaceAlertConfig` within `AlertThresholdsConfig` was changed to
         `low_disk_space_warnings: Option<Vec<DiskSpaceAlertConfig>>` and
         `low_disk_space_criticals: Option<Vec<DiskSpaceAlertConfig>>`
         to allow multiple rules for different devices/mounts and severity levels.
         The TOML example reflects this with arrays of tables.

    4. Consider moving `SystemHealthDashboardConfig` and its sub-structs
       (`AlertThresholdsConfig`, `CpuAlertConfig`, etc.) to a dedicated
       `novade-core/src/config/system_health_config.rs` file if the project
       prefers to keep type definitions separate from configuration structure definitions.
       If so, update paths accordingly. For now, it's co-located in `types/system_health.rs`
       for simplicity in this step.

    These notes serve as a guide for the subsequent subtask that will focus on
    implementing these configuration changes.
*/
