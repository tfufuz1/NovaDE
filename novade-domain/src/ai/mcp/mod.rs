pub mod types;
pub mod client_instance;
pub mod transport;
pub mod connection_service;
pub mod consent_manager;

#[cfg(test)]
pub(crate) mod client_instance_tests;

pub use types::{
    MCPServerConfig, ClientCapabilities, ServerCapabilities, ServerInfo, ConnectionStatus,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, ToolDefinition, ResourceDefinition,
    PromptDefinition, MCPError, AIInteractionContext, AIModelProfile, AIConsent,
    AIDataCategory, AttachmentData, AIInteractionError, AIConsentStatus,
};
pub use client_instance::{MCPClientInstance};
pub use transport::{IMCPTransport, StdioTransportHandler};
pub use connection_service::{MCPConnectionService, ServerId as ConnectionServiceServerId};
pub use consent_manager::{MCPConsentManager};
