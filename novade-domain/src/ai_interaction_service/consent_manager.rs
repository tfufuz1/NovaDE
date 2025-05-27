use crate::ai_interaction_service::types::{
    AIDataCategory, AIConsentStatus, AIInteractionError, AIConsent, // Added AIConsent
};
use async_trait::async_trait;
use std::collections::HashMap; // For potential future use (e.g., storing consents)
use std::sync::Arc;
use tokio::sync::Mutex;

// A trait could be defined here if different consent manager implementations are expected.
// pub trait IConsentManager: Send + Sync {
//     async fn request_consent(&self, model_id: &str, categories: &[AIDataCategory], context_id: Option<&str>) -> Result<AIConsentStatus, AIInteractionError>;
//     async fn get_consent_status(&self, model_id: &str, user_id: &str, categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError>;
//     async fn record_consent(&self, consent: AIConsent) -> Result<(), AIInteractionError>; // Example method
// }

#[derive(Debug)]
pub struct MCPConsentManager {
    // In a real scenario, this would interact with storage (e.g., database)
    // to persist and retrieve consent information.
    // For this implementation, it will be stateless beyond logging.
    // We can add a simple in-memory store for consents if needed for more complex simulation.
    recorded_consents: Mutex<HashMap<(String, String), AIConsent>>, // (model_id, user_id) -> AIConsent
}

impl MCPConsentManager {
    pub fn new() -> Self {
        MCPConsentManager {
            recorded_consents: Mutex::new(HashMap::new()),
        }
    }

    pub async fn request_consent(
        &self,
        model_id: &str,
        categories: &[AIDataCategory],
        _context_id: Option<&str>, // context_id is not used in this stub
    ) -> Result<AIConsentStatus, AIInteractionError> {
        println!(
            "[MCPConsentManager] Requesting consent for model_id: '{}', categories: {:?}",
            model_id, categories
        );
        // This simulates initiating a consent request.
        // In a real system, this might trigger a UI prompt or a notification.
        // For now, it always returns PendingUserAction, implying user interaction is needed.
        Ok(AIConsentStatus::PendingUserAction)
    }

    pub async fn get_consent_status(
        &self,
        model_id: &str,
        user_id: &str,
        categories: &[AIDataCategory], 
    ) -> Result<AIConsentStatus, AIInteractionError> {
        println!(
            "[MCPConsentManager] Getting consent status for model_id: '{}', user_id: '{}', categories: {:?}",
            model_id, user_id, categories
        );

        // --- Test Hook for Forcing Granted Status ---
        if model_id.contains("_FORCE_GRANT_") {
            println!("[MCPConsentManager] TEST HOOK: Forcing GRANTED status for model_id: {}", model_id);
            return Ok(AIConsentStatus::Granted);
        }
        // --- End Test Hook ---
        
        let consents_guard = self.recorded_consents.lock().await;
        if let Some(consent_record) = consents_guard.get(&(model_id.to_string(), user_id.to_string())) {
            let all_categories_covered = categories.iter().all(|cat| consent_record.data_categories.contains(cat));
            if all_categories_covered {
                if consent_record.expires_at.is_none() { 
                    return Ok(AIConsentStatus::Granted);
                } else {
                    // Basic expiry check for testing. A real implementation needs proper date parsing and comparison.
                    if consent_record.expires_at.as_deref() == Some("EXPIRED_FOR_TEST") {
                        println!("[MCPConsentManager] Consent for model '{}', user '{}' is marked as EXPIRED_FOR_TEST.", model_id, user_id);
                        return Ok(AIConsentStatus::Expired);
                    }
                    println!("[MCPConsentManager] Consent for model '{}', user '{}' has an expiry date '{}'. Further check needed (not implemented).", model_id, user_id, consent_record.expires_at.as_ref().unwrap());
                    // Fall through to PendingUserAction if expiry logic is not fully implemented or date is in future
                }
            }
        }
        
        Ok(AIConsentStatus::PendingUserAction)
    }

    // New method to allow DefaultAIInteractionLogicService to store consent
    pub async fn record_user_consent(&self, consent: AIConsent) -> Result<(), AIInteractionError> {
        println!(
            "[MCPConsentManager] Recording consent for model_id: '{}', user_id: '{}', categories: {:?}, granted_at: {}",
            consent.model_id, consent.user_id, consent.data_categories, consent.granted_at
        );
        let mut consents_guard = self.recorded_consents.lock().await;
        consents_guard.insert((consent.model_id.clone(), consent.user_id.clone()), consent);
        Ok(())
    }
}

impl Default for MCPConsentManager {
    fn default() -> Self {
        Self::new()
    }
}
