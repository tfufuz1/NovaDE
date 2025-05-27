use super::common_types::{WorkspaceDescriptor, WorkspaceId};

#[derive(Clone, Debug)]
pub enum WorkspaceEvent {
    ActiveWorkspaceChanged {
        new_id: WorkspaceId,
        old_id: Option<WorkspaceId>,
    },
    WorkspaceCreated {
        descriptor: WorkspaceDescriptor,
    },
    WorkspaceDeleted {
        id: WorkspaceId,
    },
    // WorkspaceUpdated { descriptor: WorkspaceDescriptor }, // Example
    // WorkspaceOrderChanged { order: Vec<WorkspaceId> }, // Example
}
