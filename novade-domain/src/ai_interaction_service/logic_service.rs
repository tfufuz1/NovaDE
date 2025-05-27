use crate::ai_interaction_service::{
    MCPConnectionService, ServerId,
    types::{
        AIInteractionContext, AIModelProfile, AIConsent, AttachmentData, JsonRpcResponse,
        AIInteractionError, ClientCapabilities, MCPServerConfig, ServerInfo, ConnectionStatus,
        AIDataCategory, AIConsentStatus, // Added AIDataCategory, AIConsentStatus
    },
    consent_manager::MCPConsentManager, // Added MCPConsentManager
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use uuid::Uuid;

#[async_trait]
pub trait AIInteractionLogicService: Send + Sync {
    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
    async fn initiate_interaction(&mut self, model_id: String, initial_prompt: Option<String>) -> Result<String, AIInteractionError>; // Returns interaction_id
    async fn get_interaction_context(&self, interaction_id: &str) -> Result<AIInteractionContext, AIInteractionError>;
    async fn update_interaction_history(&mut self, interaction_id: &str, new_entry: String) -> Result<(), AIInteractionError>;
    async fn send_prompt(&mut self, interaction_id: &str, prompt: String, attachments: Option<Vec<AttachmentData>>) -> Result<JsonRpcResponse, AIInteractionError>;
    async fn request_consent_for_interaction(&mut self, interaction_id: &str, categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError>;


    // Methods related to consent, now potentially proxied or using MCPConsentManager directly
    async fn provide_user_consent(&mut self, consent: AIConsent) -> Result<(), AIInteractionError>;
    async fn get_user_consent_status_for_model(&self, model_id: &str, user_id: &str, categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError>;
    // store_consent is an internal detail of DefaultAIInteractionLogicService if it still holds consents,
    // or it's fully delegated to MCPConsentManager. Let's remove it from the trait if MCPConsentManager handles storage.

    async fn load_model_profiles(&mut self) -> Result<(), AIInteractionError>; // Load/refresh model profiles
    // Add other methods as per "MCP-Pflichtenheft.md (Abschnitt III.A.1)"
}

pub struct DefaultAIInteractionLogicService {
    connection_service: Arc<Mutex<MCPConnectionService>>,
    interaction_contexts: HashMap<String, AIInteractionContext>,
    model_profiles: Vec<AIModelProfile>, 
    consent_manager: Arc<MCPConsentManager>, // Added consent_manager
    // The 'consents' HashMap is removed as MCPConsentManager will handle consent storage/logic.
}

impl DefaultAIInteractionLogicService {
    pub fn new(
        connection_service: Arc<Mutex<MCPConnectionService>>,
        consent_manager: Arc<MCPConsentManager>, // Added consent_manager
    ) -> Self {
        DefaultAIInteractionLogicService {
            connection_service,
            interaction_contexts: HashMap::new(),
            model_profiles: Vec::new(),
            consent_manager, // Store consent_manager
        }
    }

    // Helper to generate a unique model_id based on server details
    fn generate_model_id(server_id: &str, server_info: &ServerInfo) -> String {
        format!("{}_{}_{}", server_id, server_info.name, server_info.version).replace(" ", "_")
    }
}

#[async_trait]
impl AIInteractionLogicService for DefaultAIInteractionLogicService {
    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> {
        let mut profiles = Vec::new();
        let conn_service = self.connection_service.lock().await;
        let client_instances = conn_service.get_all_client_instances();

        for client_arc in client_instances {
            let client_instance = client_arc.lock().await;
            
            if *client_instance.get_connection_status() == ConnectionStatus::Connected {
                if let (Some(server_info), Some(config)) = (client_instance.get_server_info(), Some(client_instance.config.clone())) { // Assuming MCPClientInstance has a public 'config' field or a getter
                    let model_id = Self::generate_model_id(&config.host, server_info);
                    let profile = AIModelProfile {
                        model_id,
                        server_id: config.host.clone(), // ServerId is host in MCPConnectionService
                        server_info: server_info.clone(),
                        mcp_server_config: config.clone(),
                        name: format!("{} - {}", server_info.name, config.host),
                        description: Some(format!("Model hosted on server {} version {}", server_info.name, server_info.version)),
                    };
                    profiles.push(profile);
                }
            }
        }
        // self.model_profiles = profiles.clone(); // Update internal cache if desired
        Ok(profiles)
    }

    async fn initiate_interaction(&mut self, model_id: String, _initial_prompt: Option<String>) -> Result<String, AIInteractionError> {
        // Check if model_id is valid (e.g., exists in self.model_profiles or can be mapped to a server)
        // For now, we assume model_id is valid if it corresponds to a known server or profile
        let current_profiles = self.list_available_models().await?;
        if !current_profiles.iter().any(|p| p.model_id == model_id) {
             return Err(AIInteractionError::ModelNotFound(model_id));
        }

        let interaction_id = Uuid::new_v4().to_string();
        let context = AIInteractionContext {
            interaction_id: interaction_id.clone(),
            model_id,
            consent_given: false, // Default to false, require explicit consent
            history: Vec::new(),
            // TODO: Populate with initial_prompt if provided
        };
        self.interaction_contexts.insert(interaction_id.clone(), context);
        println!("[DefaultAIInteractionLogicService] Initiated interaction ID: {}", interaction_id);
        Ok(interaction_id)
    }

    async fn get_interaction_context(&self, interaction_id: &str) -> Result<AIInteractionContext, AIInteractionError> {
        self.interaction_contexts.get(interaction_id)
            .cloned()
            .ok_or_else(|| AIInteractionError::InteractionNotFound(interaction_id.to_string()))
    }
    
    async fn update_interaction_history(&mut self, interaction_id: &str, new_entry: String) -> Result<(), AIInteractionError> {
        match self.interaction_contexts.get_mut(interaction_id) {
            Some(context) => {
                context.history.push(new_entry);
                Ok(())
            }
            None => Err(AIInteractionError::InteractionNotFound(interaction_id.to_string())),
        }
    }

    async fn send_prompt(&mut self, interaction_id: &str, prompt: String, _attachments: Option<Vec<AttachmentData>>) -> Result<JsonRpcResponse, AIInteractionError> {
        let context = self.get_interaction_context(interaction_id).await?;
        
        // For simplicity, assume a default user_id and categories for now.
        // In a real app, user_id would come from session, categories from model requirements/request.
        let user_id = "default_user"; // Placeholder
        let required_categories = [AIDataCategory::Personal]; // Placeholder

        let consent_status = self.consent_manager.get_consent_status(
            &context.model_id,
            user_id, // Assuming user_id is part of the context or globally available
            &required_categories, 
        ).await?;

        if consent_status != AIConsentStatus::Granted {
            // If consent is PendingUserAction, we could ask for it.
            if consent_status == AIConsentStatus::PendingUserAction {
                 // The following line is commented out because request_consent_for_interaction is not fully implemented
                 // and its interaction with send_prompt needs more design (e.g. how does the user respond?)
                 // self.request_consent_for_interaction(interaction_id, &required_categories).await?;
                 return Err(AIInteractionError::ConsentRequired(format!(
                    "Consent is pending for model {}. Please grant consent.",
                    context.model_id
                )));
            }
            return Err(AIInteractionError::ConsentNotGranted(format!(
                "Consent not granted (status: {:?}) for model {} with categories {:?}. Cannot send prompt.",
                consent_status, context.model_id, required_categories
            )));
        }
        
        // Update AIInteractionContext's consent_given flag based on the check
        // This is a bit simplistic as the check above is the gate.
        // The context.consent_given might be better managed when consent is explicitly recorded.
        // self.interaction_contexts.get_mut(interaction_id).unwrap().consent_given = true;


        let conn_service = self.connection_service.lock().await;
        
        // Find the AIModelProfile to get the server_id
        let profiles = self.list_available_models().await?; // This could be cached
        let model_profile = profiles.iter().find(|p| p.model_id == context.model_id)
            .ok_or_else(|| AIInteractionError::ModelNotFound(context.model_id.clone()))?;

        let client_instance_arc = conn_service.get_client_instance(&model_profile.server_id)
            .ok_or_else(|| AIInteractionError::ConnectionError(format!("Client instance not found for server_id: {}", model_profile.server_id)))?;
        
        let mut client_instance = client_instance_arc.lock().await;

        // This is a placeholder. The actual method name and params will depend on MCP specification.
        // Assuming a "generate" or "chat" method on the MCP server.
        let response = client_instance.send_request_internal(
            "mcp.text.generate".to_string(), // Example MCP method
            serde_json::json!({
                "prompt": prompt,
                "history": context.history, // Send history if needed by the model
                // "attachments": attachments, // Handle attachments if supported
            })
        ).await.map_err(|e| AIInteractionError::ConnectionError(e.to_string()))?;
        
        // Update history with prompt and response (simplified)
        self.update_interaction_history(interaction_id, format!("User: {}", prompt)).await?;
        if let Some(result) = &response.result {
             if let Some(text_response) = result.get("text").and_then(|v| v.as_str()) {
                self.update_interaction_history(interaction_id, format!("AI: {}", text_response)).await?;
             } else if let Some(obj_response) = result.as_object() {
                // If it's an object, try to serialize it or take a meaningful part
                if let Ok(json_str) = serde_json::to_string(obj_response) {
                    self.update_interaction_history(interaction_id, format!("AI: {}", json_str)).await?;
                }
             }
        }
        Ok(response)
    }

    async fn request_consent_for_interaction(&mut self, interaction_id: &str, categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError> {
        let context = self.get_interaction_context(interaction_id).await?;
        // In a real application, user_id would be part of the context or session
        let user_id = "default_user"; // Placeholder
        
        let status = self.consent_manager.request_consent(
            &context.model_id,
            categories,
            Some(interaction_id), // Pass interaction_id as context for the consent request
        ).await?;

        // If consent is granted immediately (e.g. by a pre-existing global consent), update context.
        // However, request_consent typically returns PendingUserAction.
        if status == AIConsentStatus::Granted {
            if let Some(ctx) = self.interaction_contexts.get_mut(interaction_id) {
                ctx.consent_given = true;
            }
        }
        Ok(status)
    }

    async fn provide_user_consent(&mut self, consent: AIConsent) -> Result<(), AIInteractionError> {
        self.consent_manager.record_user_consent(consent.clone()).await?;
        // Potentially update relevant interaction contexts if consent was granted
        // This logic is simplified; a real app might need to check categories, etc.
        if true { // Assuming consent given in AIConsent means it's granted for its scope
            for context in self.interaction_contexts.values_mut() {
                if context.model_id == consent.model_id { // && context.user_id == consent.user_id (if user_id is in context)
                    // This is a simplification. A proper check would involve categories and expiry.
                    // The `get_consent_status` is the source of truth.
                    // `context.consent_given` is more like a cache or a hint.
                    // We might set it to true here, but `send_prompt` will re-verify.
                    println!("[DefaultAIInteractionLogicService] Consent recorded. Interaction context for model {} may be affected.", consent.model_id);
                }
            }
        }
        Ok(())
    }

    async fn get_user_consent_status_for_model(&self, model_id: &str, user_id: &str, categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError> {
        self.consent_manager.get_consent_status(model_id, user_id, categories).await
    }

    async fn load_model_profiles(&mut self) -> Result<(), AIInteractionError> {
        println!("[DefaultAIInteractionLogicService] STUB: load_model_profiles called. Re-evaluating available models.");
        // This method could explicitly trigger a refresh of the model_profiles list.
        // The current list_available_models fetches dynamically, so this might just update a cache.
        let profiles = self.list_available_models().await?;
        self.model_profiles = profiles; // Store them internally
        if self.model_profiles.is_empty() {
            println!("[DefaultAIInteractionLogicService] No models found/connected after load_model_profiles.");
        } else {
            println!("[DefaultAIInteractionLogicService] Loaded {} model profiles.", self.model_profiles.len());
        }
        Ok(())
    }
}

// Ensure MCPClientInstance has a way to get its config, e.g., make it public or add a getter.
// Note: The previous comment block about making `MCPClientInstance.config` public is now obsolete,
// as that change was made in a prior subtask.
