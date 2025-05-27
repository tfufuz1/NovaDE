use crate::error::SystemError;
use crate::mcp_client_service::types::StdioProcess;
use async_trait::async_trait;

#[async_trait]
pub trait IMCPClientService: Send + Sync {
    async fn spawn_stdio_server(&self, command: String, args: Vec<String>) -> Result<StdioProcess, SystemError>;
    async fn terminate_stdio_server(&self, pid: u32) -> Result<(), SystemError>;
}
