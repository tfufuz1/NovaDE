use async_trait::async_trait;
use super::types::{AIConsent, AIModelProfile};
use super::errors::AIInteractionError;

#[async_trait]
pub trait AIConsentProvider: Send + Sync {
    async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>;
    async fn save_consent(&self, consent: &AIConsent) -> Result<(), AIInteractionError>;
}

#[async_trait]
pub trait AIModelProfileProvider: Send + Sync {
    async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
}
