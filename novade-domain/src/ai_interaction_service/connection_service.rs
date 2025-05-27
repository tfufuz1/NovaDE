use crate::ai_interaction_service::{
    MCPServerConfig, ClientCapabilities, MCPClientInstance, StdioTransportHandler, IMCPTransport,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context, anyhow};

pub type ServerId = String;

pub struct MCPConnectionService {
    client_instances: HashMap<ServerId, Arc<Mutex<MCPClientInstance>>>,
    // Assuming a default ClientCapabilities for now. This could be configurable.
    default_client_capabilities: ClientCapabilities,
}

impl MCPConnectionService {
    pub fn new(default_client_capabilities: ClientCapabilities) -> Self {
        MCPConnectionService {
            client_instances: HashMap::new(),
            default_client_capabilities,
        }
    }

    pub async fn load_initial_server_configurations(&mut self, configs: Vec<MCPServerConfig>) {
        for config in configs {
            let server_id = config.host.clone(); // Using host as ServerId for simplicity
            if let Err(e) = self.connect_to_server(config).await {
                eprintln!("[MCPConnectionService] Failed to connect to server {}: {:?}", server_id, e);
                // Decide on error handling: continue, collect errors, etc.
            }
        }
    }

    pub async fn connect_to_server(&mut self, config: MCPServerConfig) -> Result<()> {
        let server_id = config.host.clone(); // Using host as ServerId for simplicity
        if self.client_instances.contains_key(&server_id) {
            return Err(anyhow!("Server {} is already connected or connection attempt is in progress.", server_id));
        }

        // For now, we default to StdioTransportHandler. This could be made more flexible.
        let transport = Arc::new(Mutex::new(StdioTransportHandler::new()));
        let mut client_instance = MCPClientInstance::new(
            config,
            self.default_client_capabilities.clone(),
            transport,
        );

        match client_instance.connect_and_initialize().await {
            Ok(_) => {
                println!("[MCPConnectionService] Successfully connected to server: {}", server_id);
                self.client_instances.insert(server_id, Arc::new(Mutex::new(client_instance)));
                Ok(())
            }
            Err(e) => {
                eprintln!("[MCPConnectionService] Error during connect_and_initialize for server {}: {:?}", server_id, e);
                Err(e).context(format!("Failed to connect and initialize server: {}", server_id))
            }
        }
    }

    pub async fn disconnect_from_server(&mut self, server_id: &str) -> Result<()> {
        if let Some(client_arc) = self.client_instances.remove(server_id) {
            let mut client_instance = client_arc.lock().await;
            client_instance.shutdown().await.context(format!("Failed to shutdown server: {}", server_id))?;
            println!("[MCPConnectionService] Successfully disconnected from server: {}", server_id);
            Ok(())
        } else {
            Err(anyhow!("Server {} not found or not connected.", server_id))
        }
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
