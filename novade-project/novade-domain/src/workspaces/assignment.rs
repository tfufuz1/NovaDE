//! Window assignment module for the NovaDE domain layer.
//!
//! This module provides functionality for assigning windows to workspaces
//! based on rules and policies.

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::shared_types::{EntityId, Version, Identifiable, Versionable};
use crate::workspaces::core::{WindowId, WorkspaceId};
use crate::error::{DomainResult, WorkspaceError};

/// A rule for assigning windows to workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRule {
    /// The unique identifier of the rule.
    id: EntityId,
    /// The name of the rule.
    name: String,
    /// The priority of the rule (higher values have higher priority).
    priority: i32,
    /// The pattern to match against window properties.
    pattern: WindowPattern,
    /// The ID of the target workspace.
    target_workspace_id: WorkspaceId,
    /// Whether the rule is enabled.
    enabled: bool,
    /// The version of the rule.
    version: Version,
}

impl AssignmentRule {
    /// Creates a new assignment rule.
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
    /// A new `AssignmentRule` with the specified properties.
    pub fn new(
        name: impl Into<String>,
        priority: i32,
        pattern: WindowPattern,
        target_workspace_id: WorkspaceId,
    ) -> Self {
        AssignmentRule {
            id: EntityId::new(),
            name: name.into(),
            priority,
            pattern,
            target_workspace_id,
            enabled: true,
            version: Version::initial(),
        }
    }

    /// Gets the name of the rule.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the name of the rule.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name of the rule
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.increment_version();
    }

    /// Gets the priority of the rule.
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Sets the priority of the rule.
    ///
    /// # Arguments
    ///
    /// * `priority` - The new priority of the rule
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
        self.increment_version();
    }

    /// Gets the pattern of the rule.
    pub fn pattern(&self) -> &WindowPattern {
        &self.pattern
    }

    /// Sets the pattern of the rule.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The new pattern of the rule
    pub fn set_pattern(&mut self, pattern: WindowPattern) {
        self.pattern = pattern;
        self.increment_version();
    }

    /// Gets the target workspace ID of the rule.
    pub fn target_workspace_id(&self) -> WorkspaceId {
        self.target_workspace_id
    }

    /// Sets the target workspace ID of the rule.
    ///
    /// # Arguments
    ///
    /// * `target_workspace_id` - The new target workspace ID of the rule
    pub fn set_target_workspace_id(&mut self, target_workspace_id: WorkspaceId) {
        self.target_workspace_id = target_workspace_id;
        self.increment_version();
    }

    /// Checks if the rule is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables the rule.
    pub fn enable(&mut self) {
        if !self.enabled {
            self.enabled = true;
            self.increment_version();
        }
    }

    /// Disables the rule.
    pub fn disable(&mut self) {
        if self.enabled {
            self.enabled = false;
            self.increment_version();
        }
    }

    /// Validates the rule.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the rule is valid, or an error if it is invalid.
    pub fn validate(&self) -> DomainResult<()> {
        if self.name.is_empty() {
            return Err(WorkspaceError::Invalid("Rule name cannot be empty".to_string()).into());
        }
        Ok(())
    }
}

impl Identifiable for AssignmentRule {
    fn id(&self) -> EntityId {
        self.id
    }
}

impl Versionable for AssignmentRule {
    fn version(&self) -> Version {
        self.version
    }

    fn increment_version(&mut self) {
        self.version = self.version.next();
    }
}

impl fmt::Display for AssignmentRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AssignmentRule[{}] '{}' (priority: {}, enabled: {})",
            self.id, self.name, self.priority, self.enabled
        )
    }
}

/// A pattern to match against window properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPattern {
    /// The application name pattern.
    pub app_name: Option<String>,
    /// The window title pattern.
    pub title: Option<String>,
    /// The window class pattern.
    pub class: Option<String>,
    /// The window role pattern.
    pub role: Option<String>,
}

impl WindowPattern {
    /// Creates a new window pattern.
    ///
    /// # Returns
    ///
    /// A new empty `WindowPattern`.
    pub fn new() -> Self {
        WindowPattern {
            app_name: None,
            title: None,
            class: None,
            role: None,
        }
    }

    /// Sets the application name pattern.
    ///
    /// # Arguments
    ///
    /// * `app_name` - The application name pattern
    ///
    /// # Returns
    ///
    /// The modified `WindowPattern`.
    pub fn with_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    /// Sets the window title pattern.
    ///
    /// # Arguments
    ///
    /// * `title` - The window title pattern
    ///
    /// # Returns
    ///
    /// The modified `WindowPattern`.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the window class pattern.
    ///
    /// # Arguments
    ///
    /// * `class` - The window class pattern
    ///
    /// # Returns
    ///
    /// The modified `WindowPattern`.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Sets the window role pattern.
    ///
    /// # Arguments
    ///
    /// * `role` - The window role pattern
    ///
    /// # Returns
    ///
    /// The modified `WindowPattern`.
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Checks if the pattern is empty (no criteria specified).
    ///
    /// # Returns
    ///
    /// `true` if the pattern is empty, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.app_name.is_none() && self.title.is_none() && self.class.is_none() && self.role.is_none()
    }
}

impl Default for WindowPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WindowPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        
        if let Some(app_name) = &self.app_name {
            parts.push(format!("app_name: '{}'", app_name));
        }
        
        if let Some(title) = &self.title {
            parts.push(format!("title: '{}'", title));
        }
        
        if let Some(class) = &self.class {
            parts.push(format!("class: '{}'", class));
        }
        
        if let Some(role) = &self.role {
            parts.push(format!("role: '{}'", role));
        }
        
        if parts.is_empty() {
            write!(f, "WindowPattern[empty]")
        } else {
            write!(f, "WindowPattern[{}]", parts.join(", "))
        }
    }
}

/// A window assignment representing the assignment of a window to a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowAssignment {
    /// The ID of the window.
    pub window_id: WindowId,
    /// The ID of the workspace.
    pub workspace_id: WorkspaceId,
    /// The ID of the rule that created this assignment, if any.
    pub rule_id: Option<EntityId>,
    /// Whether the assignment is permanent.
    pub permanent: bool,
}

impl WindowAssignment {
    /// Creates a new window assignment.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `workspace_id` - The ID of the workspace
    /// * `rule_id` - The ID of the rule that created this assignment, if any
    /// * `permanent` - Whether the assignment is permanent
    ///
    /// # Returns
    ///
    /// A new `WindowAssignment` with the specified properties.
    pub fn new(
        window_id: WindowId,
        workspace_id: WorkspaceId,
        rule_id: Option<EntityId>,
        permanent: bool,
    ) -> Self {
        WindowAssignment {
            window_id,
            workspace_id,
            rule_id,
            permanent,
        }
    }

    /// Creates a new temporary window assignment.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// A new temporary `WindowAssignment`.
    pub fn temporary(window_id: WindowId, workspace_id: WorkspaceId) -> Self {
        WindowAssignment::new(window_id, workspace_id, None, false)
    }

    /// Creates a new permanent window assignment.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// A new permanent `WindowAssignment`.
    pub fn permanent(window_id: WindowId, workspace_id: WorkspaceId) -> Self {
        WindowAssignment::new(window_id, workspace_id, None, true)
    }

    /// Creates a new rule-based window assignment.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window
    /// * `workspace_id` - The ID of the workspace
    /// * `rule_id` - The ID of the rule that created this assignment
    ///
    /// # Returns
    ///
    /// A new rule-based `WindowAssignment`.
    pub fn from_rule(window_id: WindowId, workspace_id: WorkspaceId, rule_id: EntityId) -> Self {
        WindowAssignment::new(window_id, workspace_id, Some(rule_id), false)
    }
}

impl fmt::Display for WindowAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WindowAssignment[window: {}, workspace: {}, {}]",
            self.window_id,
            self.workspace_id,
            if self.permanent {
                "permanent"
            } else if self.rule_id.is_some() {
                "rule-based"
            } else {
                "temporary"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_rule_new() {
        let workspace_id = WorkspaceId::new();
        let pattern = WindowPattern::new().with_app_name("test-app");
        let rule = AssignmentRule::new("Test Rule", 10, pattern, workspace_id);
        
        assert_eq!(rule.name(), "Test Rule");
        assert_eq!(rule.priority(), 10);
        assert_eq!(rule.target_workspace_id(), workspace_id);
        assert!(rule.is_enabled());
        assert_eq!(rule.version(), Version::initial());
    }

    #[test]
    fn test_assignment_rule_setters() {
        let workspace_id = WorkspaceId::new();
        let pattern = WindowPattern::new().with_app_name("test-app");
        let mut rule = AssignmentRule::new("Test Rule", 10, pattern, workspace_id);
        
        rule.set_name("New Name");
        assert_eq!(rule.name(), "New Name");
        
        rule.set_priority(20);
        assert_eq!(rule.priority(), 20);
        
        let new_pattern = WindowPattern::new().with_title("test-title");
        rule.set_pattern(new_pattern.clone());
        assert_eq!(rule.pattern().title, new_pattern.title);
        
        let new_workspace_id = WorkspaceId::new();
        rule.set_target_workspace_id(new_workspace_id);
        assert_eq!(rule.target_workspace_id(), new_workspace_id);
    }

    #[test]
    fn test_assignment_rule_enable_disable() {
        let workspace_id = WorkspaceId::new();
        let pattern = WindowPattern::new().with_app_name("test-app");
        let mut rule = AssignmentRule::new("Test Rule", 10, pattern, workspace_id);
        
        assert!(rule.is_enabled());
        
        rule.disable();
        assert!(!rule.is_enabled());
        
        rule.enable();
        assert!(rule.is_enabled());
    }

    #[test]
    fn test_assignment_rule_validate() {
        let workspace_id = WorkspaceId::new();
        let pattern = WindowPattern::new().with_app_name("test-app");
        let rule = AssignmentRule::new("Test Rule", 10, pattern, workspace_id);
        
        assert!(rule.validate().is_ok());
        
        let mut invalid_rule = rule.clone();
        invalid_rule.set_name("");
        assert!(invalid_rule.validate().is_err());
    }

    #[test]
    fn test_window_pattern_new() {
        let pattern = WindowPattern::new();
        
        assert!(pattern.app_name.is_none());
        assert!(pattern.title.is_none());
        assert!(pattern.class.is_none());
        assert!(pattern.role.is_none());
        assert!(pattern.is_empty());
    }

    #[test]
    fn test_window_pattern_with_methods() {
        let pattern = WindowPattern::new()
            .with_app_name("test-app")
            .with_title("test-title")
            .with_class("test-class")
            .with_role("test-role");
        
        assert_eq!(pattern.app_name, Some("test-app".to_string()));
        assert_eq!(pattern.title, Some("test-title".to_string()));
        assert_eq!(pattern.class, Some("test-class".to_string()));
        assert_eq!(pattern.role, Some("test-role".to_string()));
        assert!(!pattern.is_empty());
    }

    #[test]
    fn test_window_pattern_display() {
        let pattern = WindowPattern::new()
            .with_app_name("test-app")
            .with_title("test-title");
        
        let display = format!("{}", pattern);
        assert!(display.contains("app_name: 'test-app'"));
        assert!(display.contains("title: 'test-title'"));
    }

    #[test]
    fn test_window_assignment_new() {
        let window_id = WindowId::new();
        let workspace_id = WorkspaceId::new();
        let rule_id = EntityId::new();
        
        let assignment = WindowAssignment::new(window_id, workspace_id, Some(rule_id), true);
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert_eq!(assignment.rule_id, Some(rule_id));
        assert!(assignment.permanent);
    }

    #[test]
    fn test_window_assignment_temporary() {
        let window_id = WindowId::new();
        let workspace_id = WorkspaceId::new();
        
        let assignment = WindowAssignment::temporary(window_id, workspace_id);
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert!(assignment.rule_id.is_none());
        assert!(!assignment.permanent);
    }

    #[test]
    fn test_window_assignment_permanent() {
        let window_id = WindowId::new();
        let workspace_id = WorkspaceId::new();
        
        let assignment = WindowAssignment::permanent(window_id, workspace_id);
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert!(assignment.rule_id.is_none());
        assert!(assignment.permanent);
    }

    #[test]
    fn test_window_assignment_from_rule() {
        let window_id = WindowId::new();
        let workspace_id = WorkspaceId::new();
        let rule_id = EntityId::new();
        
        let assignment = WindowAssignment::from_rule(window_id, workspace_id, rule_id);
        
        assert_eq!(assignment.window_id, window_id);
        assert_eq!(assignment.workspace_id, workspace_id);
        assert_eq!(assignment.rule_id, Some(rule_id));
        assert!(!assignment.permanent);
    }

    #[test]
    fn test_window_assignment_display() {
        let window_id = WindowId::new();
        let workspace_id = WorkspaceId::new();
        
        let temporary = WindowAssignment::temporary(window_id, workspace_id);
        let permanent = WindowAssignment::permanent(window_id, workspace_id);
        let rule_based = WindowAssignment::from_rule(window_id, workspace_id, EntityId::new());
        
        assert!(format!("{}", temporary).contains("temporary"));
        assert!(format!("{}", permanent).contains("permanent"));
        assert!(format!("{}", rule_based).contains("rule-based"));
    }
}
