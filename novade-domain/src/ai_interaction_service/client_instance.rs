use crate::ai_interaction_service::types::{
    MCPServerConfig, ClientCapabilities, ServerCapabilities, ServerInfo, ConnectionStatus,
    JsonRpcRequest, JsonRpcResponse,
};
use crate::ai_interaction_service::transport::IMCPTransport;
use anyhow::{Result, anyhow, Context};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc}; // Added mpsc

pub struct MCPClientInstance {
    pub config: MCPServerConfig, // Made public
    client_capabilities: ClientCapabilities,
    notification_rx: Option<mpsc::UnboundedReceiver<JsonRpcRequest>>, // Added
    server_capabilities: Option<ServerCapabilities>,
    server_info: Option<ServerInfo>,
    connection_status: ConnectionStatus,
    transport: Arc<Mutex<dyn IMCPTransport>>,
    request_id_counter: u64,
}

impl MCPClientInstance {
    pub fn new(
        config: MCPServerConfig,
        client_capabilities: ClientCapabilities,
        transport: Arc<Mutex<dyn IMCPTransport>>,
        notification_rx: mpsc::UnboundedReceiver<JsonRpcRequest>, // Added parameter
    ) -> Self {
        MCPClientInstance {
            config,
            client_capabilities,
            notification_rx: Some(notification_rx), // Store it
            server_capabilities: None,
            server_info: None,
            connection_status: ConnectionStatus::Disconnected,
            transport,
            request_id_counter: 0,
        }
    }

    pub fn take_notification_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<JsonRpcRequest>> { // Added method
        self.notification_rx.take()
    }

    fn next_request_id(&mut self) -> serde_json::Value {
        self.request_id_counter += 1;
        json!(self.request_id_counter)
    }

    pub async fn connect_and_initialize(&mut self) -> Result<()> {
        self.connection_status = ConnectionStatus::Connecting;
        let mut transport_guard = self.transport.lock().await;
        transport_guard.connect().await.context("Failed to connect transport")?;

        let initialize_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: json!({ "clientCapabilities": self.client_capabilities }),
            id: Some(self.next_request_id()),
        };

        let response = transport_guard.send_request(initialize_request).await.context("Failed to send initialize request")?;

        if let Some(err) = response.error {
            self.connection_status = ConnectionStatus::Error;
            return Err(anyhow!("Initialization error: code={}, message={}", err.code, err.message));
        }

        let result = response.result.ok_or_else(|| anyhow!("Initialize response missing result"))?;
        
        let server_info: ServerInfo = serde_json::from_value(result.get("serverInfo").cloned().ok_or_else(|| anyhow!("Missing serverInfo in initialize response"))?)
            .context("Failed to deserialize ServerInfo")?;
        let server_capabilities: ServerCapabilities = serde_json::from_value(result.get("serverCapabilities").cloned().ok_or_else(|| anyhow!("Missing serverCapabilities in initialize response"))?)
            .context("Failed to deserialize ServerCapabilities")?;

        self.server_info = Some(server_info);
        self.server_capabilities = Some(server_capabilities);
        self.connection_status = ConnectionStatus::Connected;

        println!("[MCPClientInstance] Successfully connected and initialized. Server: {:?}, Capabilities: {:?}", self.server_info, self.server_capabilities);
        Ok(())
    }

    pub async fn send_request_internal(&mut self, method: String, params: serde_json::Value) -> Result<JsonRpcResponse> {
        if self.connection_status != ConnectionStatus::Connected {
            return Err(anyhow!("Client not connected. Current status: {:?}", self.connection_status));
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: Some(self.next_request_id()),
        };
        
        let mut transport_guard = self.transport.lock().await;
        transport_guard.send_request(request).await.context("Failed to send request")
    }

    pub async fn send_notification_internal(&mut self, method: String, params: serde_json::Value) -> Result<()> {
        if self.connection_status != ConnectionStatus::Connected {
            return Err(anyhow!("Client not connected. Current status: {:?}", self.connection_status));
        }

        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: None, // Notifications do not have an ID
        };

        let mut transport_guard = self.transport.lock().await;
        transport_guard.send_notification(notification).await.context("Failed to send notification")
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if self.connection_status == ConnectionStatus::Connected {
            // Attempt to send a shutdown notification if the protocol defines one.
            // For now, we'll just log it.
            println!("[MCPClientInstance] Sending shutdown notification (if applicable)...");
            // Example: self.send_notification_internal("shutdown", json!({})).await?;
        }

        let mut transport_guard = self.transport.lock().await;
        transport_guard.disconnect().await.context("Failed to disconnect transport")?;
        self.connection_status = ConnectionStatus::Disconnected;
        println!("[MCPClientInstance] Shutdown complete.");
        Ok(())
    }

    // Getter methods for status and capabilities
    pub fn get_connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn get_server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    pub fn get_server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }
}
