use crate::error::SystemError;
use crate::mcp_client_service::types::StdioProcess;
use crate::mcp_client_service::traits::IMCPClientService;
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc; // Changed from std::sync::Mutex to tokio::sync::Mutex
use tokio::process::{Child, Command};
use tokio::sync::Mutex; // Using tokio's Mutex for async code

pub struct DefaultMCPClientService {
    // Store Child processes to be able to terminate them.
    // The Child struct itself contains the PID.
    processes: Arc<Mutex<HashMap<u32, Child>>>,
}

impl DefaultMCPClientService {
    pub fn new() -> Self {
        DefaultMCPClientService {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for DefaultMCPClientService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IMCPClientService for DefaultMCPClientService {
    async fn spawn_stdio_server(&self, command: String, args: Vec<String>) -> Result<StdioProcess, SystemError> {
        let mut child_process = Command::new(&command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Capture stderr for potential debugging
            .spawn()
            .map_err(|e| SystemError::SpawnError { command: command.clone(), error: e.to_string() })?;

        let pid = child_process.id().ok_or_else(|| SystemError::SpawnError {
            command: command.clone(),
            error: "Failed to get PID from spawned process".to_string(),
        })?;

        let stdin = child_process.stdin.take().ok_or(SystemError::StdInNotAvailable(pid))?;
        let stdout = child_process.stdout.take().ok_or(SystemError::StdOutNotAvailable(pid))?;
        
        // Optional: Log stderr if needed, or handle it appropriately
        // if let Some(mut stderr) = child_process.stderr.take() {
        //     tokio::spawn(async move {
        //         let mut buffer = String::new();
        //         use tokio::io::AsyncReadExt;
        //         if let Err(e) = stderr.read_to_string(&mut buffer).await {
        //             eprintln!("[PID {}] Error reading stderr: {}", pid, e);
        //         }
        //         if !buffer.is_empty() {
        //             println!("[PID {}] Stderr: {}", pid, buffer);
        //         }
        //     });
        // }


        let mut processes_guard = self.processes.lock().await;
        processes_guard.insert(pid, child_process);

        Ok(StdioProcess { stdin, stdout, pid })
    }

    async fn terminate_stdio_server(&self, pid: u32) -> Result<(), SystemError> {
        let mut processes_guard = self.processes.lock().await;
        if let Some(mut child) = processes_guard.remove(&pid) {
            child.kill().await.map_err(|e| SystemError::TerminationError { pid, error: e.to_string() })?;
            // Optionally wait for the process to ensure it's fully cleaned up
            // child.wait().await.map_err(|e| SystemError::ProcessError(format!("Error waiting for process {} to terminate: {}", pid, e)))?;
            Ok(())
        } else {
            Err(SystemError::ProcessNotFound(pid))
        }
    }
}
