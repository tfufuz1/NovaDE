use crate::ai::mcp::types::{ // Updated path
    MCPServerConfig, ClientCapabilities, ServerCapabilities, ServerInfo, ConnectionStatus,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError as DomainJsonRpcError, MCPError,
};
use crate::ai::mcp::transport::IMCPTransport; // Updated path
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
            protocol_version: None,
            connection_status: ConnectionStatus::Disconnected,
            last_error: None,
            transport,
            request_id_counter: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }))
    }
    
    async fn process_incoming_message(
        pending_requests_arc: Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, DomainJsonRpcError>>>>>,
        // Remove direct update of server_info, server_capabilities, connection_status from here.
        // Those are handled by the main task that initiates requests like 'initialize'.
        // This handler's job is to route responses/notifications.
        message_value: Value,
    ) {
        println!("[MCPClientInstance] Processing incoming message: {:?}", message_value);
        match serde_json::from_value::<JsonRpcResponse>(message_value.clone()) {
            Ok(response) => {
                if let Some(id_val) = response.id.as_str().or_else(|| response.id.as_u64().map(|u| u.to_string())).or_else(|| response.id.as_i64().map(|i| i.to_string())) {
                    let mut pending_requests = pending_requests_arc.lock().await;
                    if let Some(sender) = pending_requests.remove(&id_val) {
                        // If the response itself contains a JsonRpcError, send that. Otherwise send the whole response.
                        if let Some(rpc_err) = response.error.clone() {
                             if sender.send(Err(rpc_err)).is_err() {
                                eprintln!("[MCPClientInstance] Failed to send error response to waiting task for ID: {}", id_val);
                            }
                        } else {
                            if sender.send(Ok(response)).is_err() {
                                eprintln!("[MCPClientInstance] Failed to send response to waiting task for ID: {}", id_val);
                            }
                        }
                    } else {
                        eprintln!("[MCPClientInstance] Received response for unknown request ID: {}", id_val);
                    }
                } else if response.id.is_null() && response.error.is_some() {
                     eprintln!("[MCPClientInstance] Received a server error notification (no ID): {:?}", response.error);
                } else if response.id.is_null() && response.method.is_some() {
                     println!("[MCPClientInstance] Received server notification: method='{}', params='{:?}'", response.method.as_ref().unwrap_or(&String::new()), response.params);
                     // TODO: Implement server-initiated notification handling if required by MCP spec.
                } else {
                     eprintln!("[MCPClientInstance] Received message with unhandled ID format or structure: {:?}", response);
                }
            }
            Err(e) => {
                eprintln!("[MCPClientInstance] Failed to parse incoming message as JsonRpcResponse: {}. Message: {:?}", e, message_value);
            }
        }
    }

    async fn next_request_id(counter_arc: Arc<Mutex<u64>>) -> String {
        let mut counter = counter_arc.lock().await;
        *counter += 1;
        counter.to_string() // JSON-RPC IDs can be numbers or strings. String is safer for general map keys.
    }

    pub async fn connect_and_initialize(instance_arc: Arc<Mutex<Self>>, process: StdioProcess) -> Result<(), MCPError> {
        let client_capabilities_clone = { // Scope to release lock quickly
            let mut instance_guard = instance_arc.lock().await;
            instance_guard.connection_status = ConnectionStatus::Connecting;
            instance_guard.last_error = None; // Clear previous errors

            let pending_requests_for_handler = instance_guard.pending_requests.clone();
            // The handler closure now only needs access to pending_requests.
            let handler_closure = Arc::new(move |msg_result: Result<Value, MCPError>| {
                let pending_req_arc = pending_requests_for_handler.clone();
                tokio::spawn(async move { // Spawn a task to handle the message
                    match msg_result {
                        Ok(value) => {
                            MCPClientInstance::process_incoming_message(pending_req_arc, value).await;
                        }
                        Err(e) => { // Transport level error
                            eprintln!("[MCPClientInstance Handler] Error from transport: {:?}", e);
                            let mut pending_requests = pending_req_arc.lock().await;
                            for (_, sender) in pending_requests.drain() {
                                let _ = sender.send(Err(DomainJsonRpcError {
                                    code: -32002, // Custom code for transport/unhandled errors
                                    message: format!("Transport error: {:?}", e),
                                    data: None,
                                }));
                            }
                            // TODO: Need a way to set instance_arc's connection_status to Error here.
                            // This might involve passing an Arc<Mutex<ConnectionStatus>> to the handler,
                            // or using a Weak<Mutex<MCPClientInstance>> to call a method on it.
                        }
                    }
                });
            });
            
            let mut transport_guard = instance_guard.transport.lock().await;
            transport_guard.register_message_handler(handler_closure).await;
            
            // Attempt to connect the transport
            if let Err(e) = transport_guard.connect(process).await {
                instance_guard.connection_status = ConnectionStatus::Error;
                instance_guard.last_error = Some(e.clone());
                return Err(e);
            }
            instance_guard.client_capabilities.clone() // Return client_capabilities for the initialize request
        };

        // Perform the initialize request using send_request_internal
        let initialize_params = json!({ "clientCapabilities": client_capabilities_clone });
        match MCPClientInstance::send_request_internal(instance_arc.clone(), "initialize".to_string(), initialize_params).await {
            Ok(response) => {
                // send_request_internal already checks for response.error and returns MCPError::RequestError
                // So if we get Ok(response) here, response.error should be None.
                let result = response.result.ok_or_else(|| {
                    let e = MCPError::InternalError; // "Initialize response missing result"
                    // Update status via lock
                    tokio::spawn(async move {
                        let mut instance_guard = instance_arc.lock().await;
                        instance_guard.connection_status = ConnectionStatus::Error;
                        instance_guard.last_error = Some(e.clone());
                    });
                    e
                })?;
                
                let server_info_val: ServerInfo = serde_json::from_value(
                    result.get("serverInfo").cloned().ok_or_else(|| MCPError::JsonRpcParseError("Missing serverInfo in initialize response".to_string()))?
                ).map_err(|e| MCPError::JsonRpcParseError(format!("Failed to parse ServerInfo: {}", e)))?;
                
                let server_capabilities_val: ServerCapabilities = serde_json::from_value(
                    result.get("serverCapabilities").cloned().ok_or_else(|| MCPError::JsonRpcParseError("Missing serverCapabilities in initialize response".to_string()))?
                ).map_err(|e| MCPError::JsonRpcParseError(format!("Failed to parse ServerCapabilities: {}", e)))?;

                let mut instance = instance_arc.lock().await;
                instance.server_info = Some(server_info_val.clone());
                instance.server_capabilities = Some(server_capabilities_val);
                instance.protocol_version = server_info_val.protocol_version;
                instance.connection_status = ConnectionStatus::Connected;
                instance.last_error = None;
                println!("[MCPClientInstance] Successfully connected and initialized. Server: {:?}, Capabilities: {:?}", instance.server_info, instance.server_capabilities);
                Ok(())
            }
            Err(e) => {
                let mut instance = instance_arc.lock().await;
                instance.connection_status = ConnectionStatus::Error;
                instance.last_error = Some(e.clone());
                Err(e)
            }
        }
    }

    pub async fn send_request_internal(instance_arc: Arc<Mutex<Self>>, method: String, params: Value) -> Result<JsonRpcResponse, MCPError> {
        let (transport_clone, pending_requests_clone, request_id_counter_clone) = {
            let instance_guard = instance_arc.lock().await;
             if instance_guard.connection_status != ConnectionStatus::Connected {
                let err = MCPError::ConnectionClosed; 
                // Set last_error only if we can get a mutable guard, which we have here.
                // However, this function is often called without holding the instance lock for its whole duration.
                // So, last_error update for this case should be handled by the caller if needed.
                // For now, just return error. If caller is another method of MCPClientInstance, it can update last_error.
                return Err(err);
            }
            (instance_guard.transport.clone(), instance_guard.pending_requests.clone(), instance_guard.request_id_counter.clone())
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

        // Send the request. If this fails, update last_error.
        if let Err(e) = transport_clone.lock().await.send_request(request).await {
            // Remove the pending request as it couldn't be sent.
            pending_requests_clone.lock().await.remove(&request_id_str);
            // Update last_error on the instance
            // This requires getting the lock again, which is fine as it's not held long.
            instance_arc.lock().await.last_error = Some(e.clone());
            return Err(e);
        }
        
        // Wait for the response from the handler
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(response))) => { // Successfully received JsonRpcResponse
                if let Some(err) = response.error.clone() { // Check if the response itself is an error response
                    let mcp_err = MCPError::RequestError(err);
                    instance_arc.lock().await.last_error = Some(mcp_err.clone());
                    return Err(mcp_err);
                }
                instance_arc.lock().await.last_error = None; // Clear error on success
                Ok(response)
            }
            Ok(Ok(Err(e))) => { // The oneshot sender sent a JsonRpcError
                let mcp_err = MCPError::RequestError(e);
                instance_arc.lock().await.last_error = Some(mcp_err.clone());
                Err(mcp_err)
            }
            Ok(Err(_recv_error)) => { // oneshot channel was closed/dropped before sending
                pending_requests_clone.lock().await.remove(&request_id_str);
                let mcp_err = MCPError::InternalError; // Or "Request cancelled or channel closed"
                instance_arc.lock().await.last_error = Some(mcp_err.clone());
                Err(mcp_err) 
            }
            Err(_timeout_error) => { // Timeout waiting for the oneshot receiver
                pending_requests_clone.lock().await.remove(&request_id_str);
                let mcp_err = MCPError::TransportIOError("Timeout waiting for response".to_string());
                instance_arc.lock().await.last_error = Some(mcp_err.clone());
                Err(mcp_err)
            }
        }
    }

    pub async fn send_notification_internal(instance_arc: Arc<Mutex<Self>>, method: String, params: Value) -> Result<(), MCPError> {
        let transport_clone = {
            let instance = instance_arc.lock().await;
             if instance.connection_status != ConnectionStatus::Connected {
                let err = MCPError::ConnectionClosed;
                // instance.last_error = Some(err.clone()); // Requires mut instance
                return Err(err);
            }
            instance.transport.clone()
        };

        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: None, 
        };
        
        if let Err(e) = transport_clone.lock().await.send_notification(notification).await {
            instance_arc.lock().await.last_error = Some(e.clone());
            return Err(e);
        }
        instance_arc.lock().await.last_error = None; // Clear error on success
        Ok(())
    }

    pub async fn shutdown(instance_arc: Arc<Mutex<Self>>) -> Result<(), MCPError> {
        let mut instance_guard = instance_arc.lock().await;
        
        if instance_guard.connection_status == ConnectionStatus::Disconnected {
            println!("[MCPClientInstance] Already disconnected or shutdown in progress.");
            return Ok(());
        }

        println!("[MCPClientInstance] Initiating shutdown...");
        instance_guard.connection_status = ConnectionStatus::Disconnected; 
        instance_guard.last_error = None; // Clear error on shutdown initiation

        let mut pending_requests = instance_guard.pending_requests.lock().await;
        for (id, sender) in pending_requests.drain() {
            println!("[MCPClientInstance] Cancelling pending request ID: {}", id);
            let _ = sender.send(Err(DomainJsonRpcError {
                code: -32001, 
                message: "Connection shut down by client".to_string(),
                data: None,
            }));
        }
        drop(pending_requests); 

        let transport_to_disconnect = instance_guard.transport.clone();
        drop(instance_guard); 

        if let Err(e) = transport_to_disconnect.lock().await.disconnect().await {
            // Even if disconnect fails, we are already in Disconnected state.
            // Log the error but don't change state back.
            eprintln!("[MCPClientInstance] Error during transport disconnect: {:?}", e);
            // Optionally, store this error if there's a way to surface it post-shutdown.
            // For now, the primary outcome is already Disconnected.
            return Err(e); // Propagate that disconnect itself had an issue
        }
        
        println!("[MCPClientInstance] Shutdown sequence complete.");
        Ok(())
    }

    // Getter methods for status and capabilities
    // These should take &self and lock the instance_arc if called from outside.
    // If MCPClientInstance is always accessed via Arc<Mutex<Self>>, these getters
    // might be better on the wrapper or require passing the Arc.
    // For now, assuming they are called when a lock is already held or are for internal use.
    // If they need to be part of public API for an Arc<Mutex<MCPClientInstance>>,
    // they would need to be static methods taking the Arc, or methods on a wrapper struct.
    // Let's assume they are for internal use for now or called on a locked guard.
    //
    // To make them callable on an `Arc<Mutex<Self>>` from outside, they would look like:
    // pub async fn get_connection_status_arc(instance_arc: Arc<Mutex<Self>>) -> ConnectionStatus {
    //     instance_arc.lock().await.connection_status.clone()
    // }
    // But the current signature is `&self`.
    // This means they are intended to be called like `instance_guard.get_connection_status()`.

    pub fn get_connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn get_server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    pub fn get_server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    pub fn get_protocol_version(&self) -> Option<&String> {
        self.protocol_version.as_ref()
    }

    pub fn get_last_error(&self) -> Option<&MCPError> {
        self.last_error.as_ref()
    }
}
