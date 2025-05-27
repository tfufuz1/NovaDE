use std::collections::HashMap;
use crate::workspaces::core::{Workspace, WorkspaceId, WindowIdentifier};
pub use self::errors::WindowAssignmentError;

pub mod errors;

/// Assigns a window to a target workspace.
///
/// # Arguments
/// * `workspaces` - A mutable hashmap of all available workspaces.
/// * `target_workspace_id` - The ID of the workspace to assign the window to.
/// * `window_id` - The identifier of the window to assign.
/// * `ensure_unique_assignment` - If true, removes the window from any other workspace it might be in.
///
/// # Returns
/// * `Ok(Option<WorkspaceId>)` - `Some(old_workspace_id)` if the window was moved from another workspace, `None` otherwise.
///
/// # Errors
/// * `WindowAssignmentError::WorkspaceNotFound` - If the `target_workspace_id` does not exist.
pub fn assign_window_to_workspace(
    workspaces: &mut HashMap<WorkspaceId, Workspace>,
    target_workspace_id: WorkspaceId,
    window_id: &WindowIdentifier,
    ensure_unique_assignment: bool,
) -> Result<Option<WorkspaceId>, WindowAssignmentError> {
    let mut old_workspace_id: Option<WorkspaceId> = None;

    if ensure_unique_assignment {
        // Find and remove the window from any other workspace
        let mut current_owner_id: Option<WorkspaceId> = None;
        for (id, ws) in workspaces.iter() {
            if ws.window_ids().contains(window_id) {
                current_owner_id = Some(*id);
                break;
            }
        }

        if let Some(owner_id) = current_owner_id {
            if owner_id != target_workspace_id {
                if let Some(owner_ws) = workspaces.get_mut(&owner_id) {
                    owner_ws.remove_window_id(window_id);
                    old_workspace_id = Some(owner_id);
                }
            } else {
                // Already in the target workspace, no actual move needed, but not an error.
                // It's effectively already uniquely assigned to the target.
                // We can return Ok(None) as no "move" from another workspace occurred.
                return Ok(None);
            }
        }
    }

    let target_workspace = workspaces
        .get_mut(&target_workspace_id)
        .ok_or(WindowAssignmentError::WorkspaceNotFound(target_workspace_id))?;

    if target_workspace.add_window_id(window_id.clone()) {
        // Window was newly added (not already present in target)
        Ok(old_workspace_id) // Return Some(old_id) if moved, None if it was only added
    } else {
        // Window was already in the target workspace.
        // If ensure_unique_assignment was true and it was moved from another workspace, old_workspace_id would be Some.
        // If it was already in target and ensure_unique was true, we returned early.
        // If ensure_unique was false and it was already in target, old_workspace_id is None.
        Ok(old_workspace_id)
    }
}

/// Removes a window from a source workspace.
///
/// # Arguments
/// * `workspaces` - A mutable hashmap of all available workspaces.
/// * `source_workspace_id` - The ID of the workspace to remove the window from.
/// * `window_id` - The identifier of the window to remove.
///
/// # Returns
/// * `Ok(true)` if the window was successfully removed.
/// * `Ok(false)` if the window was not found in the specified workspace.
///
/// # Errors
/// * `WindowAssignmentError::WorkspaceNotFound` - If the `source_workspace_id` does not exist.
pub fn remove_window_from_workspace(
    workspaces: &mut HashMap<WorkspaceId, Workspace>,
    source_workspace_id: WorkspaceId,
    window_id: &WindowIdentifier,
) -> Result<bool, WindowAssignmentError> {
    let source_workspace = workspaces
        .get_mut(&source_workspace_id)
        .ok_or(WindowAssignmentError::WorkspaceNotFound(source_workspace_id))?;

    Ok(source_workspace.remove_window_id(window_id))
}

/// Finds the workspace that currently contains the given window.
///
/// # Arguments
/// * `workspaces` - A hashmap of all available workspaces.
/// * `window_id` - The identifier of the window to find.
///
/// # Returns
/// * `Some(WorkspaceId)` if the window is found in one ofthe workspaces.
/// * `None` if the window is not found in any workspace.
pub fn find_workspace_for_window(
    workspaces: &HashMap<WorkspaceId, Workspace>,
    window_id: &WindowIdentifier,
) -> Option<WorkspaceId> {
    for (id, workspace) in workspaces {
        if workspace.window_ids().contains(window_id) {
            return Some(*id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::core::{Workspace, WindowIdentifier};
    use std::collections::HashMap;

    fn create_test_workspace(name: &str) -> Workspace {
        Workspace::new(name.to_string(), None, None, None).unwrap()
    }

    #[test]
    fn assign_window_success_no_unique_ensure() {
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let mut workspaces = HashMap::from([(ws1_id, ws1)]);
        let win_id = WindowIdentifier::new("win1".to_string()).unwrap();

        let result = assign_window_to_workspace(&mut workspaces, ws1_id, &win_id, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // Not moved from another
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
    }

    #[test]
    fn assign_window_success_with_unique_ensure_no_prior_assignment() {
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let mut workspaces = HashMap::from([(ws1_id, ws1)]);
        let win_id = WindowIdentifier::new("win1".to_string()).unwrap();

        let result = assign_window_to_workspace(&mut workspaces, ws1_id, &win_id, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // Not moved
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
    }

    #[test]
    fn assign_window_success_with_unique_ensure_moves_window() {
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let mut ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();
        let win_id = WindowIdentifier::new("win1".to_string()).unwrap();

        // Pre-assign to ws1
        ws1.add_window_id(win_id.clone());
        let mut workspaces = HashMap::from([(ws1_id, ws1), (ws2_id, ws2)]);

        // Assign to ws2 with ensure_unique_assignment
        let result = assign_window_to_workspace(&mut workspaces, ws2_id, &win_id, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(ws1_id)); // Moved from ws1

        assert!(!workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id), "Window should be removed from ws1");
        assert!(workspaces.get(&ws2_id).unwrap().window_ids().contains(&win_id), "Window should be added to ws2");
    }
    
    #[test]
    fn assign_window_to_same_workspace_with_unique_ensure() {
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let win_id = WindowIdentifier::new("win1".to_string()).unwrap();
        ws1.add_window_id(win_id.clone());
        let mut workspaces = HashMap::from([(ws1_id, ws1)]);

        // Try to assign to the same workspace ws1 with ensure_unique
        let result = assign_window_to_workspace(&mut workspaces, ws1_id, &win_id, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // No actual move occurred
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
        assert_eq!(workspaces.get(&ws1_id).unwrap().window_ids().len(), 1);
    }


    #[test]
    fn assign_window_to_non_existent_workspace() {
        let mut workspaces = HashMap::new();
        let win_id = WindowIdentifier::new("win1".to_string()).unwrap();
        let non_existent_ws_id = WorkspaceId::new_v4();

        let result = assign_window_to_workspace(&mut workspaces, non_existent_ws_id, &win_id, false);
        assert!(matches!(result, Err(WindowAssignmentError::WorkspaceNotFound(id)) if id == non_existent_ws_id));
        
        let result_unique = assign_window_to_workspace(&mut workspaces, non_existent_ws_id, &win_id, true);
        assert!(matches!(result_unique, Err(WindowAssignmentError::WorkspaceNotFound(id)) if id == non_existent_ws_id));
    }
}
