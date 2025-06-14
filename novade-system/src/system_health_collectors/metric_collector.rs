// novade-system/src/system_health_collectors/metric_collector.rs

use async_trait::async_trait;
use novade_core::types::system_health::{
    CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric,
};
// This trait is defined in novade-domain, so we need to correctly reference it.
// This might require a direct dependency or a more elaborate setup if circular deps are an issue.
// For now, assume it's accessible. A type alias or re-export might be needed in a shared crate if necessary.
use async_trait::async_trait;
use novade_core::types::system_health::{
    CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric,
};
use novade_domain::system_health_service::service::MetricCollectorAdapter;
use novade_domain::system_health_service::error::SystemHealthError; // Use domain error for adapter contract
use super::error::CollectionError; // System-specific errors

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct SystemMetricCollector {
    // Potentially holds handles to system resources or configuration for collection.
    // For example, a list of disks or network interfaces to monitor, obtained from config.
}

impl SystemMetricCollector {
    pub fn new() -> Self {
        // Initialize any necessary resources or state
        SystemMetricCollector {}
    }

    // Helper to convert specific CollectionError to SystemHealthError
    // #[allow(dead_code)] // This will be used when actual collection logic is added
    fn to_domain_error(err: CollectionError, metric_name: &str) -> SystemHealthError {
        SystemHealthError::MetricCollectionError {
            metric_name: metric_name.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

#[async_trait]
impl MetricCollectorAdapter for SystemMetricCollector {
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, SystemHealthError> {
        #[cfg(target_os = "linux")]
        {
            println!("SystemLayer: `collect_cpu_metrics` called (basic Linux /proc/stat attempt - returning dummy).");
            // Actual /proc/stat parsing for timediff CPU usage is complex for this step.
            // Returning a fixed dummy value.
            Ok(CpuMetrics { total_usage_percent: 10.0, per_core_usage_percent: vec![] })
        }
        #[cfg(not(target_os = "linux"))]
        {
            println!("SystemLayer: `collect_cpu_metrics` not implemented for this OS.");
            Err(Self::to_domain_error(CollectionError::NotImplemented("collect_cpu_metrics on non-Linux".to_string()), "CPU"))
        }
    }

    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, SystemHealthError> {
        #[cfg(target_os = "linux")]
        {
            println!("SystemLayer: `collect_memory_metrics` called (basic Linux /proc/meminfo attempt).");
            let mut mem_info = HashMap::new();
            let file = File::open("/proc/meminfo")
                .map_err(|e| Self::to_domain_error(CollectionError::OsResourceError{resource: "/proc/meminfo".to_string(), io_error: e}, "Memory"))?;
            let reader = BufReader::new(file);
            for line_result in reader.lines() {
                let line = line_result.map_err(|e| Self::to_domain_error(CollectionError::OsResourceError{resource: "line from /proc/meminfo".to_string(), io_error: e}, "Memory"))?;
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim_end_matches(':');
                    if let Ok(value) = parts[1].parse::<u64>() {
                        mem_info.insert(key.to_string(), value * 1024); // Values are in kB, convert to Bytes
                    }
                }
            }

            let total_bytes = mem_info.get("MemTotal").cloned().unwrap_or(0);
            let mem_free = mem_info.get("MemFree").cloned().unwrap_or(0);
            // MemAvailable is generally preferred over MemFree for "available" memory
            let mem_available = mem_info.get("MemAvailable").cloned().unwrap_or(mem_free);
            let used_bytes = total_bytes.saturating_sub(mem_available);

            let swap_total_bytes = mem_info.get("SwapTotal").cloned().unwrap_or(0);
            let swap_free_bytes = mem_info.get("SwapFree").cloned().unwrap_or(0);
            let swap_used_bytes = swap_total_bytes.saturating_sub(swap_free_bytes);

            Ok(MemoryMetrics {
                total_bytes,
                used_bytes,
                free_bytes: mem_free, // raw free
                available_bytes: mem_available, // typically what users consider "available"
                swap_total_bytes,
                swap_used_bytes,
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            println!("SystemLayer: `collect_memory_metrics` not implemented for this OS.");
            Err(Self::to_domain_error(CollectionError::NotImplemented("collect_memory_metrics on non-Linux".to_string()), "Memory"))
        }
    }

    async fn collect_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, SystemHealthError> {
        // Placeholder: Read from /proc/diskstats or similar
        println!("SystemLayer: `collect_disk_activity_metrics` called (placeholder)");
        Ok(vec![])
    }

    async fn collect_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, SystemHealthError> {
        // Placeholder: Use `statvfs` or similar for each mounted filesystem
        println!("SystemLayer: `collect_disk_space_metrics` called (placeholder)");
        Ok(vec![])
    }

    async fn collect_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, SystemHealthError> {
        // Placeholder: Read from /proc/net/dev or similar
        println!("SystemLayer: `collect_network_activity_metrics` called (placeholder)");
        Ok(vec![])
    }

    async fn collect_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, SystemHealthError> {
        // Placeholder: Read from /sys/class/hwmon or similar
        println!("SystemLayer: `collect_temperature_metrics` called (placeholder)");
        Ok(vec![])
    }
}
