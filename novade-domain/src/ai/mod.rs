//! AI module for the NovaDE domain layer.
//!
//! This module provides AI-related functionality for the NovaDE desktop environment,
//! including consent management and AI interaction capabilities.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::error::{DomainError, AIError};
use crate::entities::value_objects::Timestamp;

/// Represents the consent status for AI features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    /// User has not yet made a decision
    Undecided,
    /// User has granted consent
    Granted,
    /// User has denied consent
    Denied,
    /// Consent has been revoked
    Revoked,
}

/// Represents a specific AI feature that requires consent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIFeature {
    /// Unique identifier for the feature
    feature_id: String,
    /// The feature name
    name: String,
    /// The feature description
    description: String,
    /// The data usage policy for this feature
    data_usage_policy: String,
    /// Whether this feature is required for core functionality
    required: bool,
}

/// Represents a user's consent for a specific AI feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Unique identifier for the consent record
    consent_id: String,
    /// The user ID
    user_id: String,
    /// The feature ID
    feature_id: String,
    /// The consent status
    status: ConsentStatus,
    /// The timestamp when consent was last updated
    updated_at: Timestamp,
    /// Additional notes or context for this consent
    notes: Option<String>,
}

/// Interface for the AI consent manager.
#[async_trait]
pub trait ConsentManager: Send + Sync {
    /// Registers a new AI feature.
    ///
    /// # Arguments
    ///
    /// * `name` - The feature name
    /// * `description` - The feature description
    /// * `data_usage_policy` - The data usage policy
    /// * `required` - Whether this feature is required
    ///
    /// # Returns
    ///
    /// A `Result` containing the created feature ID.
    async fn register_feature(
        &self,
        name: &str,
        description: &str,
        data_usage_policy: &str,
        required: bool,
    ) -> Result<String, DomainError>;
    
    /// Gets an AI feature by ID.
    ///
    /// # Arguments
    ///
    /// * `feature_id` - The feature ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the feature if found.
    async fn get_feature(&self, feature_id: &str) -> Result<AIFeature, DomainError>;
    
    /// Lists all AI features.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all features.
    async fn list_features(&self) -> Result<Vec<AIFeature>, DomainError>;
    
    /// Records a user's consent for a feature.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `feature_id` - The feature ID
    /// * `status` - The consent status
    /// * `notes` - Additional notes or context
    ///
    /// # Returns
    ///
    /// A `Result` containing the created consent record ID.
    async fn record_consent(
        &self,
        user_id: &str,
        feature_id: &str,
        status: ConsentStatus,
        notes: Option<&str>,
    ) -> Result<String, DomainError>;
    
    /// Gets a user's consent for a feature.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `feature_id` - The feature ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the consent record if found.
    async fn get_consent(&self, user_id: &str, feature_id: &str) -> Result<ConsentRecord, DomainError>;
    
    /// Lists all consent records for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all consent records for the user.
    async fn list_consents_for_user(&self, user_id: &str) -> Result<Vec<ConsentRecord>, DomainError>;
    
    /// Checks if a user has consented to a feature.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `feature_id` - The feature ID
    ///
    /// # Returns
    ///
    /// A `Result` containing true if the user has consented, false otherwise.
    async fn has_consent(&self, user_id: &str, feature_id: &str) -> Result<bool, DomainError>;
    
    /// Revokes a user's consent for a feature.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `feature_id` - The feature ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn revoke_consent(&self, user_id: &str, feature_id: &str) -> Result<(), DomainError>;
    
    /// Revokes all consents for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn revoke_all_consents(&self, user_id: &str) -> Result<(), DomainError>;
}

/// Represents an AI interaction request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIRequest {
    /// Unique identifier for the request
    request_id: String,
    /// The user ID
    user_id: String,
    /// The request type
    request_type: String,
    /// The request content
    content: String,
    /// The request timestamp
    timestamp: Timestamp,
    /// Additional parameters for the request
    parameters: HashMap<String, String>,
}

/// Represents an AI interaction response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIResponse {
    /// Unique identifier for the response
    response_id: String,
    /// The request ID this response is for
    request_id: String,
    /// The response content
    content: String,
    /// The response timestamp
    timestamp: Timestamp,
    /// The response status
    status: AIResponseStatus,
    /// Additional metadata for the response
    metadata: HashMap<String, String>,
}

/// Represents the status of an AI response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIResponseStatus {
    /// The response was successful
    Success,
    /// The response contains a warning
    Warning,
    /// The response contains an error
    Error,
}

/// Interface for the AI interaction service.
#[async_trait]
pub trait AIInteractionService: Send + Sync {
    /// Sends a request to the AI service.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `request_type` - The request type
    /// * `content` - The request content
    /// * `parameters` - Additional parameters for the request
    ///
    /// # Returns
    ///
    /// A `Result` containing the AI response.
    async fn send_request(
        &self,
        user_id: &str,
        request_type: &str,
        content: &str,
        parameters: HashMap<String, String>,
    ) -> Result<AIResponse, DomainError>;
    
    /// Gets a request by ID.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The request ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the request if found.
    async fn get_request(&self, request_id: &str) -> Result<AIRequest, DomainError>;
    
    /// Gets a response by ID.
    ///
    /// # Arguments
    ///
    /// * `response_id` - The response ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the response if found.
    async fn get_response(&self, response_id: &str) -> Result<AIResponse, DomainError>;
    
    /// Lists all requests for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all requests for the user.
    async fn list_requests_for_user(&self, user_id: &str) -> Result<Vec<AIRequest>, DomainError>;
    
    /// Gets the response for a request.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The request ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the response if found.
    async fn get_response_for_request(&self, request_id: &str) -> Result<AIResponse, DomainError>;
}

mod default_consent_manager;
mod default_interaction_service;

pub use default_consent_manager::DefaultConsentManager;
pub use default_interaction_service::DefaultAIInteractionService;
