//! Default implementation of the AI interaction service.
//!
//! This module provides a default implementation of the AI interaction service
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, AIError};
use crate::entities::value_objects::Timestamp;
use super::{AIInteractionService, AIRequest, AIResponse, AIResponseStatus};

/// Default implementation of the AI interaction service.
pub struct DefaultAIInteractionService {
    requests: HashMap<String, AIRequest>,
    responses: HashMap<String, AIResponse>,
    request_to_response: HashMap<String, String>,
}

impl DefaultAIInteractionService {
    /// Creates a new default AI interaction service.
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            responses: HashMap::new(),
            request_to_response: HashMap::new(),
        }
    }
}

#[async_trait]
impl AIInteractionService for DefaultAIInteractionService {
    async fn send_request(
        &self,
        user_id: &str,
        request_type: &str,
        content: &str,
        parameters: HashMap<String, String>,
    ) -> Result<AIResponse, DomainError> {
        let request_id = Uuid::new_v4().to_string();
        let response_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();
        
        let request = AIRequest {
            request_id: request_id.clone(),
            user_id: user_id.to_string(),
            request_type: request_type.to_string(),
            content: content.to_string(),
            timestamp: now,
            parameters: parameters.clone(),
        };
        
        // In a real implementation, this would call an external AI service
        // For now, we just echo the request with a mock response
        let response_content = format!("AI response to: {}", content);
        
        let response = AIResponse {
            response_id: response_id.clone(),
            request_id: request_id.clone(),
            content: response_content,
            timestamp: now,
            status: AIResponseStatus::Success,
            metadata: HashMap::new(),
        };
        
        let mut requests = self.requests.clone();
        let mut responses = self.responses.clone();
        let mut request_to_response = self.request_to_response.clone();
        
        requests.insert(request_id.clone(), request);
        responses.insert(response_id.clone(), response.clone());
        request_to_response.insert(request_id, response_id);
        
        // Update self
        *self = Self {
            requests,
            responses,
            request_to_response,
        };
        
        Ok(response)
    }
    
    async fn get_request(&self, request_id: &str) -> Result<AIRequest, DomainError> {
        self.requests.get(request_id)
            .cloned()
            .ok_or_else(|| AIError::RequestNotFound(request_id.to_string()).into())
    }
    
    async fn get_response(&self, response_id: &str) -> Result<AIResponse, DomainError> {
        self.responses.get(response_id)
            .cloned()
            .ok_or_else(|| AIError::ResponseNotFound(response_id.to_string()).into())
    }
    
    async fn list_requests_for_user(&self, user_id: &str) -> Result<Vec<AIRequest>, DomainError> {
        Ok(self.requests.values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect())
    }
    
    async fn get_response_for_request(&self, request_id: &str) -> Result<AIResponse, DomainError> {
        let response_id = self.request_to_response.get(request_id)
            .ok_or_else(|| AIError::ResponseNotFound(format!("No response for request {}", request_id)))?;
        
        self.get_response(response_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_request() {
        let service = DefaultAIInteractionService::new();
        
        let user_id = "user123";
        let request_type = "text";
        let content = "Hello, AI!";
        let parameters = HashMap::new();
        
        let response = service.send_request(
            user_id,
            request_type,
            content,
            parameters,
        ).await.unwrap();
        
        assert!(!response.response_id.is_empty());
        assert!(!response.request_id.is_empty());
        assert!(response.content.contains("Hello, AI!"));
        assert_eq!(response.status, AIResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_get_request() {
        let service = DefaultAIInteractionService::new();
        
        let user_id = "user123";
        let request_type = "text";
        let content = "Hello, AI!";
        let parameters = HashMap::new();
        
        let response = service.send_request(
            user_id,
            request_type,
            content,
            parameters,
        ).await.unwrap();
        
        let request = service.get_request(&response.request_id).await.unwrap();
        assert_eq!(request.user_id, user_id);
        assert_eq!(request.request_type, request_type);
        assert_eq!(request.content, content);
    }
    
    #[tokio::test]
    async fn test_get_response() {
        let service = DefaultAIInteractionService::new();
        
        let user_id = "user123";
        let request_type = "text";
        let content = "Hello, AI!";
        let parameters = HashMap::new();
        
        let response1 = service.send_request(
            user_id,
            request_type,
            content,
            parameters,
        ).await.unwrap();
        
        let response2 = service.get_response(&response1.response_id).await.unwrap();
        assert_eq!(response2.response_id, response1.response_id);
        assert_eq!(response2.request_id, response1.request_id);
        assert_eq!(response2.content, response1.content);
    }
    
    #[tokio::test]
    async fn test_list_requests_for_user() {
        let service = DefaultAIInteractionService::new();
        
        let user_id1 = "user123";
        let user_id2 = "user456";
        let request_type = "text";
        let content = "Hello, AI!";
        let parameters = HashMap::new();
        
        // Send requests for user1
        service.send_request(
            user_id1,
            request_type,
            content,
            parameters.clone(),
        ).await.unwrap();
        
        service.send_request(
            user_id1,
            request_type,
            "Another request",
            parameters.clone(),
        ).await.unwrap();
        
        // Send request for user2
        service.send_request(
            user_id2,
            request_type,
            content,
            parameters.clone(),
        ).await.unwrap();
        
        // Check requests for user1
        let requests = service.list_requests_for_user(user_id1).await.unwrap();
        assert_eq!(requests.len(), 2);
        
        // Check requests for user2
        let requests = service.list_requests_for_user(user_id2).await.unwrap();
        assert_eq!(requests.len(), 1);
    }
    
    #[tokio::test]
    async fn test_get_response_for_request() {
        let service = DefaultAIInteractionService::new();
        
        let user_id = "user123";
        let request_type = "text";
        let content = "Hello, AI!";
        let parameters = HashMap::new();
        
        let response1 = service.send_request(
            user_id,
            request_type,
            content,
            parameters,
        ).await.unwrap();
        
        let response2 = service.get_response_for_request(&response1.request_id).await.unwrap();
        assert_eq!(response2.response_id, response1.response_id);
        assert_eq!(response2.request_id, response1.request_id);
        assert_eq!(response2.content, response1.content);
    }
}
