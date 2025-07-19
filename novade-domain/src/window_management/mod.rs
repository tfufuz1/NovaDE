//! Window management module for the NovaDE domain layer.
//!
//! This module provides window management functionality for the NovaDE desktop environment,
//! defining policies and rules for window behavior.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::error::{DomainError, WindowManagementError};
use crate::entities::value_objects::Timestamp;
use crate::types::geometry::{Point, Size, Rect};

/// Specifies the decoration mode for a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DecorationMode {
    /// The client is responsible for drawing its own decorations.
    ClientSide,
    /// The server (compositor) is responsible for drawing the decorations.
    #[default]
    ServerSide,
    /// The compositor decides based on internal policies or client hints.
    Auto,
}

/// Represents the state of a window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    /// Window is normal (neither minimized nor maximized)
    Normal,
    /// Window is minimized
    Minimized,
    /// Window is maximized
    Maximized,
    /// Window is fullscreen
    Fullscreen,
    /// Window is tiled
    Tiled,
}

/// Represents the type of a window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowType {
    /// Normal application window
    Normal,
    /// Dialog window
    Dialog,
    /// Utility window (e.g., toolbars, palettes)
    Utility,
    /// Splash screen
    Splash,
    /// Menu (e.g., dropdown menu)
    Menu,
    /// Popup window (e.g., tooltips)
    Popup,
    /// Notification window
    Notification,
}

/// Represents a window in the window management system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    /// Unique identifier for the window
    window_id: String,
    /// The window title
    title: String,
    /// The window application ID
    app_id: String,
    /// The window type
    window_type: WindowType,
    /// The window state
    state: WindowState,
    /// The window geometry
    geometry: Rect,
    /// The window minimum size
    min_size: Option<Size>,
    /// The window maximum size
    max_size: Option<Size>,
    /// Whether the window is resizable
    resizable: bool,
    /// Whether the window is movable
    movable: bool,
    /// Whether the window is closable
    closable: bool,
    /// Whether the window is focused
    focused: bool,
    /// The window creation timestamp
    created_at: Timestamp,
    /// The window last focused timestamp
    last_focused_at: Option<Timestamp>,
    /// Additional properties for the window
    properties: HashMap<String, String>,
}

impl Window {
    pub fn new(window_id: String, title: String, app_id: String, window_type: WindowType) -> Self {
        Self {
            window_id,
            title,
            app_id,
            window_type,
            state: WindowState::Normal,
            geometry: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 0.0,
                    height: 0.0,
                },
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
        }
    }
}

/// Represents a window management policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPolicy {
    /// Unique identifier for the policy
    policy_id: String,
    /// The policy name
    name: String,
    /// The policy description
    description: String,
    /// The policy priority (higher values take precedence)
    priority: i32,
    /// The window types this policy applies to
    applies_to: Vec<WindowType>,
    /// Whether new windows should be focused automatically
    auto_focus: bool,
    /// Whether windows can be moved between workspaces
    allow_workspace_movement: bool,
    /// Whether windows can be maximized
    allow_maximize: bool,
    /// Whether windows can be minimized
    allow_minimize: bool,
    /// Whether windows can be fullscreened
    allow_fullscreen: bool,
    /// Whether windows can be resized
    allow_resize: bool,
    /// Whether windows can be closed
    allow_close: bool,
    /// The preferred decoration mode for windows matching this policy.
    decoration_mode: DecorationMode,
    /// The policy creation timestamp
    created_at: Timestamp,
    /// The policy last modified timestamp
    modified_at: Timestamp,
}

/// Interface for the window policy manager.
#[async_trait]
pub trait WindowPolicyManager: Send + Sync {
    /// Creates a new window policy.
    ///
    /// # Arguments
    ///
    /// * `name` - The policy name
    /// * `description` - The policy description
    /// * `priority` - The policy priority
    /// * `applies_to` - The window types this policy applies to
    /// * `auto_focus` - Whether new windows should be focused automatically
    /// * `allow_workspace_movement` - Whether windows can be moved between workspaces
    /// * `allow_maximize` - Whether windows can be maximized
    /// * `allow_minimize` - Whether windows can be minimized
    /// * `allow_fullscreen` - Whether windows can be fullscreened
    /// * `allow_resize` - Whether windows can be resized
    /// * `allow_close` - Whether windows can be closed
    ///
    /// # Returns
    ///
    /// A `Result` containing the created policy ID.
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
        decoration_mode: DecorationMode,
    ) -> Result<String, DomainError>;
    
    /// Gets a window policy by ID.
    ///
    /// # Arguments
    ///
    /// * `policy_id` - The policy ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the policy if found.
    async fn get_policy(&self, policy_id: &str) -> Result<WindowPolicy, DomainError>;
    
    /// Updates a window policy.
    ///
    /// # Arguments
    ///
    /// * `policy_id` - The policy ID
    /// * `name` - The policy name
    /// * `description` - The policy description
    /// * `priority` - The policy priority
    /// * `applies_to` - The window types this policy applies to
    /// * `auto_focus` - Whether new windows should be focused automatically
    /// * `allow_workspace_movement` - Whether windows can be moved between workspaces
    /// * `allow_maximize` - Whether windows can be maximized
    /// * `allow_minimize` - Whether windows can be minimized
    /// * `allow_fullscreen` - Whether windows can be fullscreened
    /// * `allow_resize` - Whether windows can be resized
    /// * `allow_close` - Whether windows can be closed
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
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
        decoration_mode: DecorationMode,
    ) -> Result<(), DomainError>;
    
    /// Deletes a window policy.
    ///
    /// # Arguments
    ///
    /// * `policy_id` - The policy ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete_policy(&self, policy_id: &str) -> Result<(), DomainError>;
    
    /// Lists all window policies.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all policies.
    async fn list_policies(&self) -> Result<Vec<WindowPolicy>, DomainError>;
    
    /// Gets the effective policy for a window type.
    ///
    /// # Arguments
    ///
    /// * `window_type` - The window type
    ///
    /// # Returns
    ///
    /// A `Result` containing the effective policy for the window type.
    async fn get_effective_policy(&self, window_type: WindowType) -> Result<WindowPolicy, DomainError>;
    
    /// Checks if an action is allowed for a window.
    ///
    /// # Arguments
    ///
    /// * `window` - The window
    /// * `action` - The action to check
    ///
    /// # Returns
    ///
    /// A `Result` containing true if the action is allowed, false otherwise.
    async fn is_action_allowed(&self, window: &Window, action: WindowAction) -> Result<bool, DomainError>;

    /// Gets the decoration mode for a window.
    ///
    /// # Arguments
    ///
    /// * `window` - The window
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoration mode for the window.
    async fn get_decoration_mode_for_window(&self, window: &Window) -> Result<DecorationMode, DomainError>;
}

/// Represents a window action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowAction {
    /// Move the window
    Move,
    /// Resize the window
    Resize,
    /// Maximize the window
    Maximize,
    /// Minimize the window
    Minimize,
    /// Fullscreen the window
    Fullscreen,
    /// Close the window
    Close,
    /// Move the window to another workspace
    MoveToWorkspace,
}

mod default_policy_manager;

pub use default_policy_manager::DefaultWindowPolicyManager;
