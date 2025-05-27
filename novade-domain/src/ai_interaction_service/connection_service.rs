use crate::ai_interaction_service::{
    MCPServerConfig, ClientCapabilities, MCPClientInstance, StdioTransportHandler, IMCPTransport, MCPError,
};
use novade_system::mcp_client_service::IMCPClientService as SystemIMCPClientService; // For spawning processes
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context, anyhow}; // Keep anyhow for general Result, but MCPError for specific ops

pub type ServerId = String;

pub struct MCPConnectionService {
    client_instances: HashMap<ServerId, Arc<Mutex<MCPClientInstance>>>,
    default_client_capabilities: ClientCapabilities,
    system_mcp_service: Arc<dyn SystemIMCPClientService>, // To spawn server processes
    // Store PIDs of spawned servers to terminate them if MCPConnectionService is dropped
    // or if a server needs to be explicitly restarted/removed beyond MCPClientInstance::shutdown()
    spawned_pids: Mutex<HashMap<ServerId, u32>>,
}

impl MCPConnectionService {
    pub fn new(
        default_client_capabilities: ClientCapabilities,
        system_mcp_service: Arc<dyn SystemIMCPClientService>,
    ) -> Self {
        MCPConnectionService {
            client_instances: HashMap::new(),
            default_client_capabilities,
            system_mcp_service,
            spawned_pids: Mutex::new(HashMap::new()),
        }
    }

    // Takes MCPServerConfig which includes host, port, and potentially path or command for stdio.
    // Returns the PID of the spawned process if successful.
    pub async fn connect_to_server(&mut self, config: MCPServerConfig) -> Result<u32, MCPError> {
        let server_id = config.host.clone(); // Using host as ServerId for simplicity
        if self.client_instances.contains_key(&server_id) {
            // Consider what to do if already connected: return Ok or Err?
            // If an instance exists, it implies a process is already managed.
            // For now, let's treat it as an error to try connecting again to same server_id.
            return Err(MCPError::InternalError); // Or a more specific "AlreadyConnected"
        }

        // For stdio, config.host might be a command or path to executable.
        // The IMCPClientService::spawn_stdio_server takes command & args.
        // We need a way to determine these from MCPServerConfig.
        // Assume config.host is the command for now. Args could be part of MCPServerConfig if needed.
        let command = config.host.clone(); // This needs to be the actual server command/path
        let args: Vec<String> = Vec::new(); // Example: parse from config if needed

        let stdio_process = self.system_mcp_service.spawn_stdio_server(command, args).await
            .map_err(|sys_err| MCPError::InternalError)?; // Convert SystemError to MCPError: sys_err.to_string()

        let pid = stdio_process.pid;
        
        // The StdioTransportHandler is now an internal detail of MCPClientInstance's connect method.
        // MCPClientInstance::new returns Arc<Mutex<MCPClientInstance>>
        let transport = Arc::new(Mutex::new(StdioTransportHandler::new()));
        let client_instance_arc = MCPClientInstance::new(
            config.clone(), // Client needs its own config copy
            self.default_client_capabilities.clone(),
            transport, // Pass the StdioTransportHandler
        );

        // Call connect_and_initialize on the Arc<Mutex<MCPClientInstance>>
        match MCPClientInstance::connect_and_initialize(client_instance_arc.clone(), stdio_process).await {
            Ok(_) => {
                println!("[MCPConnectionService] Successfully connected to server: {} (PID: {})", server_id, pid);
                self.client_instances.insert(server_id.clone(), client_instance_arc);
                self.spawned_pids.lock().await.insert(server_id, pid);
                Ok(pid)
            }
            Err(mcp_err) => {
                eprintln!("[MCPConnectionService] Error during connect_and_initialize for server {}: {:?}", server_id, mcp_err);
                // Ensure the spawned process is terminated if initialization fails
                if let Err(term_err) = self.system_mcp_service.terminate_stdio_server(pid).await {
                    eprintln!("[MCPConnectionService] Additionally, failed to terminate process PID {} after init error: {:?}", pid, term_err);
                }
                Err(mcp_err)
            }
        }
    }
    
    // load_initial_server_configurations might not be needed if servers are added dynamically
    // or via other mechanisms. If kept, it should call the modified connect_to_server.
    pub async fn load_initial_server_configurations(&mut self, configs: Vec<MCPServerConfig>) {
        for config in configs {
            let server_id = config.host.clone();
            if let Err(e) = self.connect_to_server(config).await {
                eprintln!("[MCPConnectionService] Failed to connect to server {}: {:?}", server_id, e);
            }
        }
    }


    pub async fn disconnect_from_server(&mut self, server_id: &str) -> Result<(), MCPError> {
        if let Some(client_arc) = self.client_instances.remove(server_id) {
            // Shutdown the MCPClientInstance (which handles transport disconnect)
            MCPClientInstance::shutdown(client_arc).await?; // Assuming shutdown returns Result<(), MCPError>
            
            // Terminate the underlying OS process
            if let Some(pid) = self.spawned_pids.lock().await.remove(server_id) {
                self.system_mcp_service.terminate_stdio_server(pid).await
                    .map_err(|sys_err| MCPError::InternalError)?; // Convert SystemError
            }
            println!("[MCPConnectionService] Successfully disconnected from server: {}", server_id);
            Ok(())
        } else {
            Err(MCPError::InternalError) // "Server not found or not connected"
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
