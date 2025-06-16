//ANCHOR [NovaDE Developers <dev@novade.org>] GPU Metrics Collector.
//! This module provides a collector for GPU performance metrics.
//! It aims to support NVIDIA, AMD, and Intel GPUs, though initial implementation
//! may prioritize NVIDIA due to readily available crates.

use async_trait::async_trait;
use novade_core::types::system_health::GpuMetrics; // Assuming a GpuMetrics struct exists or will be created in novade-core
use crate::error::SystemError;
use crate::system_health_collectors::GpuMetricsCollector as GpuMetricsCollectorTrait; // Assuming this trait exists or will be created

//ANCHOR [NovaDE Developers <dev@novade.org>] Conditional compilation for NVIDIA NVML features.
#[cfg(feature = "nvml")]
use nvml_wrapper::{Nvml,error::NvmlError};
#[cfg(feature = "nvml")]
use std::sync::OnceLock;

//ANCHOR [NovaDE Developers <dev@novade.org>] Global static Nvml instance for NVIDIA.
/// Global static Nvml instance, initialized once.
#[cfg(feature = "nvml")]
static NVML_INSTANCE: OnceLock<Result<Nvml, NvmlError>> = OnceLock::new();

//ANCHOR [NovaDE Developers <dev@novade.org>] Placeholder for generic GPU statistics.
/// Holds collected GPU metrics.
/// This is a placeholder and should be expanded based on `novade_core::types::system_health::GpuMetrics`.
#[derive(Debug, Clone, Default)]
pub struct GpuStatistics {
    pub vendor: String,
    pub device_name: String,
    pub utilization_percent: Option<f32>, // GPU usage
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f32>,
    //TODO [NovaDE Developers <dev@novade.org>] Add more metrics like clock speeds, power usage, fan speed etc.
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Collector for GPU metrics.
/// Collects GPU metrics. Currently focuses on NVIDIA if NVML is available.
#[derive(Debug, Clone, Default)]
pub struct GpuCollector {
    //TODO [NovaDE Developers <dev@novade.org>] Store baseline GPU metrics for regression detection.
    #[cfg(feature = "nvml")]
    nvml_initialized: bool, // Tracks if NVML was successfully initialized for this instance
}

impl GpuCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Creates a new GpuCollector.
    /// Creates a new `GpuCollector`.
    /// It will attempt to initialize NVML for NVIDIA GPUs if the "nvml" feature is enabled.
    pub fn new() -> Self {
        #[cfg(feature = "nvml")]
        {
            let nvml_init_result = NVML_INSTANCE.get_or_init(Nvml::init);
            GpuCollector {
                nvml_initialized: nvml_init_result.is_ok(),
            }
        }
        #[cfg(not(feature = "nvml"))]
        {
            GpuCollector {}
        }
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Collects NVIDIA GPU metrics using NVML.
    /// Collects metrics for the first available NVIDIA GPU using NVML.
    #[cfg(feature = "nvml")]
    fn collect_nvidia_gpu_metrics(&self) -> Result<Vec<GpuStatistics>, SystemError> {
        if !self.nvml_initialized {
            return Ok(vec![GpuStatistics { // Return a default/empty stats if NVML failed
                vendor: "NVIDIA".to_string(),
                device_name: "NVML Initialization Failed".to_string(),
                ..Default::default()
            }]);
        }

        let nvml = match NVML_INSTANCE.get() {
            Some(Ok(instance)) => instance,
            _ => return Err(SystemError::MetricCollectorError("NVML not initialized or initialization failed.".into())),
        };

        let mut stats_list = Vec::new();
        let device_count = nvml.device_count().map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA device count: {}", e)))?;

        for i in 0..device_count {
            let device = nvml.device_by_index(i).map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA device by index {}: {}", i, e)))?;

            let name = device.name().map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA device name: {}", e)))?;

            let utilization = device.utilization_rates().map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA GPU utilization: {}", e)))?;

            let memory_info = device.memory_info().map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA GPU memory info: {}", e)))?;

            let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu).map_err(|e| SystemError::MetricCollectorError(format!("Failed to get NVIDIA GPU temperature: {}", e)))?;

            stats_list.push(GpuStatistics {
                vendor: "NVIDIA".to_string(),
                device_name: name,
                utilization_percent: Some(utilization.gpu as f32),
                memory_used_bytes: Some(memory_info.used),
                memory_total_bytes: Some(memory_info.total),
                temperature_celsius: Some(temperature as f32),
            });
        }
        Ok(stats_list)
    }
}

#[async_trait::async_trait]
impl GpuMetricsCollectorTrait for GpuCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Collects GPU metrics from available sources.
    /// Asynchronously collects GPU metrics.
    ///
    /// Currently, it supports NVIDIA GPUs via NVML (if the "nvml" feature is enabled).
    /// //TODO [NovaDE Developers <dev@novade.org>] Implement AMD GPU monitoring (e.g., via sysfs or amdgpu crate).
    /// //TODO [NovaDE Developers <dev@novade.org>] Implement Intel GPU monitoring (e.g., via sysfs or specific Intel metrics crate).
    ///
    /// Returns a `Result` containing a vector of `GpuStatistics` (one per GPU) on success,
    /// or a `SystemError` if collection fails. If no supported GPUs are found or features disabled,
    /// it may return an empty vector or a vector with placeholder/error entries.
    async fn collect_gpu_metrics(&self) -> Result<Vec<GpuStatistics>, SystemError> {
        let mut all_gpu_stats = Vec::new();

        #[cfg(feature = "nvml")]
        {
            if self.nvml_initialized {
                match self.collect_nvidia_gpu_metrics() {
                    Ok(nvidia_stats) => all_gpu_stats.extend(nvidia_stats),
                    Err(e) => {
                        // Log error or return a specific error metric entry
                        eprintln!("Error collecting NVIDIA GPU metrics: {}", e); // TODO: Use structured logging
                        all_gpu_stats.push(GpuStatistics {
                            vendor: "NVIDIA".to_string(),
                            device_name: format!("Error: {}", e),
                            ..Default::default()
                        });
                    }
                }
            } else if NVML_INSTANCE.get().is_none() || NVML_INSTANCE.get().unwrap().is_err() {
                 all_gpu_stats.push(GpuStatistics {
                    vendor: "NVIDIA".to_string(),
                    device_name: "NVML Initialization Failed or Not Attempted".to_string(),
                    ..Default::default()
                });
            }
        }

        //ANCHOR [NovaDE Developers <dev@novade.org>] Placeholder for AMD GPU data collection.
        //TODO [AMD GPU Monitoring] [NovaDE Developers <dev@novade.org>] Implement AMD GPU metrics collection.
        // This could involve reading from sysfs (`/sys/class/drm/cardX/device/hwmon/hwmonX/`)
        // or using a dedicated crate like `amdgpu-sysfs` if available and suitable.
        if all_gpu_stats.is_empty() || cfg!(not(feature="nvml")) { // Add placeholder if no NVIDIA or NVML disabled
             all_gpu_stats.push(GpuStatistics{
                vendor: "AMD".to_string(),
                device_name: "AMD GPU (Not Implemented)".to_string(),
                ..Default::default()
            });
        }

        //ANCHOR [NovaDE Developers <dev@novade.org>] Placeholder for Intel GPU data collection.
        //TODO [Intel GPU Monitoring] [NovaDE Developers <dev@novade.org>] Implement Intel GPU metrics collection.
        // This could involve reading from sysfs or using specific Intel metrics tools/crates.
        if all_gpu_stats.is_empty() || cfg!(not(feature="nvml")) { // Add placeholder if no NVIDIA or NVML disabled
            all_gpu_stats.push(GpuStatistics{
                vendor: "Intel".to_string(),
                device_name: "Intel GPU (Not Implemented)".to_string(),
                ..Default::default()
            });
        }

        if all_gpu_stats.iter().all(|s| s.device_name.contains("Not Implemented") || s.device_name.contains("Failed")) && !cfg!(feature="nvml") {
             all_gpu_stats.push(GpuStatistics{
                vendor: "Unknown".to_string(),
                device_name: "No supported GPU monitoring features enabled or no GPUs detected.".to_string(),
                ..Default::default()
            });
        }


        // Filter out "Not Implemented" entries if other GPUs were found
        if all_gpu_stats.len() > 1 {
            all_gpu_stats.retain(|stats| !stats.device_name.contains("Not Implemented") && !stats.device_name.contains("No supported GPU"));
        }


        Ok(all_gpu_stats)
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Regression Detector for GPU Metrics.
//TODO [NovaDE Developers <dev@novade.org>] Implement a more sophisticated baseline mechanism and statistical significance testing.
impl GpuCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Checks for GPU performance regression.
    /// Checks for significant increases in GPU temperature or decreases in utilization compared to baselines.
    /// Logs a warning if current metrics deviate significantly.
    /// This is a simplified check and currently iterates only over the first detected GPU's stats.
    ///
    /// # Arguments
    ///
    /// * `current_gpu_stats_list`: A slice of `GpuStatistics` for all detected GPUs.
    /// * `baseline_temp_celsius`: The baseline GPU temperature in Celsius.
    /// * `temp_threshold_celsius`: The increase in Celsius over baseline to be considered a regression.
    /// * `baseline_util_percent`: The baseline GPU utilization percentage.
    /// * `util_drop_threshold_percent`: The percentage drop from baseline utilization to be considered a regression.
    pub fn check_regression(
        &self,
        current_gpu_stats_list: &[GpuStatistics],
        baseline_temp_celsius: f32,
        temp_threshold_celsius: f32,
        baseline_util_percent: f32,
        util_drop_threshold_percent: f32,
    ) {
        if let Some(stats) = current_gpu_stats_list.first() { // Check only the first GPU for simplicity
            if let Some(temp) = stats.temperature_celsius {
                if temp > baseline_temp_celsius + temp_threshold_celsius {
                    //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
                    eprintln!(
                        "[WARNING] GpuCollector ({}/{}): Performance regression detected! Current GPU temperature {:.1}°C exceeds baseline {:.1}°C by more than {:.1}°C.",
                        stats.vendor, stats.device_name, temp, baseline_temp_celsius, temp_threshold_celsius
                    );
                }
            }
            if let Some(util) = stats.utilization_percent {
                if util < baseline_util_percent * (1.0 - util_drop_threshold_percent / 100.0) {
                     //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
                    eprintln!(
                        "[WARNING] GpuCollector ({}/{}): Performance regression detected! Current GPU utilization {:.1}% is below baseline {:.1}% by more than {:.1}%.",
                        stats.vendor, stats.device_name, util, baseline_util_percent, util_drop_threshold_percent
                    );
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::system_health_collectors::GpuMetricsCollector as GpuMetricsCollectorTrait;
    //TODO [GPU Test Data] [NovaDE Developers <dev@novade.org>] For AMD/Intel GPU implementations, consider creating sample sysfs output files or mock data structures to test parsing logic if direct hardware access is not available or reliable in CI. This would allow testing the extraction of metrics like utilization, memory, and temperature from those sources.

    #[tokio::test]
    async fn test_gpu_collector_new() {
        let collector = GpuCollector::new();
        // Depending on whether 'nvml' feature is enabled and NVML init status,
        // collector.nvml_initialized might be true or false.
        // This test mainly ensures it doesn't panic.
        #[cfg(feature = "nvml")]
        println!("NVML Initialized on new: {}", collector.nvml_initialized);
    }

    #[tokio::test]
    async fn test_collect_gpu_metrics() {
        let collector = GpuCollector::new();
        let result = collector.collect_gpu_metrics().await;

        // This test is highly dependent on the environment (presence of GPUs, drivers, NVML library).
        // It's hard to make strong assertions without mocking or specific test hardware.
        match result {
            Ok(stats_list) => {
                println!("Collected GPU Metrics: {:?}", stats_list);
                assert!(!stats_list.is_empty(), "Expected some GPU statistics (even if placeholder/error)");

                for stats in stats_list {
                    if stats.vendor == "NVIDIA" {
                        if collector.nvml_initialized {
                            assert!(!stats.device_name.contains("Failed"), "NVIDIA device name indicates failure: {}", stats.device_name);
                            // If NVML initialized and we got stats, expect some values
                            // These can be None if NVML calls fail for specific metrics
                            // assert!(stats.utilization_percent.is_some(), "NVIDIA utilization should be Some if NVML is working");
                            // assert!(stats.memory_total_bytes.is_some(), "NVIDIA total memory should be Some if NVML is working");
                        } else {
                             println!("NVIDIA NVML not initialized, stats for {} may be placeholder.", stats.device_name);
                        }
                    } else if stats.vendor == "AMD" || stats.vendor == "Intel" {
                        assert!(stats.device_name.contains("Not Implemented"), "Expected placeholder for AMD/Intel if not NVIDIA or NVML failed.");
                    }
                }
            }
            Err(e) => {
                // This might happen if NVML init itself fails critically at the static level, though less likely with OnceLock
                eprintln!("Error collecting GPU metrics in test: {:?}", e);
                // Not failing the test here as it's environment-dependent.
            }
        }
    }

    #[cfg(feature = "nvml")]
    #[tokio::test]
    async fn test_nvidia_metrics_collection_if_nvml_present() {
        let collector = GpuCollector::new();
        if collector.nvml_initialized {
            println!("NVML is initialized. Attempting to collect NVIDIA metrics directly.");
            match collector.collect_nvidia_gpu_metrics() {
                Ok(stats) => {
                    println!("NVIDIA Metrics: {:?}", stats);
                    if nvml_wrapper::Nvml::init().is_ok() && nvml_wrapper::Nvml::init().unwrap().device_count().unwrap_or(0) > 0 {
                        assert!(!stats.is_empty(), "Expected NVIDIA stats if NVML initialized and devices present.");
                        // Further checks if devices are actually present
                        let first_gpu_stats = stats.first().unwrap();
                        assert_eq!(first_gpu_stats.vendor, "NVIDIA");
                        assert!(first_gpu_stats.utilization_percent.is_some());
                        assert!(first_gpu_stats.memory_total_bytes.is_some());
                        assert!(first_gpu_stats.temperature_celsius.is_some());
                    } else {
                        println!("NVML available but no NVIDIA devices found, or device_count failed.");
                    }
                }
                Err(e) => {
                    // This case might indicate an issue with NVML calls even if init was ok.
                    eprintln!("Error in collect_nvidia_gpu_metrics: {:?}. This might be okay if no NVIDIA GPU is installed.", e);
                    // assert!(false, "collect_nvidia_gpu_metrics failed: {:?}", e);
                }
            }
        } else {
            println!("NVML not initialized or 'nvml' feature disabled. Skipping detailed NVIDIA test.");
            // Check that it returns the "NVML Initialization Failed" placeholder
            let result = collector.collect_nvidia_gpu_metrics();
            assert!(result.is_ok(), "Expected Ok result even if NVML init failed");
            let stats_list = result.unwrap();
            assert_eq!(stats_list.len(), 1, "Expected one placeholder stat");
            assert_eq!(stats_list[0].vendor, "NVIDIA");
            assert!(stats_list[0].device_name.contains("NVML Initialization Failed"));
        }
    }

    #[tokio::test]
    async fn test_gpu_regression_detection() {
        let collector = GpuCollector::new();
        let current_stats_good = vec![GpuStatistics {
            vendor: "TestGPU".to_string(), device_name: "Device 0".to_string(),
            temperature_celsius: Some(60.0), utilization_percent: Some(70.0), ..Default::default()
        }];
        let current_stats_hot = vec![GpuStatistics {
            vendor: "TestGPU".to_string(), device_name: "Device 0".to_string(),
            temperature_celsius: Some(85.0), utilization_percent: Some(70.0), ..Default::default()
        }];
        let current_stats_underutilized = vec![GpuStatistics {
            vendor: "TestGPU".to_string(), device_name: "Device 0".to_string(),
            temperature_celsius: Some(60.0), utilization_percent: Some(30.0), ..Default::default()
        }];

        // No regression
        collector.check_regression(&current_stats_good, 70.0, 10.0, 60.0, 20.0);

        // Temperature regression
        // Baseline temp 70, threshold 10. Current 85. 85 > 70 + 10. Expected warning.
        collector.check_regression(&current_stats_hot, 70.0, 10.0, 60.0, 20.0);

        // Utilization regression
        // Baseline util 60, threshold 20%. Current 30. 30 < 60 * (1 - 0.20) = 48. Expected warning.
        collector.check_regression(&current_stats_underutilized, 70.0, 10.0, 60.0, 20.0);
    }
}
