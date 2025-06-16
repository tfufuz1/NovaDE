//ANCHOR [NovaDE Developers <dev@novade.org>] Frame Time Collector for performance monitoring.
//! This module provides a collector for frame timing statistics.
//! It helps in understanding UI performance and detecting stutters.

use std::collections::VecDeque;
use std::time::Duration;

//ANCHOR [NovaDE Developers <dev@novade.org>] Defines aggregated frame time statistics.
/// Holds aggregated statistics for frame times over a certain window.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameTimeStatistics {
    pub min_frame_time_ms: f64,
    pub max_frame_time_ms: f64,
    pub avg_frame_time_ms: f64,
    pub p95_frame_time_ms: f64, // 95th percentile
    pub sample_count: usize,
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Collector for frame times.
/// Collects frame times and calculates statistics over a rolling window.
#[derive(Debug)]
pub struct FrameTimeCollector {
    frame_times: VecDeque<Duration>,
    max_samples: usize,
    //TODO [NovaDE Developers <dev@novade.org>] Consider making max_samples configurable.
}

impl FrameTimeCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Creates a new FrameTimeCollector.
    /// Creates a new `FrameTimeCollector` with a specified maximum number of samples.
    ///
    /// # Arguments
    ///
    /// * `max_samples`: The maximum number of frame times to keep in the rolling window.
    pub fn new(max_samples: usize) -> Self {
        FrameTimeCollector {
            frame_times: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Records a single frame time.
    /// Records a new frame time.
    ///
    /// If the number of samples exceeds `max_samples`, the oldest sample is dropped.
    ///
    /// # Arguments
    ///
    /// * `frame_duration`: The `Duration` of the last frame.
    pub fn record_frame_time(&mut self, frame_duration: Duration) {
        if self.frame_times.len() == self.max_samples && self.max_samples > 0 {
            self.frame_times.pop_front();
        }
        if self.max_samples > 0 {
            self.frame_times.push_back(frame_duration);
        }
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Calculates and returns frame time statistics.
    /// Calculates and returns aggregated statistics for the collected frame times.
    ///
    /// Returns `None` if no samples have been recorded.
    pub fn get_statistics(&self) -> Option<FrameTimeStatistics> {
        if self.frame_times.is_empty() {
            return None;
        }

        let mut sorted_times_ms: Vec<f64> = self
            .frame_times
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();

        sorted_times_ms.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sample_count = sorted_times_ms.len();
        let sum_ms: f64 = sorted_times_ms.iter().sum();
        let avg_ms = sum_ms / (sample_count as f64);
        let min_ms = *sorted_times_ms.first().unwrap_or(&0.0);
        let max_ms = *sorted_times_ms.last().unwrap_or(&0.0);

        // Calculate 95th percentile
        //TODO [NovaDE Developers <dev@novade.org>] Consider more sophisticated percentile calculation methods if needed.
        let p95_index = ( (sample_count as f64 * 0.95).ceil() as usize ).saturating_sub(1).min(sample_count.saturating_sub(1));
        let p95_ms = sorted_times_ms.get(p95_index).cloned().unwrap_or(max_ms);


        Some(FrameTimeStatistics {
            min_frame_time_ms: min_ms,
            max_frame_time_ms: max_ms,
            avg_frame_time_ms: avg_ms,
            p95_frame_time_ms: p95_ms,
            sample_count,
        })
    }

    //ANCHOR [NovaDE Developers <dev@novade.org>] Clears all recorded frame times.
    /// Clears all recorded frame times from the collector.
    pub fn clear(&mut self) {
        self.frame_times.clear();
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Regression Detector for Frame Times.
//TODO [NovaDE Developers <dev@novade.org>] This is a placeholder for a more generic regression detector.
// For now, baseline management is rudimentary.
impl FrameTimeCollector {
    //ANCHOR [NovaDE Developers <dev@novade.org>] Checks for performance regression based on a simple baseline.
    /// Checks for performance regression by comparing current average frame time
    /// to a baseline average. Logs a warning if current average is significantly higher.
    ///
    /// # Arguments
    ///
    /// * `baseline_avg_ms`: The baseline average frame time in milliseconds.
    /// * `threshold_percentage`: The percentage increase over baseline to be considered a regression.
    pub fn check_regression(&self, baseline_avg_ms: f64, threshold_percentage: f64) {
        if let Some(stats) = self.get_statistics() {
            if stats.avg_frame_time_ms > baseline_avg_ms * (1.0 + threshold_percentage / 100.0) {
                //TODO [NovaDE Developers <dev@novade.org>] Use the structured logger when available.
                eprintln!(
                    "[WARNING] FrameTimeCollector: Performance regression detected! Avg frame time {:.2}ms exceeds baseline {:.2}ms by more than {:.0}%.",
                    stats.avg_frame_time_ms, baseline_avg_ms, threshold_percentage
                );
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_frame_time_collector_new() {
        let collector = FrameTimeCollector::new(100);
        assert_eq!(collector.max_samples, 100);
        assert!(collector.frame_times.is_empty());
    }

    #[test]
    fn test_record_frame_time() {
        let mut collector = FrameTimeCollector::new(3);
        collector.record_frame_time(Duration::from_millis(16));
        collector.record_frame_time(Duration::from_millis(20));
        assert_eq!(collector.frame_times.len(), 2);
        collector.record_frame_time(Duration::from_millis(18));
        assert_eq!(collector.frame_times.len(), 3);
        // Next one should pop the oldest (16ms)
        collector.record_frame_time(Duration::from_millis(22));
        assert_eq!(collector.frame_times.len(), 3);
        assert_eq!(collector.frame_times.front().unwrap().as_millis(), 20);
    }

    #[test]
    fn test_record_frame_time_zero_max_samples() {
        let mut collector = FrameTimeCollector::new(0);
        collector.record_frame_time(Duration::from_millis(16));
        assert!(collector.frame_times.is_empty());
    }

    #[test]
    fn test_get_statistics_empty() {
        let collector = FrameTimeCollector::new(100);
        assert!(collector.get_statistics().is_none());
    }

    #[test]
    fn test_get_statistics_single_sample() {
        let mut collector = FrameTimeCollector::new(100);
        collector.record_frame_time(Duration::from_millis(16));
        let stats = collector.get_statistics().unwrap();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.min_frame_time_ms, 16.0);
        assert_eq!(stats.max_frame_time_ms, 16.0);
        assert_eq!(stats.avg_frame_time_ms, 16.0);
        assert_eq!(stats.p95_frame_time_ms, 16.0);
    }

    #[test]
    fn test_get_statistics_multiple_samples() {
        let mut collector = FrameTimeCollector::new(10);
        // 10, 20, 30, 40, 50 ms
        for i in 1..=5 {
            collector.record_frame_time(Duration::from_millis(i * 10));
        }
        let stats = collector.get_statistics().unwrap();
        assert_eq!(stats.sample_count, 5);
        assert_eq!(stats.min_frame_time_ms, 10.0);
        assert_eq!(stats.max_frame_time_ms, 50.0);
        assert_eq!(stats.avg_frame_time_ms, 30.0); // (10+20+30+40+50)/5 = 30
        // p95: 5 samples * 0.95 = 4.75, ceil = 5, index = 4. sorted_times_ms[4] is 50.0
        assert_eq!(stats.p95_frame_time_ms, 50.0);
    }

    #[test]
    fn test_get_statistics_percentile_calculation() {
        let mut collector = FrameTimeCollector::new(20);
        // Add 20 samples from 1 to 20 ms
        for i in 1..=20 {
            collector.record_frame_time(Duration::from_millis(i));
        }
        let stats = collector.get_statistics().unwrap();
        // p95: 20 * 0.95 = 19. ceil = 19. index = 18. sorted_times_ms[18] is 19ms
        assert_eq!(stats.p95_frame_time_ms, 19.0);

        let mut collector_small = FrameTimeCollector::new(3);
        collector_small.record_frame_time(Duration::from_millis(10));
        collector_small.record_frame_time(Duration::from_millis(20));
        collector_small.record_frame_time(Duration::from_millis(100));
        // 3 * 0.95 = 2.85, ceil = 3, index = 2. sorted_times_ms[2] is 100ms
        let stats_small = collector_small.get_statistics().unwrap();
        assert_eq!(stats_small.p95_frame_time_ms, 100.0);
    }


    #[test]
    fn test_clear_collector() {
        let mut collector = FrameTimeCollector::new(5);
        collector.record_frame_time(Duration::from_millis(10));
        collector.record_frame_time(Duration::from_millis(20));
        assert!(!collector.frame_times.is_empty());
        collector.clear();
        assert!(collector.frame_times.is_empty());
        assert!(collector.get_statistics().is_none());
    }

    #[test]
    fn test_regression_detection_no_regression() {
        let mut collector = FrameTimeCollector::new(5);
        collector.record_frame_time(Duration::from_millis(10));
        collector.record_frame_time(Duration::from_millis(12));
        // Avg is 11ms. Baseline 10ms. Threshold 20%. 10 * 1.20 = 12. 11 is not > 12.
        collector.check_regression(10.0, 20.0); // No warning expected
    }

    #[test]
    fn test_regression_detection_regression_detected() {
        let mut collector = FrameTimeCollector::new(5);
        collector.record_frame_time(Duration::from_millis(15));
        collector.record_frame_time(Duration::from_millis(25));
        // Avg is 20ms. Baseline 10ms. Threshold 20%. 10 * 1.20 = 12. 20 is > 12.
        // This test will print a warning to stderr, which is expected.
        collector.check_regression(10.0, 20.0);
    }
}
