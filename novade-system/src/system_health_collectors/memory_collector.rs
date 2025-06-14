//! # Memory Metrics Collector
//!
//! This module is responsible for collecting memory and swap usage statistics
//! from the Linux `/proc/meminfo` file.

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncReadExt}; // BufReader not strictly needed
use novade_core::types::system_health::MemoryMetrics;
use crate::error::SystemError;
use crate::system_health_collectors::MemoryMetricsCollector;
use std::collections::HashMap;

/// A collector for memory metrics on Linux systems.
///
/// Implements the `MemoryMetricsCollector` trait by parsing data from `/proc/meminfo`.
/// It extracts total RAM, free RAM, available RAM, total swap, and free swap,
/// then calculates used RAM and used swap. All values are reported in bytes.
pub struct LinuxMemoryMetricsCollector;

/// Asynchronously reads and parses `/proc/meminfo`.
///
/// Iterates through lines in `/proc/meminfo`, extracting key-value pairs.
/// Keys are strings (e.g., "MemTotal", "MemFree"), and values are parsed as u64 (kilobytes)
/// and then converted to bytes.
///
/// Returns a `Result` containing a `HashMap<String, u64>` where values are in bytes,
/// or a `SystemError` if reading or parsing fails.
async fn read_proc_meminfo() -> Result<HashMap<String, u64>, SystemError> {
    let mut file = File::open("/proc/meminfo").await.map_err(|e| SystemError::MetricCollectorError(format!("Failed to open /proc/meminfo: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.map_err(|e| SystemError::MetricCollectorError(format!("Failed to read /proc/meminfo: {}", e)))?;

    let mut mem_info = HashMap::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].trim_end_matches(':').to_string();
            if let Ok(value) = parts[1].parse::<u64>() {
                // Values in /proc/meminfo are in kB, convert to bytes
                mem_info.insert(key, value * 1024);
            }
        }
    }
    Ok(mem_info)
    // TODO: Unit test parsing of /proc/meminfo mock data.
}

#[async_trait::async_trait]
impl MemoryMetricsCollector for LinuxMemoryMetricsCollector {
    /// Asynchronously collects current memory and swap metrics.
    ///
    /// Parses `/proc/meminfo` to obtain values for total, free, and available memory,
    /// as well as total and free swap space. Calculates used memory based on
    /// `total - available` and used swap as `swap_total - swap_free`.
    /// `MemAvailable` is preferred over `MemFree` for a more realistic measure of
    /// available RAM for new applications.
    ///
    /// Returns a `Result` containing `MemoryMetrics` (all values in bytes) on success,
    /// or a `SystemError` if essential fields are missing from `/proc/meminfo` or parsing fails.
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, SystemError> {
        let mem_info = read_proc_meminfo().await?;

        let total_bytes = *mem_info.get("MemTotal").ok_or_else(|| SystemError::MetricCollectorError("MemTotal not found in /proc/meminfo".into()))?;
        let free_bytes = *mem_info.get("MemFree").ok_or_else(|| SystemError::MetricCollectorError("MemFree not found in /proc/meminfo".into()))?;
        // MemAvailable is generally preferred over MemFree for available memory
        let available_bytes = *mem_info.get("MemAvailable").unwrap_or(&free_bytes);

        let used_bytes = total_bytes - available_bytes;

        let swap_total_bytes = *mem_info.get("SwapTotal").ok_or_else(|| SystemError::MetricCollectorError("SwapTotal not found in /proc/meminfo".into()))?;
        let swap_free_bytes = *mem_info.get("SwapFree").ok_or_else(|| SystemError::MetricCollectorError("SwapFree not found in /proc/meminfo".into()))?;
        let swap_used_bytes = swap_total_bytes - swap_free_bytes;

        Ok(MemoryMetrics {
            total_bytes,
            used_bytes,
            available_bytes,
            free_bytes, // Still useful to report
            swap_total_bytes,
            swap_used_bytes,
            swap_free_bytes, // Still useful to report
        })
    }
}
