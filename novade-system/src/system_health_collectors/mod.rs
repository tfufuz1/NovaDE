//! # System Health Collectors
//!
//! This module provides traits and implementations for collecting various system health metrics,
//! logs, and running diagnostics. It serves as the system-specific backend for the
//! `SystemHealthService` in the domain layer.
//!
//! Each collector is responsible for gathering data from the underlying operating system
//! (primarily Linux-focused, using `/proc` and `/sys` filesystems or system utilities).
//! They are designed to be used by the domain layer, which abstracts away the
//! system-specific details.

use novade_core::types::system_health::{
    CpuMetrics, DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiskActivityMetrics,
    DiskSpaceMetrics, LogEntry, LogFilter, MemoryMetrics, NetworkActivityMetrics,
    TemperatureMetric, TimeRange,
};
use crate::error::SystemError;
use futures_core::Stream; // Add if not already used

pub mod cpu_collector;
pub mod memory_collector;
pub mod disk_collector;
pub mod network_collector;
pub mod temperature_collector;
pub mod journald_harvester;
pub mod basic_diagnostics_runner;
// Optionally: pub mod disk_smart_runner;

// Re-export collectors and traits
pub use cpu_collector::LinuxCpuMetricsCollector;
pub use memory_collector::LinuxMemoryMetricsCollector;
pub use disk_collector::LinuxDiskMetricsCollector;
pub use network_collector::LinuxNetworkMetricsCollector;
pub use temperature_collector::LinuxTemperatureMetricsCollector;
pub use journald_harvester::JournaldLogHarvester;
pub use basic_diagnostics_runner::BasicDiagnosticsRunner;

pub use self::{
    CpuMetricsCollector, DiagnosticRunner, DiskMetricsCollector, LogHarvester,
    MemoryMetricsCollector, NetworkMetricsCollector, TemperatureMetricsCollector, // DiagnosticRunner already included
};

/// Defines the contract for CPU metrics collection.
#[async_trait::async_trait]
pub trait CpuMetricsCollector {
    /// Asynchronously collects current CPU metrics.
    ///
    /// Returns a `Result` containing `CpuMetrics` on success, or a `SystemError` on failure.
    /// Common errors include inability to read or parse `/proc/stat`.
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, SystemError>;
}

/// Defines the contract for memory metrics collection.
#[async_trait::async_trait]
pub trait MemoryMetricsCollector {
    /// Asynchronously collects current memory and swap usage metrics.
    ///
    /// Returns a `Result` containing `MemoryMetrics` on success, or a `SystemError` on failure.
    /// Common errors include inability to read or parse `/proc/meminfo`.
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, SystemError>;
}

/// Defines the contract for disk I/O activity and space usage metrics collection.
#[async_trait::async_trait]
pub trait DiskMetricsCollector {
    /// Asynchronously collects current disk I/O activity metrics for relevant block devices.
    /// This typically involves calculating rates (reads/s, writes/s, bytes/s) by sampling
    /// `/proc/diskstats` at two points in time.
    ///
    /// Returns a `Result` containing a `Vec<DiskActivityMetrics>` on success, or a `SystemError`.
    async fn collect_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, SystemError>;

    /// Asynchronously collects current disk space usage metrics for mounted filesystems.
    /// This involves parsing `/proc/mounts` and using `statvfs` for each mount point.
    ///
    /// Returns a `Result` containing a `Vec<DiskSpaceMetrics>` on success, or a `SystemError`.
    async fn collect_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, SystemError>;
}

/// Defines the contract for network interface activity metrics collection.
#[async_trait::async_trait]
pub trait NetworkMetricsCollector {
    /// Asynchronously collects current network activity metrics (bytes/s, packets/s, errors/s)
    /// for relevant network interfaces. This typically involves calculating rates by sampling
    /// `/proc/net/dev` at two points in time.
    ///
    /// Returns a `Result` containing a `Vec<NetworkActivityMetrics>` on success, or a `SystemError`.
    async fn collect_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, SystemError>;
}

/// Defines the contract for temperature metrics collection from system sensors.
#[async_trait::async_trait]
pub trait TemperatureMetricsCollector {
    /// Asynchronously collects current temperature readings from available system sensors.
    /// This typically involves reading from `/sys/class/thermal/thermal_zone*`.
    ///
    /// Returns a `Result` containing a `Vec<TemperatureMetric>` on success, or a `SystemError`.
    async fn collect_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, SystemError>;
}

/// Defines the contract for harvesting system logs.
#[async_trait::async_trait]
pub trait LogHarvester {
    /// Asynchronously streams log entries based on the provided filter.
    ///
    /// # Arguments
    /// * `filter`: An optional `LogFilter` to apply to the stream.
    ///
    /// Returns a `Result` containing a boxed dynamic `Stream` of `LogEntry` results,
    /// or a `SystemError` if the stream cannot be established.
    async fn stream_logs(&self, filter: Option<LogFilter>) -> Result<Box<dyn Stream<Item = Result<LogEntry, SystemError>> + Send + Unpin>, SystemError>;

    /// Asynchronously queries historical log entries based on the provided filter, time range, and limit.
    ///
    /// # Arguments
    /// * `filter`: An optional `LogFilter` to apply.
    /// * `time_range`: An optional `TimeRange` to restrict the query period.
    /// * `limit`: An optional `usize` to cap the number of returned log entries.
    ///
    /// Returns a `Result` containing a `Vec<LogEntry>` on success, or a `SystemError`.
    async fn query_logs(&self, filter: Option<LogFilter>, time_range: Option<TimeRange>, limit: Option<usize>) -> Result<Vec<LogEntry>, SystemError>;
}

/// Defines the contract for running diagnostic tests.
#[async_trait::async_trait]
pub trait DiagnosticRunner {
    /// Returns a list of available diagnostic tests this runner can perform.
    /// Each test is described by `DiagnosticTestInfo`.
    ///
    /// Returns a `Result` containing a `Vec<DiagnosticTestInfo>` on success, or a `SystemError`.
    fn list_available_tests(&self) -> Result<Vec<DiagnosticTestInfo>, SystemError>;

    /// Asynchronously runs a specific diagnostic test identified by `test_id`.
    ///
    /// # Arguments
    /// * `test_id`: The `DiagnosticTestId` of the test to run.
    ///
    /// Returns a `Result` containing a `DiagnosticTestResult` on success, or a `SystemError`.
    async fn run_test(&self, test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, SystemError>;
}
