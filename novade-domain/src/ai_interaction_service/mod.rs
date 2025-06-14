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
pub mod nlp_processor; // Added nlp_processor module
pub mod context_manager; // Added context_manager module
pub mod skills_executor; // Added skills_executor module
pub use logic_service::{AIInteractionLogicService, DefaultAIInteractionLogicService, ProcessOutput, ExecutionResult}; // Added ProcessOutput, ExecutionResult
pub use consent_manager::MCPConsentManager; // Export MCPConsentManager
pub use nlp_processor::{NlpProcessor, BasicNlpProcessor}; // Added nlp_processor exports
pub use context_manager::{ContextManager, DefaultContextManager, PartialContextUpdate}; // Added context_manager exports
pub use skills_executor::{SkillsExecutor, DefaultSkillsExecutor}; // Added skills_executor exports

#[cfg(test)]
mod client_instance_tests;

#[cfg(test)]
mod logic_service_tests;

// TODO: Consider if this ai_interaction_service module should be under user_centric_services
// as hinted in Spezifikations-Wissensbasis.md. The current path is `novade-domain/src/ai_interaction_service`,
// but `novade-domain/src/user_centric_services/ai_interaction/` also exists and might be the
// more appropriate location for some of these components long-term.
// TODO: The DomainError enum in `crate::error` is already suitable. Ensure all new
// services and components consistently use `Result<T, DomainError>`.
