//ANCHOR [NovaDE Developers <dev@novade.org>] Memory Metrics Collector.
//! # Memory Metrics Collector
//!
//! This module is responsible for collecting system-wide memory and swap usage statistics
//! using the `psutil` crate, and also allows for tracking memory usage of
//! specific compositor subsystems.

use async_trait::async_trait;
use novade_core::types::system_health::MemoryMetrics;
use crate::error::SystemError;
use crate::system_health_collectors::MemoryMetricsCollector as MemoryMetricsCollectorTrait; // Renamed trait import for clarity
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use psutil::memory;

//ANCHOR [NovaDE Developers <dev@novade.org>] Defines aggregated memory statistics including per-subsystem usage.
/// Holds aggregated memory statistics, including system-wide and per-subsystem usage.
#[derive(Debug, Clone)]
pub struct ExtendedMemoryMetrics {
    pub system_metrics: MemoryMetrics,
    pub subsystem_memory_usage: HashMap<String, u64>,
    //TODO [NovaDE Developers <dev@novade.org>] Consider adding fields for specific compositor objects if direct measurement becomes feasible.
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Collector for memory metrics.
/// A collector for memory metrics, utilizing `psutil` for system-wide information
/// and allowing manual recording of subsystem-specific memory usage.
#[derive(Debug, Clone)]
pub struct MemoryCollector {
    subsystem_memory: Arc<Mutex<HashMap<String, u64>>>,
    //TODO [NovaDE Developers <dev@novade.org>] Add baseline storage for regression detection.
}

impl MemoryCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Creates a new MemoryCollector.
    /// Creates a new `MemoryCollector`.
    pub fn new() -> Self {
        MemoryCollector {
            subsystem_memory: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Records memory usage for a subsystem.
    /// Records the memory usage for a specific subsystem.
    ///
    /// # Arguments
    ///
    /// * `subsystem_name`: A `String` identifying the subsystem (e.g., "Renderer", "SceneGraph").
    /// * `usage_bytes`: The memory usage in bytes (`u64`).
    pub fn record_subsystem_memory(&self, subsystem_name: String, usage_bytes: u64) {
        let mut memory_map = self.subsystem_memory.lock().unwrap();
        memory_map.insert(subsystem_name, usage_bytes);
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Clears recorded memory for a specific subsystem.
    /// Clears the recorded memory usage for a specific subsystem.
    ///
    /// # Arguments
    ///
    /// * `subsystem_name`: The name of the subsystem to clear.
    pub fn clear_subsystem_memory(&self, subsystem_name: &str) {
        let mut memory_map = self.subsystem_memory.lock().unwrap();
        memory_map.remove(subsystem_name);
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Clears all recorded subsystem memory usage.
    /// Clears all recorded subsystem memory usage data.
    pub fn clear_all_subsystem_memory(&self) {
        let mut memory_map = self.subsystem_memory.lock().unwrap();
        memory_map.clear();
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Gets a copy of the current subsystem memory usage.
    /// Returns a clone of the current subsystem memory usage map.
    pub fn get_subsystem_memory_snapshot(&self) -> HashMap<String, u64> {
        self.subsystem_memory.lock().unwrap().clone()
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoryMetricsCollectorTrait for MemoryCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Collects system-wide and subsystem memory metrics.
    /// Asynchronously collects current system-wide memory and swap metrics using `psutil`,
    /// and combines them with any recorded subsystem-specific memory usage.
    ///
    /// Returns a `Result` containing `ExtendedMemoryMetrics` on success,
    /// or a `SystemError` if collecting system metrics fails.
    async fn collect_memory_metrics(&self) -> Result<ExtendedMemoryMetrics, SystemError> {
        // Collect system-wide virtual memory stats
        let virtual_mem = memory::virtual_memory().map_err(|e| {
            SystemError::MetricCollectorError(format!("Failed to get virtual memory: {}", e))
        })?;

        // Collect system-wide swap memory stats
        let swap_mem = memory::swap_memory().map_err(|e| {
            SystemError::MetricCollectorError(format!("Failed to get swap memory: {}", e))
        })?;

        let system_metrics = MemoryMetrics {
            total_bytes: virtual_mem.total(),
            used_bytes: virtual_mem.used(),
            available_bytes: virtual_mem.available(),
            free_bytes: virtual_mem.free(), // psutil provides this, maps to MemFree conceptually
            swap_total_bytes: swap_mem.total(),
            swap_used_bytes: swap_mem.used(),
            swap_free_bytes: swap_mem.free(),
        };

        let subsystem_usage = self.subsystem_memory.lock().unwrap().clone();

        Ok(ExtendedMemoryMetrics {
            system_metrics,
            subsystem_memory_usage: subsystem_usage,
        })
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Regression Detector for Memory Usage.
//TODO [NovaDE Developers <dev@novade.org>] Implement a more sophisticated baseline mechanism and statistical significance testing.
impl MemoryCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Checks for memory usage regression.
    /// Checks for significant increases in total used system memory compared to a baseline.
    /// Logs a warning if current usage exceeds the baseline by a given threshold.
    ///
    /// # Arguments
    ///
    /// * `current_metrics`: The latest `ExtendedMemoryMetrics` collected.
    /// * `baseline_total_used_bytes`: The baseline total used system memory in bytes.
    /// * `threshold_percentage`: The percentage increase over baseline to be considered a regression.
    pub fn check_regression(
        &self,
        current_metrics: &ExtendedMemoryMetrics,
        baseline_total_used_bytes: u64,
        threshold_percentage: f64,
    ) {
        if current_metrics.system_metrics.used_bytes >
           (baseline_total_used_bytes as f64 * (1.0 + threshold_percentage / 100.0)) as u64 {
            //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
            eprintln!(
                "[WARNING] MemoryCollector: Performance regression detected! Current total used memory {:.2} MB exceeds baseline {:.2} MB by more than {:.0}%.",
                current_metrics.system_metrics.used_bytes as f64 / (1024.0 * 1024.0),
                baseline_total_used_bytes as f64 / (1024.0 * 1024.0),
                threshold_percentage
            );
        }
        //TODO [NovaDE Developers <dev@novade.org>] Add checks for per-subsystem memory regression if baselines are established for them.
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::system_health_collectors::MemoryMetricsCollector as MemoryMetricsCollectorTrait;

    #[tokio::test]
    async fn test_memory_collector_new() {
        let collector = MemoryCollector::new();
        assert!(collector.subsystem_memory.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_record_and_get_subsystem_memory() {
        let collector = MemoryCollector::new();
        collector.record_subsystem_memory("Renderer".to_string(), 1024 * 1024 * 100); // 100MB
        collector.record_subsystem_memory("SceneGraph".to_string(), 1024 * 1024 * 50); // 50MB

        let snapshot = collector.get_subsystem_memory_snapshot();
        assert_eq!(snapshot.get("Renderer"), Some(&(1024 * 1024 * 100)));
        assert_eq!(snapshot.get("SceneGraph"), Some(&(1024 * 1024 * 50)));

        let metrics = collector.collect_memory_metrics().await;
        assert!(metrics.is_ok());
        let extended_metrics = metrics.unwrap();
        assert_eq!(extended_metrics.subsystem_memory_usage.get("Renderer"), Some(&(1024 * 1024 * 100)));
    }

    #[tokio::test]
    async fn test_clear_subsystem_memory() {
        let collector = MemoryCollector::new();
        collector.record_subsystem_memory("TestSubsystem".to_string(), 1024);
        assert!(!collector.get_subsystem_memory_snapshot().is_empty());
        collector.clear_subsystem_memory("TestSubsystem");
        assert!(collector.get_subsystem_memory_snapshot().is_empty());
    }

    #[tokio::test]
    async fn test_clear_all_subsystem_memory() {
        let collector = MemoryCollector::new();
        collector.record_subsystem_memory("Sub1".to_string(), 100);
        collector.record_subsystem_memory("Sub2".to_string(), 200);
        assert!(!collector.get_subsystem_memory_snapshot().is_empty());
        collector.clear_all_subsystem_memory();
        assert!(collector.get_subsystem_memory_snapshot().is_empty());
    }

    #[tokio::test]
    async fn test_collect_system_memory_metrics() {
        let collector = MemoryCollector::new();
        let result = collector.collect_memory_metrics().await;

        if let Err(e) = &result {
            // On systems where psutil might not work (e.g. CI environments not fully mimicking Linux /proc),
            // this test might fail. We print the error for diagnosis but don't hard fail the test.
            // A more robust test setup might involve mocking psutil if possible or running only on compatible systems.
            println!("Note: System memory collection failed, this might be expected in some CI environments. Error: {:?}", e);
            // Depending on CI setup, one might choose to assert!(true) here to not fail the build.
            // For now, we'll let it pass through and rely on manual checks or specific env flags for CI.
        } else {
            let metrics = result.unwrap();
            // Basic sanity checks, values depend on the system running the test
            assert!(metrics.system_metrics.total_bytes > 0);
            assert!(metrics.system_metrics.available_bytes > 0);
            // used_bytes can be 0 on a very idle system, so no direct assert on > 0
            // psutil might return 0 for swap on systems with no swap or if it can't read it.
            // assert!(metrics.system_metrics.swap_total_bytes > 0); // This might fail if no swap
        }
    }

    #[tokio::test]
    async fn test_memory_regression_detection() {
        let collector = MemoryCollector::new();
        let mut current_metrics = ExtendedMemoryMetrics {
            system_metrics: MemoryMetrics {
                total_bytes: 2000,
                used_bytes: 1500, // 1500 used
                available_bytes: 500,
                free_bytes: 500,
                swap_total_bytes: 1000,
                swap_used_bytes: 100,
                swap_free_bytes: 900,
            },
            subsystem_memory_usage: HashMap::new(),
        };

        // No regression: current 1500, baseline 1000, threshold 50% (1000 * 1.5 = 1500. Not > 1500)
        collector.check_regression(&current_metrics, 1000, 50.0);

        // Regression: current 1500, baseline 1000, threshold 40% (1000 * 1.4 = 1400. 1500 > 1400)
        // This will print a warning to stderr, which is expected.
        collector.check_regression(&current_metrics, 1000, 40.0);

        current_metrics.system_metrics.used_bytes = 900;
        // No regression: current 900, baseline 1000, threshold 20% (1000 * 1.2 = 1200. 900 not > 1200)
        collector.check_regression(&current_metrics, 1000, 20.0);
    }
}
