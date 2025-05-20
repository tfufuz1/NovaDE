//! Workspace manager module for the NovaDE domain layer.
//!
//! This module provides the workspace manager service interface and implementation
//! for managing workspaces in the NovaDE desktop environment.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use crate::common_events::{DomainEvent, WorkspaceEvent};
use crate::error::{DomainResult, WorkspaceError};
use crate::shared_types::EntityId;
use crate::workspaces::core::{Workspace, WorkspaceId, WindowId, WorkspaceState, WorkspaceType};
use crate::workspaces::assignment::{AssignmentRule, WindowAssignment, WindowPattern};
use crate::workspaces::config::WorkspaceConfig;

/// Service interface for workspace management.
#[async_trait]
pub trait WorkspaceManagerService: Send + Sync {
    /// Creates a new workspace.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the workspace
    /// * `workspace_type` - The type of the workspace
    ///
    /// # Returns
    ///
    /// The created workspace.
    async fn create_workspace(
        &self,
        name: String,
        workspace_type: WorkspaceType,
    ) -> DomainResult<Workspace>;

    /// Gets a workspace by ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// The workspace, or an error if it doesn't exist.
    async fn get_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<Workspace>;

    /// Gets all workspaces.
    ///
    /// # Returns
    ///
    /// A vector of all workspaces.
    async fn get_all_workspaces(&self) -> DomainResult<Vec<Workspace>>;

    /// Updates a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The updated workspace
    ///
    /// # Returns
    ///
    /// The updated workspace.
    async fn update_workspace(&self, workspace: Workspace) -> DomainResult<Workspace>;

    /// Deletes a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if the workspace was deleted, or an error if it doesn't exist.
    async fn delete_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<()>;

    /// Activates a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to activate
    ///
    /// # Returns
    ///
    /// The activated workspace.
    async fn activate_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<Workspace>;

    /// Gets the active workspace.
    ///
    /// # Returns
    ///
    /// The active workspace, or an error if no workspace is active.
    async fn get_active_workspace(&self) -> DomainResult<Workspace>;

    /// Assigns a window to a workspace.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `workspace_id` - The ID of the workspace
    /// * `permanent` - Whether the assignment is permanent
    ///
    /// # Returns
    ///
    /// The window assignment.
    async fn assign_window(
        &self,
        window_id: WindowId,
        workspace_id: WorkspaceId,
        permanent: bool,
    ) -> DomainResult<WindowAssignment>;

    /// Gets the workspace for a window.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    ///
    /// # Returns
    ///
    /// The workspace containing the window, or an error if the window is not assigned.
    async fn get_window_workspace(&self, window_id: WindowId) -> DomainResult<Workspace>;

    /// Creates an assignment rule.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the rule
    /// * `priority` - The priority of the rule
    /// * `pattern` - The pattern to match against window properties
    /// * `target_workspace_id` - The ID of the target workspace
    ///
    /// # Returns
    ///
    /// The created rule.
    async fn create_rule(
        &self,
        name: String,
        priority: i32,
        pattern: WindowPattern,
        target_workspace_id: WorkspaceId,
    ) -> DomainResult<AssignmentRule>;

    /// Gets a rule by ID.
    ///
    /// # Arguments
    ///
    /// * `rule_id` - The ID of the rule
    ///
    /// # Returns
    ///
    /// The rule, or an error if it doesn't exist.
    async fn get_rule(&self, rule_id: EntityId) -> DomainResult<AssignmentRule>;

    /// Gets all rules.
    ///
    /// # Returns
    ///
    /// A vector of all rules.
    async fn get_all_rules(&self) -> DomainResult<Vec<AssignmentRule>>;

    /// Updates a rule.
    ///
    /// # Arguments
    ///
    /// * `rule` - The updated rule
    ///
    /// # Returns
    ///
    /// The updated rule.
    async fn update_rule(&self, rule: AssignmentRule) -> DomainResult<AssignmentRule>;

    /// Deletes a rule.
    ///
    /// # Arguments
    ///
    /// * `rule_id` - The ID of the rule to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if the rule was deleted, or an error if it doesn't exist.
    async fn delete_rule(&self, rule_id: EntityId) -> DomainResult<()>;

    /// Applies rules to a window.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `properties` - The properties of the window
    ///
    /// # Returns
    ///
    /// The window assignment, or `None` if no rule matched.
    async fn apply_rules(
        &self,
        window_id: WindowId,
        properties: WindowProperties,
    ) -> DomainResult<Option<WindowAssignment>>;
}

/// Properties of a window.
#[derive(Debug, Clone)]
pub struct WindowProperties {
    /// The application name.
    pub app_name: Option<String>,
    /// The window title.
    pub title: Option<String>,
    /// The window class.
    pub class: Option<String>,
    /// The window role.
    pub role: Option<String>,
}

impl WindowProperties {
    /// Creates a new window properties object.
    ///
    /// # Returns
    ///
    /// A new empty `WindowProperties`.
    pub fn new() -> Self {
        WindowProperties {
            app_name: None,
            title: None,
            class: None,
            role: None,
        }
    }

    /// Sets the application name.
    ///
    /// # Arguments
    ///
    /// * `app_name` - The application name
    ///
    /// # Returns
    ///
    /// The modified `WindowProperties`.
    pub fn with_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    /// Sets the window title.
    ///
    /// # Arguments
    ///
    /// * `title` - The window title
    ///
    /// # Returns
    ///
    /// The modified `WindowProperties`.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the window class.
    ///
    /// # Arguments
    ///
    /// * `class` - The window class
    ///
    /// # Returns
    ///
    /// The modified `WindowProperties`.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Sets the window role.
    ///
    /// # Arguments
    ///
    /// * `role` - The window role
    ///
    /// # Returns
    ///
    /// The modified `WindowProperties`.
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Checks if the properties match a pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to match against
    ///
    /// # Returns
    ///
    /// `true` if the properties match the pattern, `false` otherwise.
    pub fn matches(&self, pattern: &WindowPattern) -> bool {
        // If the pattern specifies an app name, it must match
        if let Some(pattern_app_name) = &pattern.app_name {
            if let Some(app_name) = &self.app_name {
                if !app_name.contains(pattern_app_name) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // If the pattern specifies a title, it must match
        if let Some(pattern_title) = &pattern.title {
            if let Some(title) = &self.title {
                if !title.contains(pattern_title) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // If the pattern specifies a class, it must match
        if let Some(pattern_class) = &pattern.class {
            if let Some(class) = &self.class {
                if !class.contains(pattern_class) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // If the pattern specifies a role, it must match
        if let Some(pattern_role) = &pattern.role {
            if let Some(role) = &self.role {
                if !role.contains(pattern_role) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // All specified patterns matched
        true
    }
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self::new()
    }
}

/// Default implementation of the workspace manager service.
pub struct DefaultWorkspaceManager {
    /// The workspaces, keyed by ID.
    workspaces: Arc<RwLock<HashMap<WorkspaceId, Workspace>>>,
    /// The active workspace ID.
    active_workspace_id: Arc<RwLock<Option<WorkspaceId>>>,
    /// The window assignments, keyed by window ID.
    window_assignments: Arc<RwLock<HashMap<WindowId, WindowAssignment>>>,
    /// The assignment rules, keyed by ID.
    rules: Arc<RwLock<HashMap<EntityId, AssignmentRule>>>,
    /// The event publisher function.
    event_publisher: Box<dyn Fn(DomainEvent<WorkspaceEvent>) + Send + Sync>,
}

impl DefaultWorkspaceManager {
    /// Creates a new default workspace manager.
    ///
    /// # Arguments
    ///
    /// * `event_publisher` - A function to publish workspace events
    ///
    /// # Returns
    ///
    /// A new `DefaultWorkspaceManager`.
    pub fn new<F>(event_publisher: F) -> Self
    where
        F: Fn(DomainEvent<WorkspaceEvent>) + Send + Sync + 'static,
    {
        DefaultWorkspaceManager {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            active_workspace_id: Arc::new(RwLock::new(None)),
            window_assignments: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            event_publisher: Box::new(event_publisher),
        }
    }

    /// Creates a new default workspace manager with initial configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The initial workspace configuration
    /// * `event_publisher` - A function to publish workspace events
    ///
    /// # Returns
    ///
    /// A new `DefaultWorkspaceManager` with the specified configuration.
    pub fn with_config<F>(config: WorkspaceConfig, event_publisher: F) -> Self
    where
        F: Fn(DomainEvent<WorkspaceEvent>) + Send + Sync + 'static,
    {
        let manager = Self::new(event_publisher);
        
        // TODO: Initialize from config
        
        manager
    }

    /// Publishes a workspace event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    fn publish_event(&self, event: WorkspaceEvent) {
        let domain_event = DomainEvent::new(event, "WorkspaceManager");
        (self.event_publisher)(domain_event);
    }
}

#[async_trait]
impl WorkspaceManagerService for DefaultWorkspaceManager {
    async fn create_workspace(
        &self,
        name: String,
        workspace_type: WorkspaceType,
    ) -> DomainResult<Workspace> {
        let workspace = Workspace::new(name.clone(), workspace_type);
        workspace.validate()?;
        
        let workspace_id = workspace.id();
        
        {
            let mut workspaces = self.workspaces.write().unwrap();
            workspaces.insert(workspace_id, workspace.clone());
        }
        
        self.publish_event(WorkspaceEvent::WorkspaceCreated {
            workspace_id,
            name,
        });
        
        Ok(workspace)
    }

    async fn get_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<Workspace> {
        let workspaces = self.workspaces.read().unwrap();
        
        workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()).into())
    }

    async fn get_all_workspaces(&self) -> DomainResult<Vec<Workspace>> {
        let workspaces = self.workspaces.read().unwrap();
        
        let mut result: Vec<Workspace> = workspaces.values().cloned().collect();
        result.sort_by(|a, b| a.name().cmp(b.name()));
        
        Ok(result)
    }

    async fn update_workspace(&self, workspace: Workspace) -> DomainResult<Workspace> {
        workspace.validate()?;
        
        let workspace_id = workspace.id();
        let name = workspace.name().to_string();
        
        {
            let mut workspaces = self.workspaces.write().unwrap();
            
            if !workspaces.contains_key(&workspace_id) {
                return Err(WorkspaceError::NotFound(workspace_id.to_string()).into());
            }
            
            workspaces.insert(workspace_id, workspace.clone());
        }
        
        self.publish_event(WorkspaceEvent::WorkspaceUpdated {
            workspace_id,
            name,
        });
        
        Ok(workspace)
    }

    async fn delete_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<()> {
        {
            let mut workspaces = self.workspaces.write().unwrap();
            
            if !workspaces.contains_key(&workspace_id) {
                return Err(WorkspaceError::NotFound(workspace_id.to_string()).into());
            }
            
            workspaces.remove(&workspace_id);
        }
        
        // Remove any window assignments to this workspace
        {
            let mut window_assignments = self.window_assignments.write().unwrap();
            window_assignments.retain(|_, assignment| assignment.workspace_id != workspace_id);
        }
        
        // If this was the active workspace, clear the active workspace
        {
            let mut active_workspace_id = self.active_workspace_id.write().unwrap();
            if active_workspace_id.as_ref() == Some(&workspace_id) {
                *active_workspace_id = None;
            }
        }
        
        self.publish_event(WorkspaceEvent::WorkspaceDeleted { workspace_id });
        
        Ok(())
    }

    async fn activate_workspace(&self, workspace_id: WorkspaceId) -> DomainResult<Workspace> {
        let mut workspace = {
            let mut workspaces = self.workspaces.write().unwrap();
            
            let workspace = workspaces
                .get_mut(&workspace_id)
                .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))?;
            
            // Deactivate the currently active workspace
            if let Some(active_id) = {
                let active_id = self.active_workspace_id.read().unwrap().clone();
                active_id
            } {
                if active_id != workspace_id {
                    if let Some(active_workspace) = workspaces.get_mut(&active_id) {
                        active_workspace.deactivate();
                    }
                }
            }
            
            // Activate the new workspace
            workspace.activate();
            
            workspace.clone()
        };
        
        // Update the active workspace ID
        {
            let mut active_workspace_id = self.active_workspace_id.write().unwrap();
            *active_workspace_id = Some(workspace_id);
        }
        
        // Update the workspace
        workspace = self.update_workspace(workspace).await?;
        
        self.publish_event(WorkspaceEvent::ActiveWorkspaceChanged { workspace_id });
        
        Ok(workspace)
    }

    async fn get_active_workspace(&self) -> DomainResult<Workspace> {
        let active_id = {
            let active_id = self.active_workspace_id.read().unwrap().clone();
            active_id.ok_or_else(|| WorkspaceError::Generic("No active workspace".to_string()))?
        };
        
        self.get_workspace(active_id).await
    }

    async fn assign_window(
        &self,
        window_id: WindowId,
        workspace_id: WorkspaceId,
        permanent: bool,
    ) -> DomainResult<WindowAssignment> {
        // Verify the workspace exists
        let mut workspace = self.get_workspace(workspace_id).await?;
        
        // Create the assignment
        let assignment = if permanent {
            WindowAssignment::permanent(window_id, workspace_id)
        } else {
            WindowAssignment::temporary(window_id, workspace_id)
        };
        
        // Remove the window from any existing workspace
        if let Some(old_assignment) = {
            let window_assignments = self.window_assignments.read().unwrap();
            window_assignments.get(&window_id).cloned()
        } {
            if let Ok(mut old_workspace) = self.get_workspace(old_assignment.workspace_id).await {
                old_workspace.remove_window(&window_id);
                self.update_workspace(old_workspace).await?;
                
                self.publish_event(WorkspaceEvent::WindowRemoved {
                    workspace_id: old_assignment.workspace_id,
                    window_id,
                });
            }
        }
        
        // Add the window to the new workspace
        workspace.add_window(window_id);
        self.update_workspace(workspace).await?;
        
        // Store the assignment
        {
            let mut window_assignments = self.window_assignments.write().unwrap();
            window_assignments.insert(window_id, assignment.clone());
        }
        
        self.publish_event(WorkspaceEvent::WindowAssigned {
            workspace_id,
            window_id,
        });
        
        Ok(assignment)
    }

    async fn get_window_workspace(&self, window_id: WindowId) -> DomainResult<Workspace> {
        let workspace_id = {
            let window_assignments = self.window_assignments.read().unwrap();
            
            let assignment = window_assignments
                .get(&window_id)
                .ok_or_else(|| WorkspaceError::Generic(format!("Window {} is not assigned to any workspace", window_id)))?;
            
            assignment.workspace_id
        };
        
        self.get_workspace(workspace_id).await
    }

    async fn create_rule(
        &self,
        name: String,
        priority: i32,
        pattern: WindowPattern,
        target_workspace_id: WorkspaceId,
    ) -> DomainResult<AssignmentRule> {
        // Verify the workspace exists
        self.get_workspace(target_workspace_id).await?;
        
        let rule = AssignmentRule::new(name, priority, pattern, target_workspace_id);
        rule.validate()?;
        
        let rule_id = rule.id();
        
        {
            let mut rules = self.rules.write().unwrap();
            rules.insert(rule_id, rule.clone());
        }
        
        Ok(rule)
    }

    async fn get_rule(&self, rule_id: EntityId) -> DomainResult<AssignmentRule> {
        let rules = self.rules.read().unwrap();
        
        rules
            .get(&rule_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::Generic(format!("Rule {} not found", rule_id)).into())
    }

    async fn get_all_rules(&self) -> DomainResult<Vec<AssignmentRule>> {
        let rules = self.rules.read().unwrap();
        
        let mut result: Vec<AssignmentRule> = rules.values().cloned().collect();
        result.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        Ok(result)
    }

    async fn update_rule(&self, rule: AssignmentRule) -> DomainResult<AssignmentRule> {
        rule.validate()?;
        
        // Verify the target workspace exists
        self.get_workspace(rule.target_workspace_id()).await?;
        
        let rule_id = rule.id();
        
        {
            let mut rules = self.rules.write().unwrap();
            
            if !rules.contains_key(&rule_id) {
                return Err(WorkspaceError::Generic(format!("Rule {} not found", rule_id)).into());
            }
            
            rules.insert(rule_id, rule.clone());
        }
        
        Ok(rule)
    }

    async fn delete_rule(&self, rule_id: EntityId) -> DomainResult<()> {
        {
            let mut rules = self.rules.write().unwrap();
            
            if !rules.contains_key(&rule_id) {
                return Err(WorkspaceError::Generic(format!("Rule {} not found", rule_id)).into());
            }
            
            rules.remove(&rule_id);
        }
        
        Ok(())
    }

    async fn apply_rules(
        &self,
        window_id: WindowId,
        properties: WindowProperties,
    ) -> DomainResult<Option<WindowAssignment>> {
        let rules = self.get_all_rules().await?;
        
        // Find the highest priority rule that matches
        for rule in rules {
            if !rule.is_enabled() {
                continue;
            }
            
            if properties.matches(rule.pattern()) {
                let workspace_id = rule.target_workspace_id();
                let rule_id = rule.id();
                
                let assignment = WindowAssignment::from_rule(window_id, workspace_id, rule_id);
                
                // Assign the window to the workspace
                self.assign_window(window_id, workspace_id, false).await?;
                
                return Ok(Some(assignment));
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct TestContext {
        manager: DefaultWorkspaceManager,
        events: Arc<Mutex<Vec<WorkspaceEvent>>>,
    }

    impl TestContext {
        fn new() -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = events.clone();
            
            let manager = DefaultWorkspaceManager::new(move |event| {
                let mut events = events_clone.lock().unwrap();
                events.push(event.payload);
            });
            
            TestContext { manager, events }
        }
        
        fn get_events(&self) -> Vec<WorkspaceEvent> {
            let events = self.events.lock().unwrap();
            events.clone()
        }
        
        fn clear_events(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }
    }

    #[tokio::test]
    async fn test_create_workspace() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        
        assert_eq!(workspace.name(), "Test");
        assert_eq!(workspace.workspace_type(), WorkspaceType::Standard);
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            WorkspaceEvent::WorkspaceCreated { workspace_id, name } => {
                assert_eq!(*workspace_id, workspace.id());
                assert_eq!(name, "Test");
            },
            _ => panic!("Expected WorkspaceCreated event"),
        }
    }

    #[tokio::test]
    async fn test_get_workspace() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        
        let retrieved = ctx.manager.get_workspace(workspace_id).await.unwrap();
        
        assert_eq!(retrieved.id(), workspace_id);
        assert_eq!(retrieved.name(), "Test");
    }

    #[tokio::test]
    async fn test_get_nonexistent_workspace() {
        let ctx = TestContext::new();
        
        let result = ctx.manager.get_workspace(WorkspaceId::new()).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_workspaces() {
        let ctx = TestContext::new();
        
        let workspace1 = ctx.manager.create_workspace("Test 1".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace2 = ctx.manager.create_workspace("Test 2".to_string(), WorkspaceType::Specialized).await.unwrap();
        
        let workspaces = ctx.manager.get_all_workspaces().await.unwrap();
        
        assert_eq!(workspaces.len(), 2);
        assert!(workspaces.iter().any(|w| w.id() == workspace1.id()));
        assert!(workspaces.iter().any(|w| w.id() == workspace2.id()));
    }

    #[tokio::test]
    async fn test_update_workspace() {
        let ctx = TestContext::new();
        
        let mut workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        ctx.clear_events();
        
        workspace.set_name("Updated");
        
        let updated = ctx.manager.update_workspace(workspace.clone()).await.unwrap();
        
        assert_eq!(updated.name(), "Updated");
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            WorkspaceEvent::WorkspaceUpdated { workspace_id, name } => {
                assert_eq!(*workspace_id, workspace.id());
                assert_eq!(name, "Updated");
            },
            _ => panic!("Expected WorkspaceUpdated event"),
        }
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        ctx.clear_events();
        
        ctx.manager.delete_workspace(workspace_id).await.unwrap();
        
        let result = ctx.manager.get_workspace(workspace_id).await;
        assert!(result.is_err());
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            WorkspaceEvent::WorkspaceDeleted { workspace_id: deleted_id } => {
                assert_eq!(*deleted_id, workspace_id);
            },
            _ => panic!("Expected WorkspaceDeleted event"),
        }
    }

    #[tokio::test]
    async fn test_activate_workspace() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        ctx.clear_events();
        
        let activated = ctx.manager.activate_workspace(workspace_id).await.unwrap();
        
        assert_eq!(activated.state(), WorkspaceState::Active);
        
        let active = ctx.manager.get_active_workspace().await.unwrap();
        assert_eq!(active.id(), workspace_id);
        
        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, WorkspaceEvent::ActiveWorkspaceChanged { workspace_id: id } if *id == workspace_id)));
    }

    #[tokio::test]
    async fn test_assign_window() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        let window_id = WindowId::new();
        ctx.clear_events();
        
        let assignment = ctx.manager.assign_window(window_id, workspace_id, false).await.unwrap();
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert!(!assignment.permanent);
        
        let workspace = ctx.manager.get_workspace(workspace_id).await.unwrap();
        assert!(workspace.contains_window(&window_id));
        
        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, WorkspaceEvent::WindowAssigned { workspace_id: id, window_id: wid } if *id == workspace_id && *wid == window_id)));
    }

    #[tokio::test]
    async fn test_get_window_workspace() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        let window_id = WindowId::new();
        
        ctx.manager.assign_window(window_id, workspace_id, false).await.unwrap();
        
        let window_workspace = ctx.manager.get_window_workspace(window_id).await.unwrap();
        
        assert_eq!(window_workspace.id(), workspace_id);
    }

    #[tokio::test]
    async fn test_create_and_apply_rule() {
        let ctx = TestContext::new();
        
        let workspace = ctx.manager.create_workspace("Test".to_string(), WorkspaceType::Standard).await.unwrap();
        let workspace_id = workspace.id();
        
        let pattern = WindowPattern::new().with_app_name("test-app");
        
        let rule = ctx.manager.create_rule(
            "Test Rule".to_string(),
            10,
            pattern,
            workspace_id,
        ).await.unwrap();
        
        assert_eq!(rule.name(), "Test Rule");
        assert_eq!(rule.priority(), 10);
        assert_eq!(rule.target_workspace_id(), workspace_id);
        
        let window_id = WindowId::new();
        let properties = WindowProperties::new().with_app_name("test-app");
        
        let assignment = ctx.manager.apply_rules(window_id, properties).await.unwrap();
        
        assert!(assignment.is_some());
        let assignment = assignment.unwrap();
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert_eq!(assignment.rule_id, Some(rule.id()));
        
        let window_workspace = ctx.manager.get_window_workspace(window_id).await.unwrap();
        assert_eq!(window_workspace.id(), workspace_id);
    }
}
