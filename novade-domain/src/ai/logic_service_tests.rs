#![cfg(test)]

// logic_service_tests.rs is in novade-domain/src/ai/
// It tests AIInteractionLogicService and DefaultAIInteractionLogicService from super (ai/logic_service.rs)
use super::logic_service::{AIInteractionLogicService, DefaultAIInteractionLogicService};

// Items from the mcp submodule (novade-domain/src/ai/mcp/) are accessed via crate::ai::mcp::
// or via re-exports from crate::ai (if ai/mod.rs re-exports them)
use crate::ai::{ // These are re-exported by novade-domain/src/ai/mod.rs
    MCPConnectionService,
    MCPConsentManager,
    MCPServerConfig,
    ClientCapabilities,
    ConnectionStatus, // For assertions
    AIDataCategory, AIConsent, AIConsentStatus, AIInteractionError,
    // MCPClientInstance is also re-exported by ai/mod.rs but might not be directly used here
    // if MCPConnectionService abstracts it away.
};
// StdioTransportHandler and specific JSON-RPC types are not directly used in these tests,
// as they are abstracted by MCPConnectionService and DefaultAIInteractionLogicService.
// MCPClientInstance is used by MCPConnectionService, but tests here interact with LogicService.
// Use the mock that spawns the real mcp-echo-server
use novade_system::mcp_client_service::{
    IMCPClientService as SystemIMCPClientService, // Alias to avoid name clash
    MockRealProcessMCPClientService, // The service that spawns the real binary
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;


// Helper to get the path to the mcp-echo-server binary
// Assumes it's in the workspace target directory: target/debug/mcp-echo-server
fn get_echo_server_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // novade-domain
    path.pop(); // Go up to workspace root (e.g., /app)
    path.push("target/debug/mcp-echo-server");
    path
}

// RAII guard for ensuring server termination
struct TestContext {
    system_service: Arc<MockRealProcessMCPClientService>,
    logic_service: DefaultAIInteractionLogicService,
    connection_service: Arc<Mutex<MCPConnectionService>>,
    server_config: MCPServerConfig,
    server_pid: Option<u32>, // Store PID if server is successfully spawned
}

impl TestContext {
    async fn setup() -> Self {
        let echo_server_binary_path = get_echo_server_path();
        if !echo_server_binary_path.exists() {
            // Attempt to build the mcp-echo-server if it doesn't exist.
            // This is a bit of a hack for tests, assumes `cargo build` can be run.
            println!("[TestContext] mcp-echo-server binary not found at {:?}. Attempting to build...", echo_server_binary_path);
            let build_status = tokio::process::Command::new("cargo")
                .arg("build")
                .arg("--package")
                .arg("mcp-echo-server")
                .status()
                .await;
            if !build_status.map_or(false, |s| s.success()) {
                 panic!("Failed to build mcp-echo-server. Build status: {:?}", build_status);
            }
            if !echo_server_binary_path.exists() {
                 panic!("mcp-echo-server binary still not found after build attempt at {:?}. Ensure the project structure and build process are correct.", echo_server_binary_path);
            }
        }


        let system_service = Arc::new(MockRealProcessMCPClientService::new(echo_server_binary_path));
        let default_client_caps = ClientCapabilities { supports_streaming: false };
        let connection_service = Arc::new(Mutex::new(MCPConnectionService::new(
            default_client_caps.clone(),
            system_service.clone(), // MCPConnectionService needs the system service to spawn processes
        )));
        let consent_manager = Arc::new(MCPConsentManager::new());
        let logic_service = DefaultAIInteractionLogicService::new(connection_service.clone(), consent_manager.clone());

        let server_config = MCPServerConfig {
            host: "localhost".to_string(), // Hostname for ServerId, doesn't affect stdio path
            port: 0, // Port is not used for stdio, but part of the config struct
        };
        
        TestContext {
            system_service,
            logic_service,
            connection_service,
            server_config,
            server_pid: None,
        }
    }

    async fn connect_server(&mut self) {
        // Use connect_to_server which internally calls spawn_stdio_server
        match self.connection_service.lock().await.connect_to_server(self.server_config.clone()).await {
            Ok(pid) => {
                self.server_pid = Some(pid);
                println!("[TestContext] mcp-echo-server spawned and connected with PID: {}", pid);
            }
            Err(e) => {
                panic!("[TestContext] Failed to connect to server: {:?}", e);
            }
        }
    }
}

// Implement Drop for TestContext to ensure server termination
impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(pid) = self.server_pid {
            println!("[TestContext] Cleaning up: Terminating server with PID: {}", pid);
            // Need to run the async terminate_stdio_server. This is tricky in Drop.
            // Best effort: use a blocking call or spawn a task if in async context.
            // For tokio tests, the runtime might still be active.
            let system_service_clone = self.system_service.clone();
            let handle = tokio::runtime::Handle::try_current();
            
            match handle {
                Ok(h) => {
                    h.block_on(async move {
                        if let Err(e) = system_service_clone.terminate_stdio_server(pid).await {
                            eprintln!("[TestContext] Error terminating server PID {}: {:?}", pid, e);
                        } else {
                            println!("[TestContext] Server PID {} terminated successfully.", pid);
                        }
                    });
                }
                Err(_) => {
                     eprintln!("[TestContext] Could not get Tokio runtime handle to terminate server PID {}. Manual cleanup might be needed.", pid);
                     // Fallback or log: In some test environments, blocking_on may not be ideal from drop.
                     // For this setup, it's a common pattern for test cleanup.
                }
            }
        }
    }
}


#[tokio::test]
async fn test_list_available_models_with_live_echo_server() {
    let mut context = TestContext::setup().await;
    context.connect_server().await; // Spawns and connects to the echo server

    // Load model profiles by listing available models (which internally might call load_initial_server_configurations or similar logic)
    // Or directly call a method that refreshes/loads models if list_available_models doesn't trigger connection.
    // The MCPConnectionService::connect_to_server already handles the connection and initialization.
    // So, AIInteractionLogicService::list_available_models should now find it.
    
    let models = context.logic_service.list_available_models().await.expect("list_available_models failed");

    assert_eq!(models.len(), 1, "Expected one model profile from the echo server");
    let model_profile = &models[0];

    assert_eq!(model_profile.server_info.name, "MCP Echo Server");
    assert_eq!(model_profile.server_info.version, "0.1.0");
    assert_eq!(model_profile.server_info.protocol_version, Some("2025-03-26".to_string()));
    
    // Assert that the profile's associated MCPClientInstance has correct status and capabilities
    let client_instance_arc = context.connection_service.lock().await.get_client_instance(&model_profile.server_id)
        .expect("Client instance not found in connection service after listing models");
    
    let client_instance_guard = client_instance_arc.lock().await;
    assert_eq!(*client_instance_guard.get_connection_status(), ConnectionStatus::Connected);
    
    let server_caps = client_instance_guard.get_server_capabilities().expect("Server capabilities not set on client instance");
    assert!(!server_caps.supports_streaming); // As per echo server's mock response
    assert!(!server_caps.supports_batching);
    assert_eq!(server_caps.tools.len(), 1);
    assert_eq!(server_caps.tools[0].name, "echo");
    assert_eq!(server_caps.tools[0].description, "Echoes back the provided payload.");
}

#[tokio::test]
async fn test_call_echo_tool_on_live_server() {
    let mut context = TestContext::setup().await;
    context.connect_server().await; // Spawns and connects

    // Ensure models are loaded so we can get a model_id
    let models = context.logic_service.list_available_models().await.expect("list_available_models failed");
    assert!(!models.is_empty(), "No models available from live echo server.");
    let model_id = models[0].model_id.clone();

    // Initiate interaction
    let interaction_id = context.logic_service.initiate_interaction(model_id.clone(), None).await.expect("initiate_interaction failed");

    // Provide consent (using placeholder user_id and categories as per DefaultAIInteractionLogicService)
    let user_id = "default_user"; 
    let consent_record = AIConsent {
        consent_id: "test-consent-live-echo-001".to_string(),
        user_id: user_id.to_string(),
        model_id: model_id.clone(),
        data_categories: vec![AIDataCategory::Personal, AIDataCategory::Public], // Grant for required Personal category
        granted_at: "2024-01-01T00:00:00Z".to_string(),
        expires_at: None,
    };
    context.logic_service.provide_user_consent(consent_record).await.expect("provide_user_consent failed");

    // Call the "echo" tool
    let tool_name = "echo".to_string();
    let arguments = json!({ "data": "hello echo" });
    
    let result_value = context.logic_service.execute_tool_for_interaction(
        &interaction_id,
        tool_name,
        arguments.clone(), // clone arguments for assertion later
    ).await.expect("execute_tool_for_interaction failed");

    assert_eq!(result_value, arguments, "The echoed result from the tool does not match the arguments sent.");
}

// Note: The Drop implementation of TestContext handles server termination.
// MCPConnectionService constructor needs to be updated to accept Arc<dyn SystemIMCPClientService>
// This requires a change in `novade-domain/src/ai_interaction_service/connection_service.rs`
//
// The `MCPConnectionService::new` needs to be:
// pub fn new(default_client_capabilities: ClientCapabilities, system_mcp_service: Arc<dyn SystemIMCPClientService>) -> Self {
// And store `system_mcp_service`.
// Then `connect_to_server` should use `self.system_mcp_service.spawn_stdio_server(...)`
//
// I will make this change to MCPConnectionService in the next step.
