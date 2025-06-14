//! # Basic Diagnostics Runner
//!
//! This module provides a basic implementation of the `DiagnosticRunner` trait.
//! It currently includes a simple ping test to check internet connectivity and
//! placeholders for other potential diagnostics like disk SMART status.

use async_trait::async_trait;
use novade_core::types::system_health::{
    DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticTestStatus,
};
use crate::error::SystemError;
use crate::system_health_collectors::DiagnosticRunner;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime}; // Added SystemTime

/// A basic diagnostic runner that can perform simple tests like network ping.
///
/// Implements the `DiagnosticRunner` trait.
/// - `list_available_tests`: Returns a predefined list of tests, including a ping test
///   and a placeholder for disk SMART status.
/// - `run_test`: Executes the specified test. For the ping test, it uses the system's
///   `ping` command. For SMART tests, it currently returns a "Not Implemented" result.
pub struct BasicDiagnosticsRunner {
    /// The IP address or hostname to use for the ping diagnostic test.
    ping_target: String,
}

impl BasicDiagnosticsRunner {
    /// Creates a new `BasicDiagnosticsRunner`.
    ///
    /// # Arguments
    /// * `ping_target`: An optional string specifying the target for ping tests.
    ///   If `None`, defaults to "8.8.8.8" (Google's public DNS).
    pub fn new(ping_target: Option<String>) -> Self {
        Self {
            ping_target: ping_target.unwrap_or_else(|| "8.8.8.8".to_string()),
        }
    }
}

/// Identifier for the ping test.
const PING_TEST_ID: &str = "ping_default_dns";
/// Prefix for disk SMART test identifiers.
const SMART_TEST_ID_PREFIX: &str = "smart_disk_";

#[async_trait::async_trait]
impl DiagnosticRunner for BasicDiagnosticsRunner {
    /// Returns a list of available diagnostic tests.
    ///
    /// Currently includes a "Ping Default DNS" test and a placeholder for "Disk SMART Health".
    /// In a more advanced implementation, disk SMART tests might be dynamically generated
    /// based on available block devices.
    fn list_available_tests(&self) -> Result<Vec<DiagnosticTestInfo>, SystemError> {
        let tests = vec![
            DiagnosticTestInfo {
                id: DiagnosticTestId(PING_TEST_ID.to_string()),
                name: "Ping Default DNS".to_string(),
                description: format!(
                    "Pings the configured DNS server ({}) to check internet connectivity.",
                    self.ping_target
                ),
            },
            // Placeholder for SMART test. A real implementation might discover disks.
            DiagnosticTestInfo {
                id: DiagnosticTestId(format!("{}{}", SMART_TEST_ID_PREFIX, "sda")), // Example disk
                name: "Disk SMART Health (sda)".to_string(),
                description: "Checks the SMART health status of disk /dev/sda using `smartctl`. (Requires smartmontools to be installed and accessible).".to_string(),
            },
        ];
        Ok(tests)
    }

    /// Runs a specific diagnostic test by its ID.
    ///
    /// - For `PING_TEST_ID`: Executes the system `ping` command (e.g., `ping -c 4 <target>`).
    ///   Parses the output to determine success/failure and gather simple statistics.
    /// - For tests starting with `SMART_TEST_ID_PREFIX`: Returns a `NotRun` status,
    ///   as full SMART integration is not yet implemented.
    ///
    /// Returns a `DiagnosticTestResult` or a `SystemError` if the test ID is unknown
    /// or the ping command fails to execute.
    async fn run_test(&self, test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, SystemError> {
        let start_time = Instant::now();

        if test_id.0 == PING_TEST_ID {
            // Execute the ping command.
            // Assumes 'ping' utility is available in the system PATH.
            let output_result = Command::new("ping")
                .arg("-c")
                .arg("4") // Send 4 packets
                .arg(&self.ping_target)
                .output();

            let duration = start_time.elapsed();

            match output_result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let mut details = format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr);

                    if output.status.success() {
                        // Basic parsing for packet loss and rtt. This is very simple and might need refinement.
                        // TODO: Unit test parsing of mock ping command output.
                        let mut packet_loss_percent = None;
                        let mut avg_rtt_ms = None;

                        for line in stdout.lines() {
                            if line.contains("packet loss") {
                                if let Some(loss_str) = line.split(",").find(|s| s.contains("packet loss")) {
                                    if let Some(percent_str) = loss_str.split_whitespace().next() {
                                       packet_loss_percent = percent_str.trim_end_matches('%').parse::<f32>().ok();
                                    }
                                }
                            }
                            if line.starts_with("rtt min/avg/max/mdev") || line.starts_with("round-trip min/avg/max/stddev") { // Linux vs macOS
                                let parts: Vec<&str> = line.split('=').nth(1).unwrap_or("").split('/').collect();
                                if parts.len() > 1 {
                                    avg_rtt_ms = parts[1].trim().parse::<f32>().ok();
                                }
                            }
                        }
                        details.push_str(&format!("\nParsed - Packet Loss: {:?}, Avg RTT: {:?}", packet_loss_percent, avg_rtt_ms));

                        Ok(DiagnosticTestResult {
                            test_id: test_id.clone(),
                            status: DiagnosticTestStatus::Passed,
                            message: "Ping successful.".to_string(),
                            details: Some(details),
                            duration: Some(duration),
                            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
                        })
                    } else {
                        Ok(DiagnosticTestResult {
                            test_id: test_id.clone(),
                            status: DiagnosticTestStatus::Failed,
                            message: format!("Ping command failed with exit code: {:?}", output.status.code()),
                            details: Some(details),
                            duration: Some(duration),
                             timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
                        })
                    }
                }
                Err(e) => {
                    Ok(DiagnosticTestResult {
                        test_id: test_id.clone(),
                        status: DiagnosticTestStatus::Error,
                        message: format!("Failed to execute ping command: {}", e),
                        details: None,
                        duration: Some(duration),
                        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
                    })
                }
            }
        } else if test_id.0.starts_with(SMART_TEST_ID_PREFIX) {
            // Placeholder for SMART test.
            // A full implementation would involve:
            // 1. Identifying the correct device path (e.g., /dev/sda).
            // 2. Executing `smartctl -H /dev/sdX`.
            // 3. Parsing `smartctl` output to determine health status.
            // This requires `smartmontools` to be installed on the system.
            let device_name = test_id.0.strip_prefix(SMART_TEST_ID_PREFIX).unwrap_or("unknown_disk");
            Ok(DiagnosticTestResult {
                test_id: test_id.clone(),
                status: DiagnosticTestStatus::NotRun,
                message: format!("SMART test for {} not implemented yet.", device_name),
                details: Some("This test requires `smartmontools` to be installed and would involve parsing its output. This feature is planned.".to_string()),
                duration: Some(start_time.elapsed()),
                timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
            })
        } else {
            Err(SystemError::MetricCollectorError(format!("Unknown diagnostic test ID: {}", test_id.0)))
        }
    }
}
