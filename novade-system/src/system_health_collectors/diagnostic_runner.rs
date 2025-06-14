// novade-system/src/system_health_collectors/diagnostic_runner.rs

use async_trait::async_trait;
use novade_core::types::system_health::{DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticStatus};
use novade_domain::system_health_service::service::DiagnosticRunnerAdapter;
use novade_domain::system_health_service::error::SystemHealthError;
use super::error::CollectionError;
use std::process::Command; // For running external commands, carefully.
use chrono::Utc;

pub struct SystemDiagnosticRunner {
    // Could store a list of predefined diagnostic commands or scripts.
    available_tests: Vec<DiagnosticTestInfo>,
}

impl SystemDiagnosticRunner {
    pub fn new() -> Self {
        // Define some built-in basic tests
        let available_tests = vec![
            DiagnosticTestInfo {
                id: DiagnosticTestId("ping_google_dns".to_string()), // Changed ID slightly for clarity
                name: "Ping Google DNS".to_string(),
                description: "Pings Google's public DNS server (8.8.8.8) to check internet connectivity.".to_string(),
            },
            DiagnosticTestInfo {
                id: DiagnosticTestId("disk_space_check_root".to_string()),
                name: "Root Filesystem Space Check".to_string(),
                description: "Checks available disk space on the root filesystem using 'df -h /'.".to_string(),
            },
        ];
        SystemDiagnosticRunner { available_tests }
    }

    #[allow(dead_code)] // This will be used when actual collection logic is added
    fn to_domain_error(err: CollectionError, test_id: &DiagnosticTestId) -> SystemHealthError {
        SystemHealthError::DiagnosticTestExecutionError {
            test_id: test_id.clone(),
            source_description: err.to_string(),
            source: Some(Box::new(err)),
        }
    }

    // Helper to run a command and create a DiagnosticTestResult
    fn run_command_test(
        test_id: DiagnosticTestId,
        command_name: &str,
        args: &[&str],
    ) -> Result<DiagnosticTestResult, SystemHealthError> {
        let start_time = Utc::now();
        let output_result = Command::new(command_name)
            .args(args)
            .output();
        let end_time = Utc::now();

        match output_result {
            Ok(out) => {
                let status = if out.status.success() { DiagnosticStatus::Passed } else { DiagnosticStatus::Failed };
                let summary = if status == DiagnosticStatus::Passed {
                    format!("Command '{}' executed successfully.", command_name)
                } else {
                    format!("Command '{}' failed with exit code {:?}.", command_name, out.status.code())
                };
                Ok(DiagnosticTestResult {
                    id: test_id,
                    status,
                    summary,
                    details: Some(format!("Start Time: {}\nEnd Time: {}\nExit Code: {:?}\n\nStdout:\n{}\nStderr:\n{}",
                                          start_time.to_rfc3339(), end_time.to_rfc3339(), out.status.code(),
                                          String::from_utf8_lossy(&out.stdout),
                                          String::from_utf8_lossy(&out.stderr))),
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                })
            }
            Err(e) => Err(SystemHealthError::DiagnosticTestExecutionError {
                test_id,
                source_description: format!("Failed to execute command '{}': {}", command_name, e.to_string()),
                source: Some(Box::new(CollectionError::CommandExecutionError {
                    command: format!("{} {}", command_name, args.join(" ")),
                    exit_code: None,
                    stderr: e.to_string(),
                }))
            }),
        }
    }
}

#[async_trait]
impl DiagnosticRunnerAdapter for SystemDiagnosticRunner {
    async fn list_available_diagnostics(&self) -> Result<Vec<DiagnosticTestInfo>, SystemHealthError> {
        println!("SystemLayer: `list_available_diagnostics` called");
        Ok(self.available_tests.clone())
    }

    async fn run_diagnostic(&self, test_id: DiagnosticTestId, params: Option<serde_json::Value>) -> Result<DiagnosticTestResult, SystemHealthError> {
        println!("SystemLayer: `run_diagnostic` called for test_id: {:?}, params: {:?} (placeholder)", test_id, params);

        match test_id.0.as_str() {
            "ping_google_dns" => {
                // Example of using a parameter (though not used in this specific ping)
                if let Some(p) = params {
                    println!("Received params for ping_google_dns: {:?}", p);
                    // if p.get("host").and_then(|v| v.as_str()) == Some("another_host") { ... }
                }
                Self::run_command_test(test_id, "ping", &["-c", "4", "8.8.8.8"])
            }
            "disk_space_check_root" => {
                Self::run_command_test(test_id, "df", &["-h", "/"])
            }
            _ => Err(SystemHealthError::DiagnosticTestNotFound(test_id)),
        }
    }
}
