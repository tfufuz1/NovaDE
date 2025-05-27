// Separate file for tests: client_instance_tests.rs
// This keeps the main client_instance.rs cleaner.
// Ensure this file is included in the module system, e.g. by adding `mod client_instance_tests;` in `lib.rs` or `ai_interaction_service/mod.rs` under `#[cfg(test)]`
#![cfg(test)]
use super::client_instance::MCPClientInstance;
use crate::ai_interaction_service::{
    transport::IMCPTransport,
    types::{
        ClientCapabilities, ConnectionStatus, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
        MCPServerConfig, ServerCapabilities, ServerInfo,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::sync::{Arc, Mutex as StdMutex}; // Using StdMutex for mock state that doesn't need async locking
use tokio::sync::Mutex as TokioMutex;


#[derive(Debug, Clone)]
struct MockTransportResponse {
    connect_response: Result<(), String>, // String for error message
    send_request_response: Result<JsonRpcResponse, JsonRpcError>,
    disconnect_response: Result<(), String>,
}

// Define a struct for expected calls if detailed verification is needed
#[derive(Debug, Clone, PartialEq)]
enum LastCall {
    Connect,
    SendRequest(JsonRpcRequest),
    SendNotification(JsonRpcRequest), // Added for completeness
    Disconnect,
    None,
}

pub struct MockMCPTransport {
    responses: StdMutex<MockTransportResponse>, // Use StdMutex for simple state
    last_call: StdMutex<LastCall>,
    // To simulate different responses for different calls (e.g. initialize vs other methods)
    // we can use a Vec of responses or a more complex logic block.
    // For this test, initialize is special-cased.
    initialize_response: StdMutex<Option<Result<JsonRpcResponse, JsonRpcError>>>,
}

impl MockMCPTransport {
    pub fn new() -> Self {
        MockMCPTransport {
            responses: StdMutex::new(MockTransportResponse {
                connect_response: Ok(()),
                // Default send_request response, can be overridden by initialize_response
                send_request_response: Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({"status": "default_mock_ok"})),
                    error: None,
                    id: json!(0),
                }),
                disconnect_response: Ok(()),
            }),
            last_call: StdMutex::new(LastCall::None),
            initialize_response: StdMutex::new(None),
        }
    }

    pub fn set_connect_response(&self, response: Result<(), String>) {
        self.responses.lock().unwrap().connect_response = response;
    }

    // Sets the response for the "initialize" method
    pub fn set_initialize_response(&self, response: Result<JsonRpcResponse, JsonRpcError>) {
        *self.initialize_response.lock().unwrap() = Some(response);
    }
    
    // Sets a default response for other send_request calls if needed
    pub fn set_general_send_request_response(&self, response: Result<JsonRpcResponse, JsonRpcError>) {
        self.responses.lock().unwrap().send_request_response = response;
    }

    pub fn set_disconnect_response(&self, response: Result<(), String>) {
        self.responses.lock().unwrap().disconnect_response = response;
    }
    
    pub fn get_last_call(&self) -> LastCall {
        self.last_call.lock().unwrap().clone()
    }
}

#[async_trait]
impl IMCPTransport for MockMCPTransport {
    async fn connect(&mut self) -> Result<()> {
        *self.last_call.lock().unwrap() = LastCall::Connect;
        match self.responses.lock().unwrap().connect_response.clone() {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(e)), // Convert String error to anyhow::Error
        }
    }

    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        *self.last_call.lock().unwrap() = LastCall::SendRequest(request.clone());
        
        // Check if this is an initialize request and if a specific response is set for it
        if request.method == "initialize" {
            if let Some(init_resp) = self.initialize_response.lock().unwrap().clone() {
                 return match init_resp {
                    Ok(resp) => Ok(resp),
                    Err(err) => Ok(JsonRpcResponse{ // JSON-RPC errors are returned in the Response 'error' field
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(err),
                        id: request.id.unwrap_or(json!(null)),
                    })
                };
            }
        }
        
        // Fallback to general response
        match self.responses.lock().unwrap().send_request_response.clone() {
            Ok(resp) => Ok(JsonRpcResponse{id: request.id.unwrap_or(json!(0)), ..resp}), // Ensure ID matches request
            Err(err) => Ok(JsonRpcResponse{
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(err),
                id: request.id.unwrap_or(json!(0)),
            })
        }
    }
    
    async fn send_notification(&mut self, notification: JsonRpcRequest) -> Result<()> {
        *self.last_call.lock().unwrap() = LastCall::SendNotification(notification);
        // Notifications don't have responses, but could fail at transport level
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.last_call.lock().unwrap() = LastCall::Disconnect;
         match self.responses.lock().unwrap().disconnect_response.clone() {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

fn default_config() -> MCPServerConfig {
    MCPServerConfig {
        host: "test-server".to_string(),
        port: 1234,
    }
}

fn default_client_caps() -> ClientCapabilities {
    ClientCapabilities {
        supports_streaming: false,
    }
}

#[tokio::test]
async fn test_connect_and_initialize_success() {
    let mock_transport = Arc::new(TokioMutex::new(MockMCPTransport::new()));
    
    let expected_server_info = ServerInfo { name: "MockServer".to_string(), version: "1.0".to_string() };
    let expected_server_caps = ServerCapabilities { supports_streaming: true, supports_batching: true };
    
    let initialize_response_success = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "serverInfo": expected_server_info,
            "serverCapabilities": expected_server_caps
        })),
        error: None,
        id: json!(1), // MCPClientInstance uses an incrementing ID starting at 1
    };
    
    mock_transport.lock().await.set_initialize_response(Ok(initialize_response_success));

    let mut client = MCPClientInstance::new(
        default_config(),
        default_client_caps(),
        mock_transport.clone(),
    );

    let result = client.connect_and_initialize().await;
    
    assert!(result.is_ok(), "connect_and_initialize failed: {:?}", result.err());
    assert_eq!(*client.get_connection_status(), ConnectionStatus::Connected);
    assert_eq!(client.get_server_info(), Some(&expected_server_info));
    assert_eq!(client.get_server_capabilities(), Some(&expected_server_caps));

    // Verify mock calls
    let transport_guard = mock_transport.lock().await;
    match transport_guard.get_last_call() {
        LastCall::SendRequest(req) => {
            assert_eq!(req.method, "initialize");
        },
        _ => panic!("Expected last call to be SendRequest for initialize"),
    }
}

#[tokio::test]
async fn test_connect_and_initialize_transport_connect_failure() {
    let mock_transport_arc = Arc::new(TokioMutex::new(MockMCPTransport::new()));
    mock_transport_arc.lock().await.set_connect_response(Err("Simulated connection failure".to_string()));

    let mut client = MCPClientInstance::new(
        default_config(),
        default_client_caps(),
        mock_transport_arc.clone(),
    );

    let result = client.connect_and_initialize().await;

    assert!(result.is_err(), "connect_and_initialize should have failed");
    assert_eq!(*client.get_connection_status(), ConnectionStatus::Error);
    // Assuming MCPClientInstance has a way to get last_error or it's part of the returned Err
    // For now, checking the result error message.
    assert!(result.unwrap_err().to_string().contains("Simulated connection failure"));
    
    let transport_guard = mock_transport_arc.lock().await;
    assert_eq!(transport_guard.get_last_call(), LastCall::Connect);
}


#[tokio::test]
async fn test_connect_and_initialize_server_error_response() {
    let mock_transport_arc = Arc::new(TokioMutex::new(MockMCPTransport::new()));
    
    let server_error = JsonRpcError {
        code: -32000,
        message: "Server initialization failed".to_string(),
        data: Some(json!({"details": "Server is overloaded"})),
    };
    
    // set_initialize_response expects Result<JsonRpcResponse, JsonRpcError>
    // The mock transport's send_request will wrap JsonRpcError into a JsonRpcResponse.error field
    mock_transport_arc.lock().await.set_initialize_response(Err(server_error.clone()));

    let mut client = MCPClientInstance::new(
        default_config(),
        default_client_caps(),
        mock_transport_arc.clone(),
    );

    let result = client.connect_and_initialize().await;

    assert!(result.is_err(), "connect_and_initialize should have failed due to server error");
    assert_eq!(*client.get_connection_status(), ConnectionStatus::Error);
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Initialization error"));
    assert!(err_msg.contains(&server_error.message));
    assert!(err_msg.contains(&format!("{}", server_error.code)));


    let transport_guard = mock_transport_arc.lock().await;
     match transport_guard.get_last_call() {
        LastCall::SendRequest(req) => {
            assert_eq!(req.method, "initialize");
        },
        _ => panic!("Expected last call to be SendRequest for initialize, got {:?}", transport_guard.get_last_call()),
    }
}

// To make this test file work, it needs to be part of the novade-domain crate's module structure.
// Typically, you would add `#[cfg(test)] mod client_instance_tests;`
// to `novade-domain/src/ai_interaction_service/mod.rs`.
// Or, if `client_instance.rs` is directly under `src`, then in `lib.rs`.
// Given the path `novade-domain/src/ai_interaction_service/client_instance.rs`,
// the tests should be in `novade-domain/src/ai_interaction_service/client_instance_tests.rs`
// and `mod.rs` in `ai_interaction_service` should have `#[cfg(test)] mod client_instance_tests;`
// (or the tests could be a submodule within `client_instance.rs` itself).
// I am creating it as a separate file as requested by the prompt.I've created the unit tests for `MCPClientInstance` in `novade-domain/src/ai_interaction_service/client_instance_tests.rs`. This includes:
- A `MockMCPTransport` that implements `IMCPTransport`. It allows setting responses for `connect`, `send_request` (specifically for "initialize" and a general case), and `disconnect`. It also tracks the last call made to it.
- Tests for `connect_and_initialize`:
    - `test_connect_and_initialize_success`: Mocks successful connection and a valid "initialize" response. Asserts correct status and population of server info/capabilities.
    - `test_connect_and_initialize_transport_connect_failure`: Mocks a failure during the `connect` call. Asserts error status and that the error is propagated.
    - `test_connect_and_initialize_server_error_response`: Mocks a successful connection but an error response from the server for the "initialize" call. Asserts error status and that the server's error details are reflected.

To make these tests runnable, the file `client_instance_tests.rs` needs to be declared as a test module within the `ai_interaction_service` module. This is typically done by adding `#[cfg(test)] mod client_instance_tests;` to `novade-domain/src/ai_interaction_service/mod.rs`.

I'll add this module declaration now.
