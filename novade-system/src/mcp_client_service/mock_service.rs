use crate::error::SystemError;
use crate::mcp_client_service::types::StdioProcess;
use crate::mcp_client_service::traits::IMCPClientService;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio as StdStdio; // Using std::process::Stdio for Command
use std::sync::Arc;
use tokio::process::{Child, Command}; // Tokio's Command for async execution
use tokio::sync::Mutex;

// This Mock will now attempt to spawn a real process.
pub struct MockRealProcessMCPClientService {
    processes: Arc<Mutex<HashMap<u32, Child>>>,
    server_binary_path: PathBuf, // Path to the mcp-echo-server binary
}

impl MockRealProcessMCPClientService {
    /// Creates a new mock service.
    /// `server_binary_path` should be the path to the compiled `mcp-echo-server` binary.
    /// For example, relative from the workspace root: `target/debug/mcp-echo-server`.
    /// Or an absolute path.
    pub fn new(server_binary_path: PathBuf) -> Self {
        if !server_binary_path.exists() {
            // Log a warning or panic if the binary path is critical for all tests using this mock.
            // For CI/CD, this path needs to be reliable.
            eprintln!("[MockRealProcessMCPClientService] Warning: Server binary path does not exist or is not accessible: {:?}", server_binary_path);
        }
        MockRealProcessMCPClientService {
            processes: Arc::new(Mutex::new(HashMap::new())),
            server_binary_path,
        }
    }

    // Helper to get a potentially relative path to the echo server binary
    // This assumes tests might be run from different locations or in a workspace.
    // A more robust solution might involve environment variables or build script outputs.
    pub fn new_with_relative_path(relative_path_from_workspace_root: &str) -> Self {
        // Try to determine workspace root if possible, or assume CWD is workspace root for tests.
        // This is a common challenge for finding test binaries.
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // Gets /app/novade-system
        path.pop(); // Go up to /app (workspace root)
        path.push(relative_path_from_workspace_root); // e.g., /app/target/debug/mcp-echo-server
        
        // A simple fallback if CARGO_MANIFEST_DIR is not as expected in test environment
        // This might need to be adjusted based on how/where tests are executed.
        // For now, let's assume the provided path is directly usable or already adjusted.
        // PathBuf::from(relative_path_from_workspace_root)
        
        Self::new(path)
    }
}

impl Default for MockRealProcessMCPClientService {
    fn default() -> Self {
        // Default path, assuming mcp-echo-server is built and target is relative to workspace root
        // This is often "target/debug/mcp-echo-server" when run via `cargo test` from workspace root.
        // Adjust if your workspace structure or build process is different.
        // For the agent's environment, this needs to be correct relative to /app
        let path_from_workspace_root = "target/debug/mcp-echo-server";
        let mut path = PathBuf::from("/app"); // Agent's workspace root
        path.push(path_from_workspace_root);
        
        Self::new(path)
    }
}


#[async_trait]
impl IMCPClientService for MockRealProcessMCPClientService {
    async fn spawn_stdio_server(&self, _command: String, _args: Vec<String>) -> Result<StdioProcess, SystemError> {
        // The `command` and `args` are ignored; we always use self.server_binary_path.
        // This is because this mock is specifically for the mcp-echo-server.
        // If a generic mock is needed, it should use the provided command.
        
        if !self.server_binary_path.exists() {
            return Err(SystemError::SpawnError {
                command: self.server_binary_path.to_string_lossy().into_owned(),
                error: "Server binary not found at specified path".to_string(),
            });
        }

        let mut child_process = Command::new(&self.server_binary_path)
            // .args(args) // If we were to use args for the echo server
            .stdin(StdStdio::piped())  // Use std::process::Stdio here
            .stdout(StdStdio::piped())
            .stderr(StdStdio::piped()) // Capture stderr for debugging test failures
            .spawn()
            .map_err(|e| SystemError::SpawnError {
                command: self.server_binary_path.to_string_lossy().into_owned(),
                error: e.to_string(),
            })?;

        let pid = child_process.id().ok_or_else(|| SystemError::SpawnError {
            command: self.server_binary_path.to_string_lossy().into_owned(),
            error: "Failed to get PID from spawned process".to_string(),
        })?;

        let stdin = child_process.stdin.take().ok_or(SystemError::StdInNotAvailable(pid))?;
        let stdout = child_process.stdout.take().ok_or(SystemError::StdOutNotAvailable(pid))?;
        
        // Handle stderr: It's good practice to read/log stderr to avoid the child process blocking
        // if its stderr buffer fills up, and for debugging.
        if let Some(child_stderr) = child_process.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut stderr_reader = tokio::io::BufReader::new(child_stderr);
                let mut line = String::new();
                loop {
                    match stderr_reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            eprintln!("[mcp-echo-server PID {} stderr] {}", pid, line.trim_end());
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("[mcp-echo-server PID {} stderr] Error reading: {}", pid, e);
                            break;
                        }
                    }
                }
            });
        }

        let mut processes_guard = self.processes.lock().await;
        processes_guard.insert(pid, child_process); // Store the Child, not StdioProcess

        Ok(StdioProcess { stdin, stdout, pid })
    }

    async fn terminate_stdio_server(&self, pid: u32) -> Result<(), SystemError> {
        let mut processes_guard = self.processes.lock().await;
        if let Some(mut child) = processes_guard.remove(&pid) {
            println!("[MockRealProcessMCPClientService] Terminating PID: {}", pid);
            child.kill().await.map_err(|e| SystemError::TerminationError {
                pid,
                error: e.to_string(),
            })?;
            // Optionally, wait for the process to ensure it's fully cleaned up
            // let status = child.wait().await.map_err(|e| SystemError::ProcessError(format!("Error waiting for process {}: {}", pid, e)))?;
            // println!("[MockRealProcessMCPClientService] Process {} terminated with status: {}", pid, status);
            Ok(())
        } else {
            Err(SystemError::ProcessNotFound(pid))
        }
    }
}

// To make this mock usable by novade-domain for its tests,
// this file should NOT be under #[cfg(test)].
// It should be part of the novade_system library, e.g.
// pub mod mock_service; in mcp_client_service/mod.rs
// and then IMCPClientService, DefaultMCPClientService, MockRealProcessMCPClientService
// re-exported from novade_system::mcp_client_service module.
//
// For now, this file is created. The next step will be to ensure it's correctly exposed.
