//! # Network Metrics Collector
//!
//! This module collects network interface activity statistics from `/proc/net/dev` on Linux.
//! It calculates data transfer rates (bytes/sec, packets/sec) and error rates.

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncReadExt};
use tokio::time::{sleep, Duration};
use novade_core::types::system_health::NetworkActivityMetrics;
use crate::error::SystemError;
use crate::system_health_collectors::NetworkMetricsCollector;
use std::collections::HashMap;

/// A collector for network activity metrics on Linux systems.
///
/// Implements the `NetworkMetricsCollector` trait by parsing `/proc/net/dev`.
/// It reads interface statistics at two intervals to calculate rates for
/// received/sent bytes, packets, and errors per second. Loopback interfaces (`lo`)
/// are typically skipped.
pub struct LinuxNetworkMetricsCollector;

/// Holds raw statistics for a single network interface, parsed from a line in `/proc/net/dev`.
#[derive(Clone, Debug)]
struct NetworkStats {
    /// Name of the network interface (e.g., "eth0", "wlan0").
    interface_name: String,
    /// Total number of bytes received by the interface.
    received_bytes: u64,
    /// Total number of packets received.
    received_packets: u64,
    /// Total number of receive errors.
    received_errors: u64,
    /// Total number of received packets dropped.
    received_drop: u64,
    /// Total number of bytes transmitted by the interface.
    sent_bytes: u64,
    /// Total number of packets transmitted.
    sent_packets: u64,
    /// Total number of transmit errors.
    sent_errors: u64,
    /// Total number of transmitted packets dropped.
    sent_drop: u64,
}

/// Asynchronously reads and parses `/proc/net/dev`.
///
/// Skips the first two header lines and then parses each subsequent line for interface statistics.
/// Each line corresponds to an interface; columns represent various metrics like bytes, packets,
/// errors, and drops for both received and transmitted traffic.
///
/// Returns a `Result` containing a `Vec<NetworkStats>` or a `SystemError`.
async fn read_proc_net_dev() -> Result<Vec<NetworkStats>, SystemError> {
    let mut file = File::open("/proc/net/dev").await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to open /proc/net/dev: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to read /proc/net/dev: {}", e)))?;

    let mut stats_list = Vec::new();
    // Skip the first two header lines
    for line in contents.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 17 { // Expected number of fields for receive and transmit stats
            continue;
        }

        let interface_name = parts[0].trim_end_matches(':').to_string();
        // Skip loopback interface
        if interface_name == "lo" {
            continue;
        }

        // Receive stats: bytes, packets, errs, drop, fifo, frame, compressed, multicast
        // Transmit stats: bytes, packets, errs, drop, fifo, colls, carrier, compressed
        stats_list.push(NetworkStats {
            interface_name,
            received_bytes: parts[1].parse().unwrap_or(0),
            received_packets: parts[2].parse().unwrap_or(0),
            received_errors: parts[3].parse().unwrap_or(0),
            received_drop: parts[4].parse().unwrap_or(0),
            // parts[5] (fifo), parts[6] (frame), parts[7] (compressed), parts[8] (multicast) are skipped
            sent_bytes: parts[9].parse().unwrap_or(0),
            sent_packets: parts[10].parse().unwrap_or(0),
            sent_errors: parts[11].parse().unwrap_or(0),
            sent_drop: parts[12].parse().unwrap_or(0),
            // parts[13] (fifo), parts[14] (colls), parts[15] (carrier), parts[16] (compressed) are skipped
        });
    }
    Ok(stats_list)
    // TODO: Unit test parsing of /proc/net/dev mock data and rate calculation.
}

#[async_trait::async_trait]
impl NetworkMetricsCollector for LinuxNetworkMetricsCollector {
    /// Asynchronously collects network activity metrics for all relevant interfaces.
    ///
    /// This method reads `/proc/net/dev` twice with a short delay (e.g., 500ms).
    /// It then calculates the difference in byte counts, packet counts, and error counts
    /// to determine rates per second for each interface (excluding loopback).
    ///
    /// Returns a `Result` containing a `Vec<NetworkActivityMetrics>` (one entry per interface)
    /// on success, or a `SystemError` if reading or parsing `/proc/net/dev` fails.
    async fn collect_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, SystemError> {
        let stats1_map: HashMap<String, NetworkStats> = read_proc_net_dev().await?
            .into_iter()
            .map(|s| (s.interface_name.clone(), s))
            .collect();

        // Delay to allow counters to change for rate calculation.
        sleep(Duration::from_millis(500)).await;

        let stats2_list = read_proc_net_dev().await?;
        let mut activity_metrics = Vec::new();

        for stats2 in stats2_list {
            if let Some(stats1) = stats1_map.get(&stats2.interface_name) {
                let duration_s = 0.5; // Must match the sleep duration.

                let received_bytes_per_second = (stats2.received_bytes - stats1.received_bytes) as f64 / duration_s;
                let sent_bytes_per_second = (stats2.sent_bytes - stats1.sent_bytes) as f64 / duration_s;
                let received_packets_per_second = (stats2.received_packets - stats1.received_packets) as f64 / duration_s;
                let sent_packets_per_second = (stats2.sent_packets - stats1.sent_packets) as f64 / duration_s;

                // Calculate error rates if needed, or just total errors in the interval
                let new_receive_errors = stats2.received_errors - stats1.received_errors;
                let new_transmit_errors = stats2.sent_errors - stats1.sent_errors;


                activity_metrics.push(NetworkActivityMetrics {
                    interface_name: stats2.interface_name.clone(),
                    received_bytes_per_second: received_bytes_per_second as u64,
                    sent_bytes_per_second: sent_bytes_per_second as u64,
                    received_packets_per_second: received_packets_per_second as f32,
                    sent_packets_per_second: sent_packets_per_second as f32,
                    receive_errors_per_second: (new_receive_errors as f64 / duration_s) as f32,
                    transmit_errors_per_second: (new_transmit_errors as f64 / duration_s) as f32,
                    // Note: Interface speed (e.g., Mbps) and duplex status are not available
                    // directly from /proc/net/dev. They would require other system calls or utilities
                    // (like ethtool or reading from /sys/class/net/<interface>/speed).
                });
            }
        }
        Ok(activity_metrics)
    }
}
