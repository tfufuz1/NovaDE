// Declare the modules within the ai_interaction_service directory
pub mod types;
pub mod transport;
pub mod client_instance;
pub mod connection_service;

// Optionally, re-export key structs/traits for easier access from parent modules
pub mod consent_manager; // Added consent_manager module
pub use types::{
    MCPServerConfig, ClientCapabilities, ServerCapabilities, ServerInfo, ConnectionStatus,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, ToolDefinition, ResourceDefinition,
    PromptDefinition, MCPError, AIInteractionContext, AIModelProfile, AIConsent,
    AIDataCategory, AttachmentData, AIInteractionError, AIConsentStatus, // Added AIConsentStatus
};
pub use transport::{IMCPTransport, StdioTransportHandler};
pub use client_instance::MCPClientInstance;
pub use connection_service::{MCPConnectionService, ServerId};
pub mod logic_service;
pub use logic_service::{AIInteractionLogicService, DefaultAIInteractionLogicService};
pub use consent_manager::MCPConsentManager; // Export MCPConsentManager

#[cfg(test)]
mod client_instance_tests;

#[cfg(test)]
mod logic_service_tests;
