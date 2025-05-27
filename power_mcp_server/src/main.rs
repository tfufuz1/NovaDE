use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write}; // BufRead will be via tokio::io::AsyncBufReadExt
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, Mutex}; // Mutex for shared state like subscription status
use tokio::time;
use tracing::info;

// Assuming novade_domain provides these. Adjust if names are different.
use novade_domain::power_management::{
    DefaultPowerManagementService, PowerManagementService, BatteryInfo as DomainBatteryInfo, PowerState as DomainPowerState,
    PowerCapabilities as DomainPowerCapabilities,
};

// --- JSON-RPC Structures (Consider moving to a common crate later) ---
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

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

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

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC Error (code: {}): {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

// --- MCP Specific Structures ---

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerCapabilities {
    resources: ServerResourceCapabilities,
    tools: ServerToolCapabilities,
    prompts: Value, // Empty for now
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerResourceCapabilities {
    subscribe: Vec<String>, // List of subscribable resource URIs
    list: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerToolCapabilities {
    list: bool,
    call: Vec<String>, // List of callable tool names
}


#[derive(Debug, Serialize, Deserialize)]
struct InitializeResult {
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
    #[serde(rename = "serverCapabilities")]
    server_capabilities: ServerCapabilities,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
struct ResourceDescriptor {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    // Add other fields like `subscribeOnly` if needed by protocol
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResourcesResult {
    resources: Vec<ResourceDescriptor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
struct ToolDescriptor {
    name: String,
    description: String,
    arguments: Option<Value>, // JSON Schema for arguments, or null if none
    #[serde(rename = "responseType")]
    response_type: Option<Value>, // JSON Schema for response, or null
}

#[derive(Debug, Serialize, Deserialize)]
struct ListToolsResult {
    tools: Vec<ToolDescriptor>,
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
struct CallParams {
    name: String,
    arguments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)] // Added PartialEq for comparison
enum PowerState {
    Charging,
    Discharging,
    Full,
    Unknown,
    // Add other states if `novade-domain::PowerState` has more that map here
}

impl From<DomainPowerState> for PowerState {
    fn from(state: DomainPowerState) -> Self {
        match state {
            DomainPowerState::Charging => PowerState::Charging,
            DomainPowerState::Discharging => PowerState::Discharging,
            DomainPowerState::Full => PowerState::Full,
            _ => PowerState::Unknown, // Handle other states like Empty, PluggedNotCharging
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)] // Added PartialEq for comparison
struct BatteryInfo {
    percentage: u8,
    state: PowerState,
    #[serde(rename = "timeToFull", skip_serializing_if = "Option::is_none")]
    time_to_full_seconds: Option<u64>,
    #[serde(rename = "timeToEmpty", skip_serializing_if = "Option::is_none")]
    time_to_empty_seconds: Option<u64>,
    // Consider adding battery ID/name if multiple batteries are handled
}

impl From<&DomainBatteryInfo> for BatteryInfo {
    fn from(info: &DomainBatteryInfo) -> Self {
        BatteryInfo {
            percentage: info.percentage.unwrap_or(0), // Default or handle Option
            state: info.state.into(),
            time_to_full_seconds: info.time_to_full.map(|d| d.as_secs()),
            time_to_empty_seconds: info.time_to_empty.map(|d| d.as_secs()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PowerCapabilities {
    #[serde(rename = "canSuspend")]
    can_suspend: bool,
    #[serde(rename = "canHibernate")]
    can_hibernate: bool,
    #[serde(rename = "canShutdown")]
    can_shutdown: bool,
    #[serde(rename = "canRestart")]
    can_restart: bool,
}

impl From<DomainPowerCapabilities> for PowerCapabilities {
    fn from(cap: DomainPowerCapabilities) -> Self {
        PowerCapabilities {
            can_suspend: cap.can_suspend,
            can_hibernate: cap.can_hibernate,
            can_shutdown: cap.can_shutdown,
            can_restart: cap.can_restart,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionStatus {
    status: String, // "subscribed" or "unsubscribed"
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationParams<T> {
    uri: String,
    content: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCallResult {
    status: String, // e.g., "suspending", "restarting"
    message: Option<String>, // Optional details or error message
}


// --- Constants ---
const RESOURCE_URI_BATTERY_INFO: &str = "power/battery_info";
const RESOURCE_URI_CAPABILITIES: &str = "power/capabilities";

const TOOL_NAME_SUSPEND: &str = "power/suspend";
const TOOL_NAME_HIBERNATE: &str = "power/hibernate";
const TOOL_NAME_SHUTDOWN: &str = "power/shutdown";
const TOOL_NAME_RESTART: &str = "power/restart";

// --- Server State ---
struct ServerState {
    power_service: Arc<DefaultPowerManagementService>,
    subscribed_to_battery_info: bool,
    battery_info_broadcaster_handle: Option<tokio::task::JoinHandle<()>>,
    battery_info_broadcaster_stop_tx: Option<mpsc::Sender<()>>,
    last_broadcasted_battery_info: Option<BatteryInfo>,
}

impl ServerState {
    fn new(power_service: Arc<DefaultPowerManagementService>) -> Self {
        ServerState {
            power_service,
            subscribed_to_battery_info: false,
            battery_info_broadcaster_handle: None,
            battery_info_broadcaster_stop_tx: None,
            last_broadcasted_battery_info: None,
        }
    }
}

// --- Main Application Logic ---
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("power_mcp_server=info".parse()?))
        .init();

    info!("Power MCP Server starting...");

    let power_service = Arc::new(DefaultPowerManagementService::new());
    let server_state = Arc::new(Mutex::new(ServerState::new(power_service)));

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("EOF received, exiting.");
                break; // EOF
            }
            Ok(_) => {
                info!("Received line: {}", line.trim());
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&line);

                match request {
                    Ok(req) => {
                        let req_id_for_error = req.id.clone().unwrap_or(Value::Null);
                        // Clone Arc for power_service for this request
                        let current_power_service = Arc::clone(&server_state.lock().await.power_service);

                        match handle_request(req, Arc::clone(&server_state), current_power_service, &mut stdout).await {
                            Ok(Some(true)) => {
                                info!("Exit signal received, server shutting down.");
                                break;
                            }
                            Ok(Some(false)) | Ok(None) => {
                                // Continue or notification handled
                            }
                            Err(e) => {
                                eprintln!("Error handling request: {:?}", e);
                                let error_response = if let Some(json_err) = e.downcast_ref::<JsonRpcError>() {
                                    JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(json_err.clone()),
                                        id: req_id_for_error,
                                    }
                                } else {
                                     JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError::internal_error()), // Or format e.to_string()
                                        id: req_id_for_error,
                                    }
                                };
                                if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                                   if let Err(write_err) = tokio::io::AsyncWriteExt::write_all(&mut stdout, format!("{}\n", json_err_res).as_bytes()).await {
                                        eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
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
                            id: Value::Null,
                        };
                        if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                           if let Err(write_err) = tokio::io::AsyncWriteExt::write_all(&mut stdout, format!("{}\n", json_err_res).as_bytes()).await {
                                eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
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
    
    // Cleanup broadcaster on exit
    let mut state = server_state.lock().await;
    if state.subscribed_to_battery_info {
        if let Some(stop_tx) = state.battery_info_broadcaster_stop_tx.take() {
            info!("Exiting: Attempting to stop battery info broadcaster.");
            let _ = stop_tx.send(()).await; // Send gracefully
        }
        if let Some(handle) = state.battery_info_broadcaster_handle.take() {
            info!("Exiting: Waiting for battery info broadcaster to finish.");
            if let Err(e) = handle.await {
                 eprintln!("Error joining battery info broadcaster task on exit: {:?}", e);
            }
        }
    }

    info!("Power MCP Server shutting down.");
    Ok(())
}

async fn handle_request(
    req: JsonRpcRequest,
    server_state_arc: Arc<Mutex<ServerState>>, // Arc<Mutex<ServerState>> for managing subscriptions
    power_service: Arc<DefaultPowerManagementService>, // Direct Arc for service calls
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
) -> Result<Option<bool>> { // Option<bool> indicates if server should exit (Some(true)) or continue
    let req_id = req.id.clone();

    let response_val = match req.method.as_str() {
        "initialize" => {
            info!("Handling 'initialize' request");
            let result = InitializeResult {
                server_info: ServerInfo {
                    name: "PowerMCPServer".to_string(),
                    version: "0.1.0".to_string(),
                },
                server_capabilities: ServerCapabilities {
                    resources: ServerResourceCapabilities {
                        subscribe: vec![RESOURCE_URI_BATTERY_INFO.to_string()],
                        list: true,
                    },
                    tools: ServerToolCapabilities {
                        list: true,
                        call: vec![
                            TOOL_NAME_SUSPEND.to_string(),
                            TOOL_NAME_HIBERNATE.to_string(),
                            TOOL_NAME_SHUTDOWN.to_string(),
                            TOOL_NAME_RESTART.to_string(),
                        ],
                    },
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
            info!("Handling 'resources/list' request");
            let resources = vec![
                ResourceDescriptor {
                    uri: RESOURCE_URI_BATTERY_INFO.to_string(),
                    name: "Battery Information".to_string(),
                    description: "Provides current battery percentage, state, and time to full/empty.".to_string(),
                    mime_type: "application/json".to_string(),
                },
                ResourceDescriptor {
                    uri: RESOURCE_URI_CAPABILITIES.to_string(),
                    name: "Power Capabilities".to_string(),
                    description: "Provides information about supported power operations like suspend, hibernate.".to_string(),
                    mime_type: "application/json".to_string(),
                },
            ];
            let result = ListResourcesResult { resources };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "resources/read" => {
            info!("Handling 'resources/read' request: {:?}", req.params);
            let params: ReadParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/read: {e}"))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())),
            };

            match params.uri.as_str() {
                RESOURCE_URI_BATTERY_INFO => {
                    let batteries = power_service.get_batteries().await?;
                    if let Some(battery_domain_info) = batteries.get(0) { // Get first battery
                        let battery_info_mcp: BatteryInfo = battery_domain_info.into();
                        Some(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::to_value(battery_info_mcp)?),
                            error: None,
                            id: req_id.unwrap_or(Value::Null),
                        })
                    } else {
                        Some(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(Value::Null), // Or specific error for no battery found
                            error: Some(JsonRpcError::new(-32000, "No battery found".to_string())),
                            id: req_id.unwrap_or(Value::Null),
                        })
                    }
                }
                RESOURCE_URI_CAPABILITIES => {
                    let caps_domain = power_service.get_capabilities().await?;
                    let caps_mcp: PowerCapabilities = caps_domain.into();
                     Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::to_value(caps_mcp)?),
                        error: None,
                        id: req_id.unwrap_or(Value::Null),
                    })
                }
                _ => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::new(-32001, format!("Resource not found: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                }),
            }
        }
        "resources/subscribe" => {
            info!("Handling 'resources/subscribe' request: {:?}", req.params);
            let params: SubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/subscribe: {e}"))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())),
            };

            if params.uri == RESOURCE_URI_BATTERY_INFO {
                let mut state = server_state_arc.lock().await;
                if !state.subscribed_to_battery_info {
                    state.subscribed_to_battery_info = true;
                    state.last_broadcasted_battery_info = None; // Reset last broadcasted on new subscription period
                    info!("Subscribed to {}. Starting battery info broadcaster.", params.uri);

                    let (stop_tx, mut stop_rx) = mpsc::channel(1);
                    state.battery_info_broadcaster_stop_tx = Some(stop_tx);
                    
                    let service_clone = Arc::clone(&state.power_service);
                    let state_clone_for_task = Arc::clone(&server_state_arc); // Clone Arc for task

                    state.battery_info_broadcaster_handle = Some(tokio::spawn(async move {
                        let mut interval = time::interval(Duration::from_secs(5)); // Check every 5 seconds
                        let writer_mutex = Arc::new(tokio::sync::Mutex::new(tokio::io::stdout()));


                        loop {
                            tokio::select! {
                                _ = interval.tick() => {
                                    match service_clone.get_batteries().await {
                                        Ok(batteries) => {
                                            if let Some(battery_domain_info) = batteries.get(0) {
                                                let current_info: BatteryInfo = battery_domain_info.into();
                                                let mut task_state_guard = state_clone_for_task.lock().await;

                                                if task_state_guard.last_broadcasted_battery_info.as_ref() != Some(&current_info) {
                                                    info!("Battery info changed, broadcasting: {:?}", current_info);
                                                    let notification = JsonRpcNotification {
                                                        jsonrpc: "2.0".to_string(),
                                                        method: "notifications/resources/updated".to_string(),
                                                        params: Some(serde_json::to_value(NotificationParams {
                                                            uri: RESOURCE_URI_BATTERY_INFO.to_string(),
                                                            content: current_info.clone(),
                                                        }).unwrap()), // Handle error better
                                                    };
                                                    task_state_guard.last_broadcasted_battery_info = Some(current_info);
                                                    // Must release lock before await on writer
                                                    drop(task_state_guard);


                                                    match serde_json::to_string(&notification) {
                                                        Ok(json_notification) => {
                                                            let mut w = writer_mutex.lock().await;
                                                            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut *w, format!("{}\n", json_notification).as_bytes()).await {
                                                                eprintln!("Error writing notification: {}", e);
                                                                break; 
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Error serializing notification: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Error getting battery info for broadcast: {}", e);
                                        }
                                    }
                                }
                                _ = stop_rx.recv() => {
                                    info!("Battery info broadcaster received stop signal. Exiting.");
                                    break;
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
                    error: Some(JsonRpcError::new(-32002, format!("Cannot subscribe to unknown resource: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                })
            }
        }
        "resources/unsubscribe" => {
            info!("Handling 'resources/unsubscribe' request: {:?}", req.params);
            let params: UnsubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for resources/unsubscribe: {e}"))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())),
            };

            if params.uri == RESOURCE_URI_BATTERY_INFO {
                let mut state = server_state_arc.lock().await;
                if state.subscribed_to_battery_info {
                    state.subscribed_to_battery_info = false; // Assuming only one subscriber type for now
                    if let Some(stop_tx) = state.battery_info_broadcaster_stop_tx.take() {
                        info!("Unsubscribed from {}. Stopping battery info broadcaster.", params.uri);
                        if stop_tx.send(()).await.is_err() {
                             eprintln!("Failed to send stop signal to battery broadcaster");
                        }
                    }
                    if let Some(handle) = state.battery_info_broadcaster_handle.take() {
                        if let Err(e) = handle.await {
                            eprintln!("Error joining battery broadcaster task: {:?}", e);
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
                    error: Some(JsonRpcError::new(-32003, format!("Cannot unsubscribe from unknown resource: {}", params.uri))),
                    id: req_id.unwrap_or(Value::Null),
                })
            }
        }
        "tools/list" => {
            info!("Handling 'tools/list' request");
            let tools = vec![
                ToolDescriptor {
                    name: TOOL_NAME_SUSPEND.to_string(),
                    description: "Suspends the system.".to_string(),
                    arguments: None, // Or Some(json!({"type": "object", "properties": {}})) for no args
                    response_type: Some(serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}})),
                },
                ToolDescriptor {
                    name: TOOL_NAME_HIBERNATE.to_string(),
                    description: "Hibernates the system.".to_string(),
                    arguments: None,
                    response_type: Some(serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}})),
                },
                ToolDescriptor {
                    name: TOOL_NAME_SHUTDOWN.to_string(),
                    description: "Shuts down the system.".to_string(),
                    arguments: None,
                    response_type: Some(serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}})),
                },
                ToolDescriptor {
                    name: TOOL_NAME_RESTART.to_string(),
                    description: "Restarts the system.".to_string(),
                    arguments: None,
                    response_type: Some(serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}})),
                },
            ];
            let result = ListToolsResult { tools };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "tools/call" => {
            info!("Handling 'tools/call' request: {:?}", req.params);
            let params: CallParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| anyhow!("Invalid params for tools/call: {e}"))?,
                None => return Err(anyhow!(JsonRpcError::invalid_params())),
            };

            let tool_call_result = match params.name.as_str() {
                TOOL_NAME_SUSPEND => {
                    power_service.suspend().await.context("Suspend operation failed")?;
                    ToolCallResult { status: "suspending".to_string(), message: None }
                }
                TOOL_NAME_HIBERNATE => {
                    power_service.hibernate().await.context("Hibernate operation failed")?;
                    ToolCallResult { status: "hibernating".to_string(), message: None }
                }
                TOOL_NAME_SHUTDOWN => {
                    power_service.shutdown().await.context("Shutdown operation failed")?;
                    ToolCallResult { status: "shutting_down".to_string(), message: None }
                }
                TOOL_NAME_RESTART => {
                    power_service.restart().await.context("Restart operation failed")?;
                    ToolCallResult { status: "restarting".to_string(), message: None }
                }
                _ => {
                    return Ok(Some(JsonRpcResponse { // Return early with specific error for unknown tool
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError::new(-32004, format!("Tool not found: {}", params.name))),
                        id: req_id.unwrap_or(Value::Null),
                    }));
                }
            };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(tool_call_result)?),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "shutdown" => {
            info!("Handling 'shutdown' request.");
            // Perform any cleanup if necessary
            let mut state = server_state_arc.lock().await;
            if state.subscribed_to_battery_info {
                 if let Some(stop_tx) = state.battery_info_broadcaster_stop_tx.take() {
                    info!("Shutting down: Stopping battery info broadcaster.");
                     if stop_tx.send(()).await.is_err() {
                        eprintln!("Failed to send stop signal to battery broadcaster during shutdown");
                    }
                }
                if let Some(handle) = state.battery_info_broadcaster_handle.take() {
                     if let Err(e) = handle.await {
                        eprintln!("Error joining battery broadcaster task during shutdown: {:?}", e);
                    }
                }
                state.subscribed_to_battery_info = false;
            }

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::Null),
                error: None,
                id: req_id.unwrap_or(Value::Null),
            })
        }
        "exit" => {
            info!("Handling 'exit' notification. Server will terminate.");
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

    if let Some(res) = response_val {
        let json_response = serde_json::to_string(&res)
            .context("Failed to serialize response")?;
        tokio::io::AsyncWriteExt::write_all(writer, format!("{}\n", json_response).as_bytes()).await?;
    }
    Ok(Some(false)) // Continue server by default
}
