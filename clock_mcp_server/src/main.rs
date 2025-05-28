use anyhow::{anyhow, Context, Result};
use chrono::Local; // Removed Timelike
use serde::{Deserialize, Serialize};
use serde_json::Value;
// Removed HashMap
use std::io::{self, BufRead}; // Write will be from tokio::io::AsyncWriteExt
use std::sync::Arc; // Mutex will be tokio::sync::Mutex
use std::time::Duration;
use tokio::sync::{mpsc, Mutex}; // Changed to tokio::sync::Mutex
use tokio::time;
use tokio::io::AsyncWriteExt; // For async write operations
use std::fmt; // Added for Display trait
use tracing::info; // Added for consistency with power_mcp_server

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error (code: {}): {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

impl JsonRpcError {
    fn new(code: i32, message: String) -> Self {
        JsonRpcError {
            code,
            message,
            data: None,
        }
    }
    fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request".to_string())
    }
    fn method_not_found() -> Self {
        Self::new(-32601, "Method not found".to_string())
    }
    fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params".to_string())
    }
    fn internal_error() -> Self {
        Self::new(-32603, "Internal error".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TimeResourceContent {
    timestamp: i64,
    formatted_string: String,
    timezone: String,
}

fn get_current_time_resource_content() -> TimeResourceContent {
    let now = Local::now();
    TimeResourceContent {
        timestamp: now.timestamp(),
        formatted_string: now.format("%H:%M:%S").to_string(),
        timezone: now.offset().to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerCapabilities {
    resources: ServerResourceCapabilities,
    tools: Value,    // Empty for now
    prompts: Value, // Empty for now
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerResourceCapabilities {
    subscribe: bool,
    list: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeResult {
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
    #[serde(rename = "serverCapabilities")]
    server_capabilities: ServerCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceDescriptor {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResourcesResult {
    resources: Vec<ResourceDescriptor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadParams {
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeParams {
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnsubscribeParams {
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionStatus {
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationParams {
    uri: String,
    content: TimeResourceContent,
}

const TIME_RESOURCE_URI: &str = "time/current_formatted";

struct ServerState {
    subscribed_to_time: bool,
    time_broadcaster_handle: Option<tokio::task::JoinHandle<()>>,
    // Channel to signal the time broadcaster to stop
    time_broadcaster_stop_tx: Option<mpsc::Sender<()>>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            subscribed_to_time: false,
            time_broadcaster_handle: None,
            time_broadcaster_stop_tx: None,
        }
    }
}

async fn handle_request(
    req: JsonRpcRequest,
    state_arc: Arc<Mutex<ServerState>>, // Renamed for clarity
    writer: &mut (impl AsyncWriteExt + Unpin), // Changed to AsyncWriteExt
) -> Result<Option<bool>> { 
    let req_id = req.id.clone().unwrap_or(Value::Null); // Ensure req_id is available

    // Wrap the core logic to map errors to JsonRpcError
    let result_value: Result<Value, JsonRpcError> = (async {
        match req.method.as_str() {
            "initialize" => {
                info!("Handling 'initialize' request: {:?}", req.params);
                let result = InitializeResult {
                    server_info: ServerInfo {
                        name: "ClockMCPServer".to_string(),
                        version: "0.1.0".to_string(),
                    },
                    server_capabilities: ServerCapabilities {
                        resources: ServerResourceCapabilities {
                            subscribe: true,
                            list: true,
                        },
                        tools: serde_json::json!({}),
                        prompts: serde_json::json!({}),
                    },
                };
                serde_json::to_value(result).map_err(|e| JsonRpcError::new(-32000, format!("Serialization error: {}", e)))
            }
            "resources/list" => {
                info!("Handling 'resources/list' request: {:?}", req.params);
                let result = ListResourcesResult {
                    resources: vec![ResourceDescriptor {
                        uri: TIME_RESOURCE_URI.to_string(),
                        name: "Current Time".to_string(),
                        description: "Provides the current formatted time, subscribable.".to_string(),
                        mime_type: "application/json".to_string(),
                    }],
                };
                serde_json::to_value(result).map_err(|e| JsonRpcError::new(-32000, format!("Serialization error: {}", e)))
            }
            "resources/read" => {
                info!("Handling 'resources/read' request: {:?}", req.params);
                let params: ReadParams = req.params.clone()
                    .ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                if params.uri == TIME_RESOURCE_URI {
                    let content = get_current_time_resource_content();
                    serde_json::to_value(content).map_err(|e| JsonRpcError::new(-32000, format!("Serialization error: {}", e)))
                } else {
                    Err(JsonRpcError::new(-32001, format!("Resource not found: {}", params.uri)))
                }
            }
            "resources/subscribe" => {
                info!("Handling 'resources/subscribe' request: {:?}", req.params);
                let params: SubscribeParams = req.params.clone()
                    .ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                if params.uri == TIME_RESOURCE_URI {
                    let mut server_state = state_arc.lock().await; // Changed to .await
                    if !server_state.subscribed_to_time {
                        server_state.subscribed_to_time = true;
                        info!("Subscribed to {}. Starting time broadcaster.", TIME_RESOURCE_URI);

                        let (stop_tx, mut stop_rx) = mpsc::channel(1);
                        server_state.time_broadcaster_stop_tx = Some(stop_tx);
                        
                        // Each notification will acquire lock on stdout. Consider a dedicated channel to a single writer task if contention is an issue.
                        let stdout_writer_mutex = Arc::new(Mutex::new(tokio::io::stdout()));


                        server_state.time_broadcaster_handle = Some(tokio::spawn({
                            let writer_mutex_clone = Arc::clone(&stdout_writer_mutex);
                            async move {
                                let mut interval = time::interval(Duration::from_secs(1));
                                loop {
                                    tokio::select! {
                                        _ = interval.tick() => {
                                            let content = get_current_time_resource_content();
                                            let notification_params = NotificationParams {
                                                uri: TIME_RESOURCE_URI.to_string(),
                                                content,
                                            };
                                            let notification = JsonRpcNotification {
                                                jsonrpc: "2.0".to_string(),
                                                method: "notifications/resources/updated".to_string(),
                                                params: match serde_json::to_value(&notification_params) {
                                                    Ok(val) => Some(val),
                                                    Err(e) => {
                                                        eprintln!("[Broadcaster] Error serializing notification params: {}", e);
                                                        continue; // Skip this notification
                                                    }
                                                }
                                            };
                                            
                                            match serde_json::to_string(&notification) {
                                                Ok(json_notification) => {
                                                    let mut w = writer_mutex_clone.lock().await; // .await for tokio::sync::Mutex
                                                    if let Err(e) = w.write_all(format!("{}\n", json_notification).as_bytes()).await {
                                                        eprintln!("[Broadcaster] Error writing notification: {}", e);
                                                        break; 
                                                    }
                                                    if let Err(e) = w.flush().await {
                                                        eprintln!("[Broadcaster] Error flushing stdout for notification: {}", e);
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("[Broadcaster] Error serializing full notification: {}", e);
                                                }
                                            }
                                        }
                                        _ = stop_rx.recv() => {
                                            info!("[Broadcaster] Received stop signal. Exiting.");
                                            break;
                                        }
                                    }
                                }
                            }
                        }));
                    }
                    serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })
                        .map_err(|e| JsonRpcError::new(-32000, format!("Serialization error: {}", e)))
                } else {
                    Err(JsonRpcError::new(-32002, format!("Cannot subscribe to unknown resource: {}", params.uri)))
                }
            }
            "resources/unsubscribe" => {
                info!("Handling 'resources/unsubscribe' request: {:?}", req.params);
                let params: UnsubscribeParams = req.params.clone()
                    .ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                if params.uri == TIME_RESOURCE_URI {
                    let mut server_state = state_arc.lock().await; // Changed to .await
                    if server_state.subscribed_to_time {
                        server_state.subscribed_to_time = false;
                        if let Some(stop_tx) = server_state.time_broadcaster_stop_tx.take() {
                            info!("Unsubscribed from {}. Stopping time broadcaster.", TIME_RESOURCE_URI);
                            // Best effort send, ignore error if receiver is already dropped
                            let _ = stop_tx.try_send(()); 
                        }
                        if let Some(handle) = server_state.time_broadcaster_handle.take() {
                            // Wait for the task to complete, with a timeout
                            match tokio::time::timeout(Duration::from_millis(100), handle).await {
                                Ok(Ok(_)) => info!("Time broadcaster task joined successfully."),
                                Ok(Err(e)) => eprintln!("Error joining time broadcaster task: {:?}", e),
                                Err(_) => eprintln!("Timeout waiting for time broadcaster task to join."),
                            }
                        }
                    }
                    serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })
                        .map_err(|e| JsonRpcError::new(-32000, format!("Serialization error: {}", e)))
                } else {
                    Err(JsonRpcError::new(-32003, format!("Cannot unsubscribe from unknown resource: {}", params.uri)))
                }
            }
            "shutdown" => {
                info!("Handling 'shutdown' request.");
                let mut server_state = state_arc.lock().await; // Changed to .await
                if server_state.subscribed_to_time {
                    if let Some(stop_tx) = server_state.time_broadcaster_stop_tx.take() {
                        info!("Shutting down: Stopping time broadcaster.");
                        let _ = stop_tx.try_send(());
                    }
                    if let Some(handle) = server_state.time_broadcaster_handle.take() {
                         match tokio::time::timeout(Duration::from_millis(100), handle).await {
                            Ok(Ok(_)) => info!("Time broadcaster task joined successfully during shutdown."),
                            Ok(Err(e)) => eprintln!("Error joining time broadcaster task during shutdown: {:?}", e),
                            Err(_) => eprintln!("Timeout waiting for time broadcaster task to join during shutdown."),
                        }
                    }
                    server_state.subscribed_to_time = false;
                }
                // Per MCP spec, shutdown should make server stop accepting new requests.
                // Here we just send response. The main loop will exit based on return from handle_request.
                Ok(Value::Null) 
            }
            "exit" => {
                info!("Handling 'exit' notification. Server will terminate.");
                // No response for 'exit'. Signal main loop to exit.
                // This is achieved by returning a specific error that handle_request interprets.
                Err(JsonRpcError::new(0, "exit_signal".to_string())) // Special code/message
            }
            _ => {
                Err(JsonRpcError::method_not_found())
            }
        }
    })
    .await;

    let response = match result_value {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(value),
            error: None,
            id: req_id,
        },
        Err(err) => {
            if err.message == "exit_signal" { // Check for our special signal
                return Ok(Some(true)); // Signal exit, no response to send
            }
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(err),
                id: req_id,
            }
        }
    };

    let json_response = serde_json::to_string(&response)
        .map_err(|e| anyhow!("FATAL: Failed to serialize response: {}", e))?; // This error is critical
    writer.write_all(format!("{}\n", json_response).as_bytes()).await?;
    writer.flush().await?;

    if req.method == "shutdown" {
        return Ok(Some(true)); // Also signal exit after responding to shutdown
    }

    Ok(Some(false)) 
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("clock_mcp_server=info".parse()?))
        .init();

    info!("Clock MCP Server starting..."); // Changed eprintln to info
    let stdin = tokio::io::stdin(); // Use tokio's stdin
    let mut reader = tokio::io::BufReader::new(stdin); // Tokio BufReader
    let mut stdout = tokio::io::stdout(); // Tokio stdout

    let server_state = Arc::new(Mutex::new(ServerState::new()));

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await { // .await for async read
            Ok(0) => {
                info!("EOF received, exiting.");
                break; 
            }
            Ok(_) => {
                info!("Received line: {}", line.trim());
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&line);

                match request {
                    Ok(req) => {
                        let req_id_for_error = req.id.clone().unwrap_or(Value::Null);
                        match handle_request(req, Arc::clone(&server_state), &mut stdout).await {
                            Ok(Some(true)) => {
                                info!("Exit signal received, server shutting down.");
                                break;
                            }
                            Ok(Some(false)) | Ok(None) => {
                                // Continue or notification handled (None case for future if handle_request changes)
                            }
                            Err(e) => { // Errors from handle_request (like serialization or critical IO)
                                eprintln!("Critical error in handle_request: {:?}", e);
                                // Attempt to send a generic internal error if possible, then break.
                                let error_response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError::internal_error()),
                                    id: req_id_for_error,
                                };
                                if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                                    if stdout.write_all(format!("{}\n", json_err_res).as_bytes()).await.is_err() || stdout.flush().await.is_err() {
                                        eprintln!("Fatal: Could not write final error response to stdout.");
                                    }
                                }
                                break; // Break on any error from handler
                            }
                        }
                    }
                    Err(e) => { // JSON parsing error of the request itself
                        info!("Failed to parse JSON request: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::invalid_request()),
                            id: Value::Null, 
                        };
                        if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                           if stdout.write_all(format!("{}\n", json_err_res).as_bytes()).await.is_err() || stdout.flush().await.is_err() {
                                eprintln!("Fatal: Could not write parsing error response to stdout.");
                                break; 
                           }
                        } else {
                             eprintln!("Fatal: Could not serialize parsing error response itself.");
                             break;
                        }
                    }
                }
            }
            Err(e) => {
                info!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    info!("Main loop ended. Cleaning up broadcaster...");
    let mut state = server_state.lock().await; // .await for tokio Mutex
    if state.subscribed_to_time {
        if let Some(stop_tx) = state.time_broadcaster_stop_tx.take() {
            info!("Exiting: Attempting to stop time broadcaster.");
            let _ = stop_tx.try_send(()); 
        }
        if let Some(handle) = state.time_broadcaster_handle.take() {
            info!("Exiting: Waiting for time broadcaster to finish.");
            match tokio::time::timeout(Duration::from_secs(1), handle).await {
                Ok(Ok(_)) => info!("Time broadcaster task joined successfully upon exit."),
                Ok(Err(e)) => eprintln!("Error joining time broadcaster task on exit: {:?}", e),
                Err(_) => eprintln!("Timeout waiting for time broadcaster task on exit."),
            }
        }
    }
    info!("Clock MCP Server shutting down.");
=======
            eprintln!("Handling 'initialize' request: {:?}", req.params);
            let result = InitializeResult {
                server_info: ServerInfo {
                    name: "ClockMCPServer".to_string(),
                    version: "0.1.0".to_string(),
                },
                server_capabilities: ServerCapabilities {
                    resources: ServerResourceCapabilities {
                        subscribe: true,
                        list: true,
                    },
                    tools: serde_json::json!({}),
                    prompts: serde_json::json!({}),
                },
            };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "resources/list" => {
            eprintln!("Handling 'resources/list' request: {:?}", req.params);
            let result = ListResourcesResult {
                resources: vec![ResourceDescriptor {
                    uri: TIME_RESOURCE_URI.to_string(),
                    name: "Current Time".to_string(),
                    description: "Provides the current formatted time, subscribable.".to_string(),
                    mime_type: "application/json".to_string(),
                }],
            };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "resources/read" => {
            eprintln!("Handling 'resources/read' request: {:?}", req.params);
            let params: ReadParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/read: {}", e))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())), // This will be an anyhow::Error
            };

            if params.uri == TIME_RESOURCE_URI {
                let content = get_current_time_resource_content();
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::to_value(content)?),
                    error: None,
                    id: req_id.unwrap_or(Value::Null),
                })
            } else {
                 Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::new(-32000, format!("Resource not found: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                })
            }
        }
        "resources/subscribe" => {
            eprintln!("Handling 'resources/subscribe' request: {:?}", req.params);
            let params: SubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/subscribe: {}", e))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())), // This will be an anyhow::Error
            };

            if params.uri == TIME_RESOURCE_URI {
                let mut server_state = state.lock().unwrap();
                if !server_state.subscribed_to_time {
                    server_state.subscribed_to_time = true;
                    eprintln!("Subscribed to {}. Starting time broadcaster.", TIME_RESOURCE_URI);

                    let (stop_tx, mut stop_rx) = mpsc::channel(1);
                    server_state.time_broadcaster_stop_tx = Some(stop_tx);

                    let writer_mutex = Arc::new(Mutex::new(io::stdout()));

                    server_state.time_broadcaster_handle = Some(tokio::spawn({
                        let writer_mutex = Arc::clone(&writer_mutex);
                        async move {
                            let mut interval = time::interval(Duration::from_secs(1));
                            loop {
                                tokio::select! {
                                    _ = interval.tick() => {
                                        let content = get_current_time_resource_content();
                                        let notification = JsonRpcNotification {
                                            jsonrpc: "2.0".to_string(),
                                            method: "notifications/resources/updated".to_string(),
                                            params: Some(serde_json::to_value(NotificationParams {
                                                uri: TIME_RESOURCE_URI.to_string(),
                                                content,
                                            }).unwrap()), // Should handle error better
                                        };
                                        match serde_json::to_string(&notification) {
                                            Ok(json_notification) => {
                                                let mut w = writer_mutex.lock().unwrap();
                                                if let Err(e) = writeln!(w, "{}", json_notification) {
                                                    eprintln!("Error writing notification: {}", e);
                                                    break; 
                                                }
                                                if let Err(e) = w.flush() {
                                                    eprintln!("Error flushing stdout for notification: {}", e);
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Error serializing notification: {}", e);
                                            }
                                        }
                                    }
                                    _ = stop_rx.recv() => {
                                        eprintln!("Time broadcaster received stop signal. Exiting.");
                                        break;
                                    }
                                }
                            }
                        }
                    }));
                }
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })?),
                    error: None,
                    id: req_id.unwrap_or(Value::Null),
                })
            } else {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::new(-32000, format!("Cannot subscribe to unknown resource: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                })
            }
        }
        "resources/unsubscribe" => {
            eprintln!("Handling 'resources/unsubscribe' request: {:?}", req.params);
             let params: UnsubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/unsubscribe: {}", e))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())), // This will be an anyhow::Error
            };

            if params.uri == TIME_RESOURCE_URI {
                let mut server_state = state.lock().unwrap();
                if server_state.subscribed_to_time {
                    server_state.subscribed_to_time = false;
                    if let Some(stop_tx) = server_state.time_broadcaster_stop_tx.take() {
                        eprintln!("Unsubscribed from {}. Stopping time broadcaster.", TIME_RESOURCE_URI);
                        if let Err(e) = stop_tx.send(()).await {
                             eprintln!("Failed to send stop signal to time broadcaster: {}",e);
                        }
                    }
                    if let Some(handle) = server_state.time_broadcaster_handle.take() {
                        if let Err(e) = handle.await {
                            eprintln!("Error joining time broadcaster task: {:?}", e);
                        }
                    }
                }
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })?),
                    error: None,
                    id: req_id.unwrap_or(Value::Null),
                })
            } else {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::new(-32000, format!("Cannot unsubscribe from unknown resource: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                })
            }
        }
        "shutdown" => {
            eprintln!("Handling 'shutdown' request.");
            // Perform any cleanup if necessary
            let mut server_state = state.lock().unwrap();
            if server_state.subscribed_to_time {
                 if let Some(stop_tx) = server_state.time_broadcaster_stop_tx.take() {
                    eprintln!("Shutting down: Stopping time broadcaster.");
                     if let Err(e) = stop_tx.send(()).await {
                        eprintln!("Failed to send stop signal to time broadcaster during shutdown: {}",e);
                    }
                }
                if let Some(handle) = server_state.time_broadcaster_handle.take() {
                     if let Err(e) = handle.await {
                        eprintln!("Error joining time broadcaster task during shutdown: {:?}", e);
                    }
                }
                server_state.subscribed_to_time = false;
            }

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::Null), // Or some specific shutdown ack
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "exit" => {
            eprintln!("Handling 'exit' notification. Server will terminate.");
            // No response for 'exit' as it's a notification.
            // Signal the main loop to terminate.
            return Ok(Some(true)); // Signal exit
        }
        _ => {
            eprintln!("Unknown method: {}", req.method);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::method_not_found()),
                id: req_id.unwrap_or(Value::Null),
            })
        }
    };

    if let Some(res) = response {
        let json_response = serde_json::to_string(&res)
            .context("Failed to serialize response")?;
        writeln!(writer, "{}", json_response)?;
        writer.flush()?;
    }
    Ok(Some(false)) // Continue server
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("clock_mcp_server=info".parse()?))
        .init();

    eprintln!("Clock MCP Server starting...");
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    let server_state = Arc::new(Mutex::new(ServerState::new()));

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                eprintln!("EOF received, exiting.");
                break; // EOF
            }
            Ok(_) => {
                eprintln!("Received line: {}", line.trim());
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&line);

                match request {
                    Ok(req) => {
                        let req_id_for_error = req.id.clone().unwrap_or(Value::Null);
                        match handle_request(req, Arc::clone(&server_state), &mut stdout).await {
                            Ok(Some(true)) => {
                                eprintln!("Exit signal received, server shutting down.");
                                break;
                            }
                            Ok(Some(false)) => {
                                // Continue
                            }
                            Ok(None) => {
                                // Notification handled, no response to send (should not happen with current logic)
                            }
                            Err(e) => {
                                eprintln!("Error handling request: {:?}", e);
                                let error_response = if let Some(json_err) = e.downcast_ref::<JsonRpcError>() {
                                    JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError{code: json_err.code, message: json_err.message.clone(), data: json_err.data.clone() }),
                                        id: req_id_for_error,
                                    }
                                } else {
                                     JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError::internal_error()),
                                        id: req_id_for_error,
                                    }
                                };
                                if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                                    if let Err(write_err) = writeln!(stdout, "{}", json_err_res) {
                                         eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
                                         break;
                                    }
                                    if let Err(flush_err) = stdout.flush(){
                                        eprintln!("Fatal: Could not flush stdout for error response: {}", flush_err);
                                        break;
                                    }
                                } else {
                                     eprintln!("Fatal: Could not serialize error response");
                                     break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON request: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::invalid_request()),
                            id: Value::Null, // No ID if request parsing failed badly
                        };
                        if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                           if let Err(write_err) = writeln!(stdout, "{}", json_err_res) {
                                eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
                                break;
                           }
                            if let Err(flush_err) = stdout.flush(){
                                eprintln!("Fatal: Could not flush stdout for error response: {}", flush_err);
                                break;
                            }
                        } else {
                             eprintln!("Fatal: Could not serialize error response for invalid request");
                             break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    // Ensure broadcaster is stopped if loop breaks for other reasons
    let mut state = server_state.lock().unwrap();
    if state.subscribed_to_time {
        if let Some(stop_tx) = state.time_broadcaster_stop_tx.take() {
            eprintln!("Exiting: Attempting to stop time broadcaster.");
            let _ = stop_tx.try_send(()); // try_send is non-blocking
        }
        if let Some(handle) = state.time_broadcaster_handle.take() {
            eprintln!("Exiting: Waiting for time broadcaster to finish.");
            if let Err(e) = handle.await {
                 eprintln!("Error joining time broadcaster task on exit: {:?}", e);
            }
        }
    }
    eprintln!("Clock MCP Server shutting down.");
    Ok(())
}
