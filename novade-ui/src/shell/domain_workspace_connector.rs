use novade_domain::workspaces::{WorkspaceManager, WorkspaceEvent, WorkspaceDescriptor, WorkspaceId};
use crate::shell::panel_widget::workspace_indicator_widget::types::WorkspaceInfo as UiWorkspaceInfo;
use std::sync::Arc;
use tokio::runtime::Handle; // To spawn the event listener task
use glib::clone; // For glib closures

pub struct DomainWorkspaceConnector {
    domain_manager: Arc<dyn WorkspaceManager>,
    ui_event_sender: glib::Sender<Vec<UiWorkspaceInfo>>,
    // Store the Tokio runtime handle to spawn tasks if needed, or ensure one is running
    runtime_handle: Handle,
}

impl DomainWorkspaceConnector {
    pub fn new(
        domain_manager: Arc<dyn WorkspaceManager>,
        ui_event_sender: glib::Sender<Vec<UiWorkspaceInfo>>,
        runtime_handle: Handle,
    ) -> Self {
        let connector = Self {
            domain_manager,
            ui_event_sender,
            runtime_handle,
        };
        connector.start_event_listener();
        connector
    }

    fn start_event_listener(&self) {
        let mut receiver = match self.domain_manager.subscribe_to_workspace_events() {
            Ok(rx) => rx,
            Err(e) => {
                tracing::error!("Failed to subscribe to domain workspace events: {}", e);
                // In a real app, might want to panic or handle this more gracefully
                return;
            }
        };

        let domain_manager_clone = Arc::clone(&self.domain_manager);
        let ui_event_sender_clone = self.ui_event_sender.clone();

        self.runtime_handle.spawn(async move {
            tracing::info!("DomainWorkspaceConnector: Event listener task started.");
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        tracing::info!("DomainWorkspaceConnector: Received WorkspaceEvent: {:?}", event);
                        // Regardless of the specific event for this stub, we fetch all and update.
                        // A more refined approach might use the event details to avoid full refetch.
                        match domain_manager_clone.list_workspaces().await {
                            Ok(descriptors) => {
                                let active_id_res = domain_manager_clone.get_active_workspace_id().await;
                                let active_id = match active_id_res {
                                    Ok(id_opt) => id_opt,
                                    Err(e) => {
                                        tracing::error!("Failed to get active workspace ID: {}", e);
                                        None
                                    }
                                };

                                let ui_infos = Self::map_descriptors_to_ui_info(descriptors, active_id);
                                if let Err(e) = ui_event_sender_clone.send(ui_infos) {
                                    tracing::error!("Failed to send UI workspace info: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to list workspaces after event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error receiving workspace event: {}. Listener terminating.", e);
                        break; // Exit loop on error (e.g., sender dropped)
                    }
                }
            }
            tracing::info!("DomainWorkspaceConnector: Event listener task terminated.");
        });
    }

    fn map_descriptors_to_ui_info(
        descriptors: Vec<WorkspaceDescriptor>,
        active_id: Option<WorkspaceId>,
    ) -> Vec<UiWorkspaceInfo> {
        descriptors
            .into_iter()
            .enumerate() // For assigning numbers if not present in descriptor
            .map(|(index, desc)| UiWorkspaceInfo {
                id: desc.id.clone(),
                name: desc.name.clone(),
                icon_name: None, // Or map from descriptor if it has an icon field
                number: (index + 1), // Simple 1-based numbering
                is_active: active_id.as_ref() == Some(&desc.id),
                is_occupied: false, // Placeholder, domain model doesn't have this yet
            })
            .collect()
    }

    // Fetches current workspaces and maps them for the UI.
    // This is useful for initial population or manual refresh.
    pub async fn get_all_workspaces_for_ui(&self) -> Vec<UiWorkspaceInfo> {
        match self.domain_manager.list_workspaces().await {
            Ok(descriptors) => {
                let active_id_res = self.domain_manager.get_active_workspace_id().await;
                let active_id = match active_id_res {
                    Ok(id_opt) => id_opt,
                    Err(e) => {
                        tracing::error!("Failed to get active workspace ID during fetch: {}", e);
                        None
                    }
                };
                Self::map_descriptors_to_ui_info(descriptors, active_id)
            }
            Err(e) => {
                tracing::error!("Failed to get all workspaces for UI: {}", e);
                Vec::new() // Return empty on error
            }
        }
    }
    
    // Switches active workspace in the domain. UI update will happen via the event listener.
    pub async fn switch_to_workspace_in_domain(&self, new_active_id: String) -> Result<(), String> {
        self.domain_manager
            .set_active_workspace(new_active_id)
            .await
            .map_err(|e| format!("Domain error switching workspace: {}", e))
    }
}
