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
use novade_domain::{
    power_management::{
        DefaultPowerManagementService, PowerManagementService, 
        BatteryInfo as DomainBatteryInfo, 
        BatteryState as DomainBatteryState, // Changed from PowerState
        PowerState as DomainSystemPowerState, // For system states if needed, not directly used by MCP structs
        // PowerCapabilities is not a struct in domain anymore for direct mapping
    },
    error::DomainError, // For error handling
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
enum MCPBatteryState { // Renamed from PowerState to MCPBatteryState for clarity
    Charging,
    Discharging,
    FullyCharged, // Changed from Full
    Empty,
    Pending,
    Unknown,
}

impl From<DomainBatteryState> for MCPBatteryState {
    fn from(state: DomainBatteryState) -> Self {
        match state {
            DomainBatteryState::Charging => MCPBatteryState::Charging,
            DomainBatteryState::Discharging => MCPBatteryState::Discharging,
            DomainBatteryState::FullyCharged => MCPBatteryState::FullyCharged,
            DomainBatteryState::Empty => MCPBatteryState::Empty,
            DomainBatteryState::Pending => MCPBatteryState::Pending,
            DomainBatteryState::Unknown | _ => MCPBatteryState::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)] // Added PartialEq for comparison
struct BatteryInfo {
    percentage: f64, // Changed to f64 to match domain
    state: MCPBatteryState, // Changed type to MCPBatteryState
    #[serde(rename = "timeToFull", skip_serializing_if = "Option::is_none")]
    time_to_full_seconds: Option<i64>, // Changed to i64 to match domain
    #[serde(rename = "timeToEmpty", skip_serializing_if = "Option::is_none")]
    time_to_empty_seconds: Option<i64>, // Changed to i64 to match domain
    // Consider adding battery ID/name if multiple batteries are handled
}

impl From<&DomainBatteryInfo> for BatteryInfo {
    fn from(info: &DomainBatteryInfo) -> Self {
        BatteryInfo {
            percentage: info.percentage, // Domain percentage is f64
            state: info.state.into(), // Uses From<DomainBatteryState> for MCPBatteryState
            time_to_full_seconds: info.time_to_full, // Domain is Option<i64>
            time_to_empty_seconds: info.time_to_empty, // Domain is Option<i64>
        }
    }
}

// This MCP-specific struct is populated by individual capability calls, not direct struct mapping
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

// DomainPowerCapabilities struct is no longer used for direct conversion.
// PowerCapabilities in MCP is built from individual can_X calls.


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

    // Initialize DefaultPowerManagementService, handling potential errors
    let power_service = match DefaultPowerManagementService::new().await {
        Ok(service) => Arc::new(service),
        Err(e) => {
            eprintln!("Fatal: Failed to initialize PowerManagementService: {:?}", e);
            // Optionally, could try to send a specific JSON-RPC error to client if protocol allows early errors
            return Err(anyhow!("Power service initialization failed: {}", e));
        }
    };
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
                                let rpc_error = if let Some(domain_err) = e.downcast_ref::<DomainError>() {
                                    match domain_err {
                                        DomainError::PowerManagement(pm_err) => match pm_err {
                                            novade_domain::error::PowerManagementError::DbusCommunicationError(s) =>
                                                JsonRpcError::new(-32000, format!("D-Bus communication error: {}", s)),
                                            novade_domain::error::PowerManagementError::DeviceNotFound(s) =>
                                                JsonRpcError::new(-32001, format!("Power device not found: {}", s)),
                                            novade_domain::error::PowerManagementError::OperationNotSupported(s) =>
                                                JsonRpcError::new(-32002, format!("Operation not supported: {}", s)),
                                            novade_domain::error::PowerManagementError::SleepInhibited(s) =>
                                                JsonRpcError::new(-32003, format!("Sleep inhibited: {}", s)),
                                            novade_domain::error::PowerManagementError::InhibitorNotFound(s) =>
                                                JsonRpcError::new(-32004, format!("Inhibitor not found: {}", s)),
                                            _ => JsonRpcError::internal_error(),
                                        },
                                        _ => JsonRpcError::internal_error(),
                                    }
                                } else if let Some(json_err) = e.downcast_ref::<JsonRpcError>() {
                                    json_err.clone()
                                } else {
                                    JsonRpcError::internal_error()
                                };

                                let error_response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(rpc_error),
                                    id: req_id_for_error,
                                };

                                if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                                   if let Err(write_err) = tokio::io::AsyncWriteExt::write_all(&mut stdout, format!("{}\n", json_err_res).as_bytes()).await {
                                        eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
                                        // Server loop will break if stdout is broken.
                                   }
                                } else {
                                     eprintln!("Fatal: Could not serialize error response for error: {:?}", e);
                                     // Server loop might break or continue depending on where this error is.
                                }
                            }
                        }
                    }
                    Err(e) => { // Error parsing the initial JSON request
                        eprintln!("Failed to parse JSON request: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::invalid_request()),
                            id: Value::Null, // ID might be unknown if parsing failed badly
                        };
                        if let Ok(json_err_res) = serde_json::to_string(&error_response) {
                           if let Err(write_err) = tokio::io::AsyncWriteExt::write_all(&mut stdout, format!("{}\n", json_err_res).as_bytes()).await {
                                eprintln!("Fatal: Could not write error response to stdout: {}", write_err);
                                // Break if stdout is broken
                           }
                        } else {
                             eprintln!("Fatal: Could not serialize error response for invalid request");
                             // Potentially break
                        }
                    }
                }
            }
            Err(e) => { // Error reading line from stdin
                eprintln!("Error reading from stdin: {}", e);
                break; // Exit loop on stdin error
            }
        }
    }
    
    // Cleanup broadcaster on exit
    let mut state_guard = server_state.lock().await;
    if state_guard.subscribed_to_battery_info {
        if let Some(stop_tx) = state_guard.battery_info_broadcaster_stop_tx.take() {
            info!("Exiting: Attempting to stop battery info broadcaster.");
            // Try to send, but don't block/panic if receiver is gone
            let _ = stop_tx.try_send(()); 
        }
        if let Some(handle) = state_guard.battery_info_broadcaster_handle.take() {
            info!("Exiting: Waiting for battery info broadcaster to finish.");
            // Give it a moment to shutdown, but don't hang indefinitely
            match tokio::time::timeout(Duration::from_secs(1), handle).await {
                Ok(Err(e)) => eprintln!("Error joining battery info broadcaster task on exit: {:?}", e),
                Err(_timeout) => eprintln!("Timeout waiting for battery info broadcaster task on exit."),
                _ => {} // Ok(Ok(()))
            }
        }
    }
    drop(state_guard); // Release lock

    info!("Power MCP Server shutting down.");
    Ok(())
}


// Helper to convert DomainError or Anyhow to JsonRpcError
fn to_rpc_error(e: anyhow::Error) -> JsonRpcError {
    if let Some(domain_err) = e.downcast_ref::<DomainError>() {
        match domain_err {
            DomainError::PowerManagement(pm_err) => match pm_err {
                novade_domain::error::PowerManagementError::DbusCommunicationError(s) =>
                    JsonRpcError::new(-32000, format!("D-Bus error: {}", s)),
                novade_domain::error::PowerManagementError::DeviceNotFound(s) =>
                    JsonRpcError::new(-32001, format!("Device not found: {}", s)),
                novade_domain::error::PowerManagementError::OperationNotSupported(s) =>
                    JsonRpcError::new(-32002, format!("Operation not supported: {}", s)),
                novade_domain::error::PowerManagementError::SleepInhibited(s) =>
                    JsonRpcError::new(-32003, format!("Sleep inhibited: {}", s)),
                novade_domain::error::PowerManagementError::InhibitorNotFound(s) =>
                    JsonRpcError::new(-32004, format!("Inhibitor not found: {}", s)),
                novade_domain::error::PowerManagementError::Other(s) =>
                    JsonRpcError::new(-32005, format!("Power management error: {}", s)),
            },
            DomainError::Core(core_err) => JsonRpcError::new(-32010, format!("Core error: {}", core_err)),
            // Add other DomainError variants as needed
            _ => JsonRpcError::internal_error(),
        }
    } else if let Some(json_err) = e.downcast_ref::<JsonRpcError>() {
        json_err.clone()
    } else {
        // Generic internal error for other anyhow errors
        JsonRpcError::new(-32603, format!("Internal server error: {}", e))
    }
}


async fn handle_request(
    req: JsonRpcRequest,
    server_state_arc: Arc<Mutex<ServerState>>, 
    power_service: Arc<DefaultPowerManagementService>, 
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
) -> Result<Option<bool>> { 
    let req_id = req.id.clone().unwrap_or(Value::Null); // Ensure req_id is available for all responses

    // Wrap the core logic in a closure to use `?` for error propagation
    // and then map the error to JsonRpcError if it occurs.
    let result_value: Result<Value, anyhow::Error> = (async {
        match req.method.as_str() {
            "initialize" => {
                info!("Handling 'initialize' request");
            // ... (rest of the methods, with error mapping)
            // For example, in resources/read:
            // let batteries = power_service.get_batteries().await.context("DBus call failed")?;
            // ...
            // This will be handled by the .map_err(to_rpc_error) below if it's an anyhow::Error
            // or by the direct error construction for specific cases.
                let result = InitializeResult {
                    server_info: ServerInfo {
                        name: "PowerMCPServer".to_string(),
                        version: "0.1.0".to_string(),
                    },
                    server_capabilities: ServerCapabilities {
                        resources: ServerResourceCapabilities {
                            subscribe: vec![RESOURCE_URI_BATTERY_INFO.to_string(), RESOURCE_URI_CAPABILITIES.to_string()], // Added capabilities
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
                Ok(serde_json::to_value(result)?)
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
                Ok(serde_json::to_value(result)?)
            }
            "resources/read" => {
                info!("Handling 'resources/read' request: {:?}", req.params);
                let params: ReadParams = req.params.ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                match params.uri.as_str() {
                    RESOURCE_URI_BATTERY_INFO => {
                        let batteries = power_service.get_batteries().await?;
                        if let Some(battery_domain_info) = batteries.get(0) {
                            let battery_info_mcp: BatteryInfo = battery_domain_info.into();
                            Ok(serde_json::to_value(battery_info_mcp)?)
                        } else {
                            // Return null for the data part, but it's a successful call finding no battery
                            // Client should interpret null as "no battery found" for this resource.
                            // Alternatively, send a specific error if the MCP spec dictates.
                            Ok(Value::Null) 
                        }
                    }
                    RESOURCE_URI_CAPABILITIES => {
                        // Construct PowerCapabilities by calling individual can_ methods
                        let caps_mcp = PowerCapabilities {
                            can_suspend: power_service.can_suspend().await?,
                            can_hibernate: power_service.can_hibernate().await?,
                            can_shutdown: power_service.can_shutdown().await?,
                            can_restart: power_service.can_restart().await?,
                        };
                        Ok(serde_json::to_value(caps_mcp)?)
                    }
                    _ => Err(anyhow!(JsonRpcError::new(-32001, format!("Resource not found: {}", params.uri)))),
                }
            }
            "resources/subscribe" => {
                info!("Handling 'resources/subscribe' request: {:?}", req.params);
                let params: SubscribeParams = req.params.ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                if params.uri == RESOURCE_URI_BATTERY_INFO {
                    let mut state = server_state_arc.lock().await;
                    if !state.subscribed_to_battery_info {
                        state.subscribed_to_battery_info = true;
                        state.last_broadcasted_battery_info = None;
                        info!("Subscribed to {}. Starting battery info broadcaster.", params.uri);

                        let (stop_tx, mut stop_rx) = mpsc::channel(1);
                        state.battery_info_broadcaster_stop_tx = Some(stop_tx);
                        
                        let service_clone = Arc::clone(&state.power_service);
                        let state_clone_for_task = Arc::clone(&server_state_arc);
                        let writer_mutex = Arc::new(tokio::sync::Mutex::new(tokio::io::stdout())); // Assuming stdout for notifications

                        state.battery_info_broadcaster_handle = Some(tokio::spawn(async move {
                            let mut interval = time::interval(Duration::from_secs(5));
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
                                                        let notification_params = NotificationParams {
                                                            uri: RESOURCE_URI_BATTERY_INFO.to_string(),
                                                            content: current_info.clone(),
                                                        };
                                                        let notification = JsonRpcNotification {
                                                            jsonrpc: "2.0".to_string(),
                                                            method: "notifications/resources/updated".to_string(),
                                                            params: Some(serde_json::to_value(notification_params).unwrap_or(Value::Null)),
                                                        };
                                                        task_state_guard.last_broadcasted_battery_info = Some(current_info);
                                                        drop(task_state_guard);

                                                        match serde_json::to_string(&notification) {
                                                            Ok(json_notification) => {
                                                                let mut w = writer_mutex.lock().await;
                                                                if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut *w, format!("{}\n", json_notification).as_bytes()).await {
                                                                    eprintln!("Error writing notification: {}", e);
                                                                    break; 
                                                                }
                                                            }
                                                            Err(e) => eprintln!("Error serializing notification: {}", e),
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => eprintln!("Error getting battery info for broadcast: {:?}", e),
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
                    Ok(serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })?)
                } else if params.uri == RESOURCE_URI_CAPABILITIES {
                     // For capabilities, it's a read-once and then subscribe for changes if the underlying system supports it.
                     // UPower/Logind capabilities generally don't change at runtime, or if they do, it's not a common stream.
                     // So, for now, acknowledge subscription but don't start a broadcaster.
                     // If a change notification mechanism was added to PowerManagementService for capabilities, it would go here.
                    info!("Subscribed to {}. Currently, no real-time updates for capabilities are broadcasted.", params.uri);
                    Ok(serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })?)
                } else {
                    Err(anyhow!(JsonRpcError::new(-32002, format!("Cannot subscribe to unknown resource: {}", params.uri))))
                }
            }
            "resources/unsubscribe" => {
                info!("Handling 'resources/unsubscribe' request: {:?}", req.params);
                let params: UnsubscribeParams = req.params.ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

                if params.uri == RESOURCE_URI_BATTERY_INFO {
                    let mut state = server_state_arc.lock().await;
                    if state.subscribed_to_battery_info {
                        state.subscribed_to_battery_info = false;
                        if let Some(stop_tx) = state.battery_info_broadcaster_stop_tx.take() {
                            info!("Unsubscribed from {}. Stopping battery info broadcaster.", params.uri);
                            let _ = stop_tx.try_send(()); // Best effort
                        }
                        if let Some(handle) = state.battery_info_broadcaster_handle.take() {
                             match tokio::time::timeout(Duration::from_secs(1), handle).await {
                                Ok(Err(e)) => eprintln!("Error joining battery broadcaster task: {:?}", e),
                                Err(_timeout) => eprintln!("Timeout waiting for battery broadcaster task on unsubscribe."),
                                _ => {}
                            }
                        }
                    }
                    Ok(serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })?)
                } else if params.uri == RESOURCE_URI_CAPABILITIES {
                    info!("Unsubscribed from {}. No active broadcaster to stop for capabilities.", params.uri);
                    Ok(serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })?)
                } else {
                    Err(anyhow!(JsonRpcError::new(-32003, format!("Cannot unsubscribe from unknown resource: {}", params.uri))))
                }
            }
            "tools/list" => {
                info!("Handling 'tools/list' request");
                let tools = vec![
                    ToolDescriptor {
                        name: TOOL_NAME_SUSPEND.to_string(),
                        description: "Suspends the system.".to_string(),
                        arguments: None, 
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
                Ok(serde_json::to_value(result)?)
            }
            "tools/call" => {
                info!("Handling 'tools/call' request: {:?}", req.params);
                let params: CallParams = req.params.ok_or_else(JsonRpcError::invalid_params)
                    .and_then(|p| serde_json::from_value(p).map_err(|_| JsonRpcError::invalid_params()))?;

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
                    _ => return Err(anyhow!(JsonRpcError::new(-32004, format!("Tool not found: {}", params.name)))),
                };
                Ok(serde_json::to_value(tool_call_result)?)
            }
            "shutdown" => { // This is a JSON-RPC level shutdown, not system shutdown
                info!("Handling 'shutdown' request (MCP server shutdown).");
                // Cleanup is now handled when the main loop breaks after this returns Some(true)
                // No specific action here other than acknowledging.
                Ok(Value::Null) // Standard response for shutdown if no specific return value
            }
            "exit" => { // This is a JSON-RPC level exit notification
                info!("Handling 'exit' notification. Server will terminate.");
                // No response is sent for notifications. Signal to main loop to exit.
                // This is handled by returning Ok(None) for the response_val
                // and Ok(Some(true)) for the handle_request overall result.
                // This structure is a bit complex, might need refactor.
                // For now, let's assume exit means no JSON response.
                return Err(anyhow!("exit_signal")); // Special signal to indicate exit
            }
            _ => Err(anyhow!(JsonRpcError::method_not_found())),
        }
    })
    .await;

    // Construct JSON-RPC response based on the outcome of the handler
    let response = match result_value {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(value),
            error: None,
            id: req_id,
        },
        Err(e) => {
            if let Some(msg) = e.downcast_ref::<&str>() {
                if *msg == "exit_signal" {
                    return Ok(Some(true)); // Signal exit to main loop
                }
            }
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(to_rpc_error(e)),
                id: req_id,
            }
        }
    };
    
    // "exit" is a notification, no response should be sent.
    // The "exit_signal" error above handles this.
    // For "shutdown" method, a response IS expected.

    let json_response = serde_json::to_string(&response)
        .context("Failed to serialize response")?;
    tokio::io::AsyncWriteExt::write_all(writer, format!("{}\n", json_response).as_bytes()).await?;
    
    // If the method was "shutdown", also signal exit to the main loop.
    if req.method == "shutdown" {
        return Ok(Some(true));
    }

    Ok(Some(false)) // Continue server by default
}
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
