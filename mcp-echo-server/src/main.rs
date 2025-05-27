use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

// --- Simplified Structs for Echo Server ---

#[derive(Deserialize, Debug)]
struct JsonRpcRequestIncoming {
    jsonrpc: Option<String>, // Optional to catch parsing errors if missing
    method: String,
    params: Option<Value>,
    id: Option<Value>, // Can be number, string, or null
}

#[derive(Serialize, Debug)]
struct JsonRpcResponseOutgoing {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcErrorOutgoing>,
    id: Value, // Echo back the original ID
}

#[derive(Serialize, Debug)]
struct JsonRpcErrorOutgoing {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize, Debug)]
struct ServerInfoEcho {
    name: String,
    version: String,
    #[serde(rename = "protocolVersion")] // Ensure correct serialization for client
    protocol_version: String,
}

#[derive(Serialize, Debug)]
struct ToolDefinitionEcho {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

#[derive(Serialize, Debug)]
struct ServerCapabilitiesEcho {
    #[serde(rename = "supportsStreaming")]
    supports_streaming: bool,
    #[serde(rename = "supportsBatching")]
    supports_batching: bool,
    tools: Vec<ToolDefinitionEcho>,
}

#[derive(Serialize, Debug)]
struct InitializeResultEcho {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    #[serde(rename = "serverInfo")]
    server_info: ServerInfoEcho,
    #[serde(rename = "serverCapabilities")]
    server_capabilities: ServerCapabilitiesEcho,
}

#[derive(Deserialize, Debug)]
struct CallToolParamsEcho {
    name: String,
    arguments: Value,
}

// --- Helper to create error responses ---
fn create_error_response(id: Value, code: i32, message: String) -> JsonRpcResponseOutgoing {
    JsonRpcResponseOutgoing {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcErrorOutgoing {
            code,
            message,
            data: None,
        }),
        id,
    }
}

// --- Main Server Logic ---
#[tokio::main]
async fn main() {
    let mut reader = BufReader::new(stdin());
    let mut stdout = stdout();

    loop {
        let mut line_buffer = String::new();
        match reader.read_line(&mut line_buffer).await {
            Ok(0) => { // EOF
                eprintln!("[EchoServer] EOF received, shutting down.");
                break;
            }
            Ok(_) => {
                let trimmed_line = line_buffer.trim();
                if trimmed_line.is_empty() {
                    continue;
                }

                eprintln!("[EchoServer] Received line: {}", trimmed_line);

                let request_id_for_error = match serde_json::from_str::<Value>(trimmed_line) {
                    Ok(val) => val.get("id").cloned().unwrap_or(Value::Null),
                    Err(_) => Value::Null, 
                };
                
                let response = match serde_json::from_str::<JsonRpcRequestIncoming>(trimmed_line) {
                    Err(e) => {
                        eprintln!("[EchoServer] Parse error: {}", e);
                        create_error_response(request_id_for_error, -32700, format!("Parse error: {}", e))
                    }
                    Ok(request) => {
                        if request.jsonrpc.as_deref() != Some("2.0") {
                             eprintln!("[EchoServer] Invalid jsonrpc version or missing: {:?}", request.jsonrpc);
                             create_error_response(request.id.clone().unwrap_or(Value::Null), -32600, "Invalid Request: 'jsonrpc' version must be '2.0'".to_string())
                        } else {
                            let current_request_id = request.id.clone().unwrap_or(Value::Null);
                            match request.method.as_str() {
                                "initialize" => {
                                    eprintln!("[EchoServer] Handling 'initialize'");
                                    let init_result = InitializeResultEcho {
                                        protocol_version: "2025-03-26".to_string(),
                                        server_info: ServerInfoEcho {
                                            name: "MCP Echo Server".to_string(),
                                            version: "0.1.0".to_string(),
                                            protocol_version: "2025-03-26".to_string(),
                                        },
                                        server_capabilities: ServerCapabilitiesEcho {
                                            supports_streaming: false,
                                            supports_batching: false,
                                            tools: vec![ToolDefinitionEcho {
                                                name: "echo".to_string(),
                                                description: "Echoes back the provided payload.".to_string(),
                                                input_schema: serde_json::json!({"type": "object", "properties": {"payload": {"type": "any"}}}),
                                                output_schema: serde_json::json!({"type": "any"}),
                                            }],
                                        },
                                    };
                                    JsonRpcResponseOutgoing {
                                        jsonrpc: "2.0".to_string(),
                                        result: Some(serde_json::to_value(init_result).unwrap()),
                                        error: None,
                                        id: current_request_id,
                                    }
                                }
                                "tools/call" => {
                                    eprintln!("[EchoServer] Handling 'tools/call'");
                                    match request.params {
                                        Some(params_value) => {
                                            match serde_json::from_value::<CallToolParamsEcho>(params_value) {
                                                Ok(tool_params) => {
                                                    if tool_params.name == "echo" {
                                                        JsonRpcResponseOutgoing {
                                                            jsonrpc: "2.0".to_string(),
                                                            result: Some(tool_params.arguments), 
                                                            error: None,
                                                            id: current_request_id,
                                                        }
                                                    } else {
                                                        create_error_response(current_request_id, -32601, format!("Method not found (unknown tool name: {})", tool_params.name))
                                                    }
                                                }
                                                Err(e) => {
                                                    create_error_response(current_request_id, -32602, format!("Invalid params for tools/call: {}", e))
                                                }
                                            }
                                        }
                                        None => {
                                             create_error_response(current_request_id, -32602, "Invalid params: 'params' field missing for tools/call".to_string())
                                        }
                                    }
                                }
                                "ping" => {
                                     eprintln!("[EchoServer] Handling 'ping'");
                                     JsonRpcResponseOutgoing {
                                        jsonrpc: "2.0".to_string(),
                                        result: Some(serde_json::json!({ "status": "pong" })),
                                        error: None,
                                        id: current_request_id,
                                    }
                                }
                                "shutdown" => {
                                    eprintln!("[EchoServer] Handling 'shutdown'");
                                    let shutdown_response = JsonRpcResponseOutgoing {
                                        jsonrpc: "2.0".to_string(),
                                        result: Some(Value::Null), 
                                        error: None,
                                        id: current_request_id,
                                    };
                                    let response_str = serde_json::to_string(&shutdown_response).unwrap();
                                    let framed_response = format!("{}\n", response_str);
                                    if stdout.write_all(framed_response.as_bytes()).await.is_err() {
                                        eprintln!("[EchoServer] Error writing shutdown response to stdout.");
                                    }
                                    if stdout.flush().await.is_err() {
                                         eprintln!("[EchoServer] Error flushing stdout for shutdown response.");
                                    }
                                    eprintln!("[EchoServer] Sent shutdown response. Exiting.");
                                    break; 
                                }
                                _ => {
                                    eprintln!("[EchoServer] Method not found: {}", request.method);
                                    create_error_response(current_request_id, -32601, format!("Method not found: {}", request.method))
                                }
                            }
                        }
                    }
                };

                let response_str = serde_json::to_string(&response).unwrap();
                let framed_response = format!("{}\n", response_str);
                eprintln!("[EchoServer] Sending response: {}", framed_response.trim());
                if stdout.write_all(framed_response.as_bytes()).await.is_err() {
                     eprintln!("[EchoServer] Error writing to stdout. Client might have disconnected.");
                     break; 
                }
                if stdout.flush().await.is_err() {
                    eprintln!("[EchoServer] Error flushing stdout. Client might have disconnected.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("[EchoServer] Error reading line from stdin: {}. Shutting down.", e);
                break;
            }
        }
    }
    eprintln!("[EchoServer] Server main loop exited.");
}
