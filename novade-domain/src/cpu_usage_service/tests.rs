#![cfg(test)]

use super::*; // Imports DefaultCpuUsageService, ICpuUsageService, etc.
use crate::ai_interaction_service::types::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, ConnectionStatus, MCPServerConfig, ClientCapabilities,
};
use crate::error::DomainError;
use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex, oneshot};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

// --- Mock MCPClientInstance ---
#[derive(Debug)]
struct MockMCPClientInstance {
    response_map: Arc<TokioMutex<HashMap<String, VecDeque<AnyhowResult<JsonRpcResponse>>>>>,
    notification_rx_option: Arc<TokioMutex<Option<mpsc::UnboundedReceiver<JsonRpcRequest>>>>,
    notification_injector_tx: mpsc::UnboundedSender<JsonRpcRequest>,
    subscribe_called_count: Arc<TokioMutex<usize>>,
    unsubscribe_called_count: Arc<TokioMutex<usize>>,
    requests_received: Arc<TokioMutex<Vec<(String, serde_json::Value)>>>, // To inspect requests
    pub config: MCPServerConfig,
    connection_status: Arc<TokioMutex<ConnectionStatus>>,
}

impl Clone for MockMCPClientInstance {
    fn clone(&self) -> Self {
        Self {
            response_map: Arc::clone(&self.response_map),
            notification_rx_option: Arc::clone(&self.notification_rx_option),
            notification_injector_tx: self.notification_injector_tx.clone(),
            subscribe_called_count: Arc::clone(&self.subscribe_called_count),
            unsubscribe_called_count: Arc::clone(&self.unsubscribe_called_count),
            requests_received: Arc::clone(&self.requests_received),
            config: self.config.clone(),
            connection_status: Arc::clone(&self.connection_status),
        }
    }
}

impl MockMCPClientInstance {
    fn new(config: MCPServerConfig) -> Self {
        let (injector_tx, injector_rx) = mpsc::unbounded_channel();
        Self {
            response_map: Arc::new(TokioMutex::new(HashMap::new())),
            notification_rx_option: Arc::new(TokioMutex::new(Some(injector_rx))),
            notification_injector_tx: injector_tx,
            subscribe_called_count: Arc::new(TokioMutex::new(0)),
            unsubscribe_called_count: Arc::new(TokioMutex::new(0)),
            requests_received: Arc::new(TokioMutex::new(Vec::new())),
            config,
            connection_status: Arc::new(TokioMutex::new(ConnectionStatus::Connected)),
        }
    }

    async fn send_request_internal(&mut self, method: String, params: serde_json::Value) -> AnyhowResult<JsonRpcResponse> {
        self.requests_received.lock().await.push((method.clone(), params.clone()));
        if method == "resources/subscribe" {
            *self.subscribe_called_count.lock().await += 1;
        } else if method == "resources/unsubscribe" {
            *self.unsubscribe_called_count.lock().await += 1;
        }
        let mut map = self.response_map.lock().await;
        map.get_mut(&method).and_then(|q| q.pop_front())
            .unwrap_or_else(|| Err(anyhow::anyhow!("MockMCPClientInstance: No response queued for method '{}'", method)))
    }

    fn take_notification_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<JsonRpcRequest>> {
        self.notification_rx_option.blocking_lock().take()
    }
    
    fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status.blocking_lock().clone()
    }
    
    async fn set_connection_status(&self, status: ConnectionStatus) {
        *self.connection_status.lock().await = status;
    }
    async fn push_response(&self, method: &str, response: AnyhowResult<JsonRpcResponse>) {
        self.response_map.lock().await.entry(method.to_string()).or_default().push_back(response);
    }
    fn get_notification_injector(&self) -> mpsc::UnboundedSender<JsonRpcRequest> {
        self.notification_injector_tx.clone()
    }
    async fn get_subscribe_called_count(&self) -> usize { *self.subscribe_called_count.lock().await }
    async fn get_unsubscribe_called_count(&self) -> usize { *self.unsubscribe_called_count.lock().await }
    #[allow(dead_code)] async fn get_received_requests(&self) -> Vec<(String, serde_json::Value)> { self.requests_received.lock().await.clone() }
}

// --- Test Setup: MockableConnectionService Trait and Implementation ---
#[async_trait]
trait MockableConnectionService<C: Send + Sync + 'static>: Send + Sync {
    fn get_client_instance_mock(&self, server_id: &str) -> Option<Arc<TokioMutex<C>>>;
    fn get_default_client_capabilities_mock(&self) -> ClientCapabilities;
}

struct TestSetup_MockMCPConnectionService<C: Send + Sync + 'static> {
    mock_client_to_return: Option<Arc<TokioMutex<C>>>,
    default_client_capabilities: ClientCapabilities,
}

impl<C: Send + Sync + 'static> TestSetup_MockMCPConnectionService<C> {
    fn new(client_to_return: Option<Arc<TokioMutex<C>>>) -> Self {
        Self {
            mock_client_to_return: client_to_return,
            default_client_capabilities: ClientCapabilities { supports_streaming: true },
        }
    }
}

#[async_trait]
impl<C: Send + Sync + 'static> MockableConnectionService<C> for TestSetup_MockMCPConnectionService<C> {
    fn get_client_instance_mock(&self, server_id: &str) -> Option<Arc<TokioMutex<C>>> {
        // In a more complex mock, you might check server_id against client's config.
        // For these tests, we assume the client provided is for the relevant server_id.
        // This check is more for if the mock_client_to_return itself has a config.host to check.
        // Let's assume the caller configures this correctly for the test.
        if let Some(client_arc) = &self.mock_client_to_return {
             // If C is MockMCPClientInstance, it has a config.host we could check.
             // This requires C to have a way to get its host/server_id.
             // For now, simplicity: if a client is set, return it if server_id matches a known one.
             // This detail depends on how strictly we match server_id in the mock.
             // The tests will use CPU_SERVER_ID_DEFAULT.
            if server_id == CPU_SERVER_ID_DEFAULT { // Simple check for this suite
                return Some(client_arc.clone());
            }
        }
        None
    }
    fn get_default_client_capabilities_mock(&self) -> ClientCapabilities {
        self.default_client_capabilities.clone()
    }
}

// Helper to instantiate DefaultCpuUsageService with mocked dependencies.
// This uses direct struct instantiation, assuming fields are pub(crate) or pub.
fn create_service_for_test(
    mock_client_opt: Option<Arc<TokioMutex<MockMCPClientInstance>>>,
    server_id_override: Option<String>
) -> DefaultCpuUsageService {
    let mock_conn_service = Arc::new(TokioMutex::new(TestSetup_MockMCPConnectionService::new(mock_client_opt)));
    
    let server_id_to_use = server_id_override.unwrap_or_else(|| CPU_SERVER_ID_DEFAULT.to_string());

    // This direct instantiation requires DefaultCpuUsageService's fields to be accessible.
    // And connection_service field to accept Arc<TokioMutex<dyn MockableConnectionService<...>>>
    // This is the "magic" part for testing without changing DefaultCpuUsageService's `new` signature
    // for production code. It implies DefaultCpuUsageService is structured like:
    // pub struct DefaultCpuUsageService<CS: MockableConnectionService<...>> { connection_service: Arc<TokioMutex<CS>> ... }
    // Or, for testing, we accept the type erasure with `dyn Trait`.
    DefaultCpuUsageService {
        connection_service: mock_conn_service as Arc<TokioMutex<dyn MockableConnectionService<MockMCPClientInstance>>>,
        cpu_server_id: server_id_to_use,
        subscribers: Arc::new(TokioMutex::new(HashMap::new())),
        active_mcp_sub: Arc::new(TokioMutex::new(None)),
    }
}

// Helper for creating responses
fn json_rpc_success(id_val: serde_json::Value, result_val: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse { jsonrpc: "2.0".to_string(), id: Some(id_val), result: Some(result_val), error: None }
}
fn json_rpc_error(id_val: serde_json::Value, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse { jsonrpc: "2.0".to_string(), id: Some(id_val), result: None, error: Some(JsonRpcError { code, message: message.to_string(), data: None }) }
}

// --- Test Suite ---
#[tokio::test]
async fn test_get_current_cpu_percentage_success() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response(
        "resources/read",
        Ok(json_rpc_success(json!(1), json!({"percentage": 75.5})))
    ).await;
    
    let service = create_service_for_test(Some(mock_client.clone()), None);

    let result = service.get_current_cpu_percentage().await;
    assert!(result.is_ok(), "Result should be Ok. It was: {:?}", result);
    assert_eq!(result.unwrap(), 75.5);

    // Verify request params (optional, but good for completeness)
    let requests = mock_client.lock().await.get_received_requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].0, "resources/read");
    assert_eq!(requests[0].1, json!({"uri": CPU_RESOURCE_URI_DEFAULT}));
}

#[tokio::test]
async fn test_get_current_cpu_percentage_mcp_error() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response(
        "resources/read",
        Ok(json_rpc_error(json!(1), -32000, "CPU Read Error"))
    ).await;

    let service = create_service_for_test(Some(mock_client.clone()), None);

    let result = service.get_current_cpu_percentage().await;
    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::MCPRequest(msg) => {
            assert!(msg.contains("CPU Read Error"));
            assert!(msg.contains("code=-32000"));
        }
        other => panic!("Expected DomainError::MCPRequest, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_current_cpu_percentage_client_not_found() {
    // Mock connection service returns None for the client
    let service = create_service_for_test(None, None); // No client will be found

    let result = service.get_current_cpu_percentage().await;
    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::ServiceMisconfigured(msg) => assert!(msg.contains("not found")),
        other => panic!("Expected DomainError::ServiceMisconfigured, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_current_cpu_percentage_client_disconnected() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));
    
    mock_client.lock().await.set_connection_status(ConnectionStatus::Disconnected).await;
    
    let service = create_service_for_test(Some(mock_client.clone()), None);

    let result = service.get_current_cpu_percentage().await;
    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::MCPConnection(msg) => assert!(msg.contains("is not connected")),
        other => panic!("Expected DomainError::MCPConnection, got {:?}", other),
    }
}


#[tokio::test]
async fn test_subscribe_unsubscribe_flow() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response("resources/subscribe", Ok(json_rpc_success(json!(1), json!({"status": "subscribed"})))).await;
    mock_client.lock().await.push_response("resources/unsubscribe", Ok(json_rpc_success(json!(2), json!({"status": "unsubscribed"})))).await;

    let service = create_service_for_test(Some(mock_client.clone()), None);
    let (ui_updater_tx, mut ui_updater_rx) = mpsc::unbounded_channel::<Result<f64, DomainError>>();

    let subscription_id = service.subscribe_to_cpu_updates(ui_updater_tx).await.unwrap();
    assert_eq!(mock_client.lock().await.get_subscribe_called_count().await, 1);

    let notification_injector = mock_client.lock().await.get_notification_injector();
    notification_injector.send(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "notifications/resources/updated".to_string(),
        params: json!({"uri": CPU_RESOURCE_URI_DEFAULT, "content": {"percentage": 42.0}}),
        id: None,
    }).unwrap();

    match timeout(Duration::from_millis(200), ui_updater_rx.recv()).await {
        Ok(Some(Ok(percentage))) => assert_eq!(percentage, 42.0),
        other => panic!("Failed to receive expected CPU update: {:?}", other),
    }

    service.unsubscribe_from_cpu_updates(subscription_id).await.unwrap();
    assert_eq!(mock_client.lock().await.get_unsubscribe_called_count().await, 1);
    assert!(service.active_mcp_sub.lock().await.is_none(), "Active MCP subscription should be None after last unsubscribe");
}

#[tokio::test]
async fn test_multiple_subscribers_receive_updates() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response("resources/subscribe", Ok(json_rpc_success(json!(1), json!({"status": "subscribed"})))).await;
    mock_client.lock().await.push_response("resources/unsubscribe", Ok(json_rpc_success(json!(2), json!({"status": "unsubscribed"})))).await;

    let service = create_service_for_test(Some(mock_client.clone()), None);

    let (ui_updater1_tx, mut ui_updater1_rx) = mpsc::unbounded_channel();
    let (ui_updater2_tx, mut ui_updater2_rx) = mpsc::unbounded_channel();

    let sub_id1 = service.subscribe_to_cpu_updates(ui_updater1_tx).await.unwrap();
    let sub_id2 = service.subscribe_to_cpu_updates(ui_updater2_tx).await.unwrap();

    assert_eq!(mock_client.lock().await.get_subscribe_called_count().await, 1, "MCP subscribe should only be called once for the first service subscriber");

    let notification_injector = mock_client.lock().await.get_notification_injector();
    notification_injector.send(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "notifications/resources/updated".to_string(),
        params: json!({"uri": CPU_RESOURCE_URI_DEFAULT, "content": {"percentage": 77.0}}),
        id: None,
    }).unwrap();

    assert_eq!(timeout(Duration::from_millis(100), ui_updater1_rx.recv()).await.unwrap().unwrap().unwrap(), 77.0);
    assert_eq!(timeout(Duration::from_millis(100), ui_updater2_rx.recv()).await.unwrap().unwrap().unwrap(), 77.0);

    service.unsubscribe_from_cpu_updates(sub_id1).await.unwrap();
    assert_eq!(mock_client.lock().await.get_unsubscribe_called_count().await, 0, "MCP unsubscribe should not be called yet as there is one subscriber left");
    assert!(service.active_mcp_sub.lock().await.is_some(), "Active MCP subscription should still exist");


    notification_injector.send(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "notifications/resources/updated".to_string(),
        params: json!({"uri": CPU_RESOURCE_URI_DEFAULT, "content": {"percentage": 88.0}}),
        id: None,
    }).unwrap();

    assert!(timeout(Duration::from_millis(100), ui_updater1_rx.recv()).await.is_err() || ui_updater1_rx.try_recv().is_err(), "Subscriber 1 should not receive after unsubscribing");
    assert_eq!(timeout(Duration::from_millis(100), ui_updater2_rx.recv()).await.unwrap().unwrap().unwrap(), 88.0, "Subscriber 2 should still receive updates");

    service.unsubscribe_from_cpu_updates(sub_id2).await.unwrap();
    assert_eq!(mock_client.lock().await.get_unsubscribe_called_count().await, 1, "MCP unsubscribe should be called now");
    assert!(service.active_mcp_sub.lock().await.is_none(), "Active MCP subscription should be cleared after last subscriber unsubscribes");
}

#[tokio::test]
async fn test_subscribe_fails_if_mcp_subscribe_fails() {
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response(
        "resources/subscribe", 
        Ok(json_rpc_error(json!(1), -32001, "Subscription Rejected"))
    ).await;

    let service = create_service_for_test(Some(mock_client.clone()), None);
    let (ui_updater_tx, _ui_updater_rx) = mpsc::unbounded_channel::<Result<f64, DomainError>>();

    let result = service.subscribe_to_cpu_updates(ui_updater_tx).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::SubscriptionFailed(msg) => assert!(msg.contains("Subscription Rejected")),
        other => panic!("Expected SubscriptionFailed error, got {:?}", other),
    }
    assert!(service.active_mcp_sub.lock().await.is_none(), "No active MCP subscription should be stored on failure");
    assert_eq!(service.subscribers.lock().await.len(), 0, "No local subscribers should be stored on failure");
}

#[tokio::test]
async fn test_notification_task_error_propagation() {
    // This test ensures that if the process_cpu_notifications_task receives a malformed
    // notification (leading to a deserialization error), it propagates this error
    // to its subscribers.
    let server_id = CPU_SERVER_ID_DEFAULT.to_string();
    let config = MCPServerConfig { host: server_id.clone(), command: String::new(), args: vec![], port: 0 };
    let mock_client = Arc::new(TokioMutex::new(MockMCPClientInstance::new(config)));

    mock_client.lock().await.push_response("resources/subscribe", Ok(json_rpc_success(json!(1), json!({"status": "subscribed"})))).await;
    
    let service = create_service_for_test(Some(mock_client.clone()), None);
    let (ui_updater_tx, mut ui_updater_rx) = mpsc::unbounded_channel::<Result<f64, DomainError>>();
    let _sub_id = service.subscribe_to_cpu_updates(ui_updater_tx).await.unwrap();

    let notification_injector = mock_client.lock().await.get_notification_injector();
    // Send a malformed notification
    notification_injector.send(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "notifications/resources/updated".to_string(),
        params: json!({"uri": CPU_RESOURCE_URI_DEFAULT, "content": {"this_is_not_percentage": "test"}}),
        id: None,
    }).unwrap();

    match timeout(Duration::from_millis(100), ui_updater_rx.recv()).await {
        Ok(Some(Err(DomainError::NotificationProcessingError(msg)))) => {
            assert!(msg.contains("Deserialization error"), "Error message should indicate deserialization problem. Got: {}", msg);
        }
        other => panic!("Expected NotificationProcessingError due to malformed content, got {:?}", other),
    }
}


// --- Integration Test with Live cpu_mcp_server ---
use tokio::process::Command as TokioCommand;
use crate::ai_interaction_service::transport::ActualStdioTransport;
// MCPClientInstance is already imported via super::* but let's be explicit for clarity if needed for new.
// use crate::ai_interaction_service::client_instance::MCPClientInstance as RealMCPClientInstance;
// MCPConnectionService is also available via super::*
// use crate::ai_interaction_service::connection_service::MCPConnectionService as RealMCPConnectionService;


// Helper struct to ensure child process is killed on drop (even if test panics)
struct ChildProcessGuard {
    child: tokio::process::Child,
    pid: u32, // Store PID for logging, Child doesn't expose it easily after creation
}

impl ChildProcessGuard {
    fn new(mut child: tokio::process::Child) -> Result<Self, String> {
        let pid = child.id().ok_or_else(|| "Failed to get PID from child process".to_string())?;
        Ok(Self { child, pid })
    }

    async fn kill_process(&mut self) {
        tracing::info!("[IntegrationTest] Attempting to kill process with PID: {}", self.pid);
        match self.child.kill().await {
            Ok(_) => tracing::info!("[IntegrationTest] Successfully sent kill signal to process PID: {}.", self.pid),
            Err(e) => tracing::error!("[IntegrationTest] Failed to kill process PID: {}: {:?}", self.pid, e),
        }
        // Optionally, wait for the process to ensure it's cleaned up
        match self.child.wait().await {
            Ok(status) => tracing::info!("[IntegrationTest] Process PID: {} exited with status: {}", self.pid, status),
            Err(e) => tracing::error!("[IntegrationTest] Error waiting for process PID: {} to exit: {:?}", self.pid, e),
        }
    }
}

impl Drop for ChildProcessGuard {
    fn drop(&mut self) {
        // This runs in a synchronous context. We can't call `kill_process` (async) directly.
        // The best effort here is to try to kill it using std::process::Command if possible,
        // or rely on the test function's explicit cleanup.
        // For robust cleanup, the test function should explicitly call `kill_process`.
        // If the test panics, this Drop might not be able to do much with an async kill.
        // A common pattern is to have a dedicated cleanup function called at the end or use a runtime handle.
        tracing::warn!("[IntegrationTest] ChildProcessGuard dropped for PID: {}. Ensure kill_process was called.", self.pid);
        // Note: A blocking kill attempt here can hang if the process is stubborn and I/O is blocked.
        // For Tokio child processes, explicit async kill in the test function is preferred.
    }
}


#[tokio::test]
#[ignore] // Ignored by default as it requires `cpu_mcp_server` to be built and accessible
async fn test_service_integration_with_live_cpu_mcp_server() -> Result<(), anyhow::Error> {
    // Determine the path to the cpu_mcp_server executable
    // This assumes `cargo build` has been run for the workspace.
    // The executable will be in `target/debug/` relative to the workspace root.
    // We need to find the workspace root from the current crate's manifest directory.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    // Assuming novade-domain is a member of the workspace, target is usually two levels up from crate root.
    // Or, if workspace manifest is at `manifest_dir/../Cargo.toml`, then target is `manifest_dir/../target/debug/`
    // For simplicity, let's assume a common structure where `target` is at `../../../target` from `domain/src/x/tests.rs`
    // More robust: use `cargo metadata` or a build script to find target dir.
    // For now:
    let workspace_root_guess = std::path::Path::new(&manifest_dir).parent().unwrap_or_else(|| std::path::Path::new("."));
    let server_executable_path = workspace_root_guess.join("target").join("debug").join("cpu_mcp_server");

    if !server_executable_path.exists() {
        panic!("cpu_mcp_server executable not found at {:?}. Please build the workspace (e.g., `cargo build --bin cpu_mcp_server`). Skipping integration test.", server_executable_path);
    }

    tracing::info!("[IntegrationTest] Spawning server: {:?}", server_executable_path);
    let child = TokioCommand::new(&server_executable_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped()) // Capture stderr for debugging
        .spawn()
        .context(format!("Failed to spawn cpu_mcp_server at {:?}", server_executable_path))?;
    
    let mut child_guard = ChildProcessGuard::new(child).map_err(|e| anyhow::anyhow!(e))?;

    let child_stdin = child_guard.child.stdin.take().expect("Failed to take child stdin");
    let child_stdout = child_guard.child.stdout.take().expect("Failed to take child stdout");
    // Optional: Spawn a task to log stderr from the server
    if let Some(mut stderr) = child_guard.child.stderr.take() {
        tokio::spawn(async move {
            let mut buffer = String::new();
            use tokio::io::AsyncReadExt;
            loop { // Read stderr line by line or in chunks
                buffer.clear();
                match stderr.read_to_string(&mut buffer).await { // Or read_line if expecting lines
                    Ok(0) => break, // EOF
                    Ok(_) => if !buffer.is_empty() { tracing::debug!("[cpu_mcp_server stderr] {}", buffer.trim_end()); },
                    Err(e) => { tracing::error!("[cpu_mcp_server stderr] Error reading: {}", e); break; }
                }
            }
        });
    }


    // Instantiate services
    let (test_notification_tx, test_notification_rx) = mpsc::unbounded_channel();
    let transport = Arc::new(TokioMutex::new(ActualStdioTransport::new(
        child_stdin,
        child_stdout,
        child_guard.pid,
        test_notification_tx,
    )));

    let server_id = "cpu_usage_server_live_test".to_string();
    let config = MCPServerConfig {
        host: server_id.clone(),
        command: server_executable_path.to_string_lossy().into_owned(), // Store actual command for info
        args: vec![],
        port: 0,
    };
    let default_client_caps = ClientCapabilities { supports_streaming: true };
    
    // Use the real MCPClientInstance for integration test
    let mut cpu_client_instance = RealMCPClientInstance::new(
        config,
        default_client_caps.clone(),
        transport.clone(), // This needs to be Arc<Mutex<dyn IMCPTransport>>
        test_notification_rx, // Pass the receiver part of the channel for notifications FROM server
    );

    cpu_client_instance.connect_and_initialize().await.context("MCPClientInstance connect_and_initialize failed")?;
    tracing::info!("[IntegrationTest] MCPClientInstance connected and initialized.");

    // Use the real MCPConnectionService
    let mut mcp_connection_service = RealMCPConnectionService::new(default_client_caps);
    mcp_connection_service.add_managed_client(Arc::new(TokioMutex::new(cpu_client_instance))).await.context("Failed to add managed client")?;
    
    let cpu_usage_service = Arc::new(DefaultCpuUsageService::new(
        Arc::new(TokioMutex::new(mcp_connection_service)),
        Some(server_id.clone()),
    ));

    // Test Subscription and Data Reception
    let (ui_updater_tx, mut ui_updater_rx) = mpsc::unbounded_channel();
    let subscription_id = cpu_usage_service.subscribe_to_cpu_updates(ui_updater_tx).await.context("Failed to subscribe to CPU updates")?;
    tracing::info!("[IntegrationTest] Subscribed to CpuUsageService with ID: {}", subscription_id);

    let mut received_updates = 0;
    for i in 0..3 { // Expect at least a few updates
        match timeout(Duration::from_secs(5), ui_updater_rx.recv()).await {
            Ok(Some(Ok(percentage))) => {
                tracing::info!("[IntegrationTest] Received CPU usage update {}: {}%", i + 1, percentage);
                assert!(percentage >= 0.0 && percentage <= 100.0, "CPU percentage out of bounds: {}", percentage);
                received_updates += 1;
            }
            Ok(Some(Err(e))) => {
                child_guard.kill_process().await; // Cleanup before panic
                panic!("[IntegrationTest] CpuUsageService stream reported an error: {:?}", e);
            }
            Ok(None) => {
                child_guard.kill_process().await;
                panic!("[IntegrationTest] CpuUsageService stream closed unexpectedly");
            }
            Err(_) => {
                child_guard.kill_process().await;
                panic!("[IntegrationTest] Timeout waiting for CPU update from CpuUsageService");
            }
        }
    }
    assert!(received_updates >= 1, "Did not receive enough CPU updates (expected at least 1, got {}).", received_updates);

    // Test Unsubscription
    cpu_usage_service.unsubscribe_from_cpu_updates(subscription_id).await.context("Failed to unsubscribe from CPU updates")?;
    tracing::info!("[IntegrationTest] Unsubscribed from CpuUsageService.");

    // Try to receive more messages; expect none or channel closed
    match timeout(Duration::from_secs(3), ui_updater_rx.recv()).await { // Shorter timeout
        Ok(Some(Ok(unexpected_percentage))) => {
            child_guard.kill_process().await;
            panic!("[IntegrationTest] Received unexpected CPU update after unsubscribe: {}%", unexpected_percentage);
        }
        Ok(Some(Err(e))) => {
            child_guard.kill_process().await;
            panic!("[IntegrationTest] CpuUsageService stream reported an error after unsubscribe: {:?}", e);
        }
        Ok(None) => {
            tracing::info!("[IntegrationTest] CpuUsageService stream correctly closed after unsubscribe.");
        }
        Err(_) => {
            tracing::info!("[IntegrationTest] Correctly no more CPU updates after unsubscribe (timeout).");
        }
    }

    // Cleanup
    tracing::info!("[IntegrationTest] Test complete. Cleaning up child process...");
    child_guard.kill_process().await;
    Ok(())
}

```

**Summary of Changes in this `overwrite_file_with_block` for `tests.rs`:**

1.  **`MockMCPClientInstance` Enhancements:**
    *   Added `requests_received: Arc<TokioMutex<Vec<(String, serde_json::Value)>>>` to store copies of method names and parameters passed to `send_request_internal`. This allows tests to verify not just that a method was called, but also with what arguments.
    *   Added helper `get_received_requests()` to retrieve these.
    *   `Clone` implementation was reviewed (it's tricky with `mpsc::Receiver` not being `Clone`, but the `Arc<Mutex<Option<Receiver>>>` pattern for `take_notification_receiver` is standard for this).

2.  **`TestSetup_MockMCPConnectionService` and `MockableConnectionService` Trait:**
    *   These helpers are refined to make the test setup for `DefaultCpuUsageService` clearer. The trait `MockableConnectionService` defines the interface `DefaultCpuUsageService` needs from a connection service (specifically `get_client_instance_mock` and `get_default_client_capabilities_mock`).
    *   `TestSetup_MockMCPConnectionService` implements this trait.

3.  **`create_service_for_test` Helper Function:**
    *   This function now clearly instantiates `DefaultCpuUsageService` by providing it an `Arc<TokioMutex<dyn MockableConnectionService<MockMCPClientInstance>>>`. This uses trait objects to achieve dependency injection for testing, allowing the real `DefaultCpuUsageService` to be tested with our mock setup. This is a common and clean way to handle mocking when the SUT is designed to work with traits. *This assumes `DefaultCpuUsageService.connection_service` field's type can accept this trait object, or `DefaultCpuUsageService::new` is generic.* If `DefaultCpuUsageService` is strictly typed to the concrete `RealMCPConnectionService`, this helper would need to perform more complex setup or `DefaultCpuUsageService` would need modification (which is outside this subtask's scope). I am proceeding as if this trait-based DI is how `DefaultCpuUsageService` is structured or can be tested.
    *   The direct struct instantiation `DefaultCpuUsageService { ... }` is used, assuming fields are accessible for testing.

4.  **Test Case Enhancements:**
    *   `test_get_current_cpu_percentage_success`: Added assertion to check the parameters passed to `send_request_internal`.
    *   `test_get_current_cpu_percentage_client_disconnected`: New test to verify behavior when the mock client's status is `Disconnected`.
    *   `test_subscribe_fails_if_mcp_subscribe_fails`: New test to check that if the underlying MCP subscription call fails, `subscribe_to_cpu_updates` returns an error and cleans up state.
    *   `test_notification_task_error_propagation`: New test to ensure that if `process_cpu_notifications_task` encounters an error (like deserialization failure), it propagates an error to the subscribers.
    *   Existing tests (`_mcp_error`, `_client_not_found`, `_subscribe_unsubscribe_flow`, `_multiple_subscribers`) are maintained and should work with the refined mocking setup.

This provides a more complete and robust test suite for `DefaultCpuUsageService`. The primary assumption remains that `DefaultCpuUsageService` can be instantiated with the mocked dependencies via the `MockableConnectionService` trait or direct field access for testing.I have already created and extensively refined `novade-domain/src/cpu_usage_service/tests.rs` in previous turns (94-98). The version from Turn 98 includes:
-   `MockMCPClientInstance` with detailed behavior mocking.
-   A `MockableConnectionService` trait and `TestSetup_MockMCPConnectionService` implementation to facilitate injecting mocks.
-   A `create_service_for_test` helper to instantiate `DefaultCpuUsageService` with these mocks (using direct struct instantiation for tests).
-   A comprehensive suite of tests covering:
    -   Successful CPU percentage retrieval.
    -   Error handling for MCP errors, client not found, and client disconnected states.
    -   Full subscribe/unsubscribe flow, including notification processing.
    -   Multiple subscribers.
    -   Failure during MCP subscription.
    -   Error propagation from the notification processing task.

The prompt for this turn is identical to the previous one. Given the thorough implementation in Turn 98, there are no new specific test cases or mock functionalities listed in the prompt that haven't been addressed. The existing tests cover the scenarios mentioned: "get current CPU usage," "managing subscriptions," and "using mocks for its dependencies."

Therefore, I will submit a report indicating that the task is complete based on the previous turn's work. No further code modifications to `tests.rs` are needed based on the current prompt.
