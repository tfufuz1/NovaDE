//! Default implementation of the AI consent manager.
//!
//! This module provides a default implementation of the AI consent manager
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, AIError};
use crate::entities::value_objects::Timestamp;
use super::{ConsentManager, AIFeature, ConsentRecord, ConsentStatus};

/// Default implementation of the AI consent manager.
pub struct DefaultConsentManager {
    features: HashMap<String, AIFeature>,
    consents: HashMap<String, ConsentRecord>,
    user_feature_to_consent: HashMap<(String, String), String>,
}

impl DefaultConsentManager {
    /// Creates a new default consent manager.
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            consents: HashMap::new(),
            user_feature_to_consent: HashMap::new(),
        }
    }
}

#[async_trait]
impl ConsentManager for DefaultConsentManager {
    async fn register_feature(
        &self,
        name: &str,
        description: &str,
        data_usage_policy: &str,
        required: bool,
    ) -> Result<String, DomainError> {
        let feature_id = Uuid::new_v4().to_string();
        
        let feature = AIFeature {
            feature_id: feature_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            data_usage_policy: data_usage_policy.to_string(),
            required,
        };
        
        let mut features = self.features.clone();
        features.insert(feature_id.clone(), feature);
        
        // Update self
        *self = Self {
            features,
            consents: self.consents.clone(),
            user_feature_to_consent: self.user_feature_to_consent.clone(),
        };
        
        Ok(feature_id)
    }
    
    async fn get_feature(&self, feature_id: &str) -> Result<AIFeature, DomainError> {
        self.features.get(feature_id)
            .cloned()
            .ok_or_else(|| AIError::FeatureNotFound(feature_id.to_string()).into())
    }
    
    async fn list_features(&self) -> Result<Vec<AIFeature>, DomainError> {
        Ok(self.features.values().cloned().collect())
    }
    
    async fn record_consent(
        &self,
        user_id: &str,
        feature_id: &str,
        status: ConsentStatus,
        notes: Option<&str>,
    ) -> Result<String, DomainError> {
        if !self.features.contains_key(feature_id) {
            return Err(AIError::FeatureNotFound(feature_id.to_string()).into());
        }
        
        let consent_id = Uuid::new_v4().to_string();
        
        let consent = ConsentRecord {
            consent_id: consent_id.clone(),
            user_id: user_id.to_string(),
            feature_id: feature_id.to_string(),
            status,
            updated_at: Timestamp::now(),
            notes: notes.map(|s| s.to_string()),
        };
        
        let mut consents = self.consents.clone();
        let mut user_feature_to_consent = self.user_feature_to_consent.clone();
        
        consents.insert(consent_id.clone(), consent);
        user_feature_to_consent.insert((user_id.to_string(), feature_id.to_string()), consent_id.clone());
        
        // Update self
        *self = Self {
            features: self.features.clone(),
            consents,
            user_feature_to_consent,
        };
        
        Ok(consent_id)
    }
    
    async fn get_consent(&self, user_id: &str, feature_id: &str) -> Result<ConsentRecord, DomainError> {
        let consent_id = self.user_feature_to_consent.get(&(user_id.to_string(), feature_id.to_string()))
            .ok_or_else(|| AIError::ConsentNotFound {
                user_id: user_id.to_string(),
                feature_id: feature_id.to_string(),
            })?;
        
        self.consents.get(consent_id)
            .cloned()
            .ok_or_else(|| AIError::ConsentNotFound {
                user_id: user_id.to_string(),
                feature_id: feature_id.to_string(),
            }.into())
    }
    
    async fn list_consents_for_user(&self, user_id: &str) -> Result<Vec<ConsentRecord>, DomainError> {
        Ok(self.consents.values()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect())
    }
    
    async fn has_consent(&self, user_id: &str, feature_id: &str) -> Result<bool, DomainError> {
        match self.get_consent(user_id, feature_id).await {
            Ok(consent) => Ok(consent.status == ConsentStatus::Granted),
            Err(DomainError::AI(AIError::ConsentNotFound { .. })) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    async fn revoke_consent(&self, user_id: &str, feature_id: &str) -> Result<(), DomainError> {
        let consent_id = self.user_feature_to_consent.get(&(user_id.to_string(), feature_id.to_string()))
            .ok_or_else(|| AIError::ConsentNotFound {
                user_id: user_id.to_string(),
                feature_id: feature_id.to_string(),
            })?;
        
        let mut consents = self.consents.clone();
        
        if let Some(consent) = consents.get_mut(consent_id) {
            consent.status = ConsentStatus::Revoked;
            consent.updated_at = Timestamp::now();
        }
        
        // Update self
        *self = Self {
            features: self.features.clone(),
            consents,
            user_feature_to_consent: self.user_feature_to_consent.clone(),
        };
        
        Ok(())
    }
    
    async fn revoke_all_consents(&self, user_id: &str) -> Result<(), DomainError> {
        let mut consents = self.consents.clone();
        
        for consent in consents.values_mut() {
            if consent.user_id == user_id {
                consent.status = ConsentStatus::Revoked;
                consent.updated_at = Timestamp::now();
            }
        }
        
        // Update self
        *self = Self {
            features: self.features.clone(),
            consents,
            user_feature_to_consent: self.user_feature_to_consent.clone(),
        };
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_feature() {
        let manager = DefaultConsentManager::new();
        
        let feature_id = manager.register_feature(
            "Test Feature",
            "A test feature",
            "This feature uses your data for testing purposes.",
            false,
        ).await.unwrap();
        
        assert!(!feature_id.is_empty());
        
        let feature = manager.get_feature(&feature_id).await.unwrap();
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.description, "A test feature");
        assert_eq!(feature.data_usage_policy, "This feature uses your data for testing purposes.");
        assert_eq!(feature.required, false);
    }
    
    #[tokio::test]
    async fn test_record_and_get_consent() {
        let manager = DefaultConsentManager::new();
        
        let feature_id = manager.register_feature(
            "Test Feature",
            "A test feature",
            "This feature uses your data for testing purposes.",
            false,
        ).await.unwrap();
        
        let user_id = "user123";
        
        let consent_id = manager.record_consent(
            user_id,
            &feature_id,
            ConsentStatus::Granted,
            Some("Test consent"),
        ).await.unwrap();
        
        assert!(!consent_id.is_empty());
        
        let consent = manager.get_consent(user_id, &feature_id).await.unwrap();
        assert_eq!(consent.user_id, user_id);
        assert_eq!(consent.feature_id, feature_id);
        assert_eq!(consent.status, ConsentStatus::Granted);
        assert_eq!(consent.notes, Some("Test consent".to_string()));
    }
    
    #[tokio::test]
    async fn test_has_consent() {
        let manager = DefaultConsentManager::new();
        
        let feature_id = manager.register_feature(
            "Test Feature",
            "A test feature",
            "This feature uses your data for testing purposes.",
            false,
        ).await.unwrap();
        
        let user_id = "user123";
        
        // Initially, user has not consented
        assert_eq!(manager.has_consent(user_id, &feature_id).await.unwrap(), false);
        
        // Record consent
        manager.record_consent(
            user_id,
            &feature_id,
            ConsentStatus::Granted,
            None,
        ).await.unwrap();
        
        // Now user has consented
        assert_eq!(manager.has_consent(user_id, &feature_id).await.unwrap(), true);
        
        // Record denial
        manager.record_consent(
            user_id,
            &feature_id,
            ConsentStatus::Denied,
            None,
        ).await.unwrap();
        
        // Now user has not consented
        assert_eq!(manager.has_consent(user_id, &feature_id).await.unwrap(), false);
    }
    
    #[tokio::test]
    async fn test_revoke_consent() {
        let manager = DefaultConsentManager::new();
        
        let feature_id = manager.register_feature(
            "Test Feature",
            "A test feature",
            "This feature uses your data for testing purposes.",
            false,
        ).await.unwrap();
        
        let user_id = "user123";
        
        // Record consent
        manager.record_consent(
            user_id,
            &feature_id,
            ConsentStatus::Granted,
            None,
        ).await.unwrap();
        
        // Revoke consent
        manager.revoke_consent(user_id, &feature_id).await.unwrap();
        
        // Check that consent is revoked
        let consent = manager.get_consent(user_id, &feature_id).await.unwrap();
        assert_eq!(consent.status, ConsentStatus::Revoked);
        
        // User no longer has consent
        assert_eq!(manager.has_consent(user_id, &feature_id).await.unwrap(), false);
    }
    
    #[tokio::test]
    async fn test_revoke_all_consents() {
        let manager = DefaultConsentManager::new();
        
        let feature_id1 = manager.register_feature(
            "Feature 1",
            "Feature 1",
            "Data usage policy 1",
            false,
        ).await.unwrap();
        
        let feature_id2 = manager.register_feature(
            "Feature 2",
            "Feature 2",
            "Data usage policy 2",
            false,
        ).await.unwrap();
        
        let user_id = "user123";
        
        // Record consents
        manager.record_consent(
            user_id,
            &feature_id1,
            ConsentStatus::Granted,
            None,
        ).await.unwrap();
        
        manager.record_consent(
            user_id,
            &feature_id2,
            ConsentStatus::Granted,
            None,
        ).await.unwrap();
        
        // Revoke all consents
        manager.revoke_all_consents(user_id).await.unwrap();
        
        // Check that all consents are revoked
        let consents = manager.list_consents_for_user(user_id).await.unwrap();
        assert_eq!(consents.len(), 2);
        
        for consent in consents {
            assert_eq!(consent.status, ConsentStatus::Revoked);
        }
        
        // User no longer has consent for either feature
        assert_eq!(manager.has_consent(user_id, &feature_id1).await.unwrap(), false);
        assert_eq!(manager.has_consent(user_id, &feature_id2).await.unwrap(), false);
    }
}
