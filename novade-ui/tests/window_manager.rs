// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use async_trait::async_trait;
use novade_domain::window_management::{
    WindowPolicyManager, WindowAction, Window, WindowState, WindowType, WindowPolicy,
};
use novade_core::types::geometry::{Rect, Point, Size};
use std::collections::HashMap;
use novade_domain::error::DomainError;
use novade_ui::window_manager::WindowManager;
use novade_domain::entities::value_objects::Timestamp;

struct MockWindowPolicyManager {
    allow_action: bool,
}

#[async_trait]
impl WindowPolicyManager for MockWindowPolicyManager {
    async fn create_policy(
        &self,
        _name: &str,
        _description: &str,
        _priority: i32,
        _applies_to: Vec<WindowType>,
        _auto_focus: bool,
        _allow_workspace_movement: bool,
        _allow_maximize: bool,
        _allow_minimize: bool,
        _allow_fullscreen: bool,
        _allow_resize: bool,
        _allow_close: bool,
    ) -> Result<String, DomainError> {
        Ok("mock_policy".to_string())
    }

    async fn get_policy(&self, _policy_id: &str) -> Result<WindowPolicy, DomainError> {
        unimplemented!()
    }

    async fn update_policy(
        &self,
        _policy_id: &str,
        _name: &str,
        _description: &str,
        _priority: i32,
        _applies_to: Vec<WindowType>,
        _auto_focus: bool,
        _allow_workspace_movement: bool,
        _allow_maximize: bool,
        _allow_minimize: bool,
        _allow_fullscreen: bool,
        _allow_resize: bool,
        _allow_close: bool,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete_policy(&self, _policy_id: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_policies(&self) -> Result<Vec<WindowPolicy>, DomainError> {
        Ok(vec![])
    }

    async fn get_effective_policy(&self, _window_type: WindowType) -> Result<WindowPolicy, DomainError> {
        unimplemented!()
    }

    async fn is_action_allowed(&self, _window: &Window, _action: WindowAction) -> Result<bool, DomainError> {
        Ok(self.allow_action)
    }
}

fn create_dummy_window() -> Window {
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
        created_at: Timestamp::now(),
        last_focused_at: None,
        properties: HashMap::new(),
    }
}

#[tokio::test]
async fn test_window_manager_minimize_allowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: true });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.minimize_window(&window).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_minimize_disallowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: false });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.minimize_window(&window).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_window_manager_maximize_allowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: true });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.maximize_window(&window).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_maximize_disallowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: false });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.maximize_window(&window).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_window_manager_close_allowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: true });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.close_window(&window).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_close_disallowed() {
    let policy_manager = Arc::new(MockWindowPolicyManager { allow_action: false });
    let window_manager = WindowManager::new(policy_manager);
    let window = create_dummy_window();

    let result = window_manager.close_window(&window).await;
    assert!(result.is_err());
}
