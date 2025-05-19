//! Default implementation of the window policy manager.
//!
//! This module provides a default implementation of the window policy manager
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, WindowManagementError};
use crate::entities::value_objects::Timestamp;
use super::{WindowPolicyManager, WindowPolicy, WindowType, Window, WindowAction};

/// Default implementation of the window policy manager.
pub struct DefaultWindowPolicyManager {
    policies: HashMap<String, WindowPolicy>,
}

impl DefaultWindowPolicyManager {
    /// Creates a new default window policy manager.
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
    
    /// Creates a new default window policy manager with default policies.
    pub fn with_default_policies() -> Result<Self, DomainError> {
        let mut manager = Self::new();
        
        // Create default policy for normal windows
        let normal_policy_id = manager.create_policy(
            "Default Normal Window Policy",
            "Default policy for normal application windows",
            100, // Medium priority
            vec![WindowType::Normal],
            true, // Auto focus
            true, // Allow workspace movement
            true, // Allow maximize
            true, // Allow minimize
            true, // Allow fullscreen
            true, // Allow resize
            true, // Allow close
        ).await?;
        
        // Create default policy for dialog windows
        let dialog_policy_id = manager.create_policy(
            "Default Dialog Policy",
            "Default policy for dialog windows",
            200, // Higher priority than normal windows
            vec![WindowType::Dialog],
            true, // Auto focus
            false, // Don't allow workspace movement
            true, // Allow maximize
            true, // Allow minimize
            false, // Don't allow fullscreen
            true, // Allow resize
            true, // Allow close
        ).await?;
        
        // Create default policy for utility windows
        let utility_policy_id = manager.create_policy(
            "Default Utility Policy",
            "Default policy for utility windows",
            150, // Higher than normal, lower than dialog
            vec![WindowType::Utility],
            false, // Don't auto focus
            true, // Allow workspace movement
            true, // Allow maximize
            true, // Allow minimize
            false, // Don't allow fullscreen
            true, // Allow resize
            true, // Allow close
        ).await?;
        
        // Create default policy for popup windows
        let popup_policy_id = manager.create_policy(
            "Default Popup Policy",
            "Default policy for popup windows",
            300, // High priority
            vec![WindowType::Popup, WindowType::Menu],
            false, // Don't auto focus
            false, // Don't allow workspace movement
            false, // Don't allow maximize
            false, // Don't allow minimize
            false, // Don't allow fullscreen
            false, // Don't allow resize
            true, // Allow close
        ).await?;
        
        // Create default policy for notification windows
        let notification_policy_id = manager.create_policy(
            "Default Notification Policy",
            "Default policy for notification windows",
            400, // Highest priority
            vec![WindowType::Notification],
            false, // Don't auto focus
            false, // Don't allow workspace movement
            false, // Don't allow maximize
            false, // Don't allow minimize
            false, // Don't allow fullscreen
            false, // Don't allow resize
            true, // Allow close
        ).await?;
        
        Ok(manager)
    }
}

#[async_trait]
impl WindowPolicyManager for DefaultWindowPolicyManager {
    async fn create_policy(
        &self,
        name: &str,
        description: &str,
        priority: i32,
        applies_to: Vec<WindowType>,
        auto_focus: bool,
        allow_workspace_movement: bool,
        allow_maximize: bool,
        allow_minimize: bool,
        allow_fullscreen: bool,
        allow_resize: bool,
        allow_close: bool,
    ) -> Result<String, DomainError> {
        let policy_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();
        
        let policy = WindowPolicy {
            policy_id: policy_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            priority,
            applies_to,
            auto_focus,
            allow_workspace_movement,
            allow_maximize,
            allow_minimize,
            allow_fullscreen,
            allow_resize,
            allow_close,
            created_at: now,
            modified_at: now,
        };
        
        let mut policies = self.policies.clone();
        policies.insert(policy_id.clone(), policy);
        
        // Update self
        *self = Self {
            policies,
        };
        
        Ok(policy_id)
    }
    
    async fn get_policy(&self, policy_id: &str) -> Result<WindowPolicy, DomainError> {
        self.policies.get(policy_id)
            .cloned()
            .ok_or_else(|| WindowManagementError::PolicyNotFound(policy_id.to_string()).into())
    }
    
    async fn update_policy(
        &self,
        policy_id: &str,
        name: &str,
        description: &str,
        priority: i32,
        applies_to: Vec<WindowType>,
        auto_focus: bool,
        allow_workspace_movement: bool,
        allow_maximize: bool,
        allow_minimize: bool,
        allow_fullscreen: bool,
        allow_resize: bool,
        allow_close: bool,
    ) -> Result<(), DomainError> {
        if !self.policies.contains_key(policy_id) {
            return Err(WindowManagementError::PolicyNotFound(policy_id.to_string()).into());
        }
        
        let mut policies = self.policies.clone();
        
        let now = Timestamp::now();
        let created_at = policies.get(policy_id).unwrap().created_at;
        
        let policy = WindowPolicy {
            policy_id: policy_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            priority,
            applies_to,
            auto_focus,
            allow_workspace_movement,
            allow_maximize,
            allow_minimize,
            allow_fullscreen,
            allow_resize,
            allow_close,
            created_at,
            modified_at: now,
        };
        
        policies.insert(policy_id.to_string(), policy);
        
        // Update self
        *self = Self {
            policies,
        };
        
        Ok(())
    }
    
    async fn delete_policy(&self, policy_id: &str) -> Result<(), DomainError> {
        if !self.policies.contains_key(policy_id) {
            return Err(WindowManagementError::PolicyNotFound(policy_id.to_string()).into());
        }
        
        let mut policies = self.policies.clone();
        policies.remove(policy_id);
        
        // Update self
        *self = Self {
            policies,
        };
        
        Ok(())
    }
    
    async fn list_policies(&self) -> Result<Vec<WindowPolicy>, DomainError> {
        Ok(self.policies.values().cloned().collect())
    }
    
    async fn get_effective_policy(&self, window_type: WindowType) -> Result<WindowPolicy, DomainError> {
        let mut applicable_policies: Vec<&WindowPolicy> = self.policies.values()
            .filter(|p| p.applies_to.contains(&window_type))
            .collect();
        
        if applicable_policies.is_empty() {
            return Err(WindowManagementError::NoPolicyFound(format!("{:?}", window_type)).into());
        }
        
        // Sort by priority (highest first)
        applicable_policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Return the highest priority policy
        Ok(applicable_policies[0].clone())
    }
    
    async fn is_action_allowed(&self, window: &Window, action: WindowAction) -> Result<bool, DomainError> {
        let policy = self.get_effective_policy(window.window_type.clone()).await?;
        
        let allowed = match action {
            WindowAction::Move => window.movable,
            WindowAction::Resize => window.resizable && policy.allow_resize,
            WindowAction::Maximize => policy.allow_maximize,
            WindowAction::Minimize => policy.allow_minimize,
            WindowAction::Fullscreen => policy.allow_fullscreen,
            WindowAction::Close => window.closable && policy.allow_close,
            WindowAction::MoveToWorkspace => policy.allow_workspace_movement,
        };
        
        Ok(allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::geometry::{Point, Size, Rect};
    
    #[tokio::test]
    async fn test_create_policy() {
        let manager = DefaultWindowPolicyManager::new();
        
        let policy_id = manager.create_policy(
            "Test Policy",
            "A test policy",
            100,
            vec![WindowType::Normal],
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ).await.unwrap();
        
        assert!(!policy_id.is_empty());
        
        let policy = manager.get_policy(&policy_id).await.unwrap();
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.description, "A test policy");
        assert_eq!(policy.priority, 100);
        assert_eq!(policy.applies_to, vec![WindowType::Normal]);
        assert_eq!(policy.auto_focus, true);
        assert_eq!(policy.allow_workspace_movement, true);
        assert_eq!(policy.allow_maximize, true);
        assert_eq!(policy.allow_minimize, true);
        assert_eq!(policy.allow_fullscreen, true);
        assert_eq!(policy.allow_resize, true);
        assert_eq!(policy.allow_close, true);
    }
    
    #[tokio::test]
    async fn test_update_policy() {
        let manager = DefaultWindowPolicyManager::new();
        
        let policy_id = manager.create_policy(
            "Test Policy",
            "A test policy",
            100,
            vec![WindowType::Normal],
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ).await.unwrap();
        
        manager.update_policy(
            &policy_id,
            "Updated Policy",
            "An updated policy",
            200,
            vec![WindowType::Normal, WindowType::Dialog],
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ).await.unwrap();
        
        let policy = manager.get_policy(&policy_id).await.unwrap();
        assert_eq!(policy.name, "Updated Policy");
        assert_eq!(policy.description, "An updated policy");
        assert_eq!(policy.priority, 200);
        assert_eq!(policy.applies_to, vec![WindowType::Normal, WindowType::Dialog]);
        assert_eq!(policy.auto_focus, false);
        assert_eq!(policy.allow_workspace_movement, false);
        assert_eq!(policy.allow_maximize, false);
        assert_eq!(policy.allow_minimize, false);
        assert_eq!(policy.allow_fullscreen, false);
        assert_eq!(policy.allow_resize, false);
        assert_eq!(policy.allow_close, false);
    }
    
    #[tokio::test]
    async fn test_delete_policy() {
        let manager = DefaultWindowPolicyManager::new();
        
        let policy_id = manager.create_policy(
            "Test Policy",
            "A test policy",
            100,
            vec![WindowType::Normal],
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ).await.unwrap();
        
        manager.delete_policy(&policy_id).await.unwrap();
        
        let result = manager.get_policy(&policy_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_effective_policy() {
        let manager = DefaultWindowPolicyManager::new();
        
        // Create two policies for normal windows with different priorities
        let low_priority_id = manager.create_policy(
            "Low Priority",
            "Low priority policy",
            100,
            vec![WindowType::Normal],
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ).await.unwrap();
        
        let high_priority_id = manager.create_policy(
            "High Priority",
            "High priority policy",
            200,
            vec![WindowType::Normal],
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ).await.unwrap();
        
        // The effective policy should be the high priority one
        let effective_policy = manager.get_effective_policy(WindowType::Normal).await.unwrap();
        assert_eq!(effective_policy.name, "High Priority");
        assert_eq!(effective_policy.priority, 200);
    }
    
    #[tokio::test]
    async fn test_is_action_allowed() {
        let manager = DefaultWindowPolicyManager::new();
        
        // Create a policy that disallows certain actions
        let policy_id = manager.create_policy(
            "Restrictive Policy",
            "Restrictive policy",
            100,
            vec![WindowType::Normal],
            true,
            false, // Don't allow workspace movement
            false, // Don't allow maximize
            true,  // Allow minimize
            false, // Don't allow fullscreen
            true,  // Allow resize
            true,  // Allow close
        ).await.unwrap();
        
        // Create a window
        let window = Window {
            window_id: "window1".to_string(),
            title: "Test Window".to_string(),
            app_id: "test.app".to_string(),
            window_type: WindowType::Normal,
            state: super::WindowState::Normal,
            geometry: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size { width: 800.0, height: 600.0 },
            },
            min_size: None,
            max_size: None,
            resizable: true,
            movable: true,
            closable: true,
            focused: false,
            created_at: Timestamp::now(),
            last_focused_at: None,
            properties: HashMap::new(),
        };
        
        // Check allowed actions
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Move).await.unwrap(), true);
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Resize).await.unwrap(), true);
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Minimize).await.unwrap(), true);
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Close).await.unwrap(), true);
        
        // Check disallowed actions
        assert_eq!(manager.is_action_allowed(&window, WindowAction::MoveToWorkspace).await.unwrap(), false);
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Maximize).await.unwrap(), false);
        assert_eq!(manager.is_action_allowed(&window, WindowAction::Fullscreen).await.unwrap(), false);
    }
    
    #[tokio::test]
    async fn test_with_default_policies() {
        let manager = DefaultWindowPolicyManager::with_default_policies().unwrap();
        
        // Check that default policies exist
        let policies = manager.list_policies().await.unwrap();
        assert!(policies.len() >= 5); // At least 5 default policies
        
        // Check effective policies for different window types
        let normal_policy = manager.get_effective_policy(WindowType::Normal).await.unwrap();
        let dialog_policy = manager.get_effective_policy(WindowType::Dialog).await.unwrap();
        let utility_policy = manager.get_effective_policy(WindowType::Utility).await.unwrap();
        let popup_policy = manager.get_effective_policy(WindowType::Popup).await.unwrap();
        let notification_policy = manager.get_effective_policy(WindowType::Notification).await.unwrap();
        
        // Verify priorities are as expected
        assert!(dialog_policy.priority > normal_policy.priority);
        assert!(utility_policy.priority > normal_policy.priority);
        assert!(popup_policy.priority > dialog_policy.priority);
        assert!(notification_policy.priority > popup_policy.priority);
    }
}
