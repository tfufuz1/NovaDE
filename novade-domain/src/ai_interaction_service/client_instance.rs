use crate::ai_interaction_service::types::{
    MCPServerConfig, ClientCapabilities, ServerCapabilities, ServerInfo, ConnectionStatus,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError as DomainJsonRpcError, MCPError, // Added MCPError, DomainJsonRpcError
};
use crate::ai_interaction_service::transport::IMCPTransport;
use novade_system::mcp_client_service::StdioProcess; // For connect method
use anyhow::{Result, anyhow, Context};
use serde_json::{json, Value};
use std::collections::HashMap; // For pending_requests
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot}; // Added oneshot

pub struct MCPClientInstance {
    pub config: MCPServerConfig, 
    client_capabilities: ClientCapabilities,
    server_capabilities: Option<ServerCapabilities>,
    server_info: Option<ServerInfo>,
    protocol_version: Option<String>, // Added to store the protocol version
    connection_status: ConnectionStatus,
    last_error: Option<MCPError>, // Added to store the last error
    transport: Arc<Mutex<dyn IMCPTransport>>, 
    request_id_counter: Arc<Mutex<u64>>, 
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, DomainJsonRpcError>>>>>,
}

impl MCPClientInstance {
     pub fn new(
        config: MCPServerConfig,
        client_capabilities: ClientCapabilities,
        transport: Arc<Mutex<dyn IMCPTransport>>, // Pass the actual transport instance
    ) -> Arc<Mutex<Self>> { // Return Arc<Mutex<Self>>
        Arc::new(Mutex::new(MCPClientInstance {
            config,
            client_capabilities,
            server_capabilities: None,
            server_info: None,
            connection_status: ConnectionStatus::Disconnected,
            transport,
            request_id_counter: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }))
    }
    
    // Helper for handle_incoming_message closure
    // Note: This method itself cannot be directly passed as handler due to `&mut self`
    // The actual handler will be a closure.
    async fn process_incoming_message(
        pending_requests_arc: Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, DomainJsonRpcError>>>>>,
        server_info_arc: Arc<Mutex<Option<ServerInfo>>>, // Assuming we need to update these
        server_capabilities_arc: Arc<Mutex<Option<ServerCapabilities>>>,
        connection_status_arc: Arc<Mutex<ConnectionStatus>>,
        message_value: Value,
    ) {
        println!("[MCPClientInstance] Processing incoming message: {:?}", message_value);
        // Attempt to deserialize as JsonRpcResponse
        match serde_json::from_value::<JsonRpcResponse>(message_value.clone()) {
            Ok(response) => {
                if let Some(id_val) = response.id.as_str() { // Assuming ID is string for simplicity
                    let mut pending_requests = pending_requests_arc.lock().await;
                    if let Some(sender) = pending_requests.remove(id_val) {
                        if sender.send(Ok(response)).is_err() {
                            eprintln!("[MCPClientInstance] Failed to send response to waiting task for ID: {}", id_val);
                        }
                    } else {
                        eprintln!("[MCPClientInstance] Received response for unknown request ID: {}", id_val);
                    }
                } else if response.id.is_null() && response.error.is_some() {
                     // This might be a general server error not tied to a specific request.
                     eprintln!("[MCPClientInstance] Received a server error notification: {:?}", response.error);
                     // Potentially update connection status or log globally.
                } else {
                    // This could be a notification from the server (if it's not a response)
                    // Or a response with an unexpected ID format.
                    // For now, we only handle responses to known requests.
                    // Notifications should have no ID. If it has an ID but not string, it's an issue.
                    if response.id.is_null() && response.method.is_some() { // Assuming notifications have a method field
                        println!("[MCPClientInstance] Received server notification: method='{}', params='{:?}'", response.method.as_ref().unwrap_or(&String::new()), response.params);
                        // TODO: Handle server-initiated notifications if MCP spec defines them
                        // e.g. by routing them to a different handler.
                    } else {
                        eprintln!("[MCPClientInstance] Received message that is not a response to a pending request or a recognized notification: {:?}", response);
                    }
                }
            }
            Err(e) => {
                // If not a JsonRpcResponse, it might be a malformed message or a type we don't expect here.
                eprintln!("[MCPClientInstance] Failed to parse incoming message as JsonRpcResponse: {}. Message: {:?}", e, message_value);
                // If it was an error related to connection status, update it.
                // For instance, if the error was MCPError::ConnectionClosed from the transport.
                // This part is tricky as the handler gets Result<Value, MCPError>.
                // The error case for the handler is handled one level up.
            }
        }
    }


    async fn next_request_id(counter_arc: Arc<Mutex<u64>>) -> String {
        let mut counter = counter_arc.lock().await;
        *counter += 1;
        counter.to_string()
    }

    // Changed to take Arc<Mutex<Self>> to allow calling from the instance itself
    pub async fn connect_and_initialize(instance_arc: Arc<Mutex<Self>>, process: StdioProcess) -> Result<(), MCPError> {
        let (pending_requests_clone, server_info_clone, server_capabilities_clone, connection_status_clone, transport_clone, client_capabilities_clone, request_id_counter_clone) = {
            let mut instance = instance_arc.lock().await;
            instance.connection_status = ConnectionStatus::Connecting;
            
            // Clone Arcs for the handler closure
            let pending_requests_for_handler = instance.pending_requests.clone();
            // These are not Arcs yet. We need to decide how to update ServerInfo/Caps from handler.
            // For now, the handler only deals with pending_requests.
            // Let's assume ServerInfo/Caps are updated directly in connect_and_initialize.
            // This requires `handle_incoming_message` to be more sophisticated or part of the instance.
            // For now, the handler will only resolve pending requests.
            
            // Simpler handler that only knows about pending_requests
            let handler_closure = Arc::new(move |msg_result: Result<Value, MCPError>| {
                let pending_req_arc = pending_requests_for_handler.clone();
                // We need to spawn a task here because the Fn trait is not async.
                tokio::spawn(async move {
                    match msg_result {
                        Ok(value) => {
                             // Simplified: Directly call a static-like method or function
                             // that takes the necessary Arcs.
                             // This is where we'd parse `value` into `JsonRpcResponse`
                             // and complete the `oneshot::Sender`.
                            MCPClientInstance::process_incoming_message(
                                pending_req_arc,
                                Arc::new(Mutex::new(None)), // Placeholder, not actually updating server_info from here
                                Arc::new(Mutex::new(None)), // Placeholder
                                Arc::new(Mutex::new(ConnectionStatus::Connecting)), // Placeholder
                                value
                            ).await;
                        }
                        Err(e) => {
                            eprintln!("[MCPClientInstance Handler] Error from transport: {:?}", e);
                            // Potentially signal all pending requests about the error.
                            let mut pending_requests = pending_req_arc.lock().await;
                            for (_, sender) in pending_requests.drain() {
                                // It's tricky to convert MCPError to DomainJsonRpcError directly here.
                                // Let's send a generic error.
                                let _ = sender.send(Err(DomainJsonRpcError{
                                    code: -32000, // Generic error code
                                    message: format!("Transport error: {:?}", e),
                                    data: None,
                                }));
                            }
                            // TODO: Update connection status via an Arc if available to the handler
                        }
                    }
                });
            });

            let mut transport_guard = instance.transport.lock().await;
            transport_guard.register_message_handler(handler_closure).await;
            transport_guard.connect(process).await.map_err(|e| {
                // If connect fails, instance.connection_status needs to be updated.
                // This is why instance_arc is useful.
                // For now, directly returning error. The caller should update status.
                e 
            })?;
            (instance.pending_requests.clone(), Arc::new(Mutex::new(instance.server_info.clone())), Arc::new(Mutex::new(instance.server_capabilities.clone())), Arc::new(Mutex::new(instance.connection_status.clone())), instance.transport.clone(), instance.client_capabilities.clone(), instance.request_id_counter.clone())
        };


        // `initialize` request
        let init_req_id = Self::next_request_id(request_id_counter_clone.clone()).await;
        let initialize_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: json!({ "clientCapabilities": client_capabilities_clone }),
            id: Some(json!(init_req_id.clone())),
        };

        let (tx, rx) = oneshot::channel();
        pending_requests_clone.lock().await.insert(init_req_id, tx);
        
        transport_clone.lock().await.send_request(initialize_request).await?;

        // Wait for the response from the handler
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(response))) => {
                // Successfully received response for initialize
                if let Some(err) = response.error {
                    instance_arc.lock().await.connection_status = ConnectionStatus::Error;
                    return Err(MCPError::InternalError); // Convert from DomainJsonRpcError
                }
                let result = response.result.ok_or_else(|| MCPError::InternalError)?; // "Initialize response missing result"
                
                let server_info_val: ServerInfo = serde_json::from_value(result.get("serverInfo").cloned().ok_or_else(|| MCPError::InternalError)?)
                    .map_err(|_| MCPError::InternalError)?;
                let server_capabilities_val: ServerCapabilities = serde_json::from_value(result.get("serverCapabilities").cloned().ok_or_else(|| MCPError::InternalError)?)
                    .map_err(|_| MCPError::InternalError)?;

                let mut instance = instance_arc.lock().await;
                instance.server_info = Some(server_info_val);
                instance.server_capabilities = Some(server_capabilities_val);
                instance.connection_status = ConnectionStatus::Connected;
                println!("[MCPClientInstance] Successfully connected and initialized. Server: {:?}, Capabilities: {:?}", instance.server_info, instance.server_capabilities);
                Ok(())
            }
            Ok(Ok(Err(e))) => { // Error from oneshot sender (DomainJsonRpcError)
                 instance_arc.lock().await.connection_status = ConnectionStatus::Error;
                 Err(MCPError::InternalError) // Convert: format!("Initialize failed: {:?}", e)
            }
            Ok(Err(_)) => { // oneshot channel closed/recv error
                instance_arc.lock().await.connection_status = ConnectionStatus::Error;
                Err(MCPError::InternalError) // "Initialize oneshot channel closed"
            }
            Err(_) => { // Timeout
                instance_arc.lock().await.connection_status = ConnectionStatus::Error;
                Err(MCPError::TransportIOError("Timeout waiting for initialize response".to_string()))
            }
        }
    }

    pub async fn send_request_internal(instance_arc: Arc<Mutex<Self>>, method: String, params: Value) -> Result<JsonRpcResponse, MCPError> {
        let (config_clone, transport_clone, pending_requests_clone, request_id_counter_clone, current_connection_status) = {
            let instance = instance_arc.lock().await;
            // Check connection status without holding lock for too long
             if instance.connection_status != ConnectionStatus::Connected {
                return Err(MCPError::ConnectionClosed); // Or a more specific "NotConnected"
            }
            (instance.config.clone(), instance.transport.clone(), instance.pending_requests.clone(), instance.request_id_counter.clone(), instance.connection_status.clone())
        };


        let request_id_str = Self::next_request_id(request_id_counter_clone).await;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: Some(json!(request_id_str.clone())),
        };
        
        let (tx, rx) = oneshot::channel();
        pending_requests_clone.lock().await.insert(request_id_str.clone(), tx);

        transport_clone.lock().await.send_request(request).await?;

        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(response))) => { // Successfully received JsonRpcResponse
                if let Some(err) = response.error.clone() { // Check if the response itself is an error response
                    return Err(MCPError::RequestError(err));
                }
                Ok(response)
            }
            Ok(Ok(Err(e))) => Err(MCPError::RequestError(e)), // The oneshot sender sent a JsonRpcError
            Ok(Err(_recv_error)) => { // oneshot channel was closed/dropped before sending
                // This implies the handler failed to send back a response or the instance shut down.
                // Remove the sender as it's no longer valid.
                pending_requests_clone.lock().await.remove(&request_id_str);
                Err(MCPError::InternalError) // Or a more specific error like "Request cancelled or channel closed"
            }
            Err(_timeout_error) => { // Timeout waiting for the oneshot receiver
                // Remove the sender if timed out to prevent stale entries
                pending_requests_clone.lock().await.remove(&request_id_str);
                Err(MCPError::TransportIOError("Timeout waiting for response".to_string()))
            }
        }
    }

    pub async fn send_notification_internal(instance_arc: Arc<Mutex<Self>>, method: String, params: Value) -> Result<(), MCPError> {
        let (transport_clone, current_connection_status) = {
            let instance = instance_arc.lock().await;
             if instance.connection_status != ConnectionStatus::Connected {
                return Err(MCPError::ConnectionClosed);
            }
            (instance.transport.clone(), instance.connection_status.clone())
        };


        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: None, 
        };

        transport_clone.lock().await.send_notification(notification).await
    }

    pub async fn shutdown(instance_arc: Arc<Mutex<Self>>) -> Result<(), MCPError> {
        let mut instance_guard = instance_arc.lock().await;
        
        // Check if already disconnected or shutting down
        if instance_guard.connection_status == ConnectionStatus::Disconnected {
            println!("[MCPClientInstance] Already disconnected or shutdown in progress.");
            return Ok(());
        }

        println!("[MCPClientInstance] Initiating shutdown...");
        instance_guard.connection_status = ConnectionStatus::Disconnected; // Set status immediately

        // Clear any pending requests, signalling them with an error
        let mut pending_requests = instance_guard.pending_requests.lock().await;
        for (id, sender) in pending_requests.drain() {
            println!("[MCPClientInstance] Cancelling pending request ID: {}", id);
            let _ = sender.send(Err(DomainJsonRpcError {
                code: -32001, 
                message: "Connection shut down by client".to_string(),
                data: None,
            }));
        }
        drop(pending_requests); // Release lock before calling transport disconnect

        // Disconnect transport
        // No need to clone transport here, we have instance_guard
        let transport_to_disconnect = instance_guard.transport.clone();
        drop(instance_guard); // Release lock on instance before await on disconnect

        transport_to_disconnect.lock().await.disconnect().await?;
        
        println!("[MCPClientInstance] Shutdown sequence complete.");
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
