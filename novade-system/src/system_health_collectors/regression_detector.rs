//ANCHOR [NovaDE Developers <dev@novade.org>] Performance Regression Detector.
//! This module provides a basic framework for detecting performance regressions.
//!
//! It defines a generic `RegressionDetector` and specific implementations or helper functions
//! for different types of metrics. The core idea is to compare current metrics against
//! a stored baseline and flag significant deviations.
//!
//! //TODO [NovaDE Developers <dev@novade.org>] More sophisticated baseline management (e.g., rolling baselines, historical data).
//! //TODO [NovaDE Developers <dev@novade.org>] Implement statistical significance testing for regressions.
//! //TODO [NovaDE Developers <dev@novade.org>] Allow configuration of thresholds and sensitivity.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::SystemTime;

//ANCHOR [NovaDE Developers <dev@novade.org>] Generic baseline storage.
/// Stores baseline values for different metrics identified by a key.
#[derive(Debug, Clone)]
pub struct BaselineStore<K: Eq + Hash + Clone + Debug, V: Clone + Debug> {
    baselines: HashMap<K, BaselineEntry<V>>,
    default_threshold_percentage: f64, // General threshold if not specified per metric
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Entry in the baseline store.
/// Represents a single baseline entry, including the value and when it was recorded.
#[derive(Debug, Clone)]
pub struct BaselineEntry<V: Clone + Debug> {
    pub value: V,
    pub recorded_at: SystemTime,
    pub threshold_percentage: Option<f64>, // Specific threshold for this metric
}

impl<K: Eq + Hash + Clone + Debug, V: Clone + Debug> BaselineStore<K, V> {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Creates a new BaselineStore.
    /// Creates a new `BaselineStore` with a default threshold percentage.
    ///
    /// # Arguments
    /// * `default_threshold_percentage`: The default percentage deviation to consider a regression
    ///   if no specific threshold is set for a metric. For values that increase (e.g., time, memory),
    ///   this is `(current - baseline) / baseline`. For values that decrease (e.g., FPS), careful
    ///   interpretation is needed or a separate threshold type.
    pub fn new(default_threshold_percentage: f64) -> Self {
        BaselineStore {
            baselines: HashMap::new(),
            default_threshold_percentage,
        }
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Updates or sets a baseline value.
    /// Updates or sets a baseline for a given metric key.
    ///
    /// # Arguments
    /// * `key`: The identifier for the metric.
    /// * `value`: The baseline value.
    /// * `threshold_percentage`: Optional specific threshold for this metric.
    pub fn set_baseline(&mut self, key: K, value: V, threshold_percentage: Option<f64>) {
        self.baselines.insert(
            key,
            BaselineEntry {
                value,
                recorded_at: SystemTime::now(),
                threshold_percentage,
            },
        );
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Retrieves a baseline entry.
    /// Retrieves a baseline entry for a given key.
    pub fn get_baseline(&self, key: &K) -> Option<&BaselineEntry<V>> {
        self.baselines.get(key)
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Clears a specific baseline.
    /// Clears the baseline for a specific metric key.
    pub fn clear_baseline(&mut self, key: &K) {
        self.baselines.remove(key);
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Clears all baselines.
    /// Clears all stored baselines.
    pub fn clear_all_baselines(&mut self) {
        self.baselines.clear();
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Generic regression check for numeric values that are "worse" when higher.
/// Checks for regression for a numeric metric where a higher value is worse (e.g., latency, memory usage).
///
/// # Arguments
/// * `metric_name`: A descriptive name for the metric being checked (for logging).
/// * `current_value`: The current value of the metric.
/// * `baseline_entry`: Optional `BaselineEntry<f64>` containing the baseline value and threshold.
/// * `default_threshold_percentage`: Fallback threshold if `baseline_entry` or its specific threshold is None.
///
/// # Returns
/// `true` if a regression is detected, `false` otherwise.
pub fn check_regression_higher_is_worse(
    metric_name: &str,
    current_value: f64,
    baseline_entry: Option<&BaselineEntry<f64>>,
    default_threshold_percentage: f64,
) -> bool {
    if let Some(baseline) = baseline_entry {
        let threshold = baseline.threshold_percentage.unwrap_or(default_threshold_percentage);
        if current_value > baseline.value * (1.0 + threshold / 100.0) {
            //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
            eprintln!(
                "[WARNING] RegressionDetector ({}): Performance regression detected! Current value {:.2} exceeds baseline {:.2} by more than {:.0}%.",
                metric_name, current_value, baseline.value, threshold
            );
            return true;
        }
    }
    false
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Generic regression check for numeric values that are "worse" when lower.
/// Checks for regression for a numeric metric where a lower value is worse (e.g., frame rate, throughput).
///
/// # Arguments
/// * `metric_name`: A descriptive name for the metric being checked (for logging).
/// * `current_value`: The current value of the metric.
/// * `baseline_entry`: Optional `BaselineEntry<f64>` containing the baseline value and threshold.
/// * `default_threshold_percentage`: Fallback threshold if `baseline_entry` or its specific threshold is None.
///
/// # Returns
/// `true` if a regression is detected, `false` otherwise.
pub fn check_regression_lower_is_worse(
    metric_name: &str,
    current_value: f64,
    baseline_entry: Option<&BaselineEntry<f64>>,
    default_threshold_percentage: f64,
) -> bool {
    if let Some(baseline) = baseline_entry {
        let threshold = baseline.threshold_percentage.unwrap_or(default_threshold_percentage);
        if current_value < baseline.value * (1.0 - threshold / 100.0) {
            //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
            eprintln!(
                "[WARNING] RegressionDetector ({}): Performance regression detected! Current value {:.2} is below baseline {:.2} by more than {:.0}%.",
                metric_name, current_value, baseline.value, threshold
            );
            return true;
        }
    }
    false
}

// Example of how a specific collector might use this:
/*
use crate::system_health_collectors::frame_time_collector::{FrameTimeCollector, FrameTimeStatistics};

impl FrameTimeCollector {
    pub fn check_regression_with_store(
        &self,
        baseline_store: &BaselineStore<String, f64>, // Key could be "avg_frame_time"
        metric_key: &String,
    ) {
        if let Some(stats) = self.get_statistics() {
            check_regression_higher_is_worse(
                metric_key, // e.g. "Average Frame Time"
                stats.avg_frame_time_ms,
                baseline_store.get_baseline(metric_key),
                baseline_store.default_threshold_percentage
            );
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_store_new_set_get() {
        let mut store = BaselineStore::<String, f64>::new(20.0);
        assert_eq!(store.default_threshold_percentage, 20.0);

        store.set_baseline("avg_latency".to_string(), 100.0, Some(15.0));
        let entry = store.get_baseline(&"avg_latency".to_string()).unwrap();
        assert_eq!(entry.value, 100.0);
        assert_eq!(entry.threshold_percentage, Some(15.0));

        store.set_baseline("max_latency".to_string(), 200.0, None);
        let entry_max = store.get_baseline(&"max_latency".to_string()).unwrap();
        assert_eq!(entry_max.value, 200.0);
        assert_eq!(entry_max.threshold_percentage, None);
    }

    #[test]
    fn test_baseline_store_clear() {
        let mut store = BaselineStore::<String, f64>::new(10.0);
        store.set_baseline("metric1".to_string(), 1.0, None);
        store.set_baseline("metric2".to_string(), 2.0, None);

        store.clear_baseline(&"metric1".to_string());
        assert!(store.get_baseline(&"metric1".to_string()).is_none());
        assert!(store.get_baseline(&"metric2".to_string()).is_some());

        store.clear_all_baselines();
        assert!(store.get_baseline(&"metric2".to_string()).is_none());
        assert!(store.baselines.is_empty());
    }

    #[test]
    fn test_check_regression_higher_is_worse_no_baseline() {
        let detected = check_regression_higher_is_worse("test_metric", 120.0, None, 10.0);
        assert!(!detected, "Should not detect regression if no baseline is provided.");
    }

    #[test]
    fn test_check_regression_higher_is_worse_no_regression() {
        let baseline = BaselineEntry { value: 100.0, recorded_at: SystemTime::now(), threshold_percentage: Some(20.0) };
        // Current 110.0, baseline 100.0, threshold 20%. 100.0 * 1.20 = 120.0. 110.0 is not > 120.0
        let detected = check_regression_higher_is_worse("test_metric", 110.0, Some(&baseline), 25.0);
        assert!(!detected);
    }

    #[test]
    fn test_check_regression_higher_is_worse_regression_detected_specific_threshold() {
        let baseline = BaselineEntry { value: 100.0, recorded_at: SystemTime::now(), threshold_percentage: Some(10.0) };
        // Current 115.0, baseline 100.0, threshold 10%. 100.0 * 1.10 = 110.0. 115.0 is > 110.0
        let detected = check_regression_higher_is_worse("test_metric", 115.0, Some(&baseline), 25.0);
        assert!(detected, "Expected regression to be detected due to specific threshold");
    }

    #[test]
    fn test_check_regression_higher_is_worse_regression_detected_default_threshold() {
        let baseline = BaselineEntry { value: 100.0, recorded_at: SystemTime::now(), threshold_percentage: None };
        // Current 130.0, baseline 100.0, default_threshold 25%. 100.0 * 1.25 = 125.0. 130.0 is > 125.0
        let detected = check_regression_higher_is_worse("test_metric", 130.0, Some(&baseline), 25.0);
        assert!(detected, "Expected regression to be detected due to default threshold");
    }

    #[test]
    fn test_check_regression_lower_is_worse_no_baseline() {
        let detected = check_regression_lower_is_worse("test_fps", 50.0, None, 10.0);
        assert!(!detected, "Should not detect regression if no baseline is provided.");
    }

    #[test]
    fn test_check_regression_lower_is_worse_no_regression() {
        let baseline = BaselineEntry { value: 60.0, recorded_at: SystemTime::now(), threshold_percentage: Some(10.0) };
        // Current 58.0, baseline 60.0, threshold 10%. 60.0 * (1.0 - 0.10) = 54.0. 58.0 is not < 54.0
        let detected = check_regression_lower_is_worse("test_fps", 58.0, Some(&baseline), 15.0);
        assert!(!detected);
    }

    #[test]
    fn test_check_regression_lower_is_worse_regression_detected_specific_threshold() {
        let baseline = BaselineEntry { value: 60.0, recorded_at: SystemTime::now(), threshold_percentage: Some(10.0) };
        // Current 50.0, baseline 60.0, threshold 10%. 60.0 * 0.90 = 54.0. 50.0 is < 54.0
        let detected = check_regression_lower_is_worse("test_fps", 50.0, Some(&baseline), 15.0);
        assert!(detected, "Expected regression to be detected due to specific threshold");
    }

    #[test]
    fn test_check_regression_lower_is_worse_regression_detected_default_threshold() {
        let baseline = BaselineEntry { value: 60.0, recorded_at: SystemTime::now(), threshold_percentage: None };
        // Current 40.0, baseline 60.0, default_threshold 15%. 60.0 * (1.0 - 0.15) = 51.0. 40.0 is < 51.0
        let detected = check_regression_lower_is_worse("test_fps", 40.0, Some(&baseline), 15.0);
        assert!(detected, "Expected regression to be detected due to default threshold");
    }
}
