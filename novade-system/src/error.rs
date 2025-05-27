use thiserror::Error;
// Re-exporting or defining a new McpError specific to system layer,
// or it could wrap the one from novade_domain if that's preferred.
// For now, let's assume we might want to define system-level MCP errors.
use novade_domain::ai_interaction_service::MCPError as DomainMCPError;


#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Process operation failed: {0}")]
    ProcessError(String),
    #[error("Process with PID {0} not found")]
    ProcessNotFound(u32),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to spawn process '{command}': {error}")]
    SpawnError { command: String, error: String },
    #[error("Failed to terminate process with PID {pid}: {error}")]
    TerminationError { pid: u32, error: String },
    #[error("Attempted to use a process (PID: {0}) that is not running or already handled.")]
    ProcessNotAvailable(u32),
    #[error("Child process STDIN not available for PID {0}")]
    StdInNotAvailable(u32),
    #[error("Child process STDOUT not available for PID {0}")]
    StdOutNotAvailable(u32),
    #[error("MCP Domain Error: {0:?}")]
    DomainMcpError(#[from] DomainMCPError),
    // This can be expanded if the system layer needs its own distinct MCP errors
    #[error("System-level MCP error: {0}")]
    SystemMcpError(String), 
}
