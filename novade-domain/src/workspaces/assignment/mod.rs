use std::collections::HashMap;
use crate::workspaces::core::{WorkspaceId, WindowIdentifier, Workspace};
use super::errors::WindowAssignmentError; // From crate::workspaces::assignment::errors

pub fn assign_window_to_workspace(
    workspaces: &mut HashMap<WorkspaceId, Workspace>,
    target_workspace_id: WorkspaceId,
    window_id: &WindowIdentifier,
    ensure_unique_assignment: bool,
) -> Result<(), WindowAssignmentError> {
    if ensure_unique_assignment {
        for (ws_id, ws) in workspaces.iter_mut() {
            if *ws_id != target_workspace_id {
                ws.remove_window_id(window_id);
            }
        }
    }

    let target_workspace = workspaces
        .get_mut(&target_workspace_id)
        .ok_or(WindowAssignmentError::WorkspaceNotFound(target_workspace_id))?;

    target_workspace.add_window_id(window_id.clone());
    // Ignoring boolean return of add_window_id as per function signature.
    // If we needed to signal if it was newly added vs already present, error variant or Ok(bool) would be needed.
    // WindowAlreadyAssigned error could be returned here if add_window_id returned false AND that's an error condition.
    // For now, adding an already present window is idempotent and not an error.

    Ok(())
}

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

pub fn move_window_to_workspace(
    workspaces: &mut HashMap<WorkspaceId, Workspace>,
    source_workspace_id: WorkspaceId,
    target_workspace_id: WorkspaceId,
    window_id: &WindowIdentifier,
) -> Result<(), WindowAssignmentError> {
    if source_workspace_id == target_workspace_id {
        return Err(WindowAssignmentError::CannotMoveToSameWorkspace {
            workspace_id: source_workspace_id,
            window_id: window_id.clone(),
        });
    }

    // Ensure target workspace exists before attempting any move logic
    if !workspaces.contains_key(&target_workspace_id) {
        return Err(WindowAssignmentError::TargetWorkspaceNotFound(target_workspace_id));
    }
    
    // Get source workspace and remove window
    let source_workspace = workspaces
        .get_mut(&source_workspace_id)
        .ok_or(WindowAssignmentError::SourceWorkspaceNotFound(source_workspace_id))?;

    if !source_workspace.remove_window_id(window_id) {
        return Err(WindowAssignmentError::WindowNotOnSourceWorkspace {
            workspace_id: source_workspace_id,
            window_id: window_id.clone(),
        });
    }

    // This unwrap is safe because we checked contains_key earlier.
    let target_workspace = workspaces.get_mut(&target_workspace_id).unwrap();
    target_workspace.add_window_id(window_id.clone());

    Ok(())
}

pub fn find_workspace_for_window(
    workspaces: &HashMap<WorkspaceId, Workspace>,
    window_id: &WindowIdentifier,
) -> Option<WorkspaceId> {
    for workspace in workspaces.values() {
        if workspace.window_ids().contains(window_id) {
            return Some(workspace.id());
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::core::Workspace;
    use uuid::Uuid;

    fn create_test_workspace(name: &str) -> Workspace {
        Workspace::new(name.to_string(), None, None, None).unwrap()
    }

    #[test]
    fn test_assign_window_to_workspace_basic() {
        let mut workspaces = HashMap::new();
        let ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        workspaces.insert(ws1_id, ws1);

        let win_id = WindowIdentifier::from("win1");
        
        let result = assign_window_to_workspace(&mut workspaces, ws1_id, &win_id, false);
        assert!(result.is_ok());
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
    }

    #[test]
    fn test_assign_window_to_non_existent_workspace() {
        let mut workspaces = HashMap::new();
        let non_existent_ws_id = Uuid::new_v4();
        let win_id = WindowIdentifier::from("win1");

        let result = assign_window_to_workspace(&mut workspaces, non_existent_ws_id, &win_id, false);
        assert!(matches!(result, Err(WindowAssignmentError::WorkspaceNotFound(id)) if id == non_existent_ws_id));
    }

    #[test]
    fn test_assign_window_ensure_unique_assignment() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let mut ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();

        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone()); 

        workspaces.insert(ws1_id, ws1);
        workspaces.insert(ws2_id, ws2);
        
        let result = assign_window_to_workspace(&mut workspaces, ws2_id, &win_id, true);
        assert!(result.is_ok());
        
        assert!(!workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id), "Window should be removed from ws1");
        assert!(workspaces.get(&ws2_id).unwrap().window_ids().contains(&win_id), "Window should be added to ws2");
    }
    
    #[test]
    fn test_assign_window_no_unique_assignment() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();

        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone()); 

        workspaces.insert(ws1_id, ws1);
        workspaces.insert(ws2_id, ws2);
        
        let result = assign_window_to_workspace(&mut workspaces, ws2_id, &win_id, false);
        assert!(result.is_ok());
        
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id), "Window should still be in ws1");
        assert!(workspaces.get(&ws2_id).unwrap().window_ids().contains(&win_id), "Window should be added to ws2");
    }

    #[test]
    fn test_assign_window_already_present_in_target() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone());
        workspaces.insert(ws1_id, ws1);

        let result = assign_window_to_workspace(&mut workspaces, ws1_id, &win_id, false);
        assert!(result.is_ok());
        assert_eq!(workspaces.get(&ws1_id).unwrap().window_ids().len(), 1);
    }

    #[test]
    fn test_remove_window_from_workspace_existing() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone());
        workspaces.insert(ws1_id, ws1);

        let result = remove_window_from_workspace(&mut workspaces, ws1_id, &win_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); 
        assert!(!workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
    }

    #[test]
    fn test_remove_window_from_workspace_non_existing_window() {
        let mut workspaces = HashMap::new();
        let ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        workspaces.insert(ws1_id, ws1);
        
        let win_id = WindowIdentifier::from("win1");
        let result = remove_window_from_workspace(&mut workspaces, ws1_id, &win_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); 
    }

    #[test]
    fn test_remove_window_from_non_existent_workspace() {
        let mut workspaces = HashMap::new();
        let non_existent_ws_id = Uuid::new_v4();
        let win_id = WindowIdentifier::from("win1");

        let result = remove_window_from_workspace(&mut workspaces, non_existent_ws_id, &win_id);
        assert!(matches!(result, Err(WindowAssignmentError::WorkspaceNotFound(id)) if id == non_existent_ws_id));
    }

    #[test]
    fn test_move_window_to_workspace_successful() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();
        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone());
        workspaces.insert(ws1_id, ws1);
        workspaces.insert(ws2_id, ws2);

        let result = move_window_to_workspace(&mut workspaces, ws1_id, ws2_id, &win_id);
        assert!(result.is_ok());
        assert!(!workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id));
        assert!(workspaces.get(&ws2_id).unwrap().window_ids().contains(&win_id));
    }

    #[test]
    fn test_move_window_to_same_workspace() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1"); // Mutable because add_window_id takes &mut self
        let ws1_id = ws1.id();
        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone()); // Add window so it exists
        workspaces.insert(ws1_id, ws1);


        let result = move_window_to_workspace(&mut workspaces, ws1_id, ws1_id, &win_id);
        assert!(matches!(result, Err(WindowAssignmentError::CannotMoveToSameWorkspace { workspace_id, .. }) if workspace_id == ws1_id));
    }
    
    #[test]
    fn test_move_window_source_workspace_not_found() {
        let mut workspaces = HashMap::new();
        let ws1_id = Uuid::new_v4();
        let ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();
        workspaces.insert(ws2_id, ws2);
        let win_id = WindowIdentifier::from("win1");

        let result = move_window_to_workspace(&mut workspaces, ws1_id, ws2_id, &win_id);
        assert!(matches!(result, Err(WindowAssignmentError::SourceWorkspaceNotFound(id)) if id == ws1_id));
    }

    #[test]
    fn test_move_window_target_workspace_not_found() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let win_id = WindowIdentifier::from("win1");
        ws1.add_window_id(win_id.clone());
        workspaces.insert(ws1_id, ws1);
        let ws2_id = Uuid::new_v4();

        let result = move_window_to_workspace(&mut workspaces, ws1_id, ws2_id, &win_id);
        assert!(matches!(result, Err(WindowAssignmentError::TargetWorkspaceNotFound(id)) if id == ws2_id));
        assert!(workspaces.get(&ws1_id).unwrap().window_ids().contains(&win_id)); // Should not be removed
    }

    #[test]
    fn test_move_window_not_on_source_workspace() {
        let mut workspaces = HashMap::new();
        let ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();
        let win_id = WindowIdentifier::from("win1");
        workspaces.insert(ws1_id, ws1);
        workspaces.insert(ws2_id, ws2);

        let result = move_window_to_workspace(&mut workspaces, ws1_id, ws2_id, &win_id);
        assert!(matches!(result, Err(WindowAssignmentError::WindowNotOnSourceWorkspace { workspace_id, .. }) if workspace_id == ws1_id));
    }

    #[test]
    fn test_find_workspace_for_window_found() {
        let mut workspaces = HashMap::new();
        let mut ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        let ws2 = create_test_workspace("WS2");
        let ws2_id = ws2.id();
        let win_id1 = WindowIdentifier::from("win1");
        let win_id2 = WindowIdentifier::from("win2");
        ws1.add_window_id(win_id1.clone());
        ws2.add_window_id(win_id2.clone());
        workspaces.insert(ws1_id, ws1);
        workspaces.insert(ws2_id, ws2);

        assert_eq!(find_workspace_for_window(&workspaces, &win_id1), Some(ws1_id));
        assert_eq!(find_workspace_for_window(&workspaces, &win_id2), Some(ws2_id));
    }

    #[test]
    fn test_find_workspace_for_window_not_found() {
        let mut workspaces = HashMap::new();
        let ws1 = create_test_workspace("WS1");
        let ws1_id = ws1.id();
        workspaces.insert(ws1_id, ws1);
        let win_id_unassigned = WindowIdentifier::from("win_unassigned");

        assert_eq!(find_workspace_for_window(&workspaces, &win_id_unassigned), None);
    }

    #[test]
    fn test_find_workspace_for_window_empty_map() {
        let workspaces = HashMap::new();
        let win_id = WindowIdentifier::from("win1");
        assert_eq!(find_workspace_for_window(&workspaces, &win_id), None);
    }
}
