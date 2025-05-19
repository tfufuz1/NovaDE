//! Task entity for the NovaDE domain layer.
//!
//! This module provides the Task entity, which represents a task
//! to be completed in the NovaDE desktop environment.

use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entities::value_objects::{Timestamp, Status, Priority};

/// Represents a task in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    task_id: String,
    /// The task title
    title: String,
    /// The task description
    description: Option<String>,
    /// The project ID this task belongs to, if any
    project_id: Option<String>,
    /// The task status
    status: Status,
    /// The task priority
    priority: Priority,
    /// The task creation timestamp
    created_at: Timestamp,
    /// The task due date, if any
    due_date: Option<Timestamp>,
    /// The task completion timestamp, if completed
    completed_at: Option<Timestamp>,
    /// Tags associated with the task
    tags: Vec<String>,
}

impl Task {
    /// Creates a new task with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The task title
    ///
    /// # Returns
    ///
    /// A new `Task` with a generated UUID and default values.
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: None,
            project_id: None,
            status: Status::Open,
            priority: Priority::Medium,
            created_at: Timestamp::now(),
            due_date: None,
            completed_at: None,
            tags: Vec::new(),
        }
    }
    
    /// Creates a new task with the given ID and title.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task's unique identifier
    /// * `title` - The task title
    ///
    /// # Returns
    ///
    /// A new `Task` with the specified ID and default values.
    pub fn with_id<S1: Into<String>, S2: Into<String>>(task_id: S1, title: S2) -> Self {
        Self {
            task_id: task_id.into(),
            title: title.into(),
            description: None,
            project_id: None,
            status: Status::Open,
            priority: Priority::Medium,
            created_at: Timestamp::now(),
            due_date: None,
            completed_at: None,
            tags: Vec::new(),
        }
    }
    
    /// Returns the task ID.
    pub fn task_id(&self) -> &str {
        &self.task_id
    }
    
    /// Returns the task title.
    pub fn title(&self) -> &str {
        &self.title
    }
    
    /// Sets the task title.
    pub fn set_title<S: Into<String>>(&mut self, title: S) {
        self.title = title.into();
    }
    
    /// Returns the task description, if any.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// Sets the task description.
    pub fn set_description<S: Into<String>>(&mut self, description: Option<S>) {
        self.description = description.map(|d| d.into());
    }
    
    /// Returns the project ID this task belongs to, if any.
    pub fn project_id(&self) -> Option<&str> {
        self.project_id.as_deref()
    }
    
    /// Sets the project ID this task belongs to.
    pub fn set_project_id<S: Into<String>>(&mut self, project_id: Option<S>) {
        self.project_id = project_id.map(|p| p.into());
    }
    
    /// Returns the task status.
    pub fn status(&self) -> Status {
        self.status
    }
    
    /// Sets the task status.
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
    
    /// Returns the task priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }
    
    /// Sets the task priority.
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }
    
    /// Returns the task creation timestamp.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
    
    /// Returns the task due date, if any.
    pub fn due_date(&self) -> Option<Timestamp> {
        self.due_date
    }
    
    /// Sets the task due date.
    pub fn set_due_date(&mut self, due_date: Option<Timestamp>) {
        self.due_date = due_date;
    }
    
    /// Returns the task completion timestamp, if completed.
    pub fn completed_at(&self) -> Option<Timestamp> {
        self.completed_at
    }
    
    /// Returns whether the task is completed.
    pub fn is_completed(&self) -> bool {
        self.status.is_completed()
    }
    
    /// Marks the task as completed.
    pub fn complete(&mut self) {
        self.set_status(Status::Completed);
    }
    
    /// Returns whether the task is overdue.
    pub fn is_overdue(&self) -> bool {
        if self.is_completed() {
            return false;
        }
        
        if let Some(due_date) = self.due_date {
            return due_date.is_past();
        }
        
        false
    }
    
    /// Returns the tags associated with the task.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    
    /// Adds a tag to the task.
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Removes a tag from the task.
    pub fn remove_tag<S: AsRef<str>>(&mut self, tag: S) {
        let tag = tag.as_ref();
        self.tags.retain(|t| t != tag);
    }
    
    /// Clears all tags from the task.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task({}, {})", self.task_id, self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_new() {
        let task = Task::new("Test Task");
        
        assert!(!task.task_id().is_empty());
        assert_eq!(task.title(), "Test Task");
        assert_eq!(task.description(), None);
        assert_eq!(task.project_id(), None);
        assert_eq!(task.status(), Status::Open);
        assert_eq!(task.priority(), Priority::Medium);
        assert_eq!(task.due_date(), None);
        assert_eq!(task.completed_at(), None);
        assert!(task.tags().is_empty());
    }
    
    #[test]
    fn test_task_with_id() {
        let task = Task::with_id("task123", "Test Task");
        
        assert_eq!(task.task_id(), "task123");
        assert_eq!(task.title(), "Test Task");
    }
    
    #[test]
    fn test_task_setters() {
        let mut task = Task::new("Test Task");
        
        task.set_title("Updated Task");
        assert_eq!(task.title(), "Updated Task");
        
        task.set_description(Some("This is a test task"));
        assert_eq!(task.description(), Some("This is a test task"));
        
        task.set_project_id(Some("project123"));
        assert_eq!(task.project_id(), Some("project123"));
        
        task.set_status(Status::InProgress);
        assert_eq!(task.status(), Status::InProgress);
        
        task.set_priority(Priority::High);
        assert_eq!(task.priority(), Priority::High);
        
        let due_date = Timestamp::now();
        task.set_due_date(Some(due_date));
        assert_eq!(task.due_date(), Some(due_date));
    }
    
    #[test]
    fn test_task_completion() {
        let mut task = Task::new("Test Task");
        assert_eq!(task.is_completed(), false);
        assert_eq!(task.completed_at(), None);
        
        task.complete();
        assert_eq!(task.is_completed(), true);
        assert_eq!(task.status(), Status::Completed);
        assert!(task.completed_at().is_some());
        
        // Test that setting status to non-completed clears the completion timestamp
        task.set_status(Status::InProgress);
        assert_eq!(task.is_completed(), false);
        assert_eq!(task.completed_at(), None);
    }
    
    #[test]
    fn test_task_tags() {
        let mut task = Task::new("Test Task");
        assert!(task.tags().is_empty());
        
        task.add_tag("important");
        task.add_tag("work");
        assert_eq!(task.tags(), &["important", "work"]);
        
        // Adding the same tag again should have no effect
        task.add_tag("important");
        assert_eq!(task.tags(), &["important", "work"]);
        
        task.remove_tag("important");
        assert_eq!(task.tags(), &["work"]);
        
        task.clear_tags();
        assert!(task.tags().is_empty());
    }
    
    #[test]
    fn test_task_overdue() {
        use chrono::Duration;
        
        let mut task = Task::new("Test Task");
        assert_eq!(task.is_overdue(), false);
        
        // Set a due date in the past
        let past_due = Timestamp::new(Utc::now() - Duration::hours(1));
        task.set_due_date(Some(past_due));
        assert_eq!(task.is_overdue(), true);
        
        // Set a due date in the future
        let future_due = Timestamp::new(Utc::now() + Duration::hours(1));
        task.set_due_date(Some(future_due));
        assert_eq!(task.is_overdue(), false);
        
        // Completed tasks are never overdue
        task.set_due_date(Some(past_due));
        task.complete();
        assert_eq!(task.is_overdue(), false);
    }
}
