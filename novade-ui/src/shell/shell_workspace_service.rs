use crate::shell::panel_widget::workspace_indicator_widget::types::WorkspaceInfo;
use std::cell::RefCell;

pub struct ShellWorkspaceService {
    workspaces: RefCell<Vec<WorkspaceInfo>>,
    active_workspace_id: RefCell<Option<String>>,
}

impl ShellWorkspaceService {
    pub fn new() -> Self {
        let initial_workspaces = vec![
            WorkspaceInfo {
                id: "ws1".to_string(),
                name: "Workspace Alpha".to_string(),
                icon_name: Some("desktop-symbolic".to_string()),
                number: 1,
                is_active: true, // Initial active workspace
                is_occupied: true,
            },
            WorkspaceInfo {
                id: "ws2".to_string(),
                name: "Workspace Beta".to_string(),
                icon_name: Some("folder-symbolic".to_string()),
                number: 2,
                is_active: false,
                is_occupied: false,
            },
            WorkspaceInfo {
                id: "ws3".to_string(),
                name: "Workspace Gamma".to_string(),
                icon_name: Some("applications-utilities-symbolic".to_string()),
                number: 3,
                is_active: false,
                is_occupied: true,
            },
            WorkspaceInfo {
                id: "ws4".to_string(),
                name: "Workspace Delta".to_string(),
                icon_name: None,
                number: 4,
                is_active: false,
                is_occupied: false,
            },
        ];
        let initial_active_id = initial_workspaces
            .iter()
            .find(|ws| ws.is_active)
            .map(|ws| ws.id.clone());

        Self {
            workspaces: RefCell::new(initial_workspaces),
            active_workspace_id: RefCell::new(initial_active_id),
        }
    }

    pub fn get_all_workspaces(&self) -> Vec<WorkspaceInfo> {
        let active_id_opt = self.active_workspace_id.borrow();
        self.workspaces
            .borrow()
            .iter()
            .map(|ws_info| {
                let mut new_ws_info = ws_info.clone();
                new_ws_info.is_active = active_id_opt.as_ref() == Some(&new_ws_info.id);
                new_ws_info
            })
            .collect()
    }

    // This method takes &self due to interior mutability of active_workspace_id
    pub fn switch_to_workspace(&self, new_active_id: String) {
        self.active_workspace_id.replace(Some(new_active_id));
        // Note: In a real system, this might also trigger updates to the underlying
        // window manager and then a separate event would notify parts of the UI.
        // For this stub, WorkspaceIndicatorWidget will call get_all_workspaces again.
    }
}

impl Default for ShellWorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}
