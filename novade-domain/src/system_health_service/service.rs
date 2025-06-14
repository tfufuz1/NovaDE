//! # System Health Service Module
//!
//! This module defines the domain-level service for system health monitoring.
//! It provides a unified API for accessing various system metrics, logs, diagnostics,
//! and alerts, abstracting away the underlying data collection and system interactions.
//!
//! The main components are:
//! - `SystemHealthService` trait: Defines the public contract of the service.
//! - `DefaultSystemHealthService`: The default implementation of the service trait,
//!   which orchestrates data collection from various system-layer collectors and runners.

use novade_core::types::system_health::{
    Alert, CpuMetrics, DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult,
    DiskActivityMetrics, DiskSpaceMetrics, LogEntry, LogFilter, MemoryMetrics,
    NetworkActivityMetrics, SystemHealthDashboardConfig, TemperatureMetric, TimeRange,
};
use novade_core::config::CoreConfig;
use novade_system::error::SystemError as NovaSystemError; // Alias to avoid naming conflicts
use novade_system::system_health_collectors::{
    CpuMetricsCollector, DiagnosticRunner, DiskMetricsCollector, LogHarvester,
    MemoryMetricsCollector, NetworkMetricsCollector, TemperatureMetricsCollector,
};
use crate::error::SystemHealthError;
use std::sync::Arc;
use futures_core::Stream;
use async_trait::async_trait;
use futures_util::TryStreamExt; // For mapping stream errors
use tokio::sync::Mutex; // For ActiveAlerts
use std::collections::HashMap; // For ActiveAlerts
use uuid::Uuid; // For generating unique alert IDs

// Represents the internal store of active alerts.
// Key: A unique identifier for the alert condition (e.g., "cpu_usage_high", "disk_space_low_/dev/sda1").
//      This key helps in updating or removing existing alerts.
// Value: The `Alert` object itself.
type ActiveAlertsMap = HashMap<String, Alert>;

/// Trait defining the public API for the System Health Service.
///
/// This service aggregates data from various collectors and runners, evaluates alerts,
/// and provides a comprehensive view of the system's health.
#[async_trait::async_trait]
pub trait SystemHealthService: Send + Sync {
    /// Retrieves the latest CPU metrics.
    /// Returns a `Result` with `CpuMetrics` or a `SystemHealthError`.
    async fn get_cpu_metrics(&self) -> Result<CpuMetrics, SystemHealthError>;

    /// Retrieves the latest memory metrics.
    /// Returns a `Result` with `MemoryMetrics` or a `SystemHealthError`.
    async fn get_memory_metrics(&self) -> Result<MemoryMetrics, SystemHealthError>;

    /// Retrieves the latest disk I/O activity metrics for all monitored disks.
    /// Returns a `Result` with a vector of `DiskActivityMetrics` or a `SystemHealthError`.
    async fn get_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, SystemHealthError>;

    /// Retrieves the latest disk space usage metrics for all monitored filesystems.
    /// Returns a `Result` with a vector of `DiskSpaceMetrics` or a `SystemHealthError`.
    async fn get_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, SystemHealthError>;

    /// Retrieves the latest network activity metrics for all relevant interfaces.
    /// Returns a `Result` with a vector of `NetworkActivityMetrics` or a `SystemHealthError`.
    async fn get_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, SystemHealthError>;

    /// Retrieves the latest temperature metrics from available system sensors.
    /// Returns a `Result` with a vector of `TemperatureMetric` or a `SystemHealthError`.
    async fn get_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, SystemHealthError>;

    /// Queries historical log entries based on specified criteria.
    ///
    /// # Arguments
    /// * `filter`: Optional filter for log level, keywords, etc.
    /// * `time_range`: Optional time range to constrain the query.
    /// * `limit`: Optional maximum number of log entries to return.
    /// Returns a `Result` with a vector of `LogEntry` items or a `SystemHealthError`.
    async fn query_logs(&self, filter: Option<LogFilter>, time_range: Option<TimeRange>, limit: Option<usize>) -> Result<Vec<LogEntry>, SystemHealthError>;

    /// Streams live log entries, applying an optional filter.
    /// Returns a `Result` with a stream of `LogEntry` results or a `SystemHealthError`.
    async fn stream_logs(&self, filter: Option<LogFilter>) -> Result<Box<dyn Stream<Item = Result<LogEntry, SystemHealthError>> + Send + Unpin>, SystemHealthError>;

    /// Lists all available diagnostic tests that can be run.
    /// Returns a `Result` with a vector of `DiagnosticTestInfo` or a `SystemHealthError`.
    fn list_available_diagnostic_tests(&self) -> Result<Vec<DiagnosticTestInfo>, SystemHealthError>;

    /// Runs a specific diagnostic test by its ID.
    /// Returns a `Result` with `DiagnosticTestResult` or a `SystemHealthError`.
    async fn run_diagnostic_test(&self, test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, SystemHealthError>;

    /// Retrieves a list of currently active system alerts.
    /// Alerts are evaluated based on configured thresholds and current metrics.
    /// Returns a `Result` with a vector of `Alert` items or a `SystemHealthError`.
    async fn get_active_alerts(&self) -> Result<Vec<Alert>, SystemHealthError>;

    // /// Updates the configuration for the system health dashboard, particularly alert thresholds.
    // /// (Placeholder for future implementation)
    // async fn update_configuration(&self, config: SystemHealthDashboardConfig) -> Result<(), SystemHealthError>;
}

/// Default implementation of the `SystemHealthService`.
///
/// This struct holds `Arc` references to various system-layer collectors and runners,
/// an `Arc` to the `CoreConfig` for accessing configuration (like alert thresholds),
/// and an internal store for managing active alerts.
pub struct DefaultSystemHealthService {
    /// Shared reference to the application's core configuration.
    core_config: Arc<CoreConfig>,
    /// Collector for CPU metrics.
    cpu_collector: Arc<dyn CpuMetricsCollector + Send + Sync>,
    /// Collector for memory metrics.
    memory_collector: Arc<dyn MemoryMetricsCollector + Send + Sync>,
    /// Collector for disk metrics (activity and space).
    disk_collector: Arc<dyn DiskMetricsCollector + Send + Sync>,
    /// Collector for network activity metrics.
    network_collector: Arc<dyn NetworkMetricsCollector + Send + Sync>,
    /// Collector for temperature metrics.
    temperature_collector: Arc<dyn TemperatureMetricsCollector + Send + Sync>,
    /// Harvester for system logs.
    log_harvester: Arc<dyn LogHarvester + Send + Sync>,
    /// Runner for diagnostic tests.
    diagnostic_runner: Arc<dyn DiagnosticRunner + Send + Sync>,
    /// Internal store for currently active alerts, protected by a Mutex.
    active_alerts: Arc<Mutex<ActiveAlertsMap>>,
}

impl DefaultSystemHealthService {
    /// Creates a new `DefaultSystemHealthService`.
    ///
    /// # Arguments
    /// * `core_config`: Shared access to the application's `CoreConfig`.
    /// * `cpu_collector`: An implementation of `CpuMetricsCollector`.
    /// * `memory_collector`: An implementation of `MemoryMetricsCollector`.
    /// * `disk_collector`: An implementation of `DiskMetricsCollector`.
    /// * `network_collector`: An implementation of `NetworkMetricsCollector`.
    /// * `temperature_collector`: An implementation of `TemperatureMetricsCollector`.
    /// * `log_harvester`: An implementation of `LogHarvester`.
    /// * `diagnostic_runner`: An implementation of `DiagnosticRunner`.
    pub fn new(
        core_config: Arc<CoreConfig>,
        cpu_collector: Arc<dyn CpuMetricsCollector + Send + Sync>,
        memory_collector: Arc<dyn MemoryMetricsCollector + Send + Sync>,
        disk_collector: Arc<dyn DiskMetricsCollector + Send + Sync>,
        network_collector: Arc<dyn NetworkMetricsCollector + Send + Sync>,
        temperature_collector: Arc<dyn TemperatureMetricsCollector + Send + Sync>,
        log_harvester: Arc<dyn LogHarvester + Send + Sync>,
        diagnostic_runner: Arc<dyn DiagnosticRunner + Send + Sync>,
    ) -> Self {
        Self {
            core_config,
            cpu_collector,
            memory_collector,
            disk_collector,
            network_collector,
            temperature_collector,
            log_harvester,
            diagnostic_runner,
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Helper to access the specific config
    // TODO: Uncomment and use when SystemHealthDashboardConfig is integrated into CoreConfig
    // fn get_dashboard_config(&self) -> Option<&SystemHealthDashboardConfig> {
    //    self.core_config.system_health_dashboard.as_ref()
    // }

    /// Evaluates current system metrics against configured alert thresholds and updates
    /// the internal `active_alerts` map.
    ///
    /// This method is called internally by `get_active_alerts` to ensure the
    /// alert list is up-to-date before being returned.
    ///
    /// It fetches CPU, memory, and disk space metrics, then compares them against
    /// thresholds defined in `self.core_config.system_health.alert_thresholds`.
    /// - If a metric breaches a threshold, an `Alert` is created or updated in `active_alerts`.
    /// - If a metric no longer breaches a previously active threshold, the corresponding alert is removed.
    ///
    /// Error handling for metric collection failures is basic (prints to stderr); a more robust
    /// system might generate specific "metric collection failed" alerts.
    ///
    /// Note: Duration-based alerting (e.g., "CPU above X% for Y duration") is simplified
    /// in this implementation to only check the current value. True duration-based alerting
    /// would require more state management, potentially tracking breach start times.
    async fn evaluate_alerts(&self) -> Result<(), SystemHealthError> {
        // TODO: Unit test alert generation logic with mock metrics and config.
        // TODO: Unit test alert clearing logic.
        let config = if let Some(sh_config) = &self.core_config.system_health { // Corrected field name
            sh_config
        } else {
            // If no specific system_health_dashboard config in CoreConfig, no alerts can be evaluated.
            // This might happen if the config file is missing the [system_health] section
            // or if SystemHealthDashboardConfig is not yet integrated into CoreConfig defaults.
            let mut alerts_map = self.active_alerts.lock().await;
            if !alerts_map.is_empty() {
                // If there were pre-existing alerts, clear them as their config is gone.
                alerts_map.clear();
                 // Optionally, log a warning that system health config is missing for alerting.
                eprintln!("Warning: SystemHealthDashboardConfig not found in CoreConfig. Alerts disabled/cleared.");
            }
            return Ok(());
        };

        let mut current_alerts = self.active_alerts.lock().await; // Lock the mutex for the duration of evaluation

        // --- CPU Usage Alert ---
        if let Some(cpu_threshold_config) = &config.alert_thresholds.high_cpu_usage_percent {
            let cpu_alert_key = "high_cpu_usage";
            match self.get_cpu_metrics().await {
                Ok(metrics) => {
                    if metrics.total_usage_percent >= cpu_threshold_config.threshold_percent {
                        // If alert doesn't exist or needs update (e.g., timestamp), create/update it.
                        // For simplicity, we just insert/replace.
                        // A more complex system might check if the existing alert differs before replacing.
                        if !current_alerts.contains_key(cpu_alert_key) { // Only add if new
                            let alert = Alert {
                                id: Uuid::new_v4().to_string(), // Generate a new unique ID for each alert instance
                                name: "High CPU Usage".to_string(),
                                description: format!(
                                    "CPU usage is at {:.2}%, exceeding threshold of {}%.",
                                    metrics.total_usage_percent, cpu_threshold_config.threshold_percent
                                ),
                                severity: novade_core::types::system_health::AlertSeverity::Warning, // Could be configurable
                                source: "system_health_service".to_string(),
                                timestamp: std::time::SystemTime::now(),
                                acknowledged: false,
                                details: Some(format!("Current CPU Metrics: {:?}", metrics)),
                            };
                            current_alerts.insert(cpu_alert_key.to_string(), alert);
                        }
                    } else {
                        // If usage is below threshold, remove the alert if it exists.
                        current_alerts.remove(cpu_alert_key);
                    }
                }
                Err(e) => {
                    // Failed to get CPU metrics. Consider how to handle this.
                    // Option 1: Log error and do nothing about existing CPU alert.
                    // Option 2: Create a specific "CPU metric collection failed" alert.
                    // Option 3: Remove existing CPU alert as its status is unknown.
                    eprintln!("Failed to get CPU metrics for alert evaluation: {:?}", e);
                    // For now, we don't change the existing alert state on metric error.
                }
            }
        } else {
            // If this specific alert configuration is removed from CoreConfig, ensure the alert is cleared.
            current_alerts.remove("high_cpu_usage");
        }

        // --- Low Memory Available Alert ---
        if let Some(mem_threshold_config) = &config.alert_thresholds.low_memory_available_percent {
            let mem_alert_key = "low_memory_available";
            match self.get_memory_metrics().await {
                Ok(metrics) => {
                    let available_percent = if metrics.total_bytes > 0 {
                        (metrics.available_bytes as f32 / metrics.total_bytes as f32) * 100.0
                    } else {
                        100.0 // Avoid division by zero if total_bytes is 0
                    };

                    if available_percent < mem_threshold_config.threshold_percent {
                        if !current_alerts.contains_key(mem_alert_key) {
                             let alert = Alert {
                                id: Uuid::new_v4().to_string(),
                                name: "Low Memory Available".to_string(),
                                description: format!(
                                    "Available memory is at {:.2}%, below threshold of {}%.",
                                    available_percent, mem_threshold_config.threshold_percent
                                ),
                                severity: novade_core::types::system_health::AlertSeverity::Warning,
                                source: "system_health_service".to_string(),
                                timestamp: std::time::SystemTime::now(),
                                acknowledged: false,
                                details: Some(format!("Current Memory Metrics: {:?}", metrics)),
                            };
                            current_alerts.insert(mem_alert_key.to_string(), alert);
                        }
                    } else {
                        current_alerts.remove(mem_alert_key);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get Memory metrics for alert evaluation: {:?}", e);
                }
            }
        } else {
            current_alerts.remove("low_memory_available");
        }

        // --- Low Disk Space Alerts ---
        // This section needs to handle multiple disk alerts and their configurations.
        // We iterate through configured thresholds and then through actual disk metrics.

        // Critical Thresholds
        if let Some(disk_critical_configs) = &config.alert_thresholds.low_disk_space_criticals {
            match self.get_disk_space_metrics().await { // This fetches all disk metrics
                Ok(disk_metrics_vec) => {
                    for metrics in &disk_metrics_vec { // Iterate over actual disk metrics
                        for critical_config in disk_critical_configs { // Check against each critical config
                            // Match config to metric (e.g., by mount_point or wildcard)
                            let path_matches = critical_config.device_path_or_mount_point == "*" ||
                                               critical_config.device_path_or_mount_point == metrics.mount_point;
                                               // Add || critical_config.device_path_or_mount_point == metrics.filesystem_type if needed

                            if path_matches {
                                let free_percent = if metrics.total_bytes > 0 {
                                    (metrics.free_bytes as f32 / metrics.total_bytes as f32) * 100.0
                                } else { 100.0 };

                                let alert_key = format!("low_disk_space_critical_{}", metrics.mount_point.replace(|c: char| !c.is_alphanumeric(), "_"));
                                if free_percent < critical_config.threshold_percent {
                                    if !current_alerts.contains_key(&alert_key) {
                                        let alert = Alert {
                                            id: Uuid::new_v4().to_string(),
                                            name: format!("Critical Low Disk Space on {}", metrics.mount_point),
                                            description: format!(
                                                "Free disk space on {} is at {:.2}%, below critical threshold of {}%.",
                                                metrics.mount_point, free_percent, critical_config.threshold_percent
                                            ),
                                            severity: novade_core::types::system_health::AlertSeverity::Critical,
                                            source: "system_health_service".to_string(),
                                            timestamp: std::time::SystemTime::now(),
                                            acknowledged: false,
                                            details: Some(format!("Disk Space Metrics for {}: {:?}", metrics.mount_point, metrics)),
                                        };
                                        current_alerts.insert(alert_key.clone(), alert);
                                    }
                                } else {
                                    current_alerts.remove(&alert_key);
                                }
                                // Found a matching config for this metric, move to next metric
                                // This assumes only one config (critical or warning) should apply per disk.
                                // If multiple configs could apply (e.g. wildcard and specific), behavior might need adjustment.
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                     eprintln!("Failed to get Disk Space metrics for critical alert evaluation: {:?}", e);
                }
            }
        } // If no critical_configs, existing critical disk alerts would persist unless cleared by more general logic.

        // Warning Thresholds (similar logic)
        if let Some(disk_warning_configs) = &config.alert_thresholds.low_disk_space_warnings {
             match self.get_disk_space_metrics().await {
                Ok(disk_metrics_vec) => {
                    for metrics in &disk_metrics_vec {
                        for warning_config in disk_warning_configs {
                            let path_matches = warning_config.device_path_or_mount_point == "*" ||
                                               warning_config.device_path_or_mount_point == metrics.mount_point;

                            if path_matches {
                                let free_percent = if metrics.total_bytes > 0 {
                                    (metrics.free_bytes as f32 / metrics.total_bytes as f32) * 100.0
                                } else { 100.0 };

                                let critical_alert_key = format!("low_disk_space_critical_{}", metrics.mount_point.replace(|c: char| !c.is_alphanumeric(), "_"));
                                let warning_alert_key = format!("low_disk_space_warning_{}", metrics.mount_point.replace(|c: char| !c.is_alphanumeric(), "_"));

                                // Only add/check warning if no critical alert for the same disk is active
                                if !current_alerts.contains_key(&critical_alert_key) {
                                    if free_percent < warning_config.threshold_percent {
                                        if !current_alerts.contains_key(&warning_alert_key) {
                                             let alert = Alert {
                                                id: Uuid::new_v4().to_string(),
                                                name: format!("Low Disk Space Warning on {}", metrics.mount_point),
                                                description: format!(
                                                    "Free disk space on {} is at {:.2}%, below warning threshold of {}%.",
                                                    metrics.mount_point, free_percent, warning_config.threshold_percent
                                                ),
                                                severity: novade_core::types::system_health::AlertSeverity::Warning,
                                                source: "system_health_service".to_string(),
                                                timestamp: std::time::SystemTime::now(),
                                                acknowledged: false,
                                                details: Some(format!("Disk Space Metrics for {}: {:?}", metrics.mount_point, metrics)),
                                            };
                                            current_alerts.insert(warning_alert_key.clone(), alert);
                                        }
                                    } else {
                                         current_alerts.remove(&warning_alert_key); // Clear warning if above threshold
                                    }
                                } else {
                                    // If a critical alert is active, ensure any warning for the same disk is cleared.
                                    current_alerts.remove(&warning_alert_key);
                                }
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get Disk Space metrics for warning alert evaluation: {:?}", e);
                }
            }
        }
        // TODO: Add logic to remove disk alerts if their specific configurations (critical or warning) are removed from CoreConfig.
        // This would involve iterating `current_alerts` for keys matching disk alert patterns and checking
        // if a corresponding configuration (e.g., in `disk_critical_configs` or `disk_warning_configs`) still exists.

        Ok(())
    }
}

#[async_trait::async_trait]
impl SystemHealthService for DefaultSystemHealthService {
    async fn get_cpu_metrics(&self) -> Result<CpuMetrics, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.cpu_collector.collect_cpu_metrics().await.map_err(Into::into)
    }

    async fn get_memory_metrics(&self) -> Result<MemoryMetrics, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.memory_collector.collect_memory_metrics().await.map_err(Into::into)
    }

    async fn get_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.disk_collector.collect_disk_activity_metrics().await.map_err(Into::into)
    }

    async fn get_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.disk_collector.collect_disk_space_metrics().await.map_err(Into::into)
    }

    async fn get_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.network_collector.collect_network_activity_metrics().await.map_err(Into::into)
    }

    async fn get_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.temperature_collector.collect_temperature_metrics().await.map_err(Into::into)
    }

    async fn query_logs(&self, filter: Option<LogFilter>, time_range: Option<TimeRange>, limit: Option<usize>) -> Result<Vec<LogEntry>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.log_harvester.query_logs(filter, time_range, limit).await.map_err(Into::into)
    }

    async fn stream_logs(&self, filter: Option<LogFilter>) -> Result<Box<dyn Stream<Item = Result<LogEntry, SystemHealthError>> + Send + Unpin>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        let system_stream = self.log_harvester.stream_logs(filter).await?;
        // Map the error type within the stream items from novade_system::Error to SystemHealthError
        let domain_stream = system_stream.map_err(SystemHealthError::from);
        Ok(Box::pin(domain_stream))
    }

    fn list_available_diagnostic_tests(&self) -> Result<Vec<DiagnosticTestInfo>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.diagnostic_runner.list_available_tests().map_err(Into::into)
    }

    async fn run_diagnostic_test(&self, test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        self.diagnostic_runner.run_test(test_id).await.map_err(Into::into)
    }

    async fn get_active_alerts(&self) -> Result<Vec<Alert>, SystemHealthError> {
        // TODO: Integration test this service method with mock system collectors/harvesters/runners.
        // TODO: Test error propagation from system layer mock to SystemHealthError.
        // Placeholder implementation.
        // TODO: Implement actual alert logic based on self.core_config.system_health_dashboard thresholds.
        // This would involve:
        // 1. Fetching relevant metrics.
        // 2. Comparing them against thresholds in self.get_dashboard_config().
        // 3. Generating alerts if thresholds are breached.
        // 4. Storing and managing these active_alerts (e.g., in an Arc<Mutex<Vec<Alert>>>).
        // For now, returning an empty Vec.
        if self.core_config.system_health.is_none() { // Corrected field name
            // This check is a placeholder for when SystemHealthDashboardConfig is properly part of CoreConfig
            // It indicates that the config required for evaluating alerts is not available.
             return Ok(Vec::new()); // Or perhaps an error SystemHealthError::ConfigError("Dashboard config missing".to_string())
        }
        // Call evaluate_alerts to update the store before returning.
        // self.evaluate_alerts().await?; // This will be added in the next step.
        let alerts_map = self.active_alerts.lock().await;
        // Call evaluate_alerts to update the store before returning.
        self.evaluate_alerts().await?;
        let alerts_map = self.active_alerts.lock().await;
        Ok(alerts_map.values().cloned().collect())
    }

    // TODO: Implement update_configuration if/when CoreConfig supports dynamic updates
    // and SystemHealthDashboardConfig is part of it.
    // async fn update_configuration(&self, config: SystemHealthDashboardConfig) -> Result<(), SystemHealthError> {
    //     Err(SystemHealthError::ConfigError("Updating dashboard configuration not yet implemented.".to_string()))
    // }
}
