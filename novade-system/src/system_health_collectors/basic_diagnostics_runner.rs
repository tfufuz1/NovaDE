//! # Basic Diagnostics Runner
//!
//! This module provides a basic implementation of the `DiagnosticRunner` trait.
//! It currently includes a simple ping test to check internet connectivity and
//! placeholders for other potential diagnostics like disk SMART status.

use async_trait::async_trait;
use novade_core::types::system_health::{
    DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticTestStatus,
};
use novade_core::types::system_health::{
    DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticStatus,
};
use crate::error::SystemError;
use crate::system_health_collectors::DiagnosticRunner;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use std::path::Path; // Added for path checks
use glob::glob; // Added for device discovery


/// A basic diagnostic runner that can perform simple tests like network ping and SMART checks.
///
/// Implements the `DiagnosticRunner` trait.
pub struct BasicDiagnosticsRunner {
    /// The IP address or hostname to use for the ping diagnostic test.
    /// This is derived from the configuration.
    diagnostic_ping_target: Option<String>,
}

impl BasicDiagnosticsRunner {
    /// Creates a new `BasicDiagnosticsRunner`.
    ///
    /// # Arguments
    /// * `diagnostic_ping_target`: An optional string specifying the target for ping tests.
    ///   This would typically come from `SystemHealthDashboardConfig`.
    pub fn new(diagnostic_ping_target: Option<String>) -> Self {
        Self {
            diagnostic_ping_target,
        }
    }
}

/// Identifier for the ping test.
const PING_TEST_ID: &str = "ping_connectivity_check";
/// Prefix for disk SMART test identifiers.
const SMART_TEST_ID_PREFIX: &str = "smart_check_";

// Helper function to parse smartctl output. Made pub(crate) for testing.
pub(crate) fn parse_smartctl_output(
    stdout: &str,
    stderr: &str,
    exit_code: Option<i32>,
    device_name: &str, // Added for more informative messages
) -> (DiagnosticStatus, String) {
    match exit_code {
        Some(0) => { // Command executed successfully, check output content
            if stdout.contains("SMART overall-health self-assessment test result: PASSED") ||
               stdout.contains("SMART Health Status: OK") {
                (DiagnosticStatus::Passed, format!("SMART health check for {} passed.", device_name))
            } else if stdout.contains("FAILED!") || stdout.contains("UNKNOWN!") {
                (DiagnosticStatus::Failed, format!("SMART check for {} reported success exit code, but output indicates potential issues: FAILED! or UNKNOWN! found.", device_name))
            } else {
                (DiagnosticStatus::Warning, format!("SMART check for {} completed with exit code 0, but health status unclear from output.", device_name))
            }
        }
        Some(code) => { // Command executed but returned an error code
            let mut message = format!("SMART health check for {} failed or smartctl error. Exit code: {}", device_name, code);
            // Check specific bits in exit code if needed, or rely on stderr
            // Bit 1 (mask 2): Device open failed, device did not return SMART status.
            // Bit 2 (mask 4): Some SMART command to the disk failed.
            // Bit 3 (mask 8): SMART status check returned "DISK FAILING".
            let final_status = if (code & 0b10) != 0 || // Bit 1: Device open error
                               stderr.contains("Unable to detect device type") ||
                               stderr.contains("Cannot open device") ||
                               stderr.contains("No such device") ||
                               stderr.contains("Smartctl open device: failed") {
                message = format!("Error executing smartctl for /dev/{}: Could not open or detect device. Ensure device path is correct and smartctl has permissions.", device_name);
                DiagnosticStatus::Error
            } else if (code & 0b1000) != 0 { // Bit 3: Disk Failing
                 message = format!("SMART status for {} indicates DISK FAILING. Exit code: {}", device_name, code);
                 DiagnosticStatus::Failed
            } else { // Other non-zero exit codes
                DiagnosticStatus::Failed // Assume other errors also mean a failure in health.
            };
            (final_status, message)
        }
        None => { // Command failed to execute (e.g. not found) - this case is usually handled before calling this parser
            (DiagnosticStatus::Error, format!("smartctl command failed to execute for {}. Ensure smartmontools is installed and in PATH.", device_name))
        }
    }
}


#[async_trait::async_trait]
impl DiagnosticRunner for BasicDiagnosticsRunner {
    /// Returns a list of available diagnostic tests.
    /// Dynamically discovers block devices for SMART tests and includes a configurable ping test.
    fn list_available_tests(&self) -> Result<Vec<DiagnosticTestInfo>, SystemError> {
        let mut tests = Vec::new();

        // 1. Ping Test
        let ping_target_display = self
            .diagnostic_ping_target
            .as_deref()
            .unwrap_or("default (1.1.1.1)"); // Default for description if None

        tests.push(DiagnosticTestInfo {
            id: DiagnosticTestId(PING_TEST_ID.to_string()),
            name: "Ping Connectivity Test".to_string(),
            description: format!(
                "Pings the target ({}) to check internet connectivity.",
                ping_target_display
            ),
        });

        // 2. SMART Disk Health Tests
        // Discover common block device patterns.
        let patterns = ["/dev/sd[a-z]", "/dev/nvme[0-9]n[0-9]", "/dev/vd[a-z]"];
        for pattern in patterns.iter() {
            match glob(pattern) {
                Ok(paths) => {
                    for entry in paths {
                        match entry {
                            Ok(path_buf) => {
                                // Check if the path actually exists.
                                if Path::new(&path_buf).exists() {
                                    if let Some(device_name) = path_buf.file_name().and_then(|name| name.to_str()) {
                                        tests.push(DiagnosticTestInfo {
                                            id: DiagnosticTestId(format!("{}{}", SMART_TEST_ID_PREFIX, device_name)),
                                            name: format!("SMART Health Check ({})", device_name),
                                            description: format!(
                                                "Checks the SMART health status of /dev/{}. Requires smartmontools.",
                                                device_name
                                            ),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                // Using tracing::warn or similar would be appropriate in a larger app.
                                eprintln!("Warning: Error processing path from glob pattern {}: {}", pattern, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Error with glob pattern {}: {}", pattern, e);
                }
            }
        }
        Ok(tests)
    }

    /// Runs a specific diagnostic test by its ID.
    ///
    /// - For `PING_TEST_ID`: Executes the system `ping` command.
    /// - For tests starting with `SMART_TEST_ID_PREFIX`: Executes `smartctl -H /dev/{device}`.
    ///
    /// Returns a `DiagnosticTestResult` or a `SystemError` if the test ID is unknown
    /// or a command fails to execute.
    async fn run_test(&self, test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, SystemError> {
        let start_time_utc = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let start_instant = Instant::now(); // For measuring duration

        if test_id.0 == PING_TEST_ID {
            let ping_target_address = self.diagnostic_ping_target.clone().unwrap_or_else(|| {
                // TODO: This default ("1.1.1.1") should ideally be defined in a central configuration spot,
                // or the test should perhaps be disabled/warn if no target is explicitly set via config.
                "1.1.1.1".to_string()
            });

            let output_result = Command::new("ping")
                .arg("-c")
                .arg("4") // Send 4 packets
                .arg(&ping_target_address)
                .output();

            let duration = start_instant.elapsed();

            match output_result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let mut details = format!("Target: {}\nStdout:\n{}\nStderr:\n{}", ping_target_address, stdout, stderr);

                    if output.status.success() {
                        // Basic parsing for packet loss and rtt.
                        let mut packet_loss_percent = None;
                        let mut avg_rtt_ms = None;

                        for line in stdout.lines() {
                            if line.contains("packet loss") {
                                // More robustly extract percentage, e.g., "50% packet loss" or "50.0% packet loss"
                                if let Some(loss_part) = line.split(',').find(|s| s.contains("packet loss")) {
                                    if let Some(percent_str) = loss_part.trim().split_whitespace().next() {
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
                        details.push_str(&format!("\nParsed - Packet Loss: {:?}%, Avg RTT: {:?}ms", packet_loss_percent.map(|p| p.to_string()), avg_rtt_ms.map(|r| r.to_string())));

                        Ok(DiagnosticTestResult {
                            test_id: test_id.clone(),
                            status: DiagnosticStatus::Passed,
                            message: "Ping successful.".to_string(),
                            details: Some(details),
                            duration: Some(duration),
                            timestamp: start_time_utc,
                        })
                    } else {
                        Ok(DiagnosticTestResult {
                            test_id: test_id.clone(),
                            status: DiagnosticStatus::Failed,
                            message: format!("Ping command failed with exit code: {:?}", output.status.code()),
                            details: Some(details),
                            duration: Some(duration),
                            timestamp: start_time_utc,
                        })
                    }
                }
                Err(e) => {
                    Ok(DiagnosticTestResult {
                        test_id: test_id.clone(),
                        status: DiagnosticStatus::Error,
                        message: format!("Failed to execute ping command: {}", e),
                        details: Some(format!("Error: {}. Ensure 'ping' utility is installed and in PATH.", e)),
                        duration: Some(start_instant.elapsed()),
                        timestamp: start_time_utc,
                    })
                }
            }
        } else if test_id.0.starts_with(SMART_TEST_ID_PREFIX) {
            let device_name = match test_id.0.strip_prefix(SMART_TEST_ID_PREFIX) {
                Some(name) => name,
                None => {
                    // This case should not be reached if IDs are generated correctly by list_available_tests
                    return Ok(DiagnosticTestResult {
                        test_id: test_id.clone(),
                        status: DiagnosticStatus::Error,
                        message: "Invalid SMART test ID format.".to_string(),
                        details: Some(format!("Malformed ID: {}", test_id.0)),
                        duration: Some(start_instant.elapsed()),
                        timestamp: start_time_utc,
                    });
                }
            };
            let device_path = format!("/dev/{}", device_name);

            let output_result = Command::new("smartctl")
                .arg("-H") // Check health status
                .arg(&device_path)
                .output();

            let duration = start_instant.elapsed();

            match output_result {
                Ok(output) => {
                    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
                    let details = format!("Device: {}\nCommand: smartctl -H {}\nStdout:\n{}\nStderr:\n{}", device_path, device_path, stdout_str, stderr_str);

                    let (status, message) = parse_smartctl_output(&stdout_str, &stderr_str, output.status.code(), device_name);

                    Ok(DiagnosticTestResult {
                        test_id: test_id.clone(),
                        status,
                        message,
                        details: Some(details),
                        duration: Some(duration),
                        timestamp: start_time_utc,
                    })
                }
                Err(e) => { // This means `smartctl` command itself could not be executed (e.g., not found)
                    Ok(DiagnosticTestResult {
                        test_id: test_id.clone(),
                        status: DiagnosticStatus::Error,
                        message: format!("Failed to execute smartctl command for {}: {}. Ensure smartmontools is installed and in PATH.", device_path, e),
                        details: Some(e.to_string()),
                        duration: Some(start_instant.elapsed()),
                        timestamp: start_time_utc,
                    })
                }
            }
        } else {
            Err(SystemError::MetricCollectorError(format!("Unknown diagnostic test ID: {}", test_id.0)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::system_health::DiagnosticStatus;

    #[test]
    fn test_parse_smartctl_passed() {
        let stdout = "smartctl 7.2 2020-12-30 r5155 [x86_64-linux-5.10.0-8-amd64]\nCopyright (C) 2002-20, Bruce Allen, Christian Franke, www.smartmontools.org\n\n=== START OF READ SMART DATA SECTION ===\nSMART overall-health self-assessment test result: PASSED\n";
        let (status, msg) = parse_smartctl_output(stdout, "", Some(0), "sda");
        assert_eq!(status, DiagnosticStatus::Passed);
        assert!(msg.contains("passed"));
    }

    #[test]
    fn test_parse_smartctl_health_status_ok() {
        let stdout = "SMART Health Status: OK";
        let (status, msg) = parse_smartctl_output(stdout, "", Some(0), "sdb");
        assert_eq!(status, DiagnosticStatus::Passed);
        assert!(msg.contains("passed"));
    }

    #[test]
    fn test_parse_smartctl_failed_output() {
        let stdout = "SMART overall-health self-assessment test result: FAILED!\nWarning: This disk is failing!";
        let (status, msg) = parse_smartctl_output(stdout, "", Some(0), "sdc"); // smartctl might exit 0 even if test fails
        assert_eq!(status, DiagnosticStatus::Failed);
        assert!(msg.contains("FAILED!"));
    }

    #[test]
    fn test_parse_smartctl_unknown_output_success_code() {
        let stdout = "Some other output from smartctl, but no clear PASSED or FAILED.";
        let (status, msg) = parse_smartctl_output(stdout, "", Some(0), "sdd");
        assert_eq!(status, DiagnosticStatus::Warning);
        assert!(msg.contains("unclear from output"));
    }

    #[test]
    fn test_parse_smartctl_error_device_open_failure_exit_code() {
        // Bit 1 (value 2) indicates device open failed.
        let (status, msg) = parse_smartctl_output("", "Could not open device /dev/sde", Some(2), "sde");
        assert_eq!(status, DiagnosticStatus::Error);
        assert!(msg.contains("Could not open or detect device"));
    }

    #[test]
    fn test_parse_smartctl_error_disk_failing_exit_code() {
        // Bit 3 (value 8) indicates SMART status is "DISK FAILING".
        let (status, msg) = parse_smartctl_output("Disk is FAILING", "", Some(8), "sdf");
        assert_eq!(status, DiagnosticStatus::Failed);
        assert!(msg.contains("DISK FAILING"));
    }

    #[test]
    fn test_parse_smartctl_other_error_exit_code() {
        // Bit 2 (value 4) some other SMART command failed
        let (status, msg) = parse_smartctl_output("", "Some SMART command failed", Some(4), "sdg");
        assert_eq!(status, DiagnosticStatus::Failed); // General failure if not specifically device open error
        assert!(msg.contains("failed or smartctl error"));
    }

    #[test]
    fn test_ping_target_description_in_list() {
        let runner_custom_target = BasicDiagnosticsRunner::new(Some("my.server.com".to_string()));
        let tests_custom = runner_custom_target.list_available_tests().unwrap();
        let ping_test_custom = tests_custom.iter().find(|t| t.id.0 == PING_TEST_ID).unwrap();
        assert!(ping_test_custom.description.contains("my.server.com"));

        let runner_default_target = BasicDiagnosticsRunner::new(None);
        let tests_default = runner_default_target.list_available_tests().unwrap();
        let ping_test_default = tests_default.iter().find(|t| t.id.0 == PING_TEST_ID).unwrap();
        assert!(ping_test_default.description.contains("default (1.1.1.1)"));
    }

    #[test]
    fn test_smart_test_id_and_name_generation_in_list() {
        // This test is limited as it relies on the actual filesystem via `glob`.
        // It might not find any devices in some CI environments.
        // We are mostly checking the naming convention if a device is found.
        let runner = BasicDiagnosticsRunner::new(None);
        let tests = runner.list_available_tests().unwrap();

        let smart_tests: Vec<_> = tests.into_iter().filter(|t| t.id.0.starts_with(SMART_TEST_ID_PREFIX)).collect();

        // If smart_tests is not empty, check one.
        if let Some(first_smart_test) = smart_tests.first() {
            let expected_device_name = first_smart_test.id.0.strip_prefix(SMART_TEST_ID_PREFIX).unwrap();
            assert!(first_smart_test.name.contains(expected_device_name));
            assert!(first_smart_test.description.contains(&format!("/dev/{}", expected_device_name)));
        } else {
            // It's okay if no SMART devices are found in a test environment,
            // as glob might not find any /dev/sd* or /dev/nvme*.
            // We can't reliably assert that it *will* find devices here.
            println!("Warning: No SMART-eligible devices found by glob in test environment for test_smart_test_id_and_name_generation_in_list. This might be normal.");
        }
    }
}
