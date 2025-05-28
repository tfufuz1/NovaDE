use crate::ai_interaction_service::{
    MCPServerConfig, ClientCapabilities, MCPClientInstance, IMCPTransport,
};
// Removed StdioTransportHandler and ActualStdioTransport imports as they are not directly used here anymore.
// Spawning and transport creation will happen outside.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex; // mpsc is used by MCPClientInstance, but not directly by MCPConnectionService here.
use anyhow::{Result, Context, anyhow};

pub type ServerId = String;

// The placeholder StdioTransportHandler and its impl block are REMOVED from this file.
// Process spawning and ActualStdioTransport creation will now be handled externally,
// likely in novade-system/src/main.rs using DefaultMCPClientService.

pub struct MCPConnectionService {
    client_instances: HashMap<ServerId, Arc<Mutex<MCPClientInstance>>>,
    default_client_capabilities: ClientCapabilities,
}

impl MCPConnectionService {
    pub fn new(default_client_capabilities: ClientCapabilities) -> Self {
        MCPConnectionService {
            client_instances: HashMap::new(),
            default_client_capabilities,
        }
    }

    /// Adds a pre-configured and connected MCPClientInstance to the service.
    /// The MCPClientInstance should have already completed its `connect_and_initialize` sequence.
    pub async fn add_managed_client(&mut self, client_instance_arc: Arc<Mutex<MCPClientInstance>>) -> Result<()> {
        let client_instance_guard = client_instance_arc.lock().await;
        let server_id = client_instance_guard.config.host.clone(); // Use host from config as ServerId

        if client_instance_guard.get_connection_status() != &crate::ai_interaction_service::types::ConnectionStatus::Connected {
            return Err(anyhow!("MCPClientInstance for server_id '{}' is not connected.", server_id));
        }
        
        if self.client_instances.contains_key(&server_id) {
            return Err(anyhow!("Server {} is already managed.", server_id));
        }
        
        drop(client_instance_guard); // Release lock before inserting

        self.client_instances.insert(server_id.clone(), client_instance_arc);
        tracing::info!("[MCPConnectionService] Added managed client for server: {}", server_id);
        Ok(())
    }

    // load_initial_server_configurations is removed as configuration loading and client creation
    // will now be handled externally (e.g., in novade-system/src/main.rs)
    // and clients added via add_managed_client.

    // connect_to_server is removed as transport creation and client initialization
    // are now handled externally.

    /// Disconnects a specific MCP server and removes it from management.
    pub async fn disconnect_from_server(&mut self, server_id: &str) -> Result<()> {
        if let Some(client_arc) = self.client_instances.remove(server_id) {
            tracing::info!("[MCPConnectionService] Disconnecting from server: {}", server_id);
            let mut client_instance = client_arc.lock().await;
            // Attempt to gracefully shutdown the client instance (which includes transport disconnect)
            match client_instance.shutdown().await {
                Ok(_) => tracing::info!("[MCPConnectionService] Successfully shut down MCPClientInstance for server: {}", server_id),
                Err(e) => tracing::error!("[MCPConnectionService] Error shutting down MCPClientInstance for server {}: {:?}", server_id, e),
            }
            // Regardless of shutdown error, we remove it from the active list.
            // The actual server process termination (if managed by DefaultMCPClientService)
            // would need to be handled by calling DefaultMCPClientService.terminate_stdio_server(pid).
            // This service currently doesn't store PIDs directly, relying on the transport/client_instance for that.
            Ok(())
        } else {
            Err(anyhow!("Server {} not found or not managed.", server_id))
        }
    }
    
    /// Disconnects all managed MCP servers.
    pub async fn disconnect_all_servers(&mut self) -> Result<()> {
        tracing::info!("[MCPConnectionService] Disconnecting all managed servers...");
        let server_ids: Vec<ServerId> = self.client_instances.keys().cloned().collect();
        for server_id in server_ids {
            if let Err(e) = self.disconnect_from_server(&server_id).await {
                tracing::error!("[MCPConnectionService] Error disconnecting server {}: {:?}", server_id, e);
                // Continue disconnecting others
            }
        }
        self.client_instances.clear(); // Ensure all are removed
        tracing::info!("[MCPConnectionService] All managed servers disconnected process initiated.");
        Ok(())
    }


    pub fn get_client_instance(&self, server_id: &str) -> Option<Arc<Mutex<MCPClientInstance>>> {
        self.client_instances.get(server_id).cloned()
    }

    pub fn get_all_client_instances(&self) -> Vec<Arc<Mutex<MCPClientInstance>>> {
        self.client_instances.values().cloned().collect()
    }

    // Optional: A method to get default capabilities or set them
    pub fn get_default_client_capabilities(&self) -> &ClientCapabilities {
        &self.default_client_capabilities
    }
}

// Ensure tracing is imported if not already: use tracing;
