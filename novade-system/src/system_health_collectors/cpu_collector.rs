//! # CPU Metrics Collector
//!
//! This module is responsible for collecting CPU usage statistics from the Linux `/proc/stat` file.
//! It calculates overall CPU usage and per-core usage percentages.

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncReadExt}; // BufReader is not strictly necessary for this read_to_string pattern
use tokio::time::{sleep, Duration};
use novade_core::types::system_health::CpuMetrics;
use crate::error::SystemError;
use crate::system_health_collectors::CpuMetricsCollector;

/// A collector for CPU metrics on Linux systems.
///
/// This struct implements the `CpuMetricsCollector` trait by parsing data from
/// the `/proc/stat` file to determine CPU usage. It requires two readings of
/// `/proc/stat` with a short delay to calculate usage percentages.
pub struct LinuxCpuMetricsCollector;

/// Represents CPU time components read from a single line in `/proc/stat`.
/// All values are in USER_HZ (typically 1/100th of a second).
#[derive(Debug, Default, Clone)]
struct CpuTimes {
    /// Time spent in user mode.
    user: u64,
    /// Time spent in user mode with low priority (nice).
    nice: u64,
    /// Time spent in system mode.
    system: u64,
    /// Time spent in the idle task. This value should be USER_HZ times the second entry in the /proc/uptime pseudo-file.
    idle: u64,
    /// Time waiting for I/O to complete. Not accounted in user or system.
    iowait: u64,
    /// Time servicing interrupts.
    irq: u64,
    /// Time servicing softirqs.
    softirq: u64,
    /// Stolen time, which is the time spent in other operating systems when running in a virtualized environment.
    steal: u64,
    /// Time spent running a virtual CPU for guest operating systems under the control of the Linux kernel.
    guest: u64,
    /// Time spent running a niced guest (virtual CPU for guest operating systems under the control of the Linux kernel).
    guest_nice: u64,
}

impl CpuTimes {
    /// Calculates the total CPU time by summing all relevant components.
    /// Excludes `guest` and `guest_nice` as they are already included in `user` and `nice`.
    fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq + self.steal
    }

    /// Calculates the total idle time.
    fn idle_time(&self) -> u64 {
        self.idle + self.iowait // iowait is also a form of idle time for the CPU
    }
}

/// Asynchronously reads the content of `/proc/stat`.
/// Returns a `Result` with a vector of strings (lines from the file) or a `SystemError`.
async fn read_proc_stat() -> Result<Vec<String>, SystemError> {
    let mut file = File::open("/proc/stat").await.map_err(|e| SystemError::MetricCollectorError(format!("Failed to open /proc/stat: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.map_err(|e| SystemError::MetricCollectorError(format!("Failed to read /proc/stat: {}", e)))?;
    Ok(contents.lines().map(String::from).collect())
}

/// Parses a single CPU line (e.g., "cpu", "cpu0", "cpu1") from `/proc/stat` into `CpuTimes`.
/// The line is expected to be space-separated, with the first field being the CPU label
/// followed by numeric time values.
fn parse_cpu_line(line: &str) -> Result<CpuTimes, SystemError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    // Expect at least "cpu" + user, nice, system, idle, iowait, irq, softirq, steal (8 values).
    // Guest and guest_nice are optional and appear after steal.
    if parts.len() < 9 {
        return Err(SystemError::MetricCollectorError(format!("Invalid CPU line format in /proc/stat: {}", line)));
    }

    Ok(CpuTimes {
        user: parts[1].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse user time: {}", e)))?,
        nice: parts[2].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse nice time: {}", e)))?,
        system: parts[3].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse system time: {}", e)))?,
        idle: parts[4].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse idle time: {}", e)))?,
        iowait: parts[5].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse iowait time: {}", e)))?,
        irq: parts[6].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse irq time: {}", e)))?,
        softirq: parts[7].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse softirq time: {}", e)))?,
        steal: parts[8].parse().map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse steal time: {}", e)))?,
        guest: parts.get(9).and_then(|s| s.parse().ok()).unwrap_or(0),
        guest_nice: parts.get(10).and_then(|s| s.parse().ok()).unwrap_or(0),
    })
    // TODO: Unit test parsing of /proc/stat mock data.
}

#[async_trait::async_trait]
impl CpuMetricsCollector for LinuxCpuMetricsCollector {
    /// Asynchronously collects current CPU metrics.
    ///
    /// This method reads `/proc/stat` twice with a short delay, then calculates the difference
    /// in CPU time components (user, nice, system, idle, iowait, irq, softirq, steal)
    /// to determine the percentage of CPU time spent in non-idle states.
    /// It provides both total CPU usage and per-core usage percentages.
    ///
    /// The formula for CPU usage percentage is approximately: `(1 - (idle_diff / total_diff)) * 100`.
    ///
    /// Returns a `Result` containing `CpuMetrics` (total and per-core usage) on success,
    /// or a `SystemError` if reading or parsing `/proc/stat` fails or if the format is unexpected.
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, SystemError> {
        // TODO: Unit test CPU usage calculation logic.
        let lines1 = read_proc_stat().await?;
        // A short delay is crucial for calculating the difference in CPU ticks.
        // 500ms is a common compromise between responsiveness and accuracy.
        sleep(Duration::from_millis(500)).await;
        let lines2 = read_proc_stat().await?;

        let mut total_usage_percent = 0.0;
        let mut per_core_usage_percent = Vec::new();

        let overall_cpu_line1 = lines1.iter().find(|line| line.starts_with("cpu "));
        let overall_cpu_line2 = lines2.iter().find(|line| line.starts_with("cpu "));

        if let (Some(line1), Some(line2)) = (overall_cpu_line1, overall_cpu_line2) {
            let times1 = parse_cpu_line(line1)?;
            let times2 = parse_cpu_line(line2)?;

            let total_diff = times2.total() - times1.total();
            let idle_diff = times2.idle_time() - times1.idle_time();

            if total_diff > 0 {
                total_usage_percent = (1.0 - (idle_diff as f64 / total_diff as f64)) * 100.0;
            }
        } else {
            return Err(SystemError::MetricCollectorError("Could not find overall CPU usage line in /proc/stat".into()));
        }

        let core_lines1: Vec<_> = lines1.iter().filter(|line| line.starts_with("cpu") && !line.starts_with("cpu ")).collect();
        let core_lines2: Vec<_> = lines2.iter().filter(|line| line.starts_with("cpu") && !line.starts_with("cpu ")).collect();

        for i in 0..core_lines1.len() {
            if i < core_lines2.len() {
                 let core_line1_str = core_lines1[i];
                 let core_line2_str = core_lines2[i];

                // Ensure both lines refer to the same CPU core by checking the prefix e.g. "cpu0"
                let prefix1 = core_line1_str.split_whitespace().next().unwrap_or_default();
                let prefix2 = core_line2_str.split_whitespace().next().unwrap_or_default();

                if prefix1 == prefix2 && !prefix1.is_empty() {
                    let times1 = parse_cpu_line(core_line1_str)?;
                    let times2 = parse_cpu_line(core_line2_str)?;

                    let total_diff = times2.total() - times1.total();
                    let idle_diff = times2.idle_time() - times1.idle_time();

                    if total_diff > 0 {
                        per_core_usage_percent.push((1.0 - (idle_diff as f64 / total_diff as f64)) * 100.0);
                    } else {
                        per_core_usage_percent.push(0.0);
                    }
                } else {
                    // Mismatch in core lines or empty prefix, skip or log error
                    // This might happen if cores go offline/online during sampling, though rare for short intervals
                     eprintln!("Skipping core due to mismatch or empty prefix: {} vs {}", prefix1, prefix2);
                }
            }
        }


        Ok(CpuMetrics {
            total_usage_percent: total_usage_percent as f32,
            per_core_usage_percent: per_core_usage_percent.into_iter().map(|val| val as f32).collect(),
            // temperature_celsius and core_temperatures_celsius are not available from /proc/stat.
            // They would typically be sourced from /sys/class/thermal or other sensor interfaces.
        })
    }
}
