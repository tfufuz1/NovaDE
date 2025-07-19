// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Window Manager UI Module
//!
//! This module provides the UI-side window management functionality,
//! connecting the UI to the domain-level window management services.

use std::sync::Arc;
use novade_domain::window_management::{
    WindowPolicyManager,
    WindowAction,
    Window,
    WindowState,
    WindowType,
};
use novade_core::types::geometry::{Rect, Point, Size};
use std::collections::HashMap;
use novade_core::error::CoreError;
use crate::error::UiError;

pub struct WindowManager {
    policy_manager: Arc<dyn WindowPolicyManager>,
}

impl WindowManager {
    pub fn new(policy_manager: Arc<dyn WindowPolicyManager>) -> Self {
        Self { policy_manager }
    }

    pub async fn minimize_window(&self, window: &Window) -> Result<(), UiError> {
        if self.policy_manager.is_action_allowed(window, WindowAction::Minimize).await? {
            // In a real implementation, this would dispatch an event to the system layer
            // to actually minimize the window. For now, we'll just log it.
            println!("Minimizing window: {}", window.title);
            Ok(())
        } else {
            Err(UiError::ActionNotAllowed("Minimize".to_string()))
        }
    }

    pub async fn maximize_window(&self, window: &Window) -> Result<(), UiError> {
        if self.policy_manager.is_action_allowed(window, WindowAction::Maximize).await? {
            // In a real implementation, this would dispatch an event to the system layer
            // to actually maximize the window. For now, we'll just log it.
            println!("Maximizing window: {}", window.title);
            Ok(())
        } else {
            Err(UiError::ActionNotAllowed("Maximize".to_string()))
        }
    }

    pub async fn close_window(&self, window: &Window) -> Result<(), UiError> {
        if self.policy_manager.is_action_allowed(window, WindowAction::Close).await? {
            // In a real implementation, this would dispatch an event to the system layer
            // to actually close the window. For now, we'll just log it.
            println!("Closing window: {}", window.title);
            Ok(())
        } else {
            Err(UiError::ActionNotAllowed("Close".to_string()))
        }
    }
}

// Dummy window for testing purposes
pub fn create_dummy_window() -> Window {
    Window {
        window_id: "dummy_window".to_string(),
        title: "Dummy Window".to_string(),
        app_id: "dummy.app".to_string(),
        window_type: WindowType::Normal,
        state: WindowState::Normal,
        geometry: Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size { width: 800.0, height: 600.0 },
        },
        min_size: None,
        max_size: None,
        resizable: true,
        movable: true,
        closable: true,
        focused: true,
        created_at: novade_domain::entities::value_objects::Timestamp::now(),
        last_focused_at: None,
        properties: HashMap::new(),
    }
}

impl From<novade_domain::error::DomainError> for UiError {
    fn from(error: novade_domain::error::DomainError) -> Self {
        UiError::DomainError(error.to_string())
    }
}
