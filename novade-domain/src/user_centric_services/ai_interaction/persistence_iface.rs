use async_trait::async_trait;
use uuid::Uuid;

use super::types::{AIConsent, AIModelProfile};
use super::errors::AIInteractionError;

// --- AIConsentProvider Trait ---
#[async_trait]
pub trait AIConsentProvider: Send + Sync {
    async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>;
    async fn save_consent(&self, consent: &AIConsent) -> Result<(), AIInteractionError>;
    async fn revoke_consent(&self, consent_id: Uuid, user_id: &str) -> Result<(), AIInteractionError>;
}

// --- AIModelProfileProvider Trait ---
#[async_trait]
pub trait AIModelProfileProvider: Send + Sync {
    async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
    async fn save_model_profiles(&self, profiles: &[AIModelProfile]) -> Result<(), AIInteractionError>;
}

// No unit tests in this file as it only contains trait definitions.
// Mock implementations for these traits will be created in the testing scope of their consumers.
