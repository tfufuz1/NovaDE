//! Project entity for the NovaDE domain layer.
//!
//! This module provides the Project entity, which represents a group
//! of related tasks in the NovaDE desktop environment.

use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entities::value_objects::{Timestamp, Status};

/// Represents a project in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier for the project
    project_id: String,
    /// The project name
    name: String,
    /// The project description
    description: Option<String>,
    /// The project status
    status: Status,
    /// The project creation timestamp
    created_at: Timestamp,
    /// The project due date, if any
    due_date: Option<Timestamp>,
    /// The project completion timestamp, if completed
    completed_at: Option<Timestamp>,
    /// Tags associated with the project
    tags: Vec<String>,
}

impl Project {
    /// Creates a new project with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The project name
    ///
    /// # Returns
    ///
    /// A new `Project` with a generated UUID and default values.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            project_id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            status: Status::Open,
            created_at: Timestamp::now(),
            due_date: None,
            completed_at: None,
            tags: Vec::new(),
        }
    }
    
    /// Creates a new project with the given ID and name.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project's unique identifier
    /// * `name` - The project name
    ///
    /// # Returns
    ///
    /// A new `Project` with the specified ID and default values.
    pub fn with_id<S1: Into<String>, S2: Into<String>>(project_id: S1, name: S2) -> Self {
        Self {
            project_id: project_id.into(),
            name: name.into(),
            description: None,
            status: Status::Open,
            created_at: Timestamp::now(),
            due_date: None,
            completed_at: None,
            tags: Vec::new(),
        }
    }
    
    /// Returns the project ID.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
    
    /// Returns the project name.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Sets the project name.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
    }
    
    /// Returns the project description, if any.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// Sets the project description.
    pub fn set_description<S: Into<String>>(&mut self, description: Option<S>) {
        self.description = description.map(|d| d.into());
    }
    
    /// Returns the project status.
    pub fn status(&self) -> Status {
        self.status
    }
    
    /// Sets the project status.
    pub fn set_status(&mut self, status: Status) {
        // If transitioning to a completed state, set the completion timestamp
        if status.is_completed() && !self.status.is_completed() {
            self.completed_at = Some(Timestamp::now());
        } else if !status.is_completed() {
            // If transitioning from completed to non-completed, clear the completion timestamp
            self.completed_at = None;
        }
        
        self.status = status;
    }
    
    /// Returns the project creation timestamp.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
    
    /// Returns the project due date, if any.
    pub fn due_date(&self) -> Option<Timestamp> {
        self.due_date
    }
    
    /// Sets the project due date.
    pub fn set_due_date(&mut self, due_date: Option<Timestamp>) {
        self.due_date = due_date;
    }
    
    /// Returns the project completion timestamp, if completed.
    pub fn completed_at(&self) -> Option<Timestamp> {
        self.completed_at
    }
    
    /// Returns whether the project is completed.
    pub fn is_completed(&self) -> bool {
        self.status.is_completed()
    }
    
    /// Marks the project as completed.
    pub fn complete(&mut self) {
        self.set_status(Status::Completed);
    }
    
    /// Returns whether the project is overdue.
    pub fn is_overdue(&self) -> bool {
        if self.is_completed() {
            return false;
        }
        
        if let Some(due_date) = self.due_date {
            return due_date.is_past();
        }
        
        false
    }
    
    /// Returns the tags associated with the project.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    
    /// Adds a tag to the project.
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Removes a tag from the project.
    pub fn remove_tag<S: AsRef<str>>(&mut self, tag: S) {
        let tag = tag.as_ref();
        self.tags.retain(|t| t != tag);
    }
    
    /// Clears all tags from the project.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Project({}, {})", self.project_id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use chrono::Duration;
    
    #[test]
    fn test_project_new() {
        let project = Project::new("Test Project");
        
        assert!(!project.project_id().is_empty());
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.description(), None);
        assert_eq!(project.status(), Status::Open);
        assert_eq!(project.due_date(), None);
        assert_eq!(project.completed_at(), None);
        assert!(project.tags().is_empty());
    }
    
    #[test]
    fn test_project_with_id() {
        let project = Project::with_id("project123", "Test Project");
        
        assert_eq!(project.project_id(), "project123");
        assert_eq!(project.name(), "Test Project");
    }
    
    #[test]
    fn test_project_setters() {
        let mut project = Project::new("Test Project");
        
        project.set_name("Updated Project");
        assert_eq!(project.name(), "Updated Project");
        
        project.set_description(Some("This is a test project"));
        assert_eq!(project.description(), Some("This is a test project"));
        
        project.set_status(Status::InProgress);
        assert_eq!(project.status(), Status::InProgress);
        
        let due_date = Timestamp::now();
        project.set_due_date(Some(due_date));
        assert_eq!(project.due_date(), Some(due_date));
    }
    
    #[test]
    fn test_project_completion() {
        let mut project = Project::new("Test Project");
        assert_eq!(project.is_completed(), false);
        assert_eq!(project.completed_at(), None);
        
        project.complete();
        assert_eq!(project.is_completed(), true);
        assert_eq!(project.status(), Status::Completed);
        assert!(project.completed_at().is_some());
        
        // Test that setting status to non-completed clears the completion timestamp
        project.set_status(Status::InProgress);
        assert_eq!(project.is_completed(), false);
        assert_eq!(project.completed_at(), None);
    }
    
    #[test]
    fn test_project_tags() {
        let mut project = Project::new("Test Project");
        assert!(project.tags().is_empty());
        
        project.add_tag("important");
        project.add_tag("work");
        assert_eq!(project.tags(), &["important", "work"]);
        
        // Adding the same tag again should have no effect
        project.add_tag("important");
        assert_eq!(project.tags(), &["important", "work"]);
        
        project.remove_tag("important");
        assert_eq!(project.tags(), &["work"]);
        
        project.clear_tags();
        assert!(project.tags().is_empty());
    }
    
    #[test]
    fn test_project_overdue() {
        let mut project = Project::new("Test Project");
        assert_eq!(project.is_overdue(), false);
        
        // Set a due date in the past
        let past_due = Timestamp::new(Utc::now() - Duration::hours(1));
        project.set_due_date(Some(past_due));
        assert_eq!(project.is_overdue(), true);
        
        // Set a due date in the future
        let future_due = Timestamp::new(Utc::now() + Duration::hours(1));
        project.set_due_date(Some(future_due));
        assert_eq!(project.is_overdue(), false);
        
        // Completed projects are never overdue
        project.set_due_date(Some(past_due));
        project.complete();
        assert_eq!(project.is_overdue(), false);
    }
}
