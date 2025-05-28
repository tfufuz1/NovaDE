use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

// Module for CPU reading logic
mod cpu_reader;
use cpu_reader::{calculate_cpu_percentage, read_cpu_times, CpuTimes};

// --- JSON-RPC Structs ---
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: NotificationParams,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcError {
    fn new(code: i32, message: String, data: Option<Value>) -> Self {
        JsonRpcError { code, message, data }
    }

    fn invalid_request() -> Self {
        JsonRpcError::new(-32600, "Invalid Request".to_string(), None)
    }

    fn method_not_found() -> Self {
        JsonRpcError::new(-32601, "Method not found".to_string(), None)
    }

    fn invalid_params(message: Option<String>) -> Self {
        JsonRpcError::new(-32602, message.unwrap_or_else(|| "Invalid params".to_string()), None)
    }

    fn internal_error(message: String) -> Self {
        JsonRpcError::new(-32603, message, None)
    }
    
    fn resource_not_found(uri: &str) -> Self {
        JsonRpcError::new(-32000, format!("Resource not found: {}", uri), None)
    }
}

// --- MCP Parameter Structs ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerResourceCapabilities {
    subscribe: bool,
    list: bool,
    read: bool, // Added read capability
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerCapabilities {
    resources: ServerResourceCapabilities,
    tools: Value,
    prompts: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InitializeResult {
    server_info: ServerInfo,
    server_capabilities: ServerCapabilities,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ResourceDescriptor {
    uri: String,
    name: String,
    description: String,
    mime_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ListResourcesResult {
    resources: Vec<ResourceDescriptor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReadParams {
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SubscribeParams {
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnsubscribeParams {
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SubscriptionStatus {
    status: String, 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CpuUsageContent {
    percentage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NotificationParams {
    uri: String,
    content: CpuUsageContent,
}

// --- Constants ---
const CPU_RESOURCE_URI: &str = "cpu/usage_percent";
const SERVER_NAME: &str = "cpu_mcp_server";
const SERVER_VERSION: &str = "0.1.0";

// --- Server State ---
struct ServerState {
    subscribed_to_cpu: bool,
    cpu_broadcaster_handle: Option<tokio::task::JoinHandle<()>>,
    cpu_broadcaster_stop_tx: Option<mpsc::Sender<()>>,
    last_cpu_times: Option<CpuTimes>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            subscribed_to_cpu: false,
            cpu_broadcaster_handle: None,
            cpu_broadcaster_stop_tx: None,
            last_cpu_times: None,
        }
    }
}

// --- Main Function ---
#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    info!("{} v{} starting...", SERVER_NAME, SERVER_VERSION);

    let server_state = Arc::new(Mutex::new(ServerState::new()));
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    
    // Create a single Mutex-protected writer for all output (responses and notifications)
    let stdout = tokio::io::stdout();
    let writer_arc = Arc::new(Mutex::new(BufWriter::new(stdout)));


    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => { 
                info!("Input stream closed (EOF). Exiting.");
                break;
            }
            Ok(_) => {
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&line);
                let mut should_exit = false;

                match request {
                    Ok(req) => {
                        let req_id = req.id.clone();
                        match handle_request(req, Arc::clone(&server_state), Arc::clone(&writer_arc)).await {
                            Ok(Some(exit_signal)) => {
                                should_exit = exit_signal;
                            }
                            Ok(None) => {
                                // This case implies handle_request already sent a response or it's a notification.
                            }
                            Err(e) => {
                                error!("Error handling request: {:?}", e);
                                let err_resp = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: req_id,
                                    result: None,
                                    error: Some(JsonRpcError::internal_error(e.to_string())),
                                };
                                let response_json = serde_json::to_string(&err_resp)? + "\n";
                                let mut w = writer_arc.lock().await;
                                w.write_all(response_json.as_bytes()).await?;
                                w.flush().await?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request: {}", e);
                        let err_resp = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None, 
                            result: None,
                            error: Some(JsonRpcError::invalid_request()),
                        };
                        let response_json = serde_json::to_string(&err_resp)? + "\n";
                        let mut w = writer_arc.lock().await;
                        w.write_all(response_json.as_bytes()).await?;
                        w.flush().await?;
                    }
                }
                if should_exit {
                    info!("Exit signal received. Shutting down.");
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    // Cleanup broadcaster task
    info!("Main loop exited. Cleaning up server state...");
    cleanup_broadcaster(server_state.lock().await).await;
    info!("Server shut down gracefully.");
    Ok(())
}

async fn cleanup_broadcaster(mut state: tokio::sync::MutexGuard<'_, ServerState>) {
    if state.subscribed_to_cpu { // Check if it was supposed to be running
        if let Some(stop_tx) = state.cpu_broadcaster_stop_tx.take() {
            info!("Sending stop signal to CPU broadcaster task during cleanup...");
            if stop_tx.send(()).await.is_err() {
                error!("Failed to send stop signal to broadcaster during cleanup: receiver dropped.");
            }
        }
        if let Some(handle) = state.cpu_broadcaster_handle.take() {
            info!("Waiting for CPU broadcaster task to complete during cleanup...");
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(())) => info!("CPU broadcaster task joined successfully during cleanup."),
                Ok(Err(e)) => error!("CPU broadcaster task panicked or failed during cleanup: {:?}", e),
                Err(_) => error!("CPU broadcaster task timed out during cleanup."),
            }
        }
        state.subscribed_to_cpu = false; // Ensure state reflects it's stopped
    }
}


// --- Request Handler ---
async fn handle_request(
    req: JsonRpcRequest,
    state_arc: Arc<Mutex<ServerState>>,
    writer_arc: Arc<Mutex<BufWriter<tokio::io::Stdout>>>,
) -> Result<Option<bool>> { // Option<bool>: Some(true) to exit, Some(false) to continue, None if response handled
    info!("Handling request method: {}", req.method);
    let req_id = req.id.clone();
    let mut response_val: Option<Value> = None;
    let mut error_val: Option<JsonRpcError> = None;
    let mut should_exit = false;

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                server_info: ServerInfo {
                    name: SERVER_NAME.to_string(),
                    version: SERVER_VERSION.to_string(),
                },
                server_capabilities: ServerCapabilities {
                    resources: ServerResourceCapabilities {
                        subscribe: true,
                        list: true,
                        read: true,
                    },
                    tools: serde_json::json!({}),
                    prompts: serde_json::json!({}),
                },
            };
            response_val = Some(serde_json::to_value(result)?);
        }
        "resources/list" => {
            let resources = vec![ResourceDescriptor {
                uri: CPU_RESOURCE_URI.to_string(),
                name: "CPU Usage".to_string(),
                description: "Provides overall CPU usage percentage.".to_string(),
                mime_type: "application/json".to_string(),
            }];
            let result = ListResourcesResult { resources };
            response_val = Some(serde_json::to_value(result)?);
        }
        "resources/read" => {
            let params: ReadParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?,
                None => return Err(JsonRpcError::invalid_params(Some("Missing params for resources/read".to_string()))).context("Invalid params"),
            };

            if params.uri == CPU_RESOURCE_URI {
                let mut state = state_arc.lock().await;
                match read_cpu_times().await {
                    Ok(Some(current_times)) => {
                        let percentage = calculate_cpu_percentage(&state.last_cpu_times, &current_times);
                        state.last_cpu_times = Some(current_times);
                        response_val = Some(serde_json::to_value(CpuUsageContent { percentage })?);
                    }
                    Ok(None) => { // Should ideally not happen if /proc/stat is valid and contains "cpu " line
                        error_val = Some(JsonRpcError::internal_error("Could not find aggregate CPU line in /proc/stat".to_string()));
                    }
                    Err(e) => {
                        error!("Failed to read CPU times for resources/read: {:?}", e);
                        error_val = Some(JsonRpcError::internal_error(format!("Failed to read CPU times: {}", e)));
                    }
                }
            } else {
                error_val = Some(JsonRpcError::resource_not_found(&params.uri));
            }
        }
        "resources/subscribe" => {
            let params: SubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?,
                None => return Err(JsonRpcError::invalid_params(Some("Missing params for resources/subscribe".to_string()))).context("Invalid params"),
            };

            if params.uri == CPU_RESOURCE_URI {
                let mut state = state_arc.lock().await;
                if !state.subscribed_to_cpu {
                    state.subscribed_to_cpu = true;
                    let (stop_tx, mut stop_rx) = mpsc::channel(1);
                    state.cpu_broadcaster_stop_tx = Some(stop_tx);

                    let task_state_arc = Arc::clone(&state_arc);
                    let task_writer_arc = Arc::clone(&writer_arc);

                    info!("Starting CPU broadcaster task...");
                    let handle = tokio::spawn(async move {
                        let mut interval = interval(Duration::from_secs(2));
                        loop {
                            tokio::select! {
                                _ = interval.tick() => {
                                    let mut task_state = task_state_arc.lock().await;
                                    if !task_state.subscribed_to_cpu {
                                        info!("Broadcaster: Subscription ended, stopping task.");
                                        break;
                                    }
                                    match read_cpu_times().await {
                                        Ok(Some(current_times)) => {
                                            let percentage = calculate_cpu_percentage(&task_state.last_cpu_times, &current_times);
                                            task_state.last_cpu_times = Some(current_times);
                                            
                                            info!("Broadcaster: CPU Usage: {:.2}%", percentage);

                                            let notification_content = CpuUsageContent { percentage };
                                            let notification = JsonRpcNotification {
                                                jsonrpc: "2.0".to_string(),
                                                method: "notifications/resources/updated".to_string(),
                                                params: NotificationParams {
                                                    uri: CPU_RESOURCE_URI.to_string(),
                                                    content: notification_content,
                                                },
                                            };
                                            match serde_json::to_string(&notification) {
                                                Ok(json_note) => {
                                                    let mut w = task_writer_arc.lock().await;
                                                    if let Err(e) = w.write_all((json_note + "\n").as_bytes()).await {
                                                        error!("Broadcaster: Failed to write notification: {:?}", e);
                                                    }
                                                    if let Err(e) = w.flush().await {
                                                         error!("Broadcaster: Failed to flush notification: {:?}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Broadcaster: Failed to serialize notification: {:?}", e);
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                             warn!("Broadcaster: Could not find aggregate CPU line in /proc/stat. Skipping notification.");
                                        }
                                        Err(e) => {
                                            error!("Broadcaster: Failed to read CPU times: {:?}", e);
                                            // Decide if we should stop or continue
                                        }
                                    }
                                }
                                _ = stop_rx.recv() => {
                                    info!("Broadcaster: Stop signal received. Exiting task.");
                                    break;
                                }
                            }
                        }
                        info!("CPU broadcaster task finished.");
                    });
                    state.cpu_broadcaster_handle = Some(handle);
                    response_val = Some(serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })?);
                } else {
                    // Already subscribed
                    response_val = Some(serde_json::to_value(SubscriptionStatus { status: "subscribed".to_string() })?);
                }
            } else {
                error_val = Some(JsonRpcError::resource_not_found(&params.uri));
            }
        }
        "resources/unsubscribe" => {
            let params: UnsubscribeParams = match req.params {
                Some(p) => serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?,
                None => return Err(JsonRpcError::invalid_params(Some("Missing params for resources/unsubscribe".to_string()))).context("Invalid params"),
            };
            
            if params.uri == CPU_RESOURCE_URI {
                let mut state = state_arc.lock().await;
                if state.subscribed_to_cpu {
                    state.subscribed_to_cpu = false; // Signal broadcaster to stop sending
                    if let Some(stop_tx) = state.cpu_broadcaster_stop_tx.take() {
                        info!("Sending stop signal to CPU broadcaster task...");
                        if stop_tx.send(()).await.is_err() {
                            error!("Failed to send stop signal to broadcaster: receiver dropped.");
                        }
                    }
                    if let Some(handle) = state.cpu_broadcaster_handle.take() {
                         info!("Waiting for CPU broadcaster task to complete...");
                        match tokio::time::timeout(Duration::from_secs(5), handle).await {
                            Ok(Ok(())) => info!("CPU broadcaster task joined successfully."),
                            Ok(Err(e)) => error!("CPU broadcaster task panicked or failed: {:?}", e),
                            Err(_) => error!("CPU broadcaster task timed out."),
                        }
                    }
                    response_val = Some(serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })?);
                } else {
                     // Not subscribed or already unsubscribed
                    response_val = Some(serde_json::to_value(SubscriptionStatus { status: "unsubscribed".to_string() })?);
                }
            } else {
                error_val = Some(JsonRpcError::resource_not_found(&params.uri));
            }
        }
        "shutdown" | "exit" => {
            info!("Received {} request. Server will shut down.", req.method);
            // Use the standalone cleanup function
            cleanup_broadcaster(state_arc.lock().await).await;
            should_exit = true;
            if req.id.is_some() { // Only respond if it's not a notification
                 response_val = Some(Value::Null);
            }
        }
        _ => {
            error_val = Some(JsonRpcError::method_not_found());
        }
    }

    // Send response if it's not a notification or if an error occurred for a notification
    if req_id.is_some() || error_val.is_some() {
        let response_obj = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req_id,
            result: response_val,
            error: error_val,
        };
        let response_json = serde_json::to_string(&response_obj)? + "\n";
        let mut w = writer_arc.lock().await;
        w.write_all(response_json.as_bytes()).await?;
        w.flush().await?;
    }
    
    Ok(Some(should_exit))
}
```

I have implemented the detailed logic for the MCP method handlers and the CPU usage broadcasting task.

**Key Changes and Implementations:**

1.  **`JsonRpcNotification` Struct:** Added this struct to represent notifications sent by the server.
2.  **`JsonRpcError::invalid_params` & `resource_not_found`:** Enhanced error reporting.
3.  **Shared Writer:** The `main` function now creates an `Arc<Mutex<BufWriter<Stdout>>>` (`writer_arc`) which is passed to `handle_request` and then cloned for the broadcaster task. This ensures all writes to stdout (responses and notifications) are synchronized.
4.  **`handle_request` - `resources/read`:**
    *   Deserializes `ReadParams`.
    *   Checks if `params.uri == CPU_RESOURCE_URI`.
    *   Locks `ServerState`, calls `read_cpu_times()`, calculates percentage using `calculate_cpu_percentage()`, updates `state.last_cpu_times`.
    *   Sends `CpuUsageContent` as response or an error if reading fails or URI is incorrect.
5.  **`handle_request` - `resources/subscribe`:**
    *   Deserializes `SubscribeParams`.
    *   Checks URI.
    *   If not already subscribed:
        *   Sets `subscribed_to_cpu = true`.
        *   Creates an `mpsc::channel` for stopping the broadcaster; stores `tx` in `ServerState`.
        *   Spawns the broadcaster Tokio task:
            *   Clones `state_arc` and `writer_arc`.
            *   Loops every 2 seconds using `tokio::time::interval`.
            *   Inside the loop, it uses `tokio::select!` to react to interval ticks or a stop signal.
            *   On tick: locks state, reads CPU times, calculates percentage, updates `last_cpu_times`.
            *   If still subscribed, creates and sends a `JsonRpcNotification` with `notifications/resources/updated` method and `CpuUsageContent` to the shared writer.
            *   Handles errors during CPU reading/calculation within the loop (logs them).
            *   Breaks loop if stop signal received via `stop_rx` or if `subscribed_to_cpu` becomes false.
        *   Stores the `JoinHandle` in `ServerState`.
    *   Responds with `SubscriptionStatus { status: "subscribed" }`.
6.  **`handle_request` - `resources/unsubscribe`:**
    *   Deserializes `UnsubscribeParams`.
    *   Checks URI.
    *   If subscribed:
        *   Sets `subscribed_to_cpu = false` (this signals the broadcaster to stop sending notifications on its next check).
        *   Sends a stop signal via `cpu_broadcaster_stop_tx`.
        *   Awaits the `cpu_broadcaster_handle` with a 5-second timeout.
        *   Clears `cpu_broadcaster_stop_tx` and `cpu_broadcaster_handle`.
    *   Responds with `SubscriptionStatus { status: "unsubscribed" }`.
7.  **`handle_request` - `shutdown` / `exit`:**
    *   Calls a new `cleanup_broadcaster` helper function (see below).
    *   Sets `should_exit = true`.
    *   Responds with `null` result if the request had an ID.
8.  **`cleanup_broadcaster` Function:**
    *   Created a new async helper function `cleanup_broadcaster(mut state: tokio::sync::MutexGuard<'_, ServerState>)`.
    *   This function encapsulates the logic to send a stop signal and await the broadcaster task. It's used by both `shutdown` and the main loop cleanup.
9.  **Main Loop Cleanup (in `main()`):**
    *   After the main loop exits, it now explicitly calls `cleanup_broadcaster(server_state.lock().await).await` to ensure the broadcaster is stopped and awaited. This handles cases where the server exits due to EOF or errors, not just a clean `shutdown` command.

Error handling for JSON parsing in `handle_request` for params has been improved. The broadcaster loop includes basic error logging for its operations. Locking of `ServerState` and the shared `writer_arc` is managed to allow concurrent access while preventing data races.
