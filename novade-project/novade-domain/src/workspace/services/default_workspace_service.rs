//! Default workspace service implementation for the NovaDE domain layer.
//!
//! This module provides a default implementation of the workspace service
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, WorkspaceError};
use crate::entities::value_objects::Timestamp;
use super::{WorkspaceService, Workspace, WorkspaceWindowRef};

/// Default implementation of the workspace service.
pub struct DefaultWorkspaceService {
    workspaces: HashMap<String, Workspace>,
    window_refs: HashMap<String, WorkspaceWindowRef>,
    window_to_workspace: HashMap<String, String>,
    active_workspace_id: Option<String>,
}

impl DefaultWorkspaceService {
    /// Creates a new default workspace service.
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            window_refs: HashMap::new(),
            window_to_workspace: HashMap::new(),
            active_workspace_id: None,
        }
    }
    
    /// Creates a new default workspace service with a default workspace.
    pub fn with_default_workspace() -> Result<Self, DomainError> {
        let mut service = Self::new();
        
        // Create default workspace
        let default_workspace_id = service.create_workspace(
            "Default Workspace",
            "Default workspace for NovaDE",
            None,
            HashMap::new(),
        ).await?;
        
        // Set as active
        service.activate_workspace(&default_workspace_id).await?;
        
        Ok(service)
    }
}

#[async_trait]
impl WorkspaceService for DefaultWorkspaceService {
    async fn create_workspace(
        &self,
        name: &str,
        description: &str,
        icon: Option<&str>,
        properties: HashMap<String, String>,
    ) -> Result<String, DomainError> {
        let workspace_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();
        
        let workspace = Workspace {
            workspace_id: workspace_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            icon: icon.map(|s| s.to_string()),
            created_at: now,
            modified_at: now,
            last_accessed_at: now,
            properties,
        };
        
        let mut workspaces = self.workspaces.clone();
        workspaces.insert(workspace_id.clone(), workspace);
        
        // Update self
        *self = Self {
            workspaces,
            window_refs: self.window_refs.clone(),
            window_to_workspace: self.window_to_workspace.clone(),
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(workspace_id)
    }
    
    async fn get_workspace(&self, workspace_id: &str) -> Result<Workspace, DomainError> {
        let mut workspace = self.workspaces.get(workspace_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace_id.to_string()))?;
        
        // Update last accessed timestamp
        workspace.last_accessed_at = Timestamp::now();
        
        let mut workspaces = self.workspaces.clone();
        workspaces.insert(workspace_id.to_string(), workspace.clone());
        
        // Update self
        *self = Self {
            workspaces,
            window_refs: self.window_refs.clone(),
            window_to_workspace: self.window_to_workspace.clone(),
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(workspace)
    }
    
    async fn update_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        description: &str,
        icon: Option<&str>,
        properties: HashMap<String, String>,
    ) -> Result<(), DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        let mut workspaces = self.workspaces.clone();
        
        let workspace = workspaces.get_mut(workspace_id).unwrap();
        workspace.name = name.to_string();
        workspace.description = description.to_string();
        workspace.icon = icon.map(|s| s.to_string());
        workspace.properties = properties;
        workspace.modified_at = Timestamp::now();
        
        // Update self
        *self = Self {
            workspaces,
            window_refs: self.window_refs.clone(),
            window_to_workspace: self.window_to_workspace.clone(),
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(())
    }
    
    async fn delete_workspace(&self, workspace_id: &str) -> Result<(), DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        if self.active_workspace_id.as_deref() == Some(workspace_id) {
            return Err(WorkspaceError::CannotDeleteActiveWorkspace.into());
        }
        
        let mut workspaces = self.workspaces.clone();
        let mut window_refs = self.window_refs.clone();
        let mut window_to_workspace = self.window_to_workspace.clone();
        
        // Remove workspace
        workspaces.remove(workspace_id);
        
        // Remove window references for this workspace
        let refs_to_remove: Vec<String> = window_refs.iter()
            .filter(|(_, r)| r.workspace_id == workspace_id)
            .map(|(id, _)| id.clone())
            .collect();
        
        for ref_id in refs_to_remove {
            let window_ref = window_refs.remove(&ref_id).unwrap();
            window_to_workspace.remove(&window_ref.window_id);
        }
        
        // Update self
        *self = Self {
            workspaces,
            window_refs,
            window_to_workspace,
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(())
    }
    
    async fn list_workspaces(&self) -> Result<Vec<Workspace>, DomainError> {
        Ok(self.workspaces.values().cloned().collect())
    }
    
    async fn add_window_to_workspace(
        &self,
        workspace_id: &str,
        window_id: &str,
    ) -> Result<String, DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        // Check if window is already in a workspace
        if let Some(existing_workspace_id) = self.window_to_workspace.get(window_id) {
            // If it's the same workspace, just return the existing reference
            if existing_workspace_id == workspace_id {
                let ref_id = self.window_refs.iter()
                    .find(|(_, r)| r.workspace_id == workspace_id && r.window_id == window_id)
                    .map(|(id, _)| id.clone())
                    .ok_or_else(|| WorkspaceError::WindowReferenceNotFound(window_id.to_string()))?;
                
                return Ok(ref_id);
            }
            
            // Otherwise, remove from the old workspace first
            self.remove_window_from_workspace(existing_workspace_id, window_id).await?;
        }
        
        let ref_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();
        
        let window_ref = WorkspaceWindowRef {
            ref_id: ref_id.clone(),
            workspace_id: workspace_id.to_string(),
            window_id: window_id.to_string(),
            created_at: now,
        };
        
        let mut window_refs = self.window_refs.clone();
        let mut window_to_workspace = self.window_to_workspace.clone();
        
        window_refs.insert(ref_id.clone(), window_ref);
        window_to_workspace.insert(window_id.to_string(), workspace_id.to_string());
        
        // Update self
        *self = Self {
            workspaces: self.workspaces.clone(),
            window_refs,
            window_to_workspace,
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(ref_id)
    }
    
    async fn remove_window_from_workspace(
        &self,
        workspace_id: &str,
        window_id: &str,
    ) -> Result<(), DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        let ref_id = self.window_refs.iter()
            .find(|(_, r)| r.workspace_id == workspace_id && r.window_id == window_id)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| WorkspaceError::WindowNotInWorkspace {
                workspace_id: workspace_id.to_string(),
                window_id: window_id.to_string(),
            })?;
        
        let mut window_refs = self.window_refs.clone();
        let mut window_to_workspace = self.window_to_workspace.clone();
        
        window_refs.remove(&ref_id);
        window_to_workspace.remove(window_id);
        
        // Update self
        *self = Self {
            workspaces: self.workspaces.clone(),
            window_refs,
            window_to_workspace,
            active_workspace_id: self.active_workspace_id.clone(),
        };
        
        Ok(())
    }
    
    async fn list_windows_in_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceWindowRef>, DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        Ok(self.window_refs.values()
            .filter(|r| r.workspace_id == workspace_id)
            .cloned()
            .collect())
    }
    
    async fn get_workspace_for_window(
        &self,
        window_id: &str,
    ) -> Result<Workspace, DomainError> {
        let workspace_id = self.window_to_workspace.get(window_id)
            .ok_or_else(|| WorkspaceError::WindowNotInAnyWorkspace(window_id.to_string()))?;
        
        self.get_workspace(workspace_id).await
    }
    
    async fn activate_workspace(&self, workspace_id: &str) -> Result<(), DomainError> {
        if !self.workspaces.contains_key(workspace_id) {
            return Err(WorkspaceError::WorkspaceNotFound(workspace_id.to_string()).into());
        }
        
        // Update self
        *self = Self {
            workspaces: self.workspaces.clone(),
            window_refs: self.window_refs.clone(),
            window_to_workspace: self.window_to_workspace.clone(),
            active_workspace_id: Some(workspace_id.to_string()),
        };
        
        Ok(())
    }
    
    async fn get_active_workspace(&self) -> Result<Workspace, DomainError> {
        let active_workspace_id = self.active_workspace_id.as_deref()
            .ok_or_else(|| WorkspaceError::NoActiveWorkspace)?;
        
        self.get_workspace(active_workspace_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        assert!(!workspace_id.is_empty());
        
        let workspace = service.get_workspace(&workspace_id).await.unwrap();
        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.description, "A test workspace");
        assert_eq!(workspace.icon, None);
        assert!(workspace.properties.is_empty());
    }
    
    #[tokio::test]
    async fn test_update_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        let mut properties = HashMap::new();
        properties.insert("key".to_string(), "value".to_string());
        
        service.update_workspace(
            &workspace_id,
            "Updated Workspace",
            "An updated workspace",
            Some("icon.png"),
            properties.clone(),
        ).await.unwrap();
        
        let workspace = service.get_workspace(&workspace_id).await.unwrap();
        assert_eq!(workspace.name, "Updated Workspace");
        assert_eq!(workspace.description, "An updated workspace");
        assert_eq!(workspace.icon, Some("icon.png".to_string()));
        assert_eq!(workspace.properties, properties);
    }
    
    #[tokio::test]
    async fn test_delete_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        service.delete_workspace(&workspace_id).await.unwrap();
        
        let result = service.get_workspace(&workspace_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_add_window_to_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        let window_id = "window1";
        
        let ref_id = service.add_window_to_workspace(&workspace_id, window_id).await.unwrap();
        assert!(!ref_id.is_empty());
        
        let windows = service.list_windows_in_workspace(&workspace_id).await.unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].window_id, window_id);
    }
    
    #[tokio::test]
    async fn test_remove_window_from_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        let window_id = "window1";
        
        service.add_window_to_workspace(&workspace_id, window_id).await.unwrap();
        service.remove_window_from_workspace(&workspace_id, window_id).await.unwrap();
        
        let windows = service.list_windows_in_workspace(&workspace_id).await.unwrap();
        assert_eq!(windows.len(), 0);
    }
    
    #[tokio::test]
    async fn test_get_workspace_for_window() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        let window_id = "window1";
        
        service.add_window_to_workspace(&workspace_id, window_id).await.unwrap();
        
        let workspace = service.get_workspace_for_window(window_id).await.unwrap();
        assert_eq!(workspace.workspace_id, workspace_id);
    }
    
    #[tokio::test]
    async fn test_active_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        service.activate_workspace(&workspace_id).await.unwrap();
        
        let active_workspace = service.get_active_workspace().await.unwrap();
        assert_eq!(active_workspace.workspace_id, workspace_id);
    }
    
    #[tokio::test]
    async fn test_with_default_workspace() {
        let service = DefaultWorkspaceService::with_default_workspace().unwrap();
        
        // Should have a default workspace
        let workspaces = service.list_workspaces().await.unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, "Default Workspace");
        
        // Default workspace should be active
        let active_workspace = service.get_active_workspace().await.unwrap();
        assert_eq!(active_workspace.name, "Default Workspace");
    }
    
    #[tokio::test]
    async fn test_cannot_delete_active_workspace() {
        let service = DefaultWorkspaceService::new();
        
        let workspace_id = service.create_workspace(
            "Test Workspace",
            "A test workspace",
            None,
            HashMap::new(),
        ).await.unwrap();
        
        service.activate_workspace(&workspace_id).await.unwrap();
        
        let result = service.delete_workspace(&workspace_id).await;
        assert!(result.is_err());
    }
}
